use std::env;
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};

use git2::FetchOptions;
use once_cell::sync::OnceCell as SyncOnceCell;

use roaring::RoaringBitmap;

static INSTANCE: SyncOnceCell<Vec<Dataset>> = SyncOnceCell::new();

pub struct Datasets;

pub struct DatasetsIter {
    iter: std::slice::Iter<'static, Dataset>,
}

impl Iterator for DatasetsIter {
    type Item = &'static Dataset;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

impl IntoIterator for Datasets {
    type Item = &'static Dataset;
    type IntoIter = DatasetsIter;

    fn into_iter(self) -> Self::IntoIter {
        DatasetsIter {
            iter: INSTANCE
                .get_or_init(|| {
                    init_datasets().and_then(parse_datasets).expect("a collection of datasets")
                })
                .iter(),
        }
    }
}

pub struct Dataset {
    pub name: String,
    pub bitmaps: Vec<RoaringBitmap>,
}

fn init_datasets() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let out_dir = env::var_os("CARGO_MANIFEST_DIR").ok_or(env::VarError::NotPresent)?;

    let out_path = Path::new(&out_dir);
    let repo_path = out_path.join("real-roaring-datasets");

    // Check if in offline mode

    let offline = env::var("ROARINGRS_BENCH_OFFLINE");
    match offline {
        Ok(value) => {
            if value.parse::<bool>()? {
                return Ok(repo_path);
            }
        }
        Err(ref err) => match err {
            env::VarError::NotPresent => (),
            _ => {
                offline?;
            }
        },
    };

    // Setup progress callbacks

    let pb_cell = once_cell::unsync::OnceCell::new();
    let mut cb = git2::RemoteCallbacks::new();

    cb.transfer_progress(|progress| {
        let pb = pb_cell.get_or_init(|| {
            indicatif::ProgressBar::new(progress.total_objects() as u64)
                .with_style(
                    indicatif::ProgressStyle::default_bar()
                        .template(&format!(
                            "{{prefix}}{{msg:.cyan/blue}} [{{bar}}] {{pos}}/{}",
                            progress.total_objects()
                        ))
                        .progress_chars("#> "),
                )
                .with_prefix("    ")
                .with_message("Receiving objects")
        });

        pb.set_position((progress.local_objects() + progress.received_objects()) as u64);
        true
    });

    let mut fetch_opts = FetchOptions::new();
    fetch_opts.remote_callbacks(cb);

    // Do update

    if !Path::new(&repo_path).exists() {
        git2::build::RepoBuilder::new()
            .fetch_options(fetch_opts)
            .clone("https://github.com/RoaringBitmap/real-roaring-datasets.git", &repo_path)?;
    } else {
        let repo = git2::Repository::open(&repo_path)?;
        repo.find_remote("origin")?.fetch(&["master"], Some(&mut fetch_opts), None)?;

        let head = repo.head()?.peel_to_commit()?;
        let origin_master_head = repo
            .find_branch("origin/master", git2::BranchType::Remote)?
            .into_reference()
            .peel_to_commit()?;

        if head.id() != origin_master_head.id() {
            repo.reset(origin_master_head.as_object(), git2::ResetType::Hard, None)?;
        }
    }

    if let Some(pb) = pb_cell.get() {
        pb.finish()
    }

    Ok(repo_path)
}

fn parse_datasets<P: AsRef<Path>>(path: P) -> Result<Vec<Dataset>, Box<dyn std::error::Error>> {
    const DATASET_FILENAME_WHITELIST: &[&str] = &[
        "census-income.zip",
        "census-income_srt.zip",
        "census1881.zip",
        "census1881_srt.zip",
        "weather_sept_85.zip",
        "weather_sept_85_srt.zip",
        "wikileaks-noquotes.zip",
        "wikileaks-noquotes_srt.zip",
    ];

    use indicatif::{ProgressBar, ProgressStyle};
    use std::io::BufRead;
    use zip::ZipArchive;

    let dir = path.as_ref().read_dir()?;

    let mut datasets = Vec::new();

    // Future work: Reuse this buffer to parse croaring bitmaps for comparison
    let mut numbers = Vec::new();

    for dir_entry_result in dir {
        let dir_entry = dir_entry_result?;
        let metadata = dir_entry.metadata()?;
        let file_name = dir_entry.file_name();
        // TODO dont panic
        let file_name_str = file_name.to_str().expect("utf-8 filename");

        if metadata.is_file() && DATASET_FILENAME_WHITELIST.contains(&file_name_str) {
            let file = File::open(dir_entry.path())?;
            let name = file_name_str.split_at(file_name_str.len() - ".zip".len()).0.to_string();

            let mut zip = ZipArchive::new(file)?;

            let mut total_size = 0;
            for i in 0..zip.len() {
                let file = zip.by_index(i)?;
                total_size += file.size();
            }

            let pb = ProgressBar::new(total_size)
                .with_style(
                    ProgressStyle::default_bar()
                        .template("    {prefix:.green} [{bar}] {msg}")
                        .progress_chars("#> "),
                )
                .with_prefix("Parsing")
                .with_message(name.clone());

            let mut bitmaps = Vec::with_capacity(zip.len());
            for i in 0..zip.len() {
                let file = zip.by_index(i)?;
                let size = file.size();
                let buf = BufReader::new(file);

                for bytes in buf.split(b',') {
                    let bytes = bytes?;
                    let str = String::from_utf8(bytes)?;
                    let n = str.trim().parse::<u32>()?;
                    numbers.push(n);
                }

                let bitmap = RoaringBitmap::from_sorted_iter(numbers.iter().copied())?;
                numbers.clear();
                bitmaps.push(bitmap);

                pb.set_position(pb.position() + size);
            }

            pb.finish();
            datasets.push(Dataset { name, bitmaps });
        }
    }
    datasets.sort_unstable_by(|a, b| a.name.cmp(&b.name));
    println!();
    Ok(datasets)
}
