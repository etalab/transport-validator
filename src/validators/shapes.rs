use std::collections::HashSet;

use crate::issues::*;

pub fn validate(gtfs: &gtfs_structures::Gtfs) -> Vec<Issue> {
    let missing_coord = gtfs
        .shapes
        .iter()
        .filter(|(_id, shapes)| !shapes.iter().all(has_coord))
        .map(|(id, _shapes)| {
            Issue::new(Severity::Warning, IssueType::MissingCoordinates, id)
                .object_type(gtfs_structures::ObjectType::Shape)
        });
    let valid = gtfs
        .shapes
        .iter()
        .filter(|(_id, shapes)| !shapes.iter().all(valid_coord))
        .map(|(id, _shapes)| {
            Issue::new(Severity::Error, IssueType::InvalidCoordinates, id)
                .object_type(gtfs_structures::ObjectType::Shape)
        });
    let valid_shape_id = gtfs
        .trips
        .iter()
        .map(|(_id, trip)| create_invalid_shape_id_issue(trip, gtfs))
        .filter_map(|o| o);

    let used_shape_id: HashSet<String> = gtfs
        .trips
        .iter()
        .map(|(_trip_id, trip)| trip.shape_id.to_owned())
        .filter_map(|o| o)
        .collect();

    let existing_shape_id: HashSet<String> = gtfs
        .shapes
        .iter()
        .map(|(shape_id, _shape)| shape_id.to_owned())
        .collect();

    let unused_shape_id = existing_shape_id.difference(&used_shape_id).map(|id| {
        Issue::new(Severity::Information, IssueType::UnusedShapeId, id)
            .object_type(gtfs_structures::ObjectType::Shape)
    });

    missing_coord
        .chain(valid)
        .chain(valid_shape_id)
        .chain(unused_shape_id)
        .collect()
}

fn create_invalid_shape_id_issue(
    trip: &gtfs_structures::Trip,
    gtfs: &gtfs_structures::Gtfs,
) -> Option<Issue> {
    match &trip.shape_id {
        None => None,
        Some(shape_id) => {
            if gtfs.shapes.contains_key(shape_id) {
                None
            } else {
                Some(
                    Issue::new(Severity::Error, IssueType::InvalidShapeId, &trip.id)
                        .object_type(gtfs_structures::ObjectType::Trip),
                )
            }
        }
    }
}

fn has_coord(shape: &gtfs_structures::Shape) -> bool {
    shape.latitude != 0.0 && shape.longitude != 0.0
}

fn valid_coord(shape: &gtfs_structures::Shape) -> bool {
    ((shape.longitude <= 180.0) && (shape.longitude >= -180.0))
        && ((shape.latitude <= 90.0) && (shape.latitude >= -90.0))
}

#[test]
fn test_missing_coord() {
    let gtfs = gtfs_structures::Gtfs::new("test_data/shapes").unwrap();
    let issues = validate(&gtfs);
    let missing_coord_issue: Vec<_> = issues
        .iter()
        .filter(|issue| issue.issue_type == IssueType::MissingCoordinates)
        .collect();

    assert_eq!(1, missing_coord_issue.len());
    assert_eq!("A_shp", missing_coord_issue[0].object_id);
    assert_eq!(
        IssueType::MissingCoordinates,
        missing_coord_issue[0].issue_type
    );
}

#[test]
fn test_valid() {
    let gtfs = gtfs_structures::Gtfs::new("test_data/shapes").unwrap();
    let issues = validate(&gtfs);
    let invalid_coord_issue: Vec<_> = issues
        .iter()
        .filter(|issue| issue.issue_type == IssueType::InvalidCoordinates)
        .collect();

    assert_eq!(1, invalid_coord_issue.len());
    assert_eq!("A_shp", invalid_coord_issue[0].object_id);
    assert_eq!(
        IssueType::InvalidCoordinates,
        invalid_coord_issue[0].issue_type
    );
}

#[test]
fn test_invalid_shape_id() {
    let gtfs = gtfs_structures::Gtfs::new("test_data/shapes").unwrap();
    let issues = validate(&gtfs);
    let invalid_shape_id: Vec<_> = issues
        .iter()
        .filter(|issue| issue.issue_type == IssueType::InvalidShapeId)
        .collect();

    assert_eq!(1, invalid_shape_id.len());
    assert_eq!("AB1", invalid_shape_id[0].object_id);
    assert_eq!(IssueType::InvalidShapeId, invalid_shape_id[0].issue_type);
}

#[test]
fn test_unused_shape_id() {
    let gtfs = gtfs_structures::Gtfs::new("test_data/shapes").unwrap();
    let issues = validate(&gtfs);
    let unused_shape_id: Vec<_> = issues
        .iter()
        .filter(|issue| issue.issue_type == IssueType::UnusedShapeId)
        .collect();

    assert_eq!(1, unused_shape_id.len());
    assert_eq!("A_shp", unused_shape_id[0].object_id);
    assert_eq!(IssueType::UnusedShapeId, unused_shape_id[0].issue_type);
}
