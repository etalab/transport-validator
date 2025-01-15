use gtfs_structures::Trip;

use crate::issues::Issue;
use crate::{IssueType, Severity};

pub fn validate(gtfs: &gtfs_structures::Gtfs) -> Vec<Issue> {
    gtfs.trips
        .values()
        .filter_map(|trip| {
            if trip.stop_times.len() == 1 {
                Some(mk_issue(trip))
            } else {
                None
            }
        })
        .collect()
}

fn mk_issue(trip: &Trip) -> Issue {
    Issue::new_with_obj(Severity::Error, IssueType::UnusableTrip, trip)
}

#[test]
fn test() {
    let gtfs = gtfs_structures::Gtfs::new("test_data/unusable_trip").unwrap();
    let issues = validate(&gtfs);

    assert_eq!(1, issues.len());
    assert_eq!("AB1", issues[0].object_id);
}
