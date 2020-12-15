use crate::issues::{Issue, IssueType, Severity};
use geo::algorithm::haversine_distance::HaversineDistance;
use gtfs_structures::RouteType::*;
use itertools::Itertools;

fn distance_and_duration(
    departure: &gtfs_structures::StopTime,
    arrival: &gtfs_structures::StopTime,
) -> Option<(f64, f64)> {
    match (
        arrival.arrival_time,
        departure.departure_time,
        departure.stop.longitude,
        departure.stop.latitude,
        arrival.stop.longitude,
        arrival.stop.latitude,
    ) {
        (Some(arrival), Some(departure), Some(d_lon), Some(d_lat), Some(a_lon), Some(a_lat)) => {
            let dep_point = geo::Point::new(d_lon, d_lat);
            let arr_point = geo::Point::new(a_lon, a_lat);
            let duration = f64::from(arrival) - f64::from(departure);
            let distance = dep_point.haversine_distance(&arr_point);

            Some((distance, duration))
        }
        _ => None,
    }
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
        Coach => 120.0,
        Air => 1_000.0,
        Taxi => 50.0,
        Other(_) => 120.0, // We suppose it’s a bus if it is invalid
    }) / 3.6 // convert in m/s
}

fn validate_speeds(gtfs: &gtfs_structures::Gtfs) -> Result<Vec<Issue>, gtfs_structures::Error> {
    let mut issues_by_stops_and_type = std::collections::HashMap::new();

    for trip in gtfs.trips.values() {
        let route = gtfs.get_route(&trip.route_id)?;
        for (departure, arrival) in trip.stop_times.iter().tuple_windows() {
            if let Some((distance, duration)) = distance_and_duration(departure, arrival) {
                let issue_kind = if distance < 10.0 {
                    Some((
                        Severity::Information,
                        IssueType::CloseStops,
                        format!("distance between the stops is {:.0} meter(s)", distance),
                    ))
                // Some timetable are rounded to the minute. For short distances this can result in a null duration
                // If stops are more than 500m appart, they should need at least a minute
                } else if duration == 0.0 && distance > 500.0 {
                    Some((
                        Severity::Warning,
                        IssueType::NullDuration,
                        format!(
                            "travel duration is null, but there are {:.0} meters between the stops",
                            distance
                        ),
                    ))
                } else if duration > 0.0 && distance / duration > max_speed(route.route_type) {
                    Some((
                        Severity::Information,
                        IssueType::ExcessiveSpeed,
                        format!(
                            "computed speed between the stops is {:.2} km/h ({:.0} m travelled in {:.0} seconds)",
                            distance / duration * 3.6,
                            distance,
                            duration
                        ),
                    ))
                } else if duration < 0.0 {
                    Some((
                        Severity::Warning,
                        IssueType::NegativeTravelTime,
                        format!("duration is {} seconds", duration),
                    ))
                } else if distance / duration < 0.1 {
                    Some((
                        Severity::Information,
                        IssueType::Slow,
                        format!(
                            "computed speed between the stops is {:.2} km/h ({:.0} m travelled in {:.0} seconds)",
                            distance / duration * 3.6,
                            distance,
                            duration
                        ),
                    ))
                } else {
                    None
                };

                // we want to limit the number of duplicate, we we don't want an issue for all the trip between A&B
                // we group all the issue by stops (and issue type)
                if let Some((severity, issue_type, details)) = issue_kind {
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
                            .details(&details)
                    });

                    // In the past, we added each individual "trip" here, but it led to overly large
                    // payloads due to the cardinality of trips (https://github.com/etalab/transport-validator/issues/101).
                    // We now just refer to the corresponding route, and make sure the route is only added once.
                    //
                    // Because the number of routes is usually low on tested datasets, we just search if the route is already
                    // there. Alternatively we could move to using a "set" here to optimize search time, if needed.
                    if !issue.related_objects.iter().any(|i| {
                        (i.id == route.id)
                            && (i.object_type.as_ref().unwrap()
                                == &gtfs_structures::ObjectType::Route)
                    }) {
                        issue.push_related_object(route);
                    }
                }
            }
        }
    }

    Ok(issues_by_stops_and_type
        .into_iter()
        .map(|(_k, v)| v)
        .collect())
}

pub fn validate(gtfs: &gtfs_structures::Gtfs) -> Vec<Issue> {
    validate_speeds(gtfs).unwrap_or_else(|e| {
        vec![Issue::new(
            Severity::Fatal,
            IssueType::InvalidReference,
            &format!("{}", e),
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

#[test]
fn test_optimisation_route_trips() {
    let gtfs = gtfs_structures::Gtfs::new("test_data/optimisation_route_trips").unwrap();
    let issues = validate(&gtfs);

    assert_eq!(1, issues.len());
    assert_eq!(IssueType::CloseStops, issues[0].issue_type);

    assert_eq!(3, issues[0].related_objects.len());
    assert_eq!(
        issues[0].related_objects[0].object_type,
        Some(gtfs_structures::ObjectType::Stop)
    );
    // we would normally refer to N trips here, but we optimised the payload by
    // referring only to the parent route, and making sure the route appears only once.
    assert_eq!(String::from("route1"), issues[0].related_objects[1].id);
    assert_eq!(
        issues[0].related_objects[1].object_type,
        Some(gtfs_structures::ObjectType::Route)
    );
    // if multiple routes are involved, each will appear once
    assert_eq!(String::from("route2"), issues[0].related_objects[2].id);
    assert_eq!(
        issues[0].related_objects[2].object_type,
        Some(gtfs_structures::ObjectType::Route)
    );
}
