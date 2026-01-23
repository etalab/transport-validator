use crate::issues::{Issue, IssueType, Severity};
use gtfs_structures::LocationType::{StopArea, StopPoint};
use std::collections::HashSet;

pub fn validate(gtfs: &gtfs_structures::Gtfs) -> Vec<Issue> {
    let mut used_stops = HashSet::new();

    // A stop can be used for a stop time
    for trip in gtfs.trips.values() {
        for stop_time in &trip.stop_times {
            used_stops.insert(stop_time.stop.id.to_owned());
        }
    }

    // A stop can be the parent station
    for stop in gtfs.stops.values() {
        if let Some(parent) = &stop.parent_station
            && used_stops.contains(&stop.id)
        {
            used_stops.insert(parent.to_owned());
        }
    }

    gtfs.stops
        .values()
        // We ignore other location types (such as entrances or boarding points)
        .filter(|&stop| stop.location_type == StopPoint || stop.location_type == StopArea)
        .filter(|&stop| !used_stops.contains(&stop.id))
        .map(|stop| make_unused_stop_issue(stop))
        .collect()
}

fn make_unused_stop_issue(stop: &gtfs_structures::Stop) -> Issue {
    Issue::new_with_obj(Severity::Information, IssueType::UnusedStop, stop)
}

#[test]
fn test() {
    let gtfs = gtfs_structures::Gtfs::new("test_data/unused_stop").unwrap();
    let issues = validate(&gtfs);

    assert_eq!(1, issues.len());
    assert_eq!("BOGUS", issues[0].object_id);
}
