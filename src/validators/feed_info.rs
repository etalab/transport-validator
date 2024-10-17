use crate::issues::*;

pub fn validate(gtfs: &gtfs_structures::Gtfs) -> Vec<Issue> {
    let missing_url = gtfs
        .feed_info
        .iter()
        .filter(is_missing_url)
        .map(|feed_info| make_issue(feed_info, Severity::Error, IssueType::MissingUrl));
    let invalid_url = gtfs
        .feed_info
        .iter()
        .filter(|fi| !is_missing_url(fi) && is_invalid_url(fi))
        .map(|feed_info| {
            make_issue(feed_info, Severity::Error, IssueType::InvalidUrl).details(&format!(
                "The feed_publisher_url (in feed_info.txt) {} is invalid",
                feed_info.url
            ))
        });
    let missing_lang = gtfs
        .feed_info
        .iter()
        .filter(missing_lang)
        .map(|feed_info| make_issue(feed_info, Severity::Warning, IssueType::MissingLanguage));
    let invalid_lang = gtfs.feed_info.iter().filter(invalid_lang).map(|feed_info| {
        make_issue(feed_info, Severity::Warning, IssueType::InvalidLanguage)
            .details(&format!("Language code {} does not exist", feed_info.lang))
    });
    missing_url
        .chain(invalid_url)
        .chain(missing_lang)
        .chain(invalid_lang)
        .collect()
}

fn make_issue(
    feed: &gtfs_structures::FeedInfo,
    severity: Severity,
    issue_type: IssueType,
) -> Issue {
    Issue::new(severity, issue_type, "").name(&format!("{}", feed))
}

fn is_missing_url(feed: &&gtfs_structures::FeedInfo) -> bool {
    feed.url.is_empty()
}

fn is_invalid_url(feed: &&gtfs_structures::FeedInfo) -> bool {
    // https://gtfs.org/schedule/reference/#field-types
    // URL - A fully qualified URL that includes http:// or https://
    !url::Url::parse(feed.url.as_ref())
        .map(|url| ["https", "http"].contains(&url.scheme()))
        .unwrap_or(false)
}

fn missing_lang(feed: &&gtfs_structures::FeedInfo) -> bool {
    feed.lang.is_empty()
}

fn invalid_lang(feed: &&gtfs_structures::FeedInfo) -> bool {
    let lang = feed.lang.to_lowercase();
    let len = lang.len();
    !match len {
        2 => isolang::Language::from_639_1(&lang).is_some(),
        3 => isolang::Language::from_639_3(&lang).is_some(),
        4..=11 => isolang::Language::from_locale(&lang).is_some(),
        _ => false,
    }
}

#[test]
fn test_missing_url() {
    let gtfs = gtfs_structures::Gtfs::new("test_data/feed_info").unwrap();
    let issues = validate(&gtfs);
    let missing_url_issue: Vec<_> = issues
        .iter()
        .filter(|issue| issue.issue_type == IssueType::MissingUrl)
        .collect();

    assert_eq!(1, missing_url_issue.len());
    assert_eq!("SNCF", missing_url_issue[0].object_name.as_ref().unwrap());
    assert_eq!(IssueType::MissingUrl, missing_url_issue[0].issue_type);
}

#[test]
fn test_valid_url() {
    let gtfs = gtfs_structures::Gtfs::new("test_data/feed_info").unwrap();
    let issues = validate(&gtfs);
    let invalid_url_issue: Vec<_> = issues
        .iter()
        .filter(|issue| issue.issue_type == IssueType::InvalidUrl)
        .collect();

    assert_eq!(1, invalid_url_issue.len());
    assert_eq!(IssueType::InvalidUrl, invalid_url_issue[0].issue_type);
}

#[test]
fn test_missing_lang() {
    let gtfs = gtfs_structures::Gtfs::new("test_data/feed_info").unwrap();
    let issues = validate(&gtfs);
    let missing_lang_issue: Vec<_> = issues
        .iter()
        .filter(|issue| issue.issue_type == IssueType::MissingLanguage)
        .collect();

    assert_eq!(1, missing_lang_issue.len());
    assert_eq!(IssueType::MissingLanguage, missing_lang_issue[0].issue_type);
}

#[test]
fn test_valid_lang() {
    let gtfs = gtfs_structures::Gtfs::new("test_data/feed_info").unwrap();
    let issues = validate(&gtfs);
    let invalid_lang_issue: Vec<_> = issues
        .iter()
        .filter(|issue| issue.issue_type == IssueType::InvalidLanguage)
        .filter(|issue| issue.object_name == Some("SNCF".to_string()))
        .collect();
    assert_eq!(1, invalid_lang_issue.len());
    assert_eq!(IssueType::InvalidLanguage, invalid_lang_issue[0].issue_type);
}

#[test]
fn test_valid_lang_upper() {
    assert!(!invalid_lang(&&gtfs_structures::FeedInfo {
        name: "bob".to_owned(),
        url: "http://bob.com".to_owned(),
        lang: "FR".to_owned(),
        start_date: None,
        end_date: None,
        version: None,
        contact_email: None,
        contact_url: None,
        default_lang: None
    }));
}
