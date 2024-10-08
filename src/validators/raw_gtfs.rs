use itertools::Itertools;

use crate::issues::{Issue, IssueType, Severity};

const MAX_DISPLAYED_PT_SEQUENCES: usize = 10;

fn check_duplicates<O: gtfs_structures::Id + gtfs_structures::Type>(
    objects: &Result<Vec<O>, gtfs_structures::Error>,
) -> Vec<Issue> {
    let mut ids = std::collections::HashSet::<String>::new();
    let mut issues = vec![];
    for o in objects.as_ref().unwrap_or(&vec![]) {
        let id = o.id().to_owned();
        if ids.contains(&id) {
            issues.push(
                Issue::new(Severity::Error, IssueType::DuplicateObjectId, &id)
                    .object_type(o.object_type()),
            );
        }
        ids.insert(id);
    }
    issues
}

fn check_shapes_duplicates(
    objects: &Result<Vec<gtfs_structures::Shape>, gtfs_structures::Error>,
) -> Vec<Issue> {
    let mut ids = std::collections::HashSet::<(String, usize)>::new();
    let mut shape_issues = std::collections::HashMap::<String, Vec<usize>>::new();
    for shape in objects.as_ref().unwrap_or(&vec![]) {
        let id = (shape.id.clone(), shape.sequence);
        if ids.contains(&id) {
            shape_issues
                .entry(shape.id.clone())
                .or_default()
                .push(shape.sequence);
        }
        ids.insert(id);
    }

    shape_issues
        .into_iter()
        .map(|(shape_id, duplicate_sequences)| {
            let more_dup = if duplicate_sequences.len() > MAX_DISPLAYED_PT_SEQUENCES {
                "…"
            } else {
                ""
            };
            let d = format!(
                "Shape has duplicated pt_sequence: {}{}",
                duplicate_sequences
                    .iter()
                    .take(MAX_DISPLAYED_PT_SEQUENCES)
                    .join(", "),
                more_dup
            );
            Issue::new(Severity::Error, IssueType::DuplicateObjectId, &shape_id)
                .object_type(gtfs_structures::ObjectType::Shape)
                .details(&d)
        })
        .collect()
}

pub fn validate(raw_gtfs: &gtfs_structures::RawGtfs) -> Vec<Issue> {
    check_duplicates(&raw_gtfs.stops)
        .into_iter()
        .chain(check_duplicates(&raw_gtfs.routes))
        .chain(check_duplicates(&raw_gtfs.trips))
        .chain(check_duplicates(&raw_gtfs.agencies))
        .chain(check_duplicates(
            raw_gtfs.pathways.as_ref().unwrap_or(&Ok(vec![])),
        ))
        .chain(check_duplicates(
            raw_gtfs.calendar.as_ref().unwrap_or(&Ok(vec![])),
        ))
        .chain(check_duplicates(
            raw_gtfs.fare_attributes.as_ref().unwrap_or(&Ok(vec![])),
        ))
        .chain(check_shapes_duplicates(
            raw_gtfs.shapes.as_ref().unwrap_or(&Ok(vec![])),
        ))
        .collect()
}

#[test]
fn test_duplicates() {
    // in the dataset, every last line has been duplicated
    let gtfs = gtfs_structures::RawGtfs::new("test_data/duplicates").unwrap();
    let issues = dbg!(validate(&gtfs));
    assert_eq!(9, issues.len());

    assert_eq!(
        Issue::new(Severity::Error, IssueType::DuplicateObjectId, "stop5")
            .object_type(gtfs_structures::ObjectType::Stop),
        issues[0]
    );

    assert_eq!(
        Issue::new(Severity::Error, IssueType::DuplicateObjectId, "CITY")
            .object_type(gtfs_structures::ObjectType::Route),
        issues[1]
    );
    assert_eq!(
        Issue::new(Severity::Error, IssueType::DuplicateObjectId, "AAMV4")
            .object_type(gtfs_structures::ObjectType::Trip),
        issues[2]
    );
    assert_eq!(
        Issue::new(Severity::Error, IssueType::DuplicateObjectId, "DTA")
            .object_type(gtfs_structures::ObjectType::Agency),
        issues[3]
    );
    assert_eq!(
        Issue::new(Severity::Error, IssueType::DuplicateObjectId, "pathway1")
            .object_type(gtfs_structures::ObjectType::Pathway),
        issues[4]
    );
    assert_eq!(
        Issue::new(Severity::Error, IssueType::DuplicateObjectId, "WE")
            .object_type(gtfs_structures::ObjectType::Calendar),
        issues[5]
    );

    assert_eq!(
        Issue::new(Severity::Error, IssueType::DuplicateObjectId, "a")
            .object_type(gtfs_structures::ObjectType::Fare),
        issues[6]
    );
    // many_dup_shp has 11 duplicated pt_sequences (all by the first one), we only display the first 10
    let many_dup_shape_issue = issues
        .iter()
        .filter(|i| i.object_id.as_str() == "many_dup_shp")
        .next();
    assert_eq!(
        Some(
            &Issue::new(
                Severity::Error,
                IssueType::DuplicateObjectId,
                "many_dup_shp"
            )
            .object_type(gtfs_structures::ObjectType::Shape)
            .details("Shape has duplicated pt_sequence: 2, 3, 4, 5, 6, 7, 8, 9, 10, 11…")
        ),
        many_dup_shape_issue
    );
    let a_shape_issue = issues
        .iter()
        .filter(|i| i.object_id.as_str() == "A_shp")
        .next();
    assert_eq!(
        Some(
            &Issue::new(Severity::Error, IssueType::DuplicateObjectId, "A_shp")
                .object_type(gtfs_structures::ObjectType::Shape)
                .details("Shape has duplicated pt_sequence: 0")
        ),
        a_shape_issue
    );
}
