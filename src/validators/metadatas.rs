use crate::validators::issues::IssueType;
use gtfs_structures;
use itertools::Itertools;

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
}

pub fn extract_metadata(gtfs: &gtfs_structures::Gtfs) -> Metadata {
    use gtfs_structures::RouteType::*;

    Metadata {
        start_date: gtfs
            .calendar
            .values()
            .map(|c| c.start_date.format("%Y-%m-%d").to_string())
            .min(),
        end_date: gtfs
            .calendar
            .values()
            .map(|c| c.end_date.format("%Y-%m-%d").to_string())
            .max(),
        stop_areas_count: gtfs
            .stops
            .values()
            .filter(|s| s.location_type == gtfs_structures::LocationType::StopArea)
            .count(),
        stop_points_count: gtfs
            .stops
            .values()
            .filter(|s| s.location_type == gtfs_structures::LocationType::StopPoint)
            .count(),
        lines_count: gtfs.routes.iter().count(),
        networks: gtfs.agencies.iter().map(|a| a.name.to_owned()).collect(),
        modes: gtfs
            .routes
            .values()
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
    }
}
