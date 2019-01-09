use crate::validators::issues::*;

pub fn validate(gtfs: &gtfs_structures::Gtfs) -> Vec<Issue> {
    let r = gtfs.routes
            .values()
            .filter(|route| !has_id(*route))
            .map(|route| Issue {
                severity: Severity::Error,
                issue_type: IssueType::MissingId,
                object_id: route.id.to_owned(),
                object_name: Some(format!("{}", route)),
                related_object_id: None,
                details: None,
            });
    let t = gtfs.trips
                .values()
                .filter(|trip| !has_id(*trip))
                .map(|trip| Issue {
                    severity: Severity::Error,
                    issue_type: IssueType::MissingId,
                    object_id: trip.id.to_owned(),
                    object_name: Some(format!("{}", trip)),
                    related_object_id: None,
                    details: None,
                });
    let c = gtfs.calendar
                .values()
                .filter(|calendar| !has_id(*calendar))
                .map(|calendar| Issue {
                    severity: Severity::Error,
                    issue_type: IssueType::MissingId,
                    object_id: calendar.id.to_owned(),
                    object_name: Some(format!("{}", calendar)),
                    related_object_id: None,
                    details: None,
                });
    let s = gtfs.stops
                .values()
                .filter(|&stop| !has_id(&**stop))
                .map(|stop| Issue {
                    severity: Severity::Error,
                    issue_type: IssueType::MissingId,
                    object_id: stop.id.to_owned(),
                    object_name: Some(format!("{}", stop)),
                    related_object_id: None,
                    details: None,
                });
    r.chain(t)
        .chain(c)
        .chain(s)
        .collect()
}

fn has_id(object: &gtfs_structures::Id) -> bool {
    !object.id().is_empty()
}

#[test]
fn test() {
    let gtfs = gtfs_structures::Gtfs::new("test_data/check_id").unwrap();
    let issues = validate(&gtfs);
    let stop_id_issues: Vec<_> = issues.iter()
        .filter(|issue| issue.object_name == Some("Null Island".to_string()))
        .collect();
    
    assert_eq!(1, stop_id_issues.len());
    assert_eq!("Null Island", stop_id_issues[0].object_name.as_ref().unwrap());
    assert_eq!(IssueType::MissingId, stop_id_issues[0].issue_type);
}