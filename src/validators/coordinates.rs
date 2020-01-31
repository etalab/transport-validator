use crate::issues::{Issue, IssueType, Severity};

pub fn validate(gtfs: &gtfs_structures::Gtfs) -> Vec<Issue> {
    let missing_coord = gtfs
        .stops
        .values()
        .filter(|stop| !has_coord(stop))
        .map(|stop| make_missing_coord_issue(&**stop).details(missing_coord_details(stop)));
    let valid = gtfs
        .stops
        .values()
        .filter(|stop| !valid_coord(stop))
        .map(|stop| make_invalid_coord_issue(&**stop));
    missing_coord.chain(valid).collect()
}

fn has_coord(stop: &gtfs_structures::Stop) -> bool {
    stop.latitude != 0.0 && stop.longitude != 0.0
}

fn make_invalid_coord_issue<T: gtfs_structures::Id + gtfs_structures::Type + std::fmt::Display>(
    o: &T,
) -> Issue {
    Issue::new_with_obj(Severity::Error, IssueType::InvalidCoordinates, o)
}

fn make_missing_coord_issue<T: gtfs_structures::Id + gtfs_structures::Type + std::fmt::Display>(
    o: &T,
) -> Issue {
    Issue::new_with_obj(Severity::Warning, IssueType::MissingCoordinates, o)
}

fn missing_coord_details(stop: &gtfs_structures::Stop) -> &str {
    if stop.latitude == 0.0 && stop.longitude == 0.0 {
        "Latitude and longitude are missing"
    } else if stop.latitude == 0.0 {
        "Latitude is missing"
    } else {
        "Longitude is missing"
    }
}

fn valid_coord(stop: &gtfs_structures::Stop) -> bool {
    ((stop.longitude <= 180.0) && (stop.longitude >= -180.0))
        && ((stop.latitude <= 90.0) && (stop.latitude >= -90.0))
}

#[test]
fn test_missing() {
    let gtfs = gtfs_structures::Gtfs::new("test_data/coordinates").unwrap();
    let issues = validate(&gtfs);
    let missing_coord_issue: Vec<_> = issues
        .iter()
        .filter(|issue| issue.issue_type == IssueType::MissingCoordinates)
        .collect();

    assert_eq!(1, missing_coord_issue.len());
    assert_eq!("AMV", missing_coord_issue[0].object_id);
    assert_eq!(
        IssueType::MissingCoordinates,
        missing_coord_issue[0].issue_type
    );
}

#[test]
fn test_valid() {
    let gtfs = gtfs_structures::Gtfs::new("test_data/coordinates").unwrap();
    let issues = validate(&gtfs);
    let invalid_coord_issue: Vec<_> = issues
        .iter()
        .filter(|issue| issue.issue_type == IssueType::InvalidCoordinates)
        .collect();

    assert_eq!(1, invalid_coord_issue.len());
    assert_eq!("PARENT", invalid_coord_issue[0].object_id);
    assert_eq!(
        IssueType::InvalidCoordinates,
        invalid_coord_issue[0].issue_type
    );
}
