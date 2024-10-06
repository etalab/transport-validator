use crate::issues::{Issue, IssueType, Severity};
use gtfs_structures::LocationType;
use itertools::Itertools;
use std::collections::HashMap;
use std::iter::once;

/// To limit the size of the issue, we limit, by stops, the number of trip associated to a wrong stop
const MAX_TRIPS: usize = 20;

pub fn validate(gtfs: &gtfs_structures::Gtfs) -> Vec<Issue> {
    check_location_type(gtfs)
        .chain(check_valid_stop_sequence(gtfs))
        .chain(check_times_increase(gtfs))
        .collect()
}

fn check_location_type(gtfs: &gtfs_structures::Gtfs) -> impl Iterator<Item = Issue> + '_ {
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

    wrong_stops.into_values()
}

// All stop_sequence of the stop times of a given trip should be different
fn check_valid_stop_sequence(gtfs: &gtfs_structures::Gtfs) -> impl Iterator<Item = Issue> + '_ {
    gtfs.trips.values().filter_map(|trip| {
        let stop_with_same_sequence = trip
            .stop_times
            .iter()
            .tuple_windows()
            .filter_map(|(s1, s2)| {
                // All stoptimes are sorted by stopsequence (thus the importance of correct stop sequences)
                // so we just need to check the next stoptime
                if s1.stop_sequence == s2.stop_sequence {
                    Some(once(s1).chain(once(s2)))
                } else {
                    None
                }
            })
            .flatten()
            .collect::<Vec<_>>();
        if !stop_with_same_sequence.is_empty() {
            let mut issue =
                Issue::new_with_obj(Severity::Error, IssueType::DuplicateStopSequence, trip);
            for st in stop_with_same_sequence {
                issue.push_related_object(st.stop.as_ref());
            }
            Some(issue)
        } else {
            None
        }
    })
}

fn check_times_increase(gtfs: &gtfs_structures::Gtfs) -> impl Iterator<Item = Issue> + '_ {
    gtfs.trips.values().flat_map(|trip| {
        trip.stop_times
            .iter()
            .filter_map(move |st| match (st.arrival_time, st.departure_time) {
                (Some(arrival), Some(departure)) if arrival > departure => {
                    let issue = Issue::new_with_obj(
                        Severity::Warning,
                        IssueType::NegativeStopDuration,
                        trip,
                    )
                    .details(&format!(
                        "Departure time before arrival time at stop sequence {}",
                        st.stop_sequence
                    ));
                    Some(issue)
                }
                _ => None,
            })
    })
}

#[test]
fn test_location_type() {
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

#[test]
fn test_stop_sequences() {
    let gtfs = gtfs_structures::Gtfs::new("test_data/duplicate_stop_sequence").unwrap();
    let mut issues = dbg!(validate(&gtfs));

    assert_eq!(2, issues.len());

    // 3 trips should be in error
    issues.sort_by_key(|a| a.object_id.to_string());
    let first_issue = &mut issues[0];
    assert_eq!(IssueType::DuplicateStopSequence, first_issue.issue_type);
    assert_eq!("trip2", first_issue.object_id);

    // 2 trips are linked to the stop 'STOP_AREA'
    first_issue.related_objects.sort_by_key(|ro| ro.id.clone());
    assert_eq!(
        vec![
            crate::RelatedObject {
                id: "stopB".to_string(),
                name: Some("Stop B".to_string()),
                object_type: Some(gtfs_structures::ObjectType::Stop)
            },
            crate::RelatedObject {
                id: "stopC".to_string(),
                name: Some("Stop C".to_string()),
                object_type: Some(gtfs_structures::ObjectType::Stop)
            },
        ],
        first_issue.related_objects
    );
    assert_eq!("trip3", &issues[1].object_id);
    assert_eq!(2, issues[1].related_objects.len());
}

#[test]
fn test_times_increase() {
    let gtfs = gtfs_structures::Gtfs::new("test_data/stop_times_increase").unwrap();
    let issues = dbg!(validate(&gtfs));

    assert_eq!(1, issues.len());
    let first_issue = &issues[0];
    assert_eq!(IssueType::NegativeStopDuration, first_issue.issue_type);
    assert_eq!("ITB1", first_issue.object_id);
    assert_eq!(
        Some("Departure time before arrival time at stop sequence 8"),
        first_issue.details.as_deref()
    );
}
