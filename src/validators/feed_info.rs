use crate::validators::issues::*;

pub fn validate(gtfs: &gtfs_structures::Gtfs) -> Vec<Issue> {
    let missing_url = gtfs
        .feed_info
        .iter()
        .filter(|feed_info| !has_url(feed_info))
        .map(|feed_info| make_issue(feed_info, IssueType::MissingUrl));
    let invalid_url = gtfs
        .feed_info
        .iter()
        .filter(|feed_info| !valid_url(feed_info))
        .map(|feed_info| {
            make_issue(feed_info, IssueType::InvalidUrl)
                .details(&format!("Publisher url {} is invalid", feed_info.url))
        });
    let missing_lang = gtfs
        .feed_info
        .iter()
        .filter(|feed_info| !has_lang(feed_info))
        .map(|feed_info| make_issue(feed_info, IssueType::MissingLanguage));
    let invalid_lang = gtfs
        .feed_info
        .iter()
        .filter(|feed_info| !valid_lang(feed_info))
        .map(|feed_info| {
            make_issue(feed_info, IssueType::InvalidLanguage)
                .details(&format!("Language code {} does not exist", feed_info.lang))
        });
    missing_url
        .chain(invalid_url)
        .chain(missing_lang)
        .chain(invalid_lang)
        .collect()
}

fn make_issue(feed: &gtfs_structures::FeedInfo, issue_type: IssueType) -> Issue {
    Issue::new(Severity::Error, issue_type, "").name(&format!("{}", feed))
}

fn has_url(feed: &gtfs_structures::FeedInfo) -> bool {
    !feed.url.is_empty()
}

fn valid_url(feed: &gtfs_structures::FeedInfo) -> bool {
    match url::Url::parse(feed.url.as_ref()) {
        Ok(url) => vec!["https", "http", "ftp"].contains(&url.scheme()),
        _ => false,
    }
}

fn has_lang(feed: &gtfs_structures::FeedInfo) -> bool {
    !feed.lang.is_empty()
}

fn valid_lang(feed: &gtfs_structures::FeedInfo) -> bool {
    let len = feed.lang.len();
    match len {
        2 => !isolang::Language::from_639_1(&feed.lang).is_none(),
        3 => !isolang::Language::from_639_3(&feed.lang).is_none(),
        4...11 => !isolang::Language::from_locale(&feed.lang).is_none(),
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
        .filter(|issue| issue.object_name == Some("BIBUS".to_string()))
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
        .filter(|issue| issue.object_name == Some("BIBUS".to_string()))
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
