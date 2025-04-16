use crate::issues::*;

pub fn validate(gtfs: &gtfs_structures::Gtfs) -> Vec<Issue> {
    if gtfs.calendar.len() + gtfs.calendar_dates.len() == 0 {
        vec![Issue::new(Severity::Warning, IssueType::NoCalendar, "")]
    } else {
        vec![]
    }
}

#[test]
fn test_empty() {
    let gtfs = gtfs_structures::Gtfs::new("test_data/empty_calendar").unwrap();
    let issues = validate(&gtfs);
    let warning_issues: Vec<_> = issues
        .iter()
        .filter(|issue| issue.severity == Severity::Warning)
        .collect();

    assert_eq!(1, warning_issues.len());
    assert_eq!(IssueType::NoCalendar, warning_issues[0].issue_type);
}

#[test]
fn test_calendar_is_set() {
    let gtfs = gtfs_structures::Gtfs::new("test_data/feed_info").unwrap();
    let issues = validate(&gtfs);
    let no_calendar_issues: Vec<_> = issues
        .iter()
        .filter(|issue| issue.issue_type == IssueType::NoCalendar)
        .collect();

    assert!(no_calendar_issues.is_empty());
}
