use crate::validators::issues::*;

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
    missing_coord.chain(valid).collect()
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
