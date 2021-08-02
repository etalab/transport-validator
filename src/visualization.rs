use crate::issues;
use geojson::Value::Point;
use geojson::{Feature, FeatureCollection};
use gtfs_structures::{Gtfs, ObjectType};
use std::sync::Arc;

pub fn add_issue_visualization(issue: &mut issues::Issue, gtfs: &Gtfs) {
    match issue.object_type {
        Some(ObjectType::Stop) => {
            let stop_id = issue.object_id.clone();

            // a vec containing the stop_id and the related stop ids features
            let features = [stop_id]
                .iter()
                .chain(get_related_stop_ids(issue).iter())
                .map(|stop_id| geojson_feature_point(&stop_id, gtfs))
                .flatten()
                .collect();

            let feature_collection = FeatureCollection {
                bbox: None,
                features: features,
                foreign_members: None,
            };

            issue.geojson = Some(feature_collection.to_string());
        }
        _ => issue.geojson = None,
    };
}

fn geojson_feature_point(stop_id: &String, gtfs: &Gtfs) -> Option<Feature> {
    gtfs.stops.get(stop_id).map(|stop| {
        let stop_geom = get_stop_geom(stop);
        Feature {
            geometry: stop_geom,
            bbox: None,
            properties: None,
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
