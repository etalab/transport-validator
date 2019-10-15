use crate::validators::issues::{Issue, IssueType, Severity};
use geo::algorithm::haversine_distance::HaversineDistance;
use gtfs_structures::RouteType::*;
use itertools::Itertools;

fn distance_and_duration(
    departure: &gtfs_structures::StopTime,
    arrival: &gtfs_structures::StopTime,
) -> (f64, f64) {
    let dep_stop = &departure.stop;
    let arr_stop = &arrival.stop;

    let dep_point = geo::Point::new(dep_stop.longitude, dep_stop.latitude);
    let arr_point = geo::Point::new(arr_stop.longitude, arr_stop.latitude);

    let duration = f64::from(arrival.arrival_time) - f64::from(departure.departure_time);
    let distance = dep_point.haversine_distance(&arr_point);

    (distance, duration)
}

fn max_speed(route_type: gtfs_structures::RouteType) -> f64 {
    // Speeds are in km/h for convenience
    (match route_type {
        Tramway => 100.0,
        Subway => 140.0,
        Rail => 300.0,
        Bus => 120.0,
        Ferry => 90.0, // https://en.wikipedia.org/wiki/List_of_HSC_ferry_routes
        CableCar => 30.0,
        Gondola => 45.0, // https://fr.wikipedia.org/wiki/Vanoise_Express
        Funicular => 40.0,
        Other(_) => 120.0, // We suppose it’s a bus if it is invalid
    }) / 3.6 // convert in m/s
}

fn validate_speeds(
    gtfs: &gtfs_structures::Gtfs,
) -> Result<Vec<Issue>, gtfs_structures::ReferenceError> {
    let mut issues_by_stops_and_type = std::collections::HashMap::new();

    for trip in gtfs.trips.values() {
        let route = gtfs.get_route(&trip.route_id)?;
        for (departure, arrival) in trip.stop_times.iter().tuple_windows() {
            let (distance, duration) = distance_and_duration(departure, arrival);

            let issue_kind = if distance < 10.0 {
                Some((Severity::Information, IssueType::CloseStops))
            // Some timetable are rounded to the minute. For short distances this can result in a null duration
            // If stops are more than 500m appart, they should need at least a minute
            } else if duration == 0.0 && distance > 500.0 {
                Some((Severity::Warning, IssueType::NullDuration))
            } else if duration > 0.0 && distance / duration > max_speed(route.route_type) {
                Some((Severity::Information, IssueType::ExcessiveSpeed))
            } else if duration < 0.0 {
                Some((Severity::Warning, IssueType::NegativeTravelTime))
            } else if distance / duration < 0.1 {
                Some((Severity::Information, IssueType::Slow))
            } else {
                None
            };

            // we want to limit the number of duplicate, we we don't want an issue for all the trip between A&B
            // we group all the issue by stops (and issue type)
            if let Some((severity, issue_type)) = issue_kind {
                // it's a bit of a trick, if we have an issue between A&B, we don't want a duplicate issue between B&A
                let key = if departure.stop.id < arrival.stop.id {
                    (
                        departure.stop.id.clone(),
                        arrival.stop.id.clone(),
                        issue_type,
                        severity,
                    )
                } else {
                    (
                        arrival.stop.id.clone(),
                        departure.stop.id.clone(),
                        issue_type,
                        severity,
                    )
                };

                let issue = issues_by_stops_and_type.entry(key).or_insert_with(|| {
                    Issue::new_with_obj(severity, issue_type, &*departure.stop)
                        .add_related_object(&*arrival.stop)
                });
                issue.push_related_object(trip);
            }
        }
    }

    Ok(issues_by_stops_and_type
        .into_iter()
        .map(|(_k, v)| v)
        .collect())
}

pub fn validate(gtfs: &gtfs_structures::Gtfs) -> Vec<Issue> {
    validate_speeds(gtfs).unwrap_or_else(|err| {
        vec![Issue::new(
            Severity::Fatal,
            IssueType::InvalidReference,
            &err.id,
        )]
    })
}

#[test]
fn test() {
    let gtfs = gtfs_structures::Gtfs::new("test_data/duration_distance").unwrap();
    let mut issues = validate(&gtfs);
    issues.sort_by(|a, b| a.issue_type.cmp(&b.issue_type));

    assert_eq!(5, issues.len());

    assert_eq!(IssueType::Slow, issues[0].issue_type);
    assert_eq!("near1", issues[0].object_id);
    assert_eq!(String::from("near2"), issues[0].related_objects[0].id);
    assert_eq!(Some(String::from("Near1")), issues[0].object_name);

    assert_eq!(IssueType::ExcessiveSpeed, issues[1].issue_type);
    assert_eq!("near1", issues[1].object_id);
    assert_eq!(String::from("null"), issues[1].related_objects[0].id);
    assert_eq!(Some(String::from("Near1")), issues[1].object_name);

    assert_eq!(IssueType::NegativeTravelTime, issues[2].issue_type);
    assert_eq!("near1", issues[2].object_id);
    assert_eq!(String::from("near2"), issues[2].related_objects[0].id);
    assert_eq!(Some(String::from("Near1")), issues[2].object_name);

    assert_eq!(IssueType::CloseStops, issues[3].issue_type);
    assert_eq!("close1", issues[3].object_id);
    assert_eq!(String::from("close2"), issues[3].related_objects[0].id);
    assert_eq!(Some(String::from("Close 1")), issues[3].object_name);

    assert_eq!(IssueType::NullDuration, issues[4].issue_type);
    assert_eq!("near1", issues[4].object_id);
    assert_eq!(String::from("null"), issues[4].related_objects[0].id);
    assert_eq!(Some(String::from("Near1")), issues[4].object_name);
}
