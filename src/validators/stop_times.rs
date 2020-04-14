use crate::issues::{Issue, IssueType, Severity};
use gtfs_structures::LocationType;
use std::collections::HashMap;

/// To limit the size of the issue, we limit, by stops, the number of trip associated to a wrong stop
const MAX_TRIPS: usize = 20;

pub fn validate(gtfs: &gtfs_structures::Gtfs) -> Vec<Issue> {
    let mut wrong_stops = HashMap::new();
    gtfs.trips.values().for_each(|trip| {
        trip.stop_times.iter().for_each(|st| {
            if st.stop.location_type != LocationType::StopPoint {
                let issue = wrong_stops.entry(st.stop.id.clone()).or_insert_with(|| {
                    Issue::new_with_obj(
                        Severity::Warning,
                        IssueType::InvalidStopLocationTypeInTrip,
                        &*st.stop,
                    )
                    .details(&format!(
                        "A {:?} cannot be referenced by a stop time",
                        st.stop.location_type
                    ))
                });

                if issue.related_objects.len() < MAX_TRIPS {
                    // we do not add more than 20 trip as related object
                    issue.push_related_object(trip);
                }
            }
        })
    });

    // dbg!(wrong_stops);

    wrong_stops.into_iter().map(|(_, v)| v).collect()
}

#[test]
fn test() {
    let gtfs = gtfs_structures::Gtfs::new("test_data/stop_times_location_type").unwrap();
    let issues = dbg!(validate(&gtfs));

    assert_eq!(1, issues.len());
    let first_issue = &issues[0];
    assert_eq!(
        IssueType::InvalidStopLocationTypeInTrip,
        first_issue.issue_type
    );
    assert_eq!("STOP_AREA", first_issue.object_id);
    // 2 trips are linked to the stop 'STOP_AREA'
    assert_eq!(2, first_issue.related_objects.len());
}
