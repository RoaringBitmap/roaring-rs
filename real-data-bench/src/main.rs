#![feature(test)]
#![feature(duration_span)]
#![feature(slice_patterns)]

extern crate test;
extern crate roaring;
extern crate zip;

use std::io::Read;
use roaring::RoaringBitmap;

static CENSUS_INCOME: &'static [u8] = include_bytes!("../RoaringBitmap/real-roaring-dataset/src/main/resources/real-roaring-dataset/census-income.zip");

fn load(zip: &[u8]) -> Vec<RoaringBitmap<u32>> {
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(zip)).unwrap();

    (0..zip.len()).map(|i| {
      let mut file = zip.by_index(i).unwrap();
      let mut s = String::with_capacity(file.size() as usize);
      file.read_to_string(&mut s).unwrap();
      s.split(',').map(|i| i.trim().parse::<u32>().unwrap()).collect()
    }).collect()
}

fn and_test(bitmaps: &[RoaringBitmap<u32>]) -> u32 {
    bitmaps
      .windows(2)
      .map(|pair| if let [ref first, ref second] = pair { (first, second) } else { unreachable!() })
      .map(|(first, second)| (first & second).len())
      .fold(0, |total, len| total + len)
}

fn main() {
    let bitmaps = load(CENSUS_INCOME);

    println!("Testing that test is correct");
    assert_eq!(1245448, and_test(&bitmaps));
    println!("Test is correct");

    println!("Benchmarking");
    let dur = std::time::Duration::span(|| { test::black_box(and_test(&bitmaps)); });
    println!("Took {} μs", dur.as_secs() * 1_000_000 + (dur.subsec_nanos() as u64 / 1_000));
}
