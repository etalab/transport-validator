use crate::custom_rules;
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

fn max_speed(
    route_type: gtfs_structures::RouteType,
    custom_rules: &custom_rules::CustomRules,
) -> f64 {
    // Speeds are in km/h for convenience
    (match route_type {
        Tramway => custom_rules.max_tramway_speed.unwrap_or(100.0),
        Subway => custom_rules.max_subway_speed.unwrap_or(140.0),
        Rail => custom_rules.max_rail_speed.unwrap_or(320.0),
        Bus => custom_rules.max_bus_speed.unwrap_or(120.0),
        Ferry => custom_rules.max_ferry_speed.unwrap_or(90.0), // https://en.wikipedia.org/wiki/List_of_HSC_ferry_routes
        CableCar => custom_rules.max_cable_car_speed.unwrap_or(30.0),
        Gondola => custom_rules.max_gondola_speed.unwrap_or(45.0), // https://fr.wikipedia.org/wiki/Vanoise_Express
        Funicular => custom_rules.max_funicular_speed.unwrap_or(40.0),
        Coach => custom_rules.max_coach_speed.unwrap_or(120.0),
        Air => custom_rules.max_air_speed.unwrap_or(1_000.0),
        Taxi => custom_rules.max_taxi_speed.unwrap_or(50.0),
        Other(_) => custom_rules.max_other_speed.unwrap_or(120.0), // We suppose it’s a bus if it is invalid
    }) / 3.6 // convert in m/s
}

fn validate_speeds(
    gtfs: &gtfs_structures::Gtfs,
    custom_rules: &custom_rules::CustomRules,
) -> Result<Vec<Issue>, gtfs_structures::Error> {
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
                } else if duration > 0.0
                    && distance / duration > max_speed(route.route_type, custom_rules)
                {
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
                    let any_related = issue.related_objects.iter().any(|i| {
                        (i.id == route.id)
                            && (i.object_type == Some(gtfs_structures::ObjectType::Route))
                    });
                    if !any_related {
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

pub fn validate(
    gtfs: &gtfs_structures::Gtfs,
    custom_rules: &custom_rules::CustomRules,
) -> Vec<Issue> {
    validate_speeds(gtfs, custom_rules).unwrap_or_else(|e| {
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

    let custom_rules = custom_rules::CustomRules {
        ..Default::default()
    };

    let mut issues = validate(&gtfs, &custom_rules);
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

    let custom_rules = custom_rules::CustomRules {
        max_bus_speed: Some(1_000_000.0),
        ..Default::default()
    };
    let issues = validate(&gtfs, &custom_rules);
    assert_eq!(
        0,
        issues
            .iter()
            .filter(|issue| issue.issue_type == IssueType::ExcessiveSpeed)
            .count()
    );
}

#[test]
fn test_optimisation_route_trips() {
    use std::collections::BTreeSet;
    let gtfs = gtfs_structures::Gtfs::new("test_data/optimisation_route_trips").unwrap();
    let custom_rules = custom_rules::CustomRules {
        ..Default::default()
    };

    let mut issues = validate(&gtfs, &custom_rules);

    assert_eq!(1, issues.len());
    // irrelevant to the test, but this acts as a guard in case someone modifies the fixtures
    assert_eq!(IssueType::CloseStops, issues[0].issue_type);

    // the routes order (for objects with index 1 and 2) is apparently non deterministic, for some
    // reason, so we sort the array to get a stable order and avoid random test failures
    issues[0].related_objects.sort_by(|a, b| a.id.cmp(&b.id));

    assert_eq!(3, issues[0].related_objects.len());

    // we would normally find N trips here, but we optimised the payload by
    // referring only to the parent route, and making sure each route appears only once.
    let ids: BTreeSet<_> = issues[0]
        .related_objects
        .iter()
        .map(|r| r.id.as_str())
        .collect();
    assert_eq!(ids, BTreeSet::from(["stop002", "route2", "route1"]));
}
