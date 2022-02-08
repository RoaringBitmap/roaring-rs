use std::env;
use std::path::Path;
use std::process::Command;

const DATASET_CENSUS_INCOME: &str = "census-income";
const DATASET_CENSUS_INCOME_SRT: &str = "census-income_srt";
const DATASET_CENSUS1881: &str = "census1881";
const DATASET_CENSUS1881_SRT: &str = "census1881_srt";
const DATASET_DIMENSION_003: &str = "dimension_003";
const DATASET_DIMENSION_008: &str = "dimension_008";
const DATASET_DIMENSION_033: &str = "dimension_033";
const DATASET_USCENSUS2000: &str = "uscensus2000";
const DATASET_WEATHER_SEPT_85: &str = "weather_sept_85";
const DATASET_WEATHER_SEPT_85_SRT: &str = "weather_sept_85_srt";
const DATASET_WIKILEAKS_NOQUOTES: &str = "wikileaks-noquotes";
const DATASET_WIKILEAKS_NOQUOTES_SRT: &str = "wikileaks-noquotes_srt";

const DATASETS: &[&str] = &[
    DATASET_CENSUS_INCOME,
    DATASET_CENSUS_INCOME_SRT,
    DATASET_CENSUS1881,
    DATASET_CENSUS1881_SRT,
    DATASET_DIMENSION_003,
    DATASET_DIMENSION_008,
    DATASET_DIMENSION_033,
    DATASET_USCENSUS2000,
    DATASET_WEATHER_SEPT_85,
    DATASET_WEATHER_SEPT_85_SRT,
    DATASET_WIKILEAKS_NOQUOTES,
    DATASET_WIKILEAKS_NOQUOTES_SRT,
];

fn main() {
    if !Path::new("real-roaring-datasets/.git").exists() {
        let status = Command::new("git")
            .args(&["submodule", "update", "--init", "real-roaring-datasets"])
            .status()
            .unwrap();

        assert!(status.success());

        let current_dir = env::current_dir().unwrap();
        for dataset in DATASETS {
            let status = Command::new("unzip")
                .current_dir(current_dir.join("real-roaring-datasets"))
                .arg("-n") // never overwrite existing files
                .arg(dataset)
                .args(&["-d", dataset])
                .status()
                .unwrap();

            assert!(status.success());
        }
    }
}
