use std::collections::HashSet;

use gtfs_structures::Trip;

use crate::issues::Issue;
use crate::{IssueType, Severity};

pub fn validate(gtfs: &gtfs_structures::Gtfs) -> Vec<Issue> {
    gtfs.trips
        .values()
        .filter_map(|trip| {
            let mut stops = HashSet::new();
            for stop_time in &trip.stop_times {
                stops.insert(stop_time.stop.id.to_owned());
                if stops.len() > 1 {
                    break;
                }
            }
            if stops.len() < 2 {
                Some(mk_issue(trip))
            } else {
                None
            }
        })
        .collect()
}

fn mk_issue(trip: &Trip) -> Issue {
    Issue::new_with_obj(Severity::Warning, IssueType::UnusableTrip, trip)
}

#[test]
fn test() {
    let gtfs = gtfs_structures::Gtfs::new("test_data/unusable_trip").unwrap();
    let issues = validate(&gtfs);

    assert_eq!(1, issues.len());
    assert_eq!("AB1", issues[0].object_id);
}
