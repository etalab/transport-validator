use crate::issues::*;

pub fn validate(gtfs: &gtfs_structures::Gtfs) -> Vec<Issue> {
    let r = gtfs
        .routes
        .values()
        .filter(|&route| !has_id(route))
        .map(make_missing_id_issue);
    let t = gtfs
        .trips
        .values()
        .filter(|&trip| !has_id(trip))
        .map(make_missing_id_issue);
    let c = gtfs
        .calendar
        .values()
        .filter(|&calendar| !has_id(calendar))
        .map(make_missing_id_issue);
    let st = gtfs
        .stops
        .values()
        .filter(|&stop| !has_id(stop.as_ref()))
        .map(|stop| make_missing_id_issue(stop.as_ref()));
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
    let ag = if gtfs.agencies.len() <= 1 {
        vec![]
    } else {
        gtfs.agencies
            .iter()
            .filter(|agency| !has_id(*agency))
            .map(make_missing_id_issue)
            .collect()
    };
    r.chain(t)
        .chain(c)
        .chain(st)
        .chain(sh)
        .chain(ag.into_iter())
        .collect()
}

fn has_id(object: &dyn gtfs_structures::Id) -> bool {
    !object.id().is_empty()
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
