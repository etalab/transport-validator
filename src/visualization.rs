use crate::issues;
use geojson::Value::Point;
use geojson::{Feature, FeatureCollection};
use gtfs_structures::{Gtfs, ObjectType};
use std::sync::Arc;

pub fn add_issue_visualization(issue: &mut issues::Issue, gtfs: &Gtfs) {
    match issue.object_type {
        Some(ObjectType::Stop) => {
            let stop_id = issue.object_id.clone();
            let stop_feature = geojson_feature_point(&stop_id, gtfs);
            let mut related_stops_features: Vec<Option<Feature>> = get_related_stops(issue)
                .iter()
                .map(|stop_id| geojson_feature_point(&stop_id, gtfs))
                .collect();

            // il doit exister plus simple...
            let mut features = vec![];
            features.push(stop_feature);
            features.append(&mut related_stops_features);

            let filtered_features = features.into_iter().flatten().collect();

            let feature_collection = FeatureCollection {
                bbox: None,
                features: filtered_features,
                foreign_members: None,
            };

            issue.geojson = Some(feature_collection.to_string());
        }
        _ => issue.geojson = None,
    };
}

fn geojson_feature_point(stop_id: &String, gtfs: &Gtfs) -> Option<Feature> {
    let obj = gtfs.stops.get(stop_id);
    return match obj {
        Some(stop) => {
            let stop_geom = get_stop_geom(stop);
            let feature = Feature {
                geometry: stop_geom,
                bbox: None,
                properties: None,
                id: None,
                foreign_members: None,
            };
            Some(feature)
        }
        None => None,
    };
}

fn get_stop_geom(stop: &Arc<gtfs_structures::Stop>) -> Option<geojson::Geometry> {
    match (&stop.longitude, &stop.latitude) {
        (Some(lon), Some(lat)) => Some(geojson::Geometry::new(Point(vec![*lon, *lat]))),
        _ => None,
    }
}

fn get_related_stops(issue: &issues::Issue) -> Vec<String> {
    let related_objects = &issue.related_objects;
    return related_objects
        .iter()
        .filter(|o| o.object_type == Some(ObjectType::Stop))
        .map(|s| s.id.clone())
        .collect();
}
