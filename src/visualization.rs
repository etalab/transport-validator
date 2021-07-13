use crate::issues;
use geojson::Feature;
use geojson::Value::Point;
use gtfs_structures::{Gtfs, ObjectType};
use std::sync::Arc;

pub fn add_issue_visualization(issue: &mut issues::Issue, gtfs: &Gtfs) {
    match issue.object_type {
        Some(ObjectType::Stop) => {
            let stop_id = issue.object_id.clone();
            issue.geojson = geojson_feature_point(&stop_id, gtfs);
        }
        _ => issue.geojson = None,
    };
}

fn geojson_feature_point(stop_id: &String, gtfs: &Gtfs) -> Option<String> {
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
            Some(feature.to_string())
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
