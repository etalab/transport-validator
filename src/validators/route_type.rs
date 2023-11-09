use crate::issues::{Issue, IssueType, Severity};
use gtfs_structures::{Gtfs, Route, RouteType};

pub fn validate(gtfs: &Gtfs) -> Vec<Issue> {
    gtfs.routes
        .iter()
        .filter_map(|(_, route)| get_non_standard_route_type(route))
        .map(|(route, route_type)| {
            Issue::new_with_obj(Severity::Information, IssueType::InvalidRouteType, route).details(
                &format!(
                    "The route type '{}' is not part of the main GTFS specification",
                    route_type
                ),
            )
        })
        .collect()
}

fn get_non_standard_route_type(route: &Route) -> Option<(&Route, i16)> {
    match route.route_type {
        RouteType::Other(rt) => Some((route, rt)),
        _ => None,
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
    use crate::custom_rules;

    let custom_rules = custom_rules::CustomRules {
        ..Default::default()
    };
    let validations =
        crate::validate::generate_validation("test_data/route_type_missing", 10, &custom_rules)
            .validations;
    let invalid_archive_validations = validations.get(&IssueType::UnloadableModel).unwrap();

    assert_eq!(1, invalid_archive_validations.len());
    assert_eq!(Severity::Fatal, invalid_archive_validations[0].severity);
    assert_eq!(
        IssueType::UnloadableModel,
        invalid_archive_validations[0].issue_type
    );
}
