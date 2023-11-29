use crate::issues::{Issue, IssueType, Severity};

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

pub fn validate(raw_gtfs: &gtfs_structures::RawGtfs) -> Vec<Issue> {
    check_duplicates(&raw_gtfs.stops)
        .into_iter()
        .chain(check_duplicates(&raw_gtfs.routes).into_iter())
        .chain(check_duplicates(&raw_gtfs.trips).into_iter())
        .chain(check_duplicates(&raw_gtfs.agencies).into_iter())
        .chain(check_duplicates(&raw_gtfs.pathways.as_ref().unwrap_or(&Ok(vec![]))).into_iter())
        .chain(check_duplicates(&raw_gtfs.shapes.as_ref().unwrap_or(&Ok(vec![]))).into_iter())
        .chain(check_duplicates(raw_gtfs.calendar.as_ref().unwrap_or(&Ok(vec![]))).into_iter())
        .chain(
            check_duplicates(raw_gtfs.fare_attributes.as_ref().unwrap_or(&Ok(vec![]))).into_iter(),
        )
        .collect()
}

#[test]
fn test_duplicates() {
    // in the dataset, every last line has been duplicated
    let gtfs = gtfs_structures::RawGtfs::new("test_data/duplicates").unwrap();
    let issues = dbg!(validate(&gtfs));
    assert_eq!(7, issues.len());

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
}
