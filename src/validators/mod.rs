pub mod issues;
mod unused_stop;

extern crate gtfs_structures;

pub fn validate(gtfs: &gtfs_structures::Gtfs) -> Vec<issues::Issue> {
    unused_stop::validate(gtfs)
}
