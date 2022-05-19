use crate::issues::*;

pub fn validate(gtfs: &gtfs_structures::Gtfs) -> Vec<Issue> {
    let r = gtfs.routes.values().filter_map(valid_id);
    let t = gtfs.trips.values().filter_map(valid_id);
    let c = gtfs.calendar.values().filter_map(valid_id);
    let st = gtfs
        .stops
        .values()
        .filter_map(|stop| valid_id(stop.as_ref()));
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
    if gtfs.agencies.len() > 1 {
        gtfs.agencies
            .iter()
            .filter_map(valid_id)
            .chain(chain)
            .collect()
    } else {
        chain.collect()
    }
}

fn valid_id<T: gtfs_structures::Id + gtfs_structures::Type + std::fmt::Display>(
    o: &T,
) -> Option<Issue> {
    if !o.id().is_ascii() {
        Some(Issue::new_with_obj(
            Severity::Warning,
            IssueType::IdNotAscii,
            o,
        ))
    } else if o.id().is_empty() {
        Some(Issue::new_with_obj(
            Severity::Error,
            IssueType::MissingId,
            o,
        ))
    } else {
        None
    }
}

#[test]
fn test_empty() {
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

#[test]
fn test_ascii() {
    let gtfs = gtfs_structures::Gtfs::new("test_data/check_id").unwrap();
    let issues = validate(&gtfs);
    let stop_id_issues: Vec<_> = issues
        .iter()
        .filter(|issue| issue.object_name == Some("AsciiParty".to_string()))
        .collect();

    assert_eq!(1, stop_id_issues.len());
    assert_eq!(
        "AsciiParty",
        stop_id_issues[0].object_name.as_ref().unwrap()
    );
    assert_eq!(IssueType::IdNotAscii, stop_id_issues[0].issue_type);
}
