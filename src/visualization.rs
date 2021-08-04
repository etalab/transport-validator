use crate::issues;
use geojson::Value::Point;
use geojson::{Feature, FeatureCollection};
use gtfs_structures::{Gtfs, ObjectType};
use serde_json::{to_value, Map};
use std::sync::Arc;

pub fn generate_issue_visualization(
    issue: &issues::Issue,
    gtfs: &Gtfs,
) -> Option<FeatureCollection> {
    match issue.object_type {
        Some(ObjectType::Stop) => {
            let stop_id = issue.object_id.clone();
            let related_stop_ids = get_related_stop_ids(issue);

            // a vec containing the stop_id and the related stop ids features
            let stop_features: Vec<_> = [stop_id.clone()]
                .iter()
                .chain(related_stop_ids.iter())
                .map(|stop_id| geojson_feature_point(&stop_id, gtfs))
                .flatten()
                .collect();

            let line_string_features: Vec<_> = related_stop_ids
                .iter()
                .map(|related_stop| {
                    geojson_feature_line_string(&stop_id, related_stop, gtfs, issue)
                })
                .flatten()
                .collect();

            let features = stop_features
                .into_iter()
                .chain(line_string_features.into_iter())
                .collect();

            let feature_collection = FeatureCollection {
                bbox: None,
                features: features,
                foreign_members: None,
            };

            Some(feature_collection)
        }
        _ => None,
    }
}

fn geojson_feature_point(stop_id: &String, gtfs: &Gtfs) -> Option<Feature> {
    gtfs.stops.get(stop_id).map(|stop| {
        let stop_geom = get_stop_geom(stop);
        let mut properties = Map::new();

        properties.insert(String::from("id"), to_value(&stop.id).unwrap());
        properties.insert(String::from("name"), to_value(&stop.name).unwrap());

        Feature {
            geometry: stop_geom,
            bbox: None,
            properties: Some(properties),
            id: None,
            foreign_members: None,
        }
    })
}

fn get_stop_geom(stop: &Arc<gtfs_structures::Stop>) -> Option<geojson::Geometry> {
    match (&stop.longitude, &stop.latitude) {
        (Some(lon), Some(lat)) => Some(geojson::Geometry::new(Point(vec![*lon, *lat]))),
        _ => None,
    }
}

fn get_related_stop_ids(issue: &issues::Issue) -> Vec<String> {
    let related_objects = &issue.related_objects;
    related_objects
        .iter()
        .filter(|o| o.object_type == Some(ObjectType::Stop))
        .map(|s| s.id.clone())
        .collect()
}

fn geojson_feature_line_string(
    stop1_id: &String,
    stop2_id: &String,
    gtfs: &Gtfs,
    issue: &issues::Issue,
) -> Option<Feature> {
    let stop1 = gtfs.stops.get(stop1_id);
    let stop2 = gtfs.stops.get(stop2_id);

    match (stop1, stop2) {
        (Some(stop1), Some(stop2)) => {
            let geom = line_geometry_between_stops(stop1, stop2);
            let mut properties = Map::new();

            if issue.details.is_some() {
                properties.insert(
                    String::from("details"),
                    to_value(issue.details.as_ref().clone()).unwrap(),
                );
            }

            Some(Feature {
                geometry: geom,
                bbox: None,
                properties: Some(properties),
                id: None,
                foreign_members: None,
            })
        }
        _ => None,
    }
}

fn line_geometry_between_stops(
    stop1: &Arc<gtfs_structures::Stop>,
    stop2: &Arc<gtfs_structures::Stop>,
) -> Option<geojson::Geometry> {
    match (
        &stop1.longitude,
        &stop1.latitude,
        &stop2.longitude,
        &stop2.latitude,
    ) {
        (Some(lon1), Some(lat1), Some(lon2), Some(lat2)) => {
            if *lon1 == *lon2 && *lat1 == *lat2 {
                return None;
            }

            Some(geojson::Geometry::new(geojson::Value::LineString(vec![
                vec![*lon1, *lat1],
                vec![*lon2, *lat2],
            ])))
        }
        _ => None,
    }
}
