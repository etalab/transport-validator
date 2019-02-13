use crate::validators::issues::*;
use geo::algorithm::haversine_distance::HaversineDistance;
use geo::Point;
use itertools::Itertools;

pub fn validate(gtfs: &gtfs_structures::Gtfs) -> Vec<Issue> {
    let mut issues = vec![];

    for (stop_a, stop_b) in gtfs
        .stops
        .values()
        .filter(|stop| stop.location_type != gtfs_structures::LocationType::StationEntrance)
        .tuple_combinations()
    {
        if duplicate_stops(stop_a, stop_b) {
            issues.push(make_duplicate_stops_issue(&**stop_a).add_related_object(&**stop_b));
        }
    }
    issues
}

fn duplicate_stops(stop_a: &gtfs_structures::Stop, stop_b: &gtfs_structures::Stop) -> bool {
    stop_a.name == stop_b.name
        && stop_a.location_type == stop_b.location_type
        && too_close_stops(stop_a, stop_b)
}

fn too_close_stops(stop_a: &gtfs_structures::Stop, stop_b: &gtfs_structures::Stop) -> bool {
    let a = Point::new(stop_a.longitude, stop_a.latitude);
    let b = Point::new(stop_b.longitude, stop_b.latitude);
    match stop_a.location_type {
        gtfs_structures::LocationType::StopPoint => a.haversine_distance(&b) < 2.,
        gtfs_structures::LocationType::StopArea => a.haversine_distance(&b) < 100.,
        _ => false,
    }
}

fn make_duplicate_stops_issue<
    T: gtfs_structures::Id + gtfs_structures::Type + std::fmt::Display,
>(
    o: &T,
) -> Issue {
    Issue::new_with_obj(Severity::Error, IssueType::DuplicateStops, o)
}

#[test]
fn test_stop_points() {
    let gtfs = gtfs_structures::Gtfs::new("test_data/duplicate_stops").unwrap();
    let issues = validate(&gtfs);
    let duplicate_stops_issues: Vec<_> = issues
        .iter()
        .filter(|issue| issue.issue_type == IssueType::DuplicateStops)
        .filter(|issue| issue.object_name == Some("Stop Point".to_string()))
        .collect();

    assert_eq!(1, duplicate_stops_issues.len());
    assert_eq!(
        "Stop Point",
        duplicate_stops_issues[0].object_name.as_ref().unwrap()
    );
}

#[test]
fn test_stop_areas() {
    let gtfs = gtfs_structures::Gtfs::new("test_data/duplicate_stops").unwrap();
    let issues = validate(&gtfs);
    let duplicate_stops_issues: Vec<_> = issues
        .iter()
        .filter(|issue| issue.issue_type == IssueType::DuplicateStops)
        .filter(|issue| issue.object_name == Some("Stop Area".to_string()))
        .collect();

    assert_eq!(1, duplicate_stops_issues.len());
    assert_eq!(
        "Stop Area",
        duplicate_stops_issues[0].object_name.as_ref().unwrap()
    );
}

#[test]
fn test_stop_entrances() {
    let gtfs = gtfs_structures::Gtfs::new("test_data/duplicate_stops").unwrap();
    let issues = validate(&gtfs);
    let entrance_issues: Vec<_> = issues
        .iter()
        .filter(|issue| issue.issue_type == IssueType::DuplicateStops)
        .filter(|issue| issue.object_name == Some("Stop Entrance".to_string()))
        .collect();

    assert_eq!(0, entrance_issues.len());
}
