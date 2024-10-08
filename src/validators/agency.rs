use crate::issues::*;

pub fn validate(gtfs: &gtfs_structures::Gtfs) -> Vec<Issue> {
    let missing_url = gtfs
        .agencies
        .iter()
        .filter(|agency| !has_url(agency))
        .map(|agency| Issue::new_with_obj(Severity::Error, IssueType::MissingUrl, agency));
    let invalid_url =
        gtfs.agencies
            .iter()
            .filter(|agency| has_url(agency) && !valid_url(agency))
            .map(|agency| {
                Issue::new_with_obj(Severity::Error, IssueType::InvalidUrl, agency).details(
                    &format!("The agency_url (in agency.txt) {} is invalid", agency.url),
                )
            });
    let invalid_tz = gtfs
        .agencies
        .iter()
        .filter(|agency| !valid_timezone(agency))
        .map(|agency| Issue::new_with_obj(Severity::Error, IssueType::InvalidTimezone, agency));
    missing_url.chain(invalid_url).chain(invalid_tz).collect()
}

fn has_url(agency: &gtfs_structures::Agency) -> bool {
    !agency.url.is_empty()
}

fn valid_url(agency: &gtfs_structures::Agency) -> bool {
    // https://gtfs.org/schedule/reference/#field-types
    // URL - A fully qualified URL that includes http:// or https://
    match url::Url::parse(agency.url.as_ref()) {
        Ok(url) => ["https", "http"].contains(&url.scheme()),
        _ => false,
    }
}

fn valid_timezone(agency: &gtfs_structures::Agency) -> bool {
    let tz: Result<chrono_tz::Tz, _> = agency.timezone.parse();
    tz.is_ok()
}

#[test]
fn test_missing_url() {
    let gtfs = gtfs_structures::Gtfs::new("test_data/agency").unwrap();
    let issues = validate(&gtfs);
    let missing_url_issue: Vec<_> = issues
        .iter()
        .filter(|issue| issue.issue_type == IssueType::MissingUrl)
        .collect();
    let invalid_url_count = issues
        .iter()
        .filter(|issue| issue.issue_type == IssueType::InvalidUrl)
        .filter(|issue| issue.object_name == Some("BIBUS".to_string()))
        .count();

    assert_eq!(1, missing_url_issue.len());
    assert_eq!("BIBUS", missing_url_issue[0].object_name.as_ref().unwrap());
    assert_eq!(IssueType::MissingUrl, missing_url_issue[0].issue_type);
    assert_eq!(0, invalid_url_count);
}

#[test]
fn test_valid_timezone() {
    let gtfs = gtfs_structures::Gtfs::new("test_data/agency").unwrap();
    let issues = validate(&gtfs);
    let invalid_tz_issue: Vec<_> = issues
        .iter()
        .filter(|issue| issue.issue_type == IssueType::InvalidTimezone)
        .collect();

    assert_eq!(1, invalid_tz_issue.len());
    assert_eq!("BIBUS", invalid_tz_issue[0].object_name.as_ref().unwrap());
    assert_eq!(IssueType::InvalidTimezone, invalid_tz_issue[0].issue_type);
}

#[test]
fn test_valid_url() {
    let gtfs = gtfs_structures::Gtfs::new("test_data/agency").unwrap();
    let issues = validate(&gtfs);
    let invalid_url_issue: Vec<_> = issues
        .iter()
        .filter(|issue| issue.issue_type == IssueType::InvalidUrl)
        .filter(|issue| issue.object_name == Some("Ter".to_string()))
        .collect();

    assert_eq!(1, invalid_url_issue.len());
    assert_eq!("2", invalid_url_issue[0].object_id);
    assert_eq!(IssueType::InvalidUrl, invalid_url_issue[0].issue_type);
}
