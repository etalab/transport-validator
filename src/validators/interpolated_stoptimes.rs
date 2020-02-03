//! To be correclty interpolated, the first and last stop of a trip cannot have undefined
//! departure / arrival
use crate::issues::{Issue, IssueType, Severity};

pub fn validate(gtfs: &gtfs_structures::Gtfs) -> Vec<Issue> {
    gtfs.trips
        .values()
        .filter_map(impossible_to_interpolate_st)
        .collect()
}

fn impossible_to_interpolate_st(trip: &gtfs_structures::Trip) -> Option<Issue> {
    let first_st = trip.stop_times.first();
    let last_st = trip.stop_times.last();

    if let (Some(first_st), Some(last_st)) = (first_st, last_st) {
        if first_st.departure_time.is_none()
            || first_st.arrival_time.is_none()
            || last_st.departure_time.is_none()
            || last_st.arrival_time.is_none()
        {
            Some(
                Issue::new_with_obj(
                    Severity::Warning,
                    IssueType::ImpossibleToInterpolateStopTimes,
                    trip,
                )
                .details("The first and last stop time of a trip cannot have empty departure/arrivals as they cannot be interpolated"),
            )
        } else {
            None
        }
    } else {
        None
    }
}

#[test]
fn test_stop_points() {
    // in the `interpolated_stop_times` GTFS, there are 2 trips
    // Trip 1 has a stop time without departure/arrival, but it's neither the first nor the last, so it's no problem
    // Trip 2 has its first stop time without departure/arrival, so we create and issue
    let gtfs = gtfs_structures::Gtfs::new("test_data/interpolated_stop_times").unwrap();
    let issues = validate(&gtfs);

    assert_eq!(1, issues.len());
    let issue = &issues[0];

    assert_eq!(
        issue.issue_type,
        IssueType::ImpossibleToInterpolateStopTimes
    );
    assert_eq!("trip2", issue.object_id);
}
