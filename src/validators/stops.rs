use crate::issues::{Issue, IssueType, Severity};
use gtfs_structures::LocationType;

pub fn validate(gtfs: &gtfs_structures::Gtfs) -> Vec<Issue> {
    validate_coord(gtfs)
        .into_iter()
        .chain(validate_parent_id(gtfs))
        .collect()
}

fn validate_coord(gtfs: &gtfs_structures::Gtfs) -> Vec<Issue> {
    let missing_coord = gtfs.stops.values().filter_map(|stop| check_coord(stop));
    let valid = gtfs
        .stops
        .values()
        .filter(|stop| !valid_coord(stop))
        .map(|stop| make_invalid_coord_issue(&**stop));
    missing_coord.chain(valid).collect()
}

// Check if the parent of the stop is correct
// Note: we don't check if the parent exists, because it is checked by the `InvalidReference` issue
fn validate_parent_id(gtfs: &gtfs_structures::Gtfs) -> Vec<Issue> {
    let stops_by_id: std::collections::HashMap<_, _> = gtfs
        .stops
        .iter()
        .map(|(_, stop)| (stop.id.clone(), stop.clone()))
        .collect();

    gtfs.stops
        .iter()
        .filter_map(|(_, stop)| {
            let parent = stop
                .parent_station
                .as_ref()
                .and_then(|parent| stops_by_id.get(parent));
            let details = match stop.location_type {
                LocationType::StopArea => {
                    // a stop area is forbidden to have a parent station
                    stop.parent_station
                        .as_ref()
                        .map(|_p| "it's not valid for a stop area to have a parent station")
                }
                LocationType::StopPoint => {
                    // the parent station of a StopPoint is optional, but should only be a stop area
                    parent.and_then(|parent| {
                        if parent.location_type != LocationType::StopArea {
                            Some("The parent of a stop point should be a stop area")
                        } else {
                            None
                        }
                    })
                }
                LocationType::GenericNode | LocationType::StationEntrance => {
                    // the parent station of a generic node or entrance is mandatory and should be a stop area
                    if parent
                        .map(|parent| parent.location_type != LocationType::StopArea)
                        .unwrap_or(true)
                    {
                        Some("The parent of a generic node or an entrance should be a stop area")
                    } else {
                        None
                    }
                }
                LocationType::BoardingArea => {
                    // the parent station of a boarding are is mandatory and should be a stop point
                    if parent
                        .map(|parent| parent.location_type != LocationType::StopPoint)
                        .unwrap_or(true)
                    {
                        Some("The parent of a boarding area should be a stop point")
                    } else {
                        None
                    }
                }
                _ => None,
            };
            if let Some(details) = details {
                let mut issue = make_invalid_parent_issue(&**stop).details(details);
                if let Some(parent) = parent {
                    issue.push_related_object(&**parent);
                }
                Some(issue)
            } else {
                None
            }
        })
        .collect()
}

fn check_coord(stop: &gtfs_structures::Stop) -> Option<Issue> {
    if stop.location_type != LocationType::GenericNode
        && stop.location_type != LocationType::BoardingArea
        && !has_coord(stop)
    {
        // the coordinates are optional for generic nodes and boarding area
        Some(
            make_missing_coord_issue(stop).details(match (stop.longitude, stop.latitude) {
                (None, None) => "Latitude and longitude are missing",
                (Some(lon), Some(lat)) if lon == 0.0 && lat == 0.0 => {
                    "Latitude and longitude are missing"
                }
                (Some(lon), _) if lon == 0.0 => "Longitude is missing",
                (_, Some(lat)) if lat == 0.0 => "Latitude is missing",
                _ => "Coordinates are ok",
            }),
        )
    } else {
        None
    }
}

fn has_coord(stop: &gtfs_structures::Stop) -> bool {
    match (stop.latitude, stop.longitude) {
        (Some(lon), Some(lat)) => lon != 0.0 && lat != 0.0,
        _ => false,
    }
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
fn make_invalid_parent_issue<T: gtfs_structures::Id + gtfs_structures::Type + std::fmt::Display>(
    o: &T,
) -> Issue {
    Issue::new_with_obj(Severity::Warning, IssueType::InvalidStopParent, o)
}

fn valid_coord(stop: &gtfs_structures::Stop) -> bool {
    match (stop.longitude, stop.latitude) {
        (Some(lon), Some(lat)) => (-180.0..=180.0).contains(&lon) && (-90.0..=90.0).contains(&lat),
        _ => false, // there is already an issue if the coord is missing
    }
}

#[test]
fn test_missing() {
    let gtfs = gtfs_structures::Gtfs::new("test_data/stops").unwrap();
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
    let gtfs = gtfs_structures::Gtfs::new("test_data/stops").unwrap();
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

#[test]
fn test_stop_parent() {
    let gtfs = gtfs_structures::Gtfs::new("test_data/stops").unwrap();
    let issues = validate(&gtfs);
    let invalid_coord_issue: Vec<_> = dbg!(issues
        .iter()
        .filter(|issue| issue.issue_type == IssueType::InvalidStopParent)
        .collect());

    assert_eq!(6, invalid_coord_issue.len());
    let stop_area_issue = invalid_coord_issue
        .iter()
        .find(|i| i.object_id == "BEATTY_AIRPORT")
        .expect("impossible to find the BEATTY_AIRPORT issue");
    assert_eq!(
        Some("it's not valid for a stop area to have a parent station".to_owned()),
        stop_area_issue.details
    );
    assert_eq!(
        vec![crate::issues::RelatedObject {
            id: "BULLFROG".to_owned(),
            object_type: Some(gtfs_structures::ObjectType::Stop),
            name: Some("Bullfrog (Demo)".to_owned())
        }],
        stop_area_issue.related_objects
    );
    let stop_point_with_parent_stop = invalid_coord_issue
        .iter()
        .find(|i| i.object_id == "STAGECOACH")
        .expect("impossible to find the STAGECOACH issue");
    assert_eq!(
        Some("The parent of a stop point should be a stop area".to_owned()),
        stop_point_with_parent_stop.details
    );
    let boarding_with_parent_stop_area = invalid_coord_issue
        .iter()
        .find(|i| i.object_id == "boarding_bad_parent")
        .expect("impossible to find the boarding_bad_parent issue");
    assert_eq!(
        Some("The parent of a boarding area should be a stop point".to_owned()),
        boarding_with_parent_stop_area.details
    );
    let boarding_without_parent = invalid_coord_issue
        .iter()
        .find(|i| i.object_id == "boarding_no_parent")
        .expect("impossible to find the boarding_no_parent issue");
    assert_eq!(
        Some("The parent of a boarding area should be a stop point".to_owned()),
        boarding_without_parent.details
    );
    let entrance_with_parent_stop_point = invalid_coord_issue
        .iter()
        .find(|i| i.object_id == "entrance_bad_parent")
        .expect("impossible to find the entrance_bad_parent issue");
    assert_eq!(
        Some("The parent of a generic node or an entrance should be a stop area".to_owned()),
        entrance_with_parent_stop_point.details
    );
    let entrance_without_parent = invalid_coord_issue
        .iter()
        .find(|i| i.object_id == "entrance_no_parent")
        .expect("impossible to find the entrance_no_parent issue");
    assert_eq!(
        Some("The parent of a generic node or an entrance should be a stop area".to_owned()),
        entrance_without_parent.details
    );
}
