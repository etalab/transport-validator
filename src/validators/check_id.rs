use crate::issues::*;

pub fn validate(gtfs: &gtfs_structures::Gtfs) -> Vec<Issue> {
    let r = gtfs
        .routes
        .values()
        .filter(missing_id)
        .map(make_missing_id_issue);
    let t = gtfs
        .trips
        .values()
        .filter(missing_id)
        .map(make_missing_id_issue);
    let c = gtfs
        .calendar
        .values()
        .filter(missing_id)
        .map(make_missing_id_issue);
    let st = gtfs
        .stops
        .values()
        .filter(missing_id)
        .map(make_missing_id_issue);
    let sh = gtfs
        .shapes
        .keys()
        .filter(|id| id.is_empty())
        .map(|_id| Issue {
            object_id: "".to_owned(),
            severity: Severity::Error,
            issue_type: IssueType::MissingId,
            object_type: Some(gtfs_structures::ObjectType::Shape),
            object_name: None,
            related_objects: vec![],
            details: None,
            related_file: None,
            geojson: None,
        });

    let chain = r.chain(t).chain(c).chain(st).chain(sh);
    // The Id of an agency needs only to be specified if there are more than one agency
    if gtfs.agencies.len() > 1 {
        gtfs.agencies
            .iter()
            .filter(missing_id)
            .map(make_missing_id_issue)
            .chain(chain)
            .collect()
    } else {
        chain.collect()
    }
}

fn missing_id<T: gtfs_structures::Id>(object: &&T) -> bool {
    object.id().is_empty()
}

fn make_missing_id_issue<T: gtfs_structures::Id + gtfs_structures::Type + std::fmt::Display>(
    o: &T,
) -> Issue {
    Issue::new_with_obj(Severity::Error, IssueType::MissingId, o)
}

#[test]
fn test() {
    let gtfs = gtfs_structures::Gtfs::new("test_data/check_id").unwrap();
    let issues = validate(&gtfs);
    let stop_id_issues: Vec<_> = issues
        .iter()
        .filter(|issue| issue.object_name == Some("Null Island".to_string()))
        .collect();

    assert_eq!(1, stop_id_issues.len());
    assert_eq!(
        "Null Island",
        stop_id_issues[0].object_name.as_ref().unwrap()
    );
    assert_eq!(IssueType::MissingId, stop_id_issues[0].issue_type);
}
