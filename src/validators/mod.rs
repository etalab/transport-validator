pub mod issues;
mod unused_stop;

extern crate serde_json;
use failure::Error;

extern crate gtfs_structures;

pub fn validate_gtfs(gtfs: &gtfs_structures::Gtfs) -> Vec<issues::Issue> {
    unused_stop::validate(gtfs)
}

pub fn validate(input: &str) -> Result<String, Error> {
    let gtfs = if input.starts_with("http") {
        gtfs_structures::Gtfs::from_url(input)
    } else if input.to_lowercase().ends_with(".zip") {
        gtfs_structures::Gtfs::from_zip(input)
    } else {
        gtfs_structures::Gtfs::new(input)
    };

    gtfs.map(|gtfs| self::validate_gtfs(&gtfs))
        .and_then(|validation| Ok(serde_json::to_string(&validation)?))
}
