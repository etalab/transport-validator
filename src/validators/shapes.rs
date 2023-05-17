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
    let invalid_shape_id = gtfs
        .trips
        .iter()
        .filter_map(|(_id, trip)| create_invalid_shape_id_issue(trip, gtfs));

    let used_shape_id: HashSet<&String> = gtfs
        .trips
        .iter()
        .filter_map(|(_trip_id, trip)| trip.shape_id.as_ref())
        .collect();

    let existing_shape_id: HashSet<&String> = gtfs
        .shapes
        .iter()
        .map(|(shape_id, _shape)| shape_id)
        .collect();

    let unused_shape_id = existing_shape_id.difference(&used_shape_id).map(|id| {
        Issue::new(Severity::Information, IssueType::UnusedShapeId, id)
            .object_type(gtfs_structures::ObjectType::Shape)
    });

    let trip_without_shaped = gtfs
        .trips
        .iter()
        .filter(|(_trip_id, trip)| trip.shape_id.is_none())
        .map(|(_trip_id, trip)| {
            Issue::new_with_obj(Severity::Information, IssueType::NoShape, trip)
        });

    missing_coord
        .chain(valid)
        .chain(invalid_shape_id)
        .chain(unused_shape_id)
        .chain(trip_without_shaped)
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
                        .object_type(gtfs_structures::ObjectType::Trip)
                        .details(&format!("invalid shape id: {}", shape_id)),
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
    assert_eq!(
        "invalid shape id: non_existing_shape_id",
        invalid_shape_id[0].details.as_ref().unwrap()
    );
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

#[test]
fn test_no_shape() {
    use std::iter::FromIterator;
    let gtfs = gtfs_structures::Gtfs::new("test_data/shapes").unwrap();
    let issues = validate(&gtfs);
    let no_shape: Vec<_> = issues
        .iter()
        .filter(|issue| issue.issue_type == IssueType::NoShape)
        .collect();

    assert_eq!(
        no_shape.iter().map(|i| i.severity).collect::<HashSet<_>>(),
        HashSet::from_iter(vec!(Severity::Information).into_iter())
    );
    // Only `STBA` has a valid shape
    // `AB1` has a shape, even if it's an invalid one (cf `test_invalid_shape_id`)
    assert_eq!(
        no_shape
            .iter()
            .map(|i| i.object_id.as_str())
            .collect::<HashSet<_>>(),
        HashSet::from_iter(
            vec!("BFC1", "AAMV3", "CITY1", "CITY2", "AB2", "AAMV4", "BFC2", "AAMV2", "AAMV1")
                .into_iter()
        )
    );
}
