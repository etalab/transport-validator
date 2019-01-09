use crate::validators::issues::*;

pub fn validate(gtfs: &gtfs_structures::Gtfs) -> Vec<Issue> {
    let missing_coord = gtfs.stops
            .values()
            .filter(|stop| !has_coord(stop))
            .map(|stop | Issue {
                severity: Severity::Error,
                issue_type: IssueType::MissingCoordinates,
                object_id: stop.id.to_owned(),
                object_name: None,
                related_object_id: None,
                details: missing_coord_details(stop),
            });
    let valid = gtfs.stops
            .values()
            .filter(|stop| !valid_coord(stop))
            .map(|stop | Issue {
                severity: Severity::Error,
                issue_type: IssueType::InvalidCoordinates,
                object_id: stop.id.to_owned(),
                object_name: None,
                related_object_id: None,
                details: None,
            });
    missing_coord
        .chain(valid)
        .collect()
}

fn has_coord(stop: &gtfs_structures::Stop) -> bool {
    stop.latitude != 0.0 && stop.longitude != 0.0
}

fn missing_coord_details(stop: &gtfs_structures::Stop) -> Option<String> {
    if stop.latitude == 0.0 && stop.longitude == 0.0 {
        Some("Latitude and longitude are missing".to_string())
    } else if stop.latitude == 0.0 {
        Some("Latitude is missing".to_string())
    } else if stop.longitude == 0.0 {
        Some("Longitude is missing".to_string())
    } else {
        None
    }
}

fn valid_coord(stop: &gtfs_structures::Stop) -> bool {
    ((stop.longitude <= 180.0) && (stop.longitude >= -180.0)) && ((stop.latitude <= 90.0) && (stop.latitude >= -90.0))
}

#[test]
fn test_missing() {
    let gtfs = gtfs_structures::Gtfs::new("test_data/coordinates").unwrap();
    let issues = validate(&gtfs);
    let missing_coord_issue: Vec<_> = issues.iter()
        .filter(|issue| issue.issue_type == IssueType::MissingCoordinates)
        .collect();
    
    assert_eq!(1, missing_coord_issue.len());
    assert_eq!("AMV", missing_coord_issue[0].object_id);
    assert_eq!(IssueType::MissingCoordinates, missing_coord_issue[0].issue_type);
}

#[test]
fn test_valid() {
    let gtfs = gtfs_structures::Gtfs::new("test_data/coordinates").unwrap();
    let issues = validate(&gtfs);
    let invalid_coord_issue: Vec<_> = issues.iter()
        .filter(|issue| issue.issue_type == IssueType::InvalidCoordinates)
        .collect();

    assert_eq!(1, invalid_coord_issue.len());
    assert_eq!("PARENT", invalid_coord_issue[0].object_id);
    assert_eq!(IssueType::InvalidCoordinates, invalid_coord_issue[0].issue_type);
}