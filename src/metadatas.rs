use crate::issues::IssueType;
use chrono::NaiveDate;
use gtfs_structures::Availability;
use itertools::Itertools;
use rgb::RGB;
use serde::Serialize;
use std::collections::HashMap;

#[derive(Serialize, Debug)]
pub struct Metadata {
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub stops_count: usize,
    pub stop_areas_count: usize,
    pub stop_points_count: usize,
    pub stops_with_wheelchair_info_count: Option<usize>,
    pub lines_count: usize,
    pub trips_count: usize,
    pub trips_with_bike_info_count: usize,
    pub trips_with_wheelchair_info_count: usize,
    pub networks: Vec<String>,
    pub networks_start_end_dates: Option<HashMap<String, Option<(String, String)>>>,
    pub modes: Vec<String>,
    pub issues_count: std::collections::BTreeMap<IssueType, usize>,
    pub has_fares: bool,
    pub has_shapes: bool,
    pub has_pathways: bool,
    pub lines_with_custom_color_count: usize,
    // some stops have a pickup_type or drop_off_type equal to "ArrangeByPhone"
    pub some_stops_need_phone_agency: bool,
    // some stops have a pickup_type or drop_off_type equal to "CoordinateWithDriver"
    pub some_stops_need_phone_driver: bool,
    pub validator_version: String,
}

pub fn extract_metadata(gtfs: &gtfs_structures::RawGtfs) -> Metadata {
    use gtfs_structures::PickupDropOffType;
    use gtfs_structures::RouteType::*;

    let start_end = gtfs
        .calendar
        .as_ref()
        .and_then(|c| c.as_ref().ok())
        .unwrap_or(&vec![])
        .iter()
        .flat_map(|c| vec![c.start_date, c.end_date].into_iter())
        .chain(
            gtfs.calendar_dates
                .as_ref()
                .and_then(|c| c.as_ref().ok())
                .unwrap_or(&vec![])
                .iter()
                .filter(|cd| cd.exception_type == gtfs_structures::Exception::Added)
                .map(|c| c.date),
        )
        .minmax()
        .into_option();
    let format = |d: chrono::NaiveDate| d.format("%Y-%m-%d").to_string();
    let validator_version = env!("CARGO_PKG_VERSION");

    Metadata {
        start_date: start_end.map(|(s, _)| format(s)),
        end_date: start_end.map(|(_, e)| format(e)),
        stops_count: gtfs.stops.as_ref().map_or(0, |stops| stops.len()),
        stop_areas_count: gtfs
            .stops
            .as_ref()
            .unwrap_or(&vec![])
            .iter()
            .filter(|s| s.location_type == gtfs_structures::LocationType::StopArea)
            .count(),
        stop_points_count: gtfs
            .stops
            .as_ref()
            .unwrap_or(&vec![])
            .iter()
            .filter(|s| s.location_type == gtfs_structures::LocationType::StopPoint)
            .count(),
        stops_with_wheelchair_info_count: None,
        lines_count: gtfs.routes.as_ref().map(|r| r.len()).unwrap_or(0),
        trips_count: gtfs.trips.as_ref().map(|t| t.len()).unwrap_or(0),
        trips_with_bike_info_count: trips_with_bike_info_count(gtfs),
        trips_with_wheelchair_info_count: trips_with_wheelchair_info_count(gtfs),
        networks: gtfs
            .agencies
            .as_ref()
            .unwrap_or(&vec![])
            .iter()
            .map(|a| a.name.to_owned())
            .unique()
            .collect(),
        networks_start_end_dates: None,
        modes: gtfs
            .routes
            .as_ref()
            .unwrap_or(&vec![])
            .iter()
            .filter_map(|r| match r.route_type {
                Tramway => Some("tramway".to_owned()),
                Subway => Some("subway".to_owned()),
                Rail => Some("rail".to_owned()),
                Bus => Some("bus".to_owned()),
                Ferry => Some("ferry".to_owned()),
                CableCar => Some("cable_car".to_owned()),
                Gondola => Some("gondola".to_owned()),
                Funicular => Some("funicular".to_owned()),
                Coach => Some("coach".to_owned()),
                Air => Some("air".to_owned()),
                Taxi => Some("taxi".to_owned()),
                Other(_) => None,
            })
            .unique()
            .collect(),
        issues_count: std::collections::BTreeMap::new(),
        has_fares: match &gtfs.fare_attributes {
            Some(Ok(fa)) => !fa.is_empty(),
            _ => false,
        },
        has_shapes: match &gtfs.shapes {
            Some(Ok(s)) => !s.is_empty(),
            _ => false,
        },
        has_pathways: match &gtfs.pathways {
            Some(Ok(p)) => !p.is_empty(),
            _ => false,
        },
        lines_with_custom_color_count: gtfs
            .routes
            .as_ref()
            .unwrap_or(&vec![])
            .iter()
            .filter(|r| {
                let text_default_color = RGB { r: 0, g: 0, b: 0 }; // black
                let route_default_color = RGB {
                    r: 255,
                    g: 255,
                    b: 255,
                }; // white
                r.text_color != text_default_color || r.color != route_default_color
            })
            .count(),
        some_stops_need_phone_agency: gtfs
            .stop_times
            .as_ref()
            .unwrap_or(&vec![])
            .iter()
            .any(|st| has_on_demand_pickup_dropoff(st, PickupDropOffType::ArrangeByPhone)),
        some_stops_need_phone_driver: gtfs
            .stop_times
            .as_ref()
            .unwrap_or(&vec![])
            .iter()
            .any(|st| has_on_demand_pickup_dropoff(st, PickupDropOffType::CoordinateWithDriver)),
        validator_version: validator_version.to_owned(),
    }
}

impl Metadata {
    pub fn enrich_with_advanced_infos(&mut self, gtfs: &gtfs_structures::Gtfs) {
        self.stops_with_wheelchair_info_count = Some(stops_with_wheelchair_info_count(gtfs));
        self.networks_start_end_dates = Some(networks_start_end_dates(gtfs));
    }
}

fn stops_with_wheelchair_info_count(gtfs: &gtfs_structures::Gtfs) -> usize {
    gtfs.stops
        .iter()
        .filter(|(_, stop)| {
            if stop.wheelchair_boarding != gtfs_structures::Availability::InformationNotAvailable {
                // information is present
                return true;
            } else {
                // information not present, check for inheritance from parent station
                let parent_wheelchair_boarding = stop
                    .parent_station
                    .as_ref()
                    .and_then(|parent| gtfs.stops.get(parent))
                    .map(|s| s.wheelchair_boarding);

                // true if parent has information about accessibility
                return parent_wheelchair_boarding == Some(Availability::Available)
                    || parent_wheelchair_boarding == Some(Availability::NotAvailable);
            }
        })
        .count()
}

fn extract_calendar_dates(
    trip: &gtfs_structures::Trip,
    gtfs: &gtfs_structures::Gtfs,
) -> Vec<NaiveDate> {
    let mut from_calendar = gtfs
        .calendar
        .get(&trip.service_id)
        // take both start and end dates
        .map(|c| vec![c.start_date, c.end_date])
        .unwrap_or(vec![]);

    let from_calendar_dates: Vec<NaiveDate> = gtfs
        .calendar_dates
        .get(&trip.service_id)
        .unwrap_or(&vec![])
        .iter()
        // keep only added calendar dates
        .filter(|cd| cd.exception_type == gtfs_structures::Exception::Added)
        .map(|cd| cd.date)
        .collect();

    from_calendar.extend(from_calendar_dates);
    from_calendar
}

fn networks_start_end_dates(
    gtfs: &gtfs_structures::Gtfs,
) -> HashMap<String, Option<(String, String)>> {
    let mut res = HashMap::new();
    let format = |d: chrono::NaiveDate| d.format("%Y-%m-%d").to_string();

    for agency in gtfs.agencies.iter() {
        // for each agency
        let date = gtfs
            .routes
            .values()
            // filter the routes corresponding to this agency
            .filter(|route| route.agency_id == agency.id)
            // take corresponding trips
            .flat_map(|route| {
                gtfs.trips
                    .values()
                    .filter(move |trips| trips.route_id == route.to_owned().id)
            })
            // get all dates in calendar files linked to those trips
            .flat_map(|trip| extract_calendar_dates(trip, gtfs))
            .minmax()
            .into_option()
            .map(|(d1, d2)| (format(d1), format(d2)));

        res.insert(agency.name.to_owned(), date);
    }

    res
}

fn has_on_demand_pickup_dropoff(
    stop_time: &gtfs_structures::RawStopTime,
    pickup_dropoff_type: gtfs_structures::PickupDropOffType,
) -> bool {
    stop_time.pickup_type == pickup_dropoff_type || stop_time.drop_off_type == pickup_dropoff_type
}

fn trips_with_bike_info_count(gtfs: &gtfs_structures::RawGtfs) -> usize {
    gtfs.trips
        .as_ref()
        .unwrap_or(&vec![])
        .iter()
        .filter(|t| t.bikes_allowed != gtfs_structures::BikesAllowedType::NoBikeInfo)
        .count()
}

fn trips_with_wheelchair_info_count(gtfs: &gtfs_structures::RawGtfs) -> usize {
    gtfs.trips
        .as_ref()
        .unwrap_or(&vec![])
        .iter()
        .filter(|t| {
            t.wheelchair_accessible != gtfs_structures::Availability::InformationNotAvailable
        })
        .count()
}

#[test]
fn show_validation_version() {
    let raw_gtfs =
        gtfs_structures::RawGtfs::new("test_data/fare_attributes").expect("Failed to load data");
    let metadatas = extract_metadata(&raw_gtfs);
    assert_eq!(metadatas.validator_version.is_empty(), false);
}

#[test]
fn test_has_fares() {
    let raw_gtfs =
        gtfs_structures::RawGtfs::new("test_data/fare_attributes").expect("Failed to load data");
    let metadatas = extract_metadata(&raw_gtfs);
    assert!(metadatas.has_fares);
}

#[test]
fn test_has_shapes() {
    let raw_gtfs = gtfs_structures::RawGtfs::new("test_data/shapes").expect("Failed to load data");
    let metadatas = extract_metadata(&raw_gtfs);
    assert!(metadatas.has_shapes);
}

#[test]
fn test_no_optional_files() {
    let raw_gtfs =
        gtfs_structures::RawGtfs::new("test_data/no_optional_files").expect("Failed to load data");
    let metadatas = extract_metadata(&raw_gtfs);
    assert!(!metadatas.has_fares);
    assert!(!metadatas.has_shapes);
    assert!(!metadatas.has_pathways);
}

#[test]
fn test_stop_need_phone_agency() {
    let raw_gtfs = gtfs_structures::RawGtfs::new("test_data/arrange_by_phone_stops")
        .expect("Failed to load data");
    let metadatas = extract_metadata(&raw_gtfs);
    assert!(metadatas.some_stops_need_phone_agency);
    assert!(!metadatas.some_stops_need_phone_driver);
}

#[test]
fn test_stop_need_phone_driver() {
    let raw_gtfs = gtfs_structures::RawGtfs::new("test_data/coordinate_with_driver_stops")
        .expect("Failed to load data");
    let metadatas = extract_metadata(&raw_gtfs);
    assert!(!metadatas.some_stops_need_phone_agency);
    assert!(metadatas.some_stops_need_phone_driver);
}

#[test]
fn test_has_pathways() {
    let raw_gtfs = gtfs_structures::RawGtfs::new("test_data/missing_mandatory_files")
        .expect("Failed to load data");
    let metadatas = extract_metadata(&raw_gtfs);
    assert!(metadatas.has_pathways);
}

#[test]
fn test_count_lines_with_custom_color() {
    let raw_gtfs =
        gtfs_structures::RawGtfs::new("test_data/custom_route_color").expect("Failed to load data");
    let metadatas = extract_metadata(&raw_gtfs);
    assert_eq!(3, metadatas.lines_with_custom_color_count);
}

#[test]
fn test_count_trips_with_bike_infos() {
    let raw_gtfs = gtfs_structures::RawGtfs::new("test_data/stops").expect("Failed to load data");
    let metadatas = extract_metadata(&raw_gtfs);
    assert_eq!(11, metadatas.trips_count);
    assert_eq!(3, metadatas.trips_with_bike_info_count);
}

#[test]
fn test_count_trips_with_accessibility_infos() {
    let raw_gtfs = gtfs_structures::RawGtfs::new("test_data/stops").expect("Failed to load data");
    let metadatas = extract_metadata(&raw_gtfs);
    assert_eq!(11, metadatas.trips_count);
    assert_eq!(3, metadatas.trips_with_wheelchair_info_count);
}

#[test]
fn test_count_stops_with_accessibility_infos() {
    let raw_gtfs =
        gtfs_structures::RawGtfs::new("test_data/accessibility").expect("Failed to load data");
    let mut metadatas = extract_metadata(&raw_gtfs);
    let gtfs = gtfs_structures::Gtfs::try_from(raw_gtfs).expect("Failed to load GTFS");

    assert_eq!(16, metadatas.stops_count);
    assert_eq!(None, metadatas.stops_with_wheelchair_info_count);

    metadatas.enrich_with_advanced_infos(&gtfs);

    assert_eq!(Some(10), metadatas.stops_with_wheelchair_info_count);
}
