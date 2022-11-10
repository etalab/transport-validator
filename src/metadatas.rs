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
    pub networks_start_end_dates: Option<HashMap<String, Option<HashMap<String, String>>>>,
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
        self.networks_start_end_dates = Some(networks_start_end_dates(self, gtfs));
    }
}

fn stops_with_wheelchair_info_count(gtfs: &gtfs_structures::Gtfs) -> usize {
    gtfs.stops
        .iter()
        .filter(|(_, stop)| {
            if stop.wheelchair_boarding != gtfs_structures::Availability::InformationNotAvailable {
                // information is present
                true
            } else {
                // information not present, check for inheritance from parent station
                let parent_wheelchair_boarding = stop
                    .parent_station
                    .as_ref()
                    .and_then(|parent| gtfs.stops.get(parent))
                    .map(|s| s.wheelchair_boarding);

                // true if parent has information about accessibility
                parent_wheelchair_boarding == Some(Availability::Available)
                    || parent_wheelchair_boarding == Some(Availability::NotAvailable)
            }
        })
        .count()
}

#[derive(Clone, Copy)]
struct Interval(chrono::NaiveDate, chrono::NaiveDate);

impl Interval {
    fn update_bounds(&mut self, other: &Interval) {
        self.update_bounds_with_date(&other.0);
        self.update_bounds_with_date(&other.1);
    }
    fn update_bounds_with_date(&mut self, d: &NaiveDate) {
        if self.0 > *d {
            self.0 = *d;
        }
        if self.1 < *d {
            self.1 = *d;
        }
    }
}

fn compute_services_start_end_dates(gtfs: &gtfs_structures::Gtfs) -> HashMap<&str, Interval> {
    let mut res: HashMap<&str, Interval> = gtfs
        .calendar
        .iter()
        .map(|(id, c)| (id.as_str(), Interval(c.start_date, c.end_date)))
        .collect();
    for (calendar_id, calendar_dates) in &gtfs.calendar_dates {
        for d in calendar_dates
            .iter()
            .filter(|cd| cd.exception_type == gtfs_structures::Exception::Added)
        {
            res.entry(calendar_id)
                .and_modify(|i| i.update_bounds_with_date(&d.date))
                .or_insert(Interval(d.date, d.date));
        }
    }
    res
}

fn networks_start_end_dates(
    metadata: &Metadata,
    gtfs: &gtfs_structures::Gtfs,
) -> HashMap<String, Option<HashMap<String, String>>> {
    let format = |d: chrono::NaiveDate| d.format("%Y-%m-%d").to_string();
    let result: HashMap<String, Option<HashMap<String, String>>> = if metadata.networks.len() == 1 {
        // if there is only one agency, get data from existing metadata
        let mut agency_start_end_dates = HashMap::default();
        let start_end = match (metadata.start_date.as_ref(), metadata.end_date.as_ref()) {
            (Some(sd), Some(ed)) => Some(HashMap::from([
                (String::from("start_date"), sd.to_owned()),
                (String::from("end_date"), ed.to_owned()),
            ])),
            _ => None,
        };
        agency_start_end_dates.insert(metadata.networks[0].to_owned(), start_end);
        agency_start_end_dates
    } else {
        // multi-agency case
        let mut agencies_start_end_dates: HashMap<Option<String>, Interval> = HashMap::default();
        let services_start_end_dates = compute_services_start_end_dates(gtfs);

        for (agency_id, service_id) in gtfs.trips.iter().filter_map(|(_trip_id, trip)| {
            gtfs.get_route(&trip.route_id)
                .map(|route| (route.agency_id.clone(), trip.service_id.as_str()))
                .ok()
        }) {
            if let Some(service_bounds) = services_start_end_dates.get(&service_id) {
                agencies_start_end_dates
                    .entry(agency_id)
                    .and_modify(|i| i.update_bounds(service_bounds))
                    .or_insert(service_bounds.clone());
            }
        }

        agencies_start_end_dates
            .into_iter()
            .map(|(id, i)| {
                (
                    gtfs.agencies
                        .iter()
                        .find(|a| &a.id == &id)
                        .map(|a| a.name.clone())
                        .unwrap_or("default_agency".to_string()),
                    Some(HashMap::from([
                        (String::from("start_date"), format(i.0)),
                        (String::from("end_date"), format(i.1)),
                    ])),
                )
            })
            .collect()
    };

    result
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::convert::TryFrom;

    #[test]
    fn show_validation_version() {
        let raw_gtfs = gtfs_structures::RawGtfs::new("test_data/fare_attributes")
            .expect("Failed to load data");
        let metadatas = extract_metadata(&raw_gtfs);
        assert!(!metadatas.validator_version.is_empty());
    }

    #[test]
    fn test_has_fares() {
        let raw_gtfs = gtfs_structures::RawGtfs::new("test_data/fare_attributes")
            .expect("Failed to load data");
        let metadatas = extract_metadata(&raw_gtfs);
        assert!(metadatas.has_fares);
    }

    #[test]
    fn test_has_shapes() {
        let raw_gtfs =
            gtfs_structures::RawGtfs::new("test_data/shapes").expect("Failed to load data");
        let metadatas = extract_metadata(&raw_gtfs);
        assert!(metadatas.has_shapes);
    }

    #[test]
    fn test_no_optional_files() {
        let raw_gtfs = gtfs_structures::RawGtfs::new("test_data/no_optional_files")
            .expect("Failed to load data");
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
        let raw_gtfs = gtfs_structures::RawGtfs::new("test_data/custom_route_color")
            .expect("Failed to load data");
        let metadatas = extract_metadata(&raw_gtfs);
        assert_eq!(3, metadatas.lines_with_custom_color_count);
    }

    #[test]
    fn test_count_trips_with_bike_infos() {
        let raw_gtfs =
            gtfs_structures::RawGtfs::new("test_data/stops").expect("Failed to load data");
        let metadatas = extract_metadata(&raw_gtfs);
        assert_eq!(11, metadatas.trips_count);
        assert_eq!(3, metadatas.trips_with_bike_info_count);
    }

    #[test]
    fn test_count_trips_with_accessibility_infos() {
        let raw_gtfs =
            gtfs_structures::RawGtfs::new("test_data/stops").expect("Failed to load data");
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
}

#[test]
fn test_network_start_end_dates() {
    use std::convert::TryFrom;

    let raw_gtfs =
        gtfs_structures::RawGtfs::new("test_data/agency_single").expect("Failed to load data");
    let mut metadatas = extract_metadata(&raw_gtfs);
    let gtfs = gtfs_structures::Gtfs::try_from(raw_gtfs).expect("Failed to load GTFS");
    metadatas.enrich_with_advanced_infos(&gtfs);

    let networks_start_end_dates = metadatas.networks_start_end_dates.unwrap();

    assert_eq!(1, networks_start_end_dates.len());

    let start_end = networks_start_end_dates
        .get("BIBUS")
        .unwrap()
        .as_ref()
        .unwrap();
    assert_eq!("2017-01-01", start_end.get("start_date").unwrap());
    assert_eq!("2017-01-15", start_end.get("end_date").unwrap());

    assert_eq!(
        metadatas.start_date.unwrap(),
        start_end.get("start_date").unwrap().to_owned()
    );
    assert_eq!(
        metadatas.end_date.unwrap(),
        start_end.get("end_date").unwrap().to_owned()
    );
}

#[test]
fn test_networks_start_end_dates() {
    use std::convert::TryFrom;

    let raw_gtfs =
        gtfs_structures::RawGtfs::new("test_data/agency_multiple").expect("Failed to load data");
    let mut metadatas = extract_metadata(&raw_gtfs);
    let gtfs = gtfs_structures::Gtfs::try_from(raw_gtfs).expect("Failed to load GTFS");

    assert_eq!(None, metadatas.networks_start_end_dates);

    metadatas.enrich_with_advanced_infos(&gtfs);

    let networks_start_end_dates = metadatas.networks_start_end_dates.unwrap();

    assert_eq!(2, networks_start_end_dates.len());

    let start_end = networks_start_end_dates
        .get("Ter")
        .unwrap()
        .as_ref()
        .unwrap();
    assert_eq!("2019-01-01", start_end.get("start_date").unwrap());
    assert_eq!("2022-01-01", start_end.get("end_date").unwrap());

    let start_end = networks_start_end_dates
        .get("BIBUS")
        .unwrap()
        .as_ref()
        .unwrap();
    assert_eq!("2016-01-01", start_end.get("start_date").unwrap());
    assert_eq!("2023-01-01", start_end.get("end_date").unwrap());
}
