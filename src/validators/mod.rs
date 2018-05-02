pub mod issues;
mod unused_stop;
mod duration_distance;

extern crate serde_json;
use failure::Error;

extern crate gtfs_structures;

pub fn validate_gtfs(gtfs: &gtfs_structures::Gtfs) -> Vec<issues::Issue> {
    unused_stop::validate(gtfs)
        .into_iter()
        .chain(duration_distance::validate(gtfs))
        .collect()
}

pub fn validate(input: &str) -> Result<String, Error> {
    info!("Starting validation: {}", input);
    let gtfs = if input.starts_with("http") {
        info!("Starting download of {}", input);
        let result = gtfs_structures::Gtfs::from_url(input);
        info!("Download done of {}", input);
        result
    } else if input.to_lowercase().ends_with(".zip") {
        gtfs_structures::Gtfs::from_zip(input)
    } else {
        gtfs_structures::Gtfs::new(input)
    };

    gtfs.map(|gtfs| self::validate_gtfs(&gtfs))
        .and_then(|validation| Ok(serde_json::to_string(&validation)?))
        .or_else(|err| {
            Ok(serde_json::to_string(&[
                issues::Issue {
                    severity: issues::Severity::Fatal,
                    issue_type: issues::IssueType::InvalidArchive,
                    object_id: format!("{}", err),
                    object_name: None,
                    related_object_id: None,
                },
            ])?)
        })
}
