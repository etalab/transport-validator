use crate::validators::issues::*;

pub fn validate(gtfs: &gtfs_structures::Gtfs) -> Vec<Issue> {
    if gtfs.agencies.len() <= 1 {
        vec![]
    } else {
        gtfs.agencies
            .iter()
            .filter(|agency| !has_id(agency))
            .map(|agency| Issue {
                severity: Severity::Error,
                issue_type: IssueType::MissingId,
                object_id: "".to_owned(),
                object_name: Some(format!("{}", agency)),
                related_objects: vec![],
                details: None,
            })
            .collect()
    }
}

fn has_id(agency: &gtfs_structures::Agency) -> bool {
    match &agency.id {
        None => false,
        Some(id) => !id.is_empty(),
    }
}

#[test]
fn test_multiple_agencies() {
    let gtfs = gtfs_structures::Gtfs::new("test_data/agency_multiple").unwrap();
    let issues = validate(&gtfs);
    let agency_id_issues: Vec<_> = issues
        .iter()
        .filter(|issue| issue.issue_type == IssueType::MissingId)
        .collect();

    assert_eq!(1, agency_id_issues.len());
    assert_eq!("BIBUS", agency_id_issues[0].object_name.as_ref().unwrap());
    assert_eq!(IssueType::MissingId, agency_id_issues[0].issue_type);
}

#[test]
fn test_single_agency() {
    let gtfs = gtfs_structures::Gtfs::new("test_data/agency_single").unwrap();
    let issues = validate(&gtfs);
    let agency_id_issues: Vec<_> = issues
        .iter()
        .filter(|issue| issue.issue_type == IssueType::MissingId)
        .collect();

    assert_eq!(0, agency_id_issues.len());
}
