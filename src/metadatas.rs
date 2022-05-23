use crate::issues::IssueType;
use itertools::Itertools;
use rgb::RGB;
use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct Metadata {
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub stop_areas_count: usize,
    pub stop_points_count: usize,
    pub lines_count: usize,
    pub networks: Vec<String>,
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

    Metadata {
        start_date: start_end.map(|(s, _)| format(s)),
        end_date: start_end.map(|(_, e)| format(e)),
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
        lines_count: gtfs.routes.as_ref().map(|r| r.len()).unwrap_or(0),
        networks: gtfs
            .agencies
            .as_ref()
            .unwrap_or(&vec![])
            .iter()
            .map(|a| a.name.to_owned())
            .unique()
            .collect(),
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
    }
}

fn has_on_demand_pickup_dropoff(
    stop_time: &gtfs_structures::RawStopTime,
    pickup_dropoff_type: gtfs_structures::PickupDropOffType,
) -> bool {
    stop_time.pickup_type == pickup_dropoff_type || stop_time.drop_off_type == pickup_dropoff_type
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
