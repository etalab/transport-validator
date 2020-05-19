use crate::issues::IssueType;
use gtfs_structures;
use itertools::Itertools;
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
}

pub fn extract_metadata(gtfs: &gtfs_structures::RawGtfs) -> Metadata {
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
            .collect(),
        modes: gtfs
            .routes
            .as_ref()
            .unwrap_or(&vec![])
            .iter()
            .map(|r| match r.route_type {
                Tramway => "tramway".to_owned(),
                Subway => "subway".to_owned(),
                Rail => "rail".to_owned(),
                Bus => "bus".to_owned(),
                Ferry => "ferry".to_owned(),
                CableCar => "cable_car".to_owned(),
                Gondola => "gondola".to_owned(),
                Funicular => "funicular".to_owned(),
                Other(_) => "invalid".to_owned(),
            })
            .unique()
            .collect(),
        issues_count: std::collections::BTreeMap::new(),
        has_fares: match &gtfs.fare_attributes {
            Some(Ok(fa)) => fa.len() > 0,
            _ => false,
        },
        has_shapes: match &gtfs.shapes {
            Some(Ok(s)) => s.len() > 0,
            _ => false,
        },
    }
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
fn test_no_fares_no_shapes() {
    let raw_gtfs =
        gtfs_structures::RawGtfs::new("test_data/no_fares_no_shapes").expect("Failed to load data");
    let metadatas = extract_metadata(&raw_gtfs);
    assert!(!metadatas.has_fares);
    assert!(!metadatas.has_shapes);
}
