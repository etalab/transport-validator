extern crate failure;
extern crate geo;
extern crate gtfs_structures;
extern crate itertools;

use self::geo::algorithm::haversine_distance::HaversineDistance;
use self::itertools::Itertools;
use gtfs_structures::RouteType::*;
use validators::issues::*;

fn distance_and_duration(
    departure: &gtfs_structures::StopTime,
    arrival: &gtfs_structures::StopTime,
    gtfs: &gtfs_structures::Gtfs,
) -> Result<(f64, f64), gtfs_structures::ReferenceError> {
    let dep_stop = gtfs.get_stop(&departure.stop.id)?;
    let arr_stop = gtfs.get_stop(&arrival.stop.id)?;

    let dep_point = geo::Point::new(dep_stop.longitude, dep_stop.latitude);
    let arr_point = geo::Point::new(arr_stop.longitude, arr_stop.latitude);

    let duration = arrival.arrival_time as f64 - departure.departure_time as f64;
    let distance = dep_point.haversine_distance(&arr_point);

    Ok((distance, duration))
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
    }) / 3.6 // convert in m/s
}

fn validate_speeds(
    gtfs: &gtfs_structures::Gtfs,
) -> Result<Vec<Issue>, gtfs_structures::ReferenceError> {
    let mut result = Vec::new();

    for (trip_id, trip) in &gtfs.trips {
        let route = gtfs.get_route(&trip.route_id)?;
        for (departure, arrival) in trip.stop_times.iter().tuple_windows() {
            let (distance, duration) = distance_and_duration(departure, arrival, gtfs)?;

            if distance < 10.0 {
                result.push(Issue {
                    severity: Severity::Warning,
                    issue_type: IssueType::CloseStops,
                    object_id: departure.stop.id.to_owned(),
                    object_name: Some(format!("Trip: {}", trip_id)),
                    related_object_id: Some(arrival.stop.id.to_owned()),
                })
            // Some timetable are rounded to the minute. For short distances this can result in a null duration
            // If stops are more than 500m appart, they should need at least a minute
            } else if duration == 0.0 && distance > 500.0 {
                result.push(Issue {
                    severity: Severity::Warning,
                    issue_type: IssueType::NullDuration,
                    object_id: departure.stop.id.to_owned(),
                    object_name: Some(format!("Trip: {}", trip_id)),
                    related_object_id: Some(arrival.stop.id.to_owned()),
                })
            } else if duration > 0.0 && distance / duration > max_speed(route.route_type) {
                result.push(Issue {
                    severity: Severity::Warning,
                    issue_type: IssueType::ExcessiveSpeed,
                    object_id: departure.stop.id.to_owned(),
                    object_name: Some(format!("Trip: {}", trip_id)),
                    related_object_id: Some(arrival.stop.id.to_owned()),
                })
            } else if duration < 0.0 {
                result.push(Issue {
                    severity: Severity::Error,
                    issue_type: IssueType::NegativeTravelTime,
                    object_id: departure.stop.id.to_owned(),
                    object_name: Some(format!("Trip: {}", trip_id)),
                    related_object_id: Some(arrival.stop.id.to_owned()),
                })
            } else if distance / duration < 0.1 {
                result.push(Issue {
                    severity: Severity::Warning,
                    issue_type: IssueType::Slow,
                    object_id: departure.stop.id.to_owned(),
                    object_name: Some(format!("Trip: {}", trip_id)),
                    related_object_id: Some(arrival.stop.id.to_owned()),
                })
            }
        }
    }

    Ok(result)
}

pub fn validate(gtfs: &gtfs_structures::Gtfs) -> Vec<Issue> {
    validate_speeds(gtfs).unwrap_or_else(|err| {
        vec![Issue {
            severity: Severity::Fatal,
            issue_type: IssueType::InvalidReference,
            object_id: err.id,
            object_name: None,
            related_object_id: None,
        }]
    })
}

#[test]
fn test() {
    let gtfs = gtfs_structures::Gtfs::new("test_data/duration_distance").unwrap();
    let mut issues = validate(&gtfs);
    issues.sort_by(|a, b| a.object_name.cmp(&b.object_name));

    assert_eq!(5, issues.len());

    assert_eq!(IssueType::CloseStops, issues[0].issue_type);
    assert_eq!("close1", issues[0].object_id);
    assert_eq!(Some(String::from("close2")), issues[0].related_object_id);
    assert_eq!(
        Some(String::from("Trip: close_stops")),
        issues[0].object_name
    );

    assert_eq!(IssueType::NullDuration, issues[1].issue_type);
    assert_eq!("near1", issues[1].object_id);
    assert_eq!(Some(String::from("null")), issues[1].related_object_id);
    assert_eq!(
        Some(String::from("Trip: null_duration")),
        issues[1].object_name
    );

    assert_eq!(IssueType::NegativeTravelTime, issues[2].issue_type);
    assert_eq!("near1", issues[2].object_id);
    assert_eq!(Some(String::from("near2")), issues[2].related_object_id);
    assert_eq!(
        Some(String::from("Trip: time_traveler")),
        issues[2].object_name
    );

    assert_eq!(IssueType::ExcessiveSpeed, issues[3].issue_type);
    assert_eq!("near1", issues[3].object_id);
    assert_eq!(Some(String::from("null")), issues[3].related_object_id);
    assert_eq!(Some(String::from("Trip: too_fast")), issues[3].object_name);

    assert_eq!(IssueType::Slow, issues[4].issue_type);
    assert_eq!("near1", issues[4].object_id);
    assert_eq!(Some(String::from("near2")), issues[4].related_object_id);
    assert_eq!(Some(String::from("Trip: too_slow")), issues[4].object_name);
}
