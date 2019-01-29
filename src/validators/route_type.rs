use crate::validators::issues::{Issue, IssueType, Severity};

pub fn validate(gtfs: &gtfs_structures::Gtfs) -> Vec<Issue> {
    gtfs.routes
        .iter()
        .filter(|&(_, route)| !has_valid_route_type(route))
        .map(|(_, route)| Issue::new_with_obj(Severity::Error, IssueType::InvalidRouteType, route))
        .collect()
}

fn has_valid_route_type(route: &gtfs_structures::Route) -> bool {
    match route.route_type {
        gtfs_structures::RouteType::Other(_) => false,
        _ => true,
    }
}

#[test]
fn test_valid() {
    let gtfs = gtfs_structures::Gtfs::new("test_data/route_type_invalid").unwrap();
    let issues = validate(&gtfs);

    assert_eq!(1, issues.len());
    assert_eq!("CITY", issues[0].object_id);
    assert_eq!(IssueType::InvalidRouteType, issues[0].issue_type);
}

#[test]
fn test_missing() {
    let validations = crate::validators::create_issues("test_data/route_type_missing").validations;
    let invalid_archive_validations = validations.get(&IssueType::InvalidArchive).unwrap();

    assert_eq!(1, invalid_archive_validations.len());
    assert_eq!(Severity::Fatal, invalid_archive_validations[0].severity);
    assert_eq!(
        IssueType::InvalidArchive,
        invalid_archive_validations[0].issue_type
    );
}
