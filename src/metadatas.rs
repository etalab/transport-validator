use crate::issues::IssueType;
use chrono::NaiveDate;
use gtfs_structures::{Availability, Error};
use itertools::Itertools;
use rgb::RGB;
use serde::Serialize;
use std::collections::HashMap;

#[derive(Serialize, Debug)]
pub struct Metadata {
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub feed_contact_emails: HashMap<String, String>,
    pub feed_start_dates: HashMap<String, String>,
    pub feed_end_dates: HashMap<String, String>,
    pub networks: Vec<String>,
    pub networks_start_end_dates: Option<HashMap<String, Option<Interval>>>,
    pub modes: Vec<String>,
    pub issues_count: std::collections::BTreeMap<IssueType, usize>,
    pub has_fares: bool,
    pub has_shapes: bool,
    pub has_pathways: bool,
    // some stops have a pickup_type or drop_off_type equal to "ArrangeByPhone"
    pub some_stops_need_phone_agency: bool,
    // some stops have a pickup_type or drop_off_type equal to "CoordinateWithDriver"
    pub some_stops_need_phone_driver: bool,
    pub validator_version: String,
    pub stops_count: usize,

    pub stats: Stats,
}

#[derive(Serialize, Debug)]
pub struct Stats {
    pub stops_count: usize,
    pub stop_areas_count: usize,
    pub stop_points_count: usize,
    pub stops_with_wheelchair_info_count: Option<usize>,

    pub routes_count: usize,
    pub routes_with_custom_color_count: usize,
    pub routes_with_short_name_count: usize,
    pub routes_with_long_name_count: usize,

    pub trips_count: usize,
    pub trips_with_bike_info_count: usize,
    pub trips_with_wheelchair_info_count: usize,
    pub trips_with_shape_count: usize,
    pub trips_with_trip_headsign_count: usize,

    pub transfers_count: usize,
    pub fares_attribute_count: usize,
    pub fares_rules_count: usize,
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
    let stats = compute_stats(gtfs);
    let default = vec![];
    let feed_info = gtfs
        .feed_info
        .as_ref()
        .and_then(|f| f.as_ref().ok())
        .unwrap_or(&default);

    Metadata {
        start_date: start_end.map(|(s, _)| format(s)),
        end_date: start_end.map(|(_, e)| format(e)),
        feed_contact_emails: feed_info
            .iter()
            .filter(|f| f.contact_email.is_some())
            .map(|f| (f.name.to_owned(), f.contact_email.to_owned().unwrap()))
            .collect(),
        feed_start_dates: feed_info
            .iter()
            .filter(|f| f.start_date.is_some())
            .map(|f| (f.name.to_owned(), format(f.start_date.unwrap())))
            .collect(),
        feed_end_dates: feed_info
            .iter()
            .filter(|f| f.end_date.is_some())
            .map(|f| (f.name.to_owned(), format(f.end_date.unwrap())))
            .collect(),
        stops_count: stats.stops_count,
        stats,
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
        self.stats.stops_with_wheelchair_info_count = Some(stops_with_wheelchair_info_count(gtfs));
        self.networks_start_end_dates = Some(networks_start_end_dates(self, gtfs));
    }
}

pub fn compute_stats(gtfs: &gtfs_structures::RawGtfs) -> Stats {
    Stats {
        stops_count: gtfs.stops.as_ref().map_or(0, |stops| stops.len()),
        stop_areas_count: counts_objects(&gtfs.stops, |s| {
            s.location_type == gtfs_structures::LocationType::StopArea
        }),
        stop_points_count: counts_objects(&gtfs.stops, |s| {
            s.location_type == gtfs_structures::LocationType::StopPoint
        }),
        stops_with_wheelchair_info_count: None,

        routes_count: gtfs.routes.as_ref().map(|r| r.len()).unwrap_or(0),
        routes_with_custom_color_count: counts_objects(&gtfs.routes, |r| {
            let text_default_color = RGB { r: 0, g: 0, b: 0 }; // black
            let route_default_color = RGB {
                r: 255,
                g: 255,
                b: 255,
            }; // white
            r.text_color != text_default_color || r.color != route_default_color
        }),
        routes_with_long_name_count: counts_objects(&gtfs.routes, |r| {
            !r.long_name.as_ref().map(|n| n.is_empty()).unwrap_or(true)
        }),
        routes_with_short_name_count: counts_objects(&gtfs.routes, |r| {
            !r.short_name.as_ref().map(|n| n.is_empty()).unwrap_or(true)
        }),

        trips_count: gtfs.trips.as_ref().map(|t| t.len()).unwrap_or(0),
        trips_with_bike_info_count: counts_objects(&gtfs.trips, |t| {
            t.bikes_allowed != gtfs_structures::BikesAllowedType::NoBikeInfo
        }),
        trips_with_wheelchair_info_count: counts_objects(&gtfs.trips, |t| {
            t.wheelchair_accessible != gtfs_structures::Availability::InformationNotAvailable
        }),
        trips_with_shape_count: counts_objects(&gtfs.trips, |t| t.shape_id.is_some()),
        trips_with_trip_headsign_count: counts_objects(&gtfs.trips, |t| {
            t.trip_headsign.is_some() && t.trip_headsign != Some("".to_string())
        }),

        fares_attribute_count: gtfs
            .fare_attributes
            .as_ref()
            .and_then(|r| r.as_ref().ok().map(|v| v.len()))
            .unwrap_or(0),
        fares_rules_count: gtfs
            .fare_rules
            .as_ref()
            .and_then(|r| r.as_ref().ok().map(|v| v.len()))
            .unwrap_or(0),

        transfers_count: gtfs
            .transfers
            .as_ref()
            .and_then(|r| r.as_ref().ok().map(|v| v.len()))
            .unwrap_or(0),
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

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct Interval {
    start_date: chrono::NaiveDate,
    end_date: chrono::NaiveDate,
}

impl Interval {
    fn update_bounds(&mut self, other: &Interval) {
        self.update_bounds_with_date(&other.start_date);
        self.update_bounds_with_date(&other.end_date);
    }
    fn update_bounds_with_date(&mut self, d: &NaiveDate) {
        self.start_date = core::cmp::min(self.start_date, *d);
        self.end_date = core::cmp::max(self.end_date, *d);
    }
}

fn compute_services_start_end_dates(gtfs: &gtfs_structures::Gtfs) -> HashMap<&str, Interval> {
    let mut res: HashMap<&str, Interval> = gtfs
        .calendar
        .iter()
        .map(|(id, c)| {
            (
                id.as_str(),
                Interval {
                    start_date: c.start_date,
                    end_date: c.end_date,
                },
            )
        })
        .collect();
    for (calendar_id, calendar_dates) in &gtfs.calendar_dates {
        for d in calendar_dates
            .iter()
            .filter(|cd| cd.exception_type == gtfs_structures::Exception::Added)
        {
            res.entry(calendar_id)
                .and_modify(|i| i.update_bounds_with_date(&d.date))
                .or_insert(Interval {
                    start_date: d.date,
                    end_date: d.date,
                });
        }
    }
    res
}

fn networks_start_end_dates(
    metadata: &Metadata,
    gtfs: &gtfs_structures::Gtfs,
) -> HashMap<String, Option<Interval>> {
    let result: HashMap<String, Option<Interval>> = if metadata.networks.len() == 1 {
        // if there is only one agency, get data from existing metadata
        let mut agency_start_end_dates = HashMap::default();
        let start_end = match (metadata.start_date.as_ref(), metadata.end_date.as_ref()) {
            (Some(sd), Some(ed)) => Some(Interval {
                start_date: sd.parse().unwrap(),
                end_date: ed.parse().unwrap(),
            }),
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
                    .or_insert(*service_bounds);
            }
        }

        agencies_start_end_dates
            .into_iter()
            .map(|(id, i)| {
                (
                    gtfs.agencies
                        .iter()
                        .find(|a| a.id == id)
                        .map(|a| a.name.clone())
                        .unwrap_or("default_agency".to_string()),
                    Some(i),
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

fn counts_objects<T>(
    objects: &Result<Vec<T>, Error>,
    matches: for<'a> fn(&'a &T) -> bool,
) -> usize {
    objects
        .as_ref()
        .unwrap_or(&Vec::new())
        .iter()
        .filter(matches)
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
    fn test_feed_info_metadata() {
        let raw_gtfs =
            gtfs_structures::RawGtfs::new("test_data/feed_info").expect("Failed to load data");
        let metadatas = extract_metadata(&raw_gtfs);
        assert_eq!(
            HashMap::from([
                ("SNCF".to_owned(), "contact@sncf.fr".to_owned()),
                ("BIBUS".to_owned(), "contact@bibus.fr".to_owned())
            ]),
            metadatas.feed_contact_emails
        );
        assert_eq!(
            HashMap::from([
                ("SNCF".to_owned(), "2018-07-09".to_owned()),
                ("BIBUS".to_owned(), "2019-01-02".to_owned())
            ]),
            metadatas.feed_start_dates
        );
        assert_eq!(
            HashMap::from([
                ("SNCF".to_owned(), "2018-09-27".to_owned()),
                ("BIBUS".to_owned(), "2019-02-04".to_owned())
            ]),
            metadatas.feed_end_dates
        );
    }

    #[test]
    fn test_feed_info_not_present_metadata() {
        // feed_info.txt is not present
        let raw_gtfs = gtfs_structures::RawGtfs::new("test_data/fare_attributes")
            .expect("Failed to load data");
        let metadatas = extract_metadata(&raw_gtfs);

        assert!(!raw_gtfs.files.contains(&"feed_info.txt".to_owned()));
        assert_eq!(HashMap::new(), metadatas.feed_contact_emails);
        assert_eq!(HashMap::new(), metadatas.feed_start_dates);
        assert_eq!(HashMap::new(), metadatas.feed_end_dates);
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
    fn test_count_routes_with_custom_color() {
        let raw_gtfs = gtfs_structures::RawGtfs::new("test_data/custom_route_color")
            .expect("Failed to load data");
        let metadatas = extract_metadata(&raw_gtfs);
        assert_eq!(3, metadatas.stats.routes_with_custom_color_count);
        assert_eq!(3, metadatas.stats.routes_with_long_name_count);
        assert_eq!(4, metadatas.stats.routes_with_short_name_count);
    }

    #[test]
    fn test_count_trips() {
        let raw_gtfs =
            gtfs_structures::RawGtfs::new("test_data/stops").expect("Failed to load data");
        let mut metadatas = extract_metadata(&raw_gtfs);

        assert_eq!(11, metadatas.stats.trips_count);
        assert_eq!(3, metadatas.stats.trips_with_bike_info_count);
        assert_eq!(3, metadatas.stats.trips_with_wheelchair_info_count);
        assert_eq!(0, metadatas.stats.trips_with_shape_count);
        assert_eq!(9, metadatas.stats.trips_with_trip_headsign_count);

        // also check the fares in the tests data
        assert_eq!(2, metadatas.stats.fares_attribute_count);
        assert_eq!(4, metadatas.stats.fares_rules_count);

        assert_eq!(
            serde_json::to_string_pretty(&metadatas.stats).unwrap(),
            r#"{
  "stops_count": 19,
  "stop_areas_count": 2,
  "stop_points_count": 11,
  "stops_with_wheelchair_info_count": null,
  "routes_count": 5,
  "routes_with_custom_color_count": 0,
  "routes_with_short_name_count": 0,
  "routes_with_long_name_count": 4,
  "trips_count": 11,
  "trips_with_bike_info_count": 3,
  "trips_with_wheelchair_info_count": 3,
  "trips_with_shape_count": 0,
  "trips_with_trip_headsign_count": 9,
  "transfers_count": 0,
  "fares_attribute_count": 2,
  "fares_rules_count": 4
}"#
        );

        // we'll get the stops_with_wheelchair_info_count stat after enriching the stats with the GTFS instead of the RawGTFS
        let gtfs = gtfs_structures::Gtfs::try_from(raw_gtfs).expect("Failed to load GTFS");
        metadatas.enrich_with_advanced_infos(&gtfs);
        assert_eq!(
            serde_json::to_string_pretty(&metadatas.stats).unwrap(),
            r#"{
  "stops_count": 19,
  "stop_areas_count": 2,
  "stop_points_count": 11,
  "stops_with_wheelchair_info_count": 0,
  "routes_count": 5,
  "routes_with_custom_color_count": 0,
  "routes_with_short_name_count": 0,
  "routes_with_long_name_count": 4,
  "trips_count": 11,
  "trips_with_bike_info_count": 3,
  "trips_with_wheelchair_info_count": 3,
  "trips_with_shape_count": 0,
  "trips_with_trip_headsign_count": 9,
  "transfers_count": 0,
  "fares_attribute_count": 2,
  "fares_rules_count": 4
}"#
        );
    }

    #[test]
    fn test_count_stops_with_accessibility_infos() {
        let raw_gtfs =
            gtfs_structures::RawGtfs::new("test_data/accessibility").expect("Failed to load data");
        let mut metadatas = extract_metadata(&raw_gtfs);
        let gtfs = gtfs_structures::Gtfs::try_from(raw_gtfs).expect("Failed to load GTFS");
        assert_eq!(16, metadatas.stops_count);

        metadatas.enrich_with_advanced_infos(&gtfs);
        assert_eq!(
            serde_json::to_string_pretty(&metadatas.stats).unwrap(),
            r#"{
  "stops_count": 16,
  "stop_areas_count": 4,
  "stop_points_count": 6,
  "stops_with_wheelchair_info_count": 10,
  "routes_count": 1,
  "routes_with_custom_color_count": 1,
  "routes_with_short_name_count": 1,
  "routes_with_long_name_count": 1,
  "trips_count": 6,
  "trips_with_bike_info_count": 0,
  "trips_with_wheelchair_info_count": 0,
  "trips_with_shape_count": 0,
  "trips_with_trip_headsign_count": 6,
  "transfers_count": 0,
  "fares_attribute_count": 0,
  "fares_rules_count": 0
}"#
        );
    }

    #[test]
    fn test_count_trips_without_shapes() {
        let raw_gtfs =
            gtfs_structures::RawGtfs::new("test_data/shapes").expect("Failed to load data");
        let metadatas = extract_metadata(&raw_gtfs);

        // only `STBA` and `AB1` have a shape, even if `AB1` has an invalid one, it will be counted (but it will have an InvalidShape issue)
        assert_eq!(2, metadatas.stats.trips_with_shape_count);
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
    assert_eq!(
        Interval {
            start_date: "2017-01-01".parse().unwrap(),
            end_date: "2017-01-15".parse().unwrap()
        },
        start_end.to_owned()
    );
    assert_eq!(
        Interval {
            start_date: metadatas.start_date.unwrap().parse().unwrap(),
            end_date: metadatas.end_date.unwrap().parse().unwrap()
        },
        start_end.to_owned()
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

    assert_eq!(
        Interval {
            start_date: "2019-01-01".parse().unwrap(),
            end_date: "2022-01-01".parse().unwrap()
        },
        start_end.to_owned()
    );

    let start_end = networks_start_end_dates
        .get("BIBUS")
        .unwrap()
        .as_ref()
        .unwrap();

    assert_eq!(
        Interval {
            start_date: "2016-01-01".parse().unwrap(),
            end_date: "2023-01-01".parse().unwrap()
        },
        start_end.to_owned()
    );
}

#[test]
fn test_interval_serialization() {
    let i = Interval {
        start_date: NaiveDate::from_ymd_opt(2022, 1, 1).unwrap(),
        end_date: NaiveDate::from_ymd_opt(2023, 1, 2).unwrap(),
    };

    assert_eq!(
        serde_json::to_string(&i).unwrap(),
        "{\"start_date\":\"2022-01-01\",\"end_date\":\"2023-01-02\"}"
    )
}
