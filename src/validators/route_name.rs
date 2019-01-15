use crate::validators::issues::*;

pub fn validate(gtfs: &gtfs_structures::Gtfs) -> Vec<Issue> {
    gtfs.routes
        .iter()
        .filter(|&(_, route)| !has_name(route))
        .map(|(_, route)| make_missing_name_issue(route))
        .collect()
}

fn has_name(route: &gtfs_structures::Route) -> bool {
    !route.short_name.is_empty() || !route.long_name.is_empty()
}

fn make_missing_name_issue<T: gtfs_structures::Id + std::fmt::Display>(o: &T) -> Issue {
    Issue::new(Severity::Error, IssueType::MissingRouteName, o.id())
}

#[test]
fn test() {
    let gtfs = gtfs_structures::Gtfs::new("test_data/unused_stop").unwrap();
    let issues = validate(&gtfs);

    assert_eq!(1, issues.len());
    assert_eq!("CITY", issues[0].object_id);
    assert_eq!(IssueType::MissingRouteName, issues[0].issue_type);
}
