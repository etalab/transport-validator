use crate::validators::issues::*;

pub fn validate(gtfs: &gtfs_structures::Gtfs) -> Vec<Issue> {
    let r = gtfs
        .routes
        .values()
        .filter(|route| !has_id(*route))
        .map(|route| make_missing_id_issue(route));
    let t = gtfs
        .trips
        .values()
        .filter(|trip| !has_id(*trip))
        .map(|trip| make_missing_id_issue(trip));
    let c = gtfs
        .calendar
        .values()
        .filter(|calendar| !has_id(*calendar))
        .map(|calendar| make_missing_id_issue(calendar));
    let st = gtfs
        .stops
        .values()
        .filter(|&stop| !has_id(&**stop))
        .map(|stop| make_missing_id_issue(&**stop));
    let sh = gtfs
        .shapes
        .keys()
        .filter(|id| id.is_empty())
        .map(|_id| Issue::new(Severity::Error, IssueType::MissingId, ""));
    let ag = if gtfs.agencies.len() <= 1 {
        vec![]
    } else {
        gtfs.agencies
            .iter()
            .filter(|agency| !has_id(*agency))
            .map(|agency| Issue::new_with_obj(Severity::Error, IssueType::MissingId, agency))
            .collect()
    };
    r.chain(t)
        .chain(c)
        .chain(st)
        .chain(sh)
        .chain(ag.into_iter())
        .collect()
}

fn has_id(object: &gtfs_structures::Id) -> bool {
    !object.id().is_empty()
}

fn make_missing_id_issue<T: gtfs_structures::Id + std::fmt::Display>(o: &T) -> Issue {
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
