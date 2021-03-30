use crate::issues::*;

pub fn validate(gtfs: &gtfs_structures::Gtfs) -> Vec<Issue> {
    let missing_price = gtfs
        .fare_attributes
        .values()
        .filter(|fare_attributes| !has_price(*fare_attributes))
        .map(|fare_attributes| make_issue(fare_attributes, IssueType::MissingPrice));
    let invalid_currency = gtfs
        .fare_attributes
        .values()
        .filter(|fare_attributes| !valid_currency(*fare_attributes))
        .map(|fare_attributes| make_issue(fare_attributes, IssueType::InvalidCurrency));
    let invalid_transfers = gtfs
        .fare_attributes
        .values()
        .filter(|fare_attributes| !valid_transfers(*fare_attributes))
        .map(|fare_attributes| make_issue(fare_attributes, IssueType::InvalidTransfers));
    let invalid_duration = gtfs
        .fare_attributes
        .values()
        .filter(|fare_attributes| !valid_duration(*fare_attributes))
        .map(|fare_attributes| make_issue(fare_attributes, IssueType::InvalidTransferDuration));
    missing_price
        .chain(invalid_currency)
        .chain(invalid_transfers)
        .chain(invalid_duration)
        .collect()
}

fn make_issue<T: gtfs_structures::Id>(o: &T, issue_type: IssueType) -> Issue {
    Issue::new(Severity::Error, issue_type, o.id()).object_type(gtfs_structures::ObjectType::Fare)
}

fn has_price(fare_attributes: &gtfs_structures::FareAttribute) -> bool {
    !fare_attributes.price.is_empty()
}

fn valid_currency(fare_attributes: &gtfs_structures::FareAttribute) -> bool {
    iso4217::alpha3(&fare_attributes.currency).is_some()
}

fn valid_transfers(fare_attributes: &gtfs_structures::FareAttribute) -> bool {
    !matches!(
        fare_attributes.transfers,
        gtfs_structures::Transfers::Other(_)
    )
}

fn valid_duration(fare_attributes: &gtfs_structures::FareAttribute) -> bool {
    fare_attributes.transfer_duration.is_none() || fare_attributes.transfer_duration >= Some(0)
}

#[test]
fn test_missing_price() {
    let gtfs = gtfs_structures::Gtfs::new("test_data/fare_attributes").unwrap();
    let issues = validate(&gtfs);
    let missing_price_issue: Vec<_> = issues
        .iter()
        .filter(|issue| issue.issue_type == IssueType::MissingPrice)
        .collect();

    assert_eq!(1, missing_price_issue.len());
    assert_eq!("50", missing_price_issue[0].object_id);
    assert_eq!(IssueType::MissingPrice, missing_price_issue[0].issue_type);
}

#[test]
fn test_valid_currency() {
    let gtfs = gtfs_structures::Gtfs::new("test_data/fare_attributes").unwrap();
    let issues = validate(&gtfs);
    let invalid_currency_issue: Vec<_> = issues
        .iter()
        .filter(|issue| issue.issue_type == IssueType::InvalidCurrency)
        .collect();

    assert_eq!(1, invalid_currency_issue.len());
    assert_eq!("61", invalid_currency_issue[0].object_id);
    assert_eq!(
        IssueType::InvalidCurrency,
        invalid_currency_issue[0].issue_type
    );
}

#[test]
fn test_valid_transfers() {
    let gtfs = gtfs_structures::Gtfs::new("test_data/fare_attributes").unwrap();
    let issues = validate(&gtfs);
    let invalid_transfers_issue: Vec<_> = issues
        .iter()
        .filter(|issue| issue.issue_type == IssueType::InvalidTransfers)
        .collect();

    assert_eq!(1, invalid_transfers_issue.len());
    assert_eq!("61", invalid_transfers_issue[0].object_id);
    assert_eq!(
        IssueType::InvalidTransfers,
        invalid_transfers_issue[0].issue_type
    );
}
