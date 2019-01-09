extern crate gtfs_structures;
use validators::issues::*;

pub fn validate(gtfs: &gtfs_structures::Gtfs) -> Vec<Issue> {
    
    gtfs.routes
        .iter()
        .filter(|&(_, route)| !has_name(route))
        .map(|(_, route)| Issue {
            severity: Severity::Error,
            issue_type: IssueType::MissingRouteName,
            object_id: route.id.to_owned(),
            object_name: None,
            related_object_id: None,
        }).collect()
}

fn has_name(route :&gtfs_structures::Route) -> bool {
    !route.short_name.is_empty() || !route.long_name.is_empty()
}

#[test]
fn test() {
    let gtfs = gtfs_structures::Gtfs::new("test_data/unused_stop").unwrap();
    let issues = validate(&gtfs);

    assert_eq!(1, issues.len());
    assert_eq!("CITY", issues[0].object_id);
    assert_eq!(IssueType::MissingRouteName, issues[0].issue_type);
}
