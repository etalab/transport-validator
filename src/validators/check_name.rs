use crate::issues::{Issue, IssueType, Severity};

pub fn validate(gtfs: &gtfs_structures::Gtfs) -> Vec<Issue> {
    let route_issues = gtfs
        .routes
        .iter()
        .filter(|&(_, route)| !has_name(route))
        .map(|(_, route)| make_missing_name_issue(route));
    let stop_issues = gtfs
        .stops
        .values()
        .filter(|&stop| !has_name(&**stop))
        .map(|stop| make_missing_name_issue(&**stop));
    let agency_issues = gtfs
        .agencies
        .iter()
        .filter(|&agency| !has_name(&*agency))
        .map(|agency| make_missing_name_issue(agency));
    let feed_info_issues = gtfs
        .feed_info
        .iter()
        .filter(|&feed_info| !has_name(&*feed_info))
        .map(|_feed_info| Issue::new(Severity::Warning, IssueType::MissingName, ""));
    route_issues
        .chain(stop_issues)
        .chain(agency_issues)
        .chain(feed_info_issues)
        .collect()
}

fn has_name<T: std::fmt::Display>(o: &T) -> bool {
    !format!("{}", o).is_empty()
}

fn make_missing_name_issue<T: gtfs_structures::Id + gtfs_structures::Type>(o: &T) -> Issue {
    Issue::new(Severity::Warning, IssueType::MissingName, o.id()).object_type(o.object_type())
}

#[test]
fn test_routes() {
    let gtfs = gtfs_structures::Gtfs::new("test_data/check_name").unwrap();
    let issues = validate(&gtfs);
    let route_name_issues: Vec<_> = issues
        .iter()
        .filter(|issue| issue.object_id == "35".to_string())
        .collect();

    assert_eq!(1, route_name_issues.len());
    assert_eq!("35", route_name_issues[0].object_id);
    assert_eq!(IssueType::MissingName, route_name_issues[0].issue_type);
}

#[test]
fn test_stops() {
    let gtfs = gtfs_structures::Gtfs::new("test_data/check_name").unwrap();
    let issues = validate(&gtfs);
    let stop_name_issues: Vec<_> = issues
        .iter()
        .filter(|issue| issue.object_id == "close1".to_string())
        .collect();

    assert_eq!(1, stop_name_issues.len());
    assert_eq!("close1", stop_name_issues[0].object_id);
    assert_eq!(IssueType::MissingName, stop_name_issues[0].issue_type);
}

#[test]
fn test_agencies() {
    let gtfs = gtfs_structures::Gtfs::new("test_data/check_name").unwrap();
    let issues = validate(&gtfs);
    let agency_name_issues: Vec<_> = issues
        .iter()
        .filter(|issue| issue.object_id == "1".to_string())
        .collect();

    assert_eq!(1, agency_name_issues.len());
    assert_eq!("1", agency_name_issues[0].object_id);
    assert_eq!(IssueType::MissingName, agency_name_issues[0].issue_type);
}

#[test]
fn test_feed_info() {
    let gtfs = gtfs_structures::Gtfs::new("test_data/check_name").unwrap();
    let issues = validate(&gtfs);
    let publisher_name_issues: Vec<_> = issues
        .iter()
        .filter(|issue| issue.object_id == "".to_string())
        .collect();

    assert_eq!(1, publisher_name_issues.len());
    assert_eq!("", publisher_name_issues[0].object_id);
    assert_eq!(IssueType::MissingName, publisher_name_issues[0].issue_type);
}
