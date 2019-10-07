use crate::validators::issues::{Issue, IssueType, Severity};

fn check_duplicates<O: gtfs_structures::Id + gtfs_structures::Type>(
    objects: &Result<Vec<O>, failure::Error>,
) -> Vec<Issue> {
    let mut ids = std::collections::HashSet::<String>::new();
    let mut issues = vec![];
    for o in objects.as_ref().unwrap_or(&vec![]) {
        let id = o.id().to_owned();
        if ids.contains(&id) {
            issues.push(
                Issue::new(Severity::Information, IssueType::DuplicateObjectId, &id)
                    .object_type(o.object_type()),
            );
        }
        ids.insert(id);
    }
    issues
}

pub fn validate(raw_gtfs: &gtfs_structures::RawGtfs) -> Vec<Issue> {
    check_duplicates(&raw_gtfs.stops)
        .into_iter()
        .chain(check_duplicates(&raw_gtfs.routes).into_iter())
        .chain(check_duplicates(&raw_gtfs.trips).into_iter())
        .chain(check_duplicates(&raw_gtfs.calendar.as_ref().unwrap_or(&Ok(vec![]))).into_iter())
        .chain(
            check_duplicates(&raw_gtfs.fare_attributes.as_ref().unwrap_or(&Ok(vec![]))).into_iter(),
        )
        .collect()
}

#[test]
fn test_duplicates() {
    // in the dataset, every last line has been duplicated
    let gtfs = gtfs_structures::RawGtfs::new("test_data/duplicates").unwrap();
    let issues = validate(&gtfs);
    assert_eq!(5, issues.len());
    assert_eq!("stop5", issues[0].object_id);
    assert_eq!(IssueType::DuplicateObjectId, issues[0].issue_type);
    assert_eq!(
        Some(gtfs_structures::ObjectType::Stop),
        issues[0].object_type
    );

    assert_eq!("CITY", issues[1].object_id);
    assert_eq!(IssueType::DuplicateObjectId, issues[1].issue_type);
    assert_eq!(
        Some(gtfs_structures::ObjectType::Route),
        issues[1].object_type
    );

    assert_eq!("AAMV4", issues[2].object_id);
    assert_eq!(IssueType::DuplicateObjectId, issues[2].issue_type);
    assert_eq!(
        Some(gtfs_structures::ObjectType::Trip),
        issues[2].object_type
    );

    assert_eq!("WE", issues[3].object_id);
    assert_eq!(IssueType::DuplicateObjectId, issues[3].issue_type);
    assert_eq!(
        Some(gtfs_structures::ObjectType::Calendar),
        issues[3].object_type
    );

    assert_eq!("a", issues[4].object_id);
    assert_eq!(IssueType::DuplicateObjectId, issues[4].issue_type);
    assert_eq!(
        Some(gtfs_structures::ObjectType::Fare),
        issues[4].object_type
    );
}
