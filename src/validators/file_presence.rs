use crate::issues::{Issue, IssueType, Severity};

const MANDATORY_FILES: &[&str] = &[
    "agency.txt",
    "routes.txt",
    "stops.txt",
    "stop_times.txt",
    "trips.txt",
];

const OPTIONAL_FILES: &[&str] = &[
    "fare_attributes.txt",
    "calendar.txt",
    "calendar_dates.txt",
    "fare_rules.txt",
    "fare_media.txt",
    "fare_products.txt",
    "fare_leg_rules.txt",
    "fare_leg_join_rules.txt",
    "feed_info.txt",
    "frequencies.txt",
    "transfers.txt",
    "shapes.txt",
    "pathways.txt",
    "levels.txt",
    "feed_info.txt",
    "translations.txt",
    "attributions.txt",
    "timeframes.txt",
    "areas.txt",
    "stop_areas.txt",
    "networks.txt",
    "route_networks.txt",
    "location_groups.txt",
    "location_group_stops.txt",
    "locations.geojson",
    "booking_rules.txt",
];

fn missing_files(raw_gtfs: &gtfs_structures::RawGtfs) -> Vec<Issue> {
    MANDATORY_FILES
        .iter()
        .filter(|m| !raw_gtfs.files.iter().any(|f| f.ends_with(*m)))
        .map(|m| {
            Issue::new(Severity::Fatal, IssueType::MissingMandatoryFile, m)
                .details("The mandatory file was not found")
        })
        .collect()
}

fn extra_files(raw_gtfs: &gtfs_structures::RawGtfs) -> Vec<Issue> {
    raw_gtfs
        .files
        .iter()
        .filter(|f| {
            !MANDATORY_FILES.iter().any(|m| f.ends_with(m))
                && !OPTIONAL_FILES.iter().any(|o| f.ends_with(o))
        })
        .map(|f| {
            Issue::new(Severity::Information, IssueType::ExtraFile, f)
                .details("This file shouldn’t be in the archive")
        })
        .collect()
}

pub fn validate(raw_gtfs: &gtfs_structures::RawGtfs) -> Vec<Issue> {
    missing_files(raw_gtfs)
        .into_iter()
        .chain(extra_files(raw_gtfs))
        .collect()
}

#[test]
fn test_missing() {
    let raw = gtfs_structures::RawGtfs::new("test_data/missing_mandatory_files").unwrap();
    let validations = missing_files(&raw);
    assert_eq!(1, validations.len());
    assert_eq!(IssueType::MissingMandatoryFile, validations[0].issue_type);
    assert_eq!("stop_times.txt", validations[0].object_id);
    assert_eq!(Severity::Fatal, validations[0].severity);
}

#[test]
fn test_extra() {
    let raw = gtfs_structures::RawGtfs::new("test_data/missing_mandatory_files").unwrap();
    let validations = extra_files(&raw);
    assert_eq!(1, validations.len());
    assert_eq!(IssueType::ExtraFile, validations[0].issue_type);
    assert_eq!(Severity::Information, validations[0].severity);
}
