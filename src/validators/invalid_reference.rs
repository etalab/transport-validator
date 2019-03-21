use crate::validators::issues::{Issue, IssueType, Severity};
use gtfs_structures::ObjectType;
use std::collections::{HashMap, HashSet};

struct Ids {
    ids: HashMap<gtfs_structures::ObjectType, HashSet<String>>,
}

impl Ids {
    fn new(raw_gtfs: &gtfs_structures::RawGtfs) -> Self {
        let mut ids = HashMap::new();

        ids.insert(
            ObjectType::Trip,
            raw_gtfs.trips.iter().map(|t| t.id.clone()).collect(),
        );
        ids.insert(
            ObjectType::Stop,
            raw_gtfs.stops.iter().map(|t| t.id.clone()).collect(),
        );
        ids.insert(
            ObjectType::Route,
            raw_gtfs.routes.iter().map(|t| t.id.clone()).collect(),
        );
        ids.insert(
            ObjectType::Calendar,
            raw_gtfs
                .calendar
                .iter()
                .map(|t| t.id.clone())
                .chain(raw_gtfs.calendar_dates.iter().map(|t| t.service_id.clone()))
                .collect(),
        );
        Ids { ids }
    }

    fn check_ref(&self, id: &str, object_type: gtfs_structures::ObjectType) -> Option<Issue> {
        match self.ids[&object_type].contains(id) {
            true => None,
            false => Some(
                Issue::new(Severity::Fatal, IssueType::InvalidReference, id)
                    .object_type(object_type),
            ),
        }
    }

    fn check_stop_times(&self, raw_gtfs: &gtfs_structures::RawGtfs) -> Vec<Issue> {
        raw_gtfs
            .stop_times
            .iter()
            .filter_map(|st| {
                self.check_ref(&st.trip_id, gtfs_structures::ObjectType::Trip)
                    .map(|i| i.details("The trip is referenced by a stop time but does not exists"))
            })
            .chain(raw_gtfs.stop_times.iter().filter_map(|st| {
                self.check_ref(&st.stop_id, gtfs_structures::ObjectType::Stop)
                    .map(|i| i.details("The stop is referenced by a stop time but does not exists"))
            }))
            .map(|i| (i.object_id.clone(), i))
            .collect::<HashMap<_, _>>() // we don't want too many invalid reference dupplicate, so we keep one by object
            .into_iter()
            .map(|(_, i)| i)
            .collect()
    }

    fn check_trips(&self, raw_gtfs: &gtfs_structures::RawGtfs) -> Vec<Issue> {
        raw_gtfs
            .trips
            .iter()
            .filter_map(|trip| {
                self.check_ref(&trip.service_id, gtfs_structures::ObjectType::Calendar)
                    .map(|i| {
                        i.details("The service is referenced by a trip but does not exists")
                            .add_related_object(trip)
                    })
            })
            .chain(raw_gtfs.trips.iter().filter_map(|trip| {
                self.check_ref(&trip.route_id, gtfs_structures::ObjectType::Route)
                    .map(|i| {
                        i.details("The route is referenced by a trip but does not exists")
                            .add_related_object(trip)
                    })
            }))
            .map(|i| (i.object_id.clone(), i))
            .collect::<HashMap<_, _>>()
            .into_iter()
            .map(|(_, i)| i)
            .collect()
    }
}

/// Check that the links in the GTFS are valid
///
/// There are not that many link in the gtfs, we check:
/// * the stop times's stops and trips
/// * the trips routes and calendar
pub fn validate(raw_gtfs: &gtfs_structures::RawGtfs) -> Vec<Issue> {
    let id_container = Ids::new(raw_gtfs);

    id_container
        .check_stop_times(&raw_gtfs)
        .into_iter()
        .chain(id_container.check_trips(&raw_gtfs))
        .collect()
}

#[test]
fn test() {
    use crate::validators::issues::RelatedObject;
    let gtfs = gtfs_structures::RawGtfs::new("test_data/invalid_references").unwrap();
    let issues = validate(&gtfs);

    assert_eq!(issues.len(), 4);

    let unknown_stop_issue = issues
        .iter()
        .find(|i| i.object_id == "unknown_stop")
        .expect("impossible to find the issue");

    assert_eq!(unknown_stop_issue.issue_type, IssueType::InvalidReference);
    assert_eq!(unknown_stop_issue.object_type, Some(ObjectType::Stop));
    assert_eq!(
        unknown_stop_issue.details,
        Some("The stop is referenced by a stop time but does not exists".to_owned())
    );

    let unknown_trip_issue = issues
        .iter()
        .find(|i| i.object_id == "unknown_trip")
        .expect("impossible to find the issue");

    assert_eq!(unknown_trip_issue.issue_type, IssueType::InvalidReference);
    assert_eq!(unknown_trip_issue.object_type, Some(ObjectType::Trip));
    assert_eq!(
        unknown_trip_issue.details,
        Some("The trip is referenced by a stop time but does not exists".to_owned())
    );

    let unknown_service_issue = issues
        .iter()
        .find(|i| i.object_id == "unknown_service")
        .expect("impossible to find the issue");

    assert_eq!(
        unknown_service_issue.issue_type,
        IssueType::InvalidReference
    );
    assert_eq!(
        unknown_service_issue.object_type,
        Some(ObjectType::Calendar)
    );
    assert_eq!(
        unknown_service_issue.details,
        Some("The service is referenced by a trip but does not exists".to_owned())
    );
    assert_eq!(
        unknown_service_issue.related_objects,
        vec![RelatedObject {
            id: "trip_with_unknown_service".to_owned(),
            object_type: Some(ObjectType::Trip),
            name: Some("route id: AAMV, service id: unknown_service".to_owned())
        }]
    );

    let unknown_route_issue = issues
        .iter()
        .find(|i| i.object_id == "unkown_route")
        .expect("impossible to find the issue");

    assert_eq!(unknown_route_issue.issue_type, IssueType::InvalidReference);
    assert_eq!(unknown_route_issue.object_type, Some(ObjectType::Route));
    assert_eq!(
        unknown_route_issue.details,
        Some("The route is referenced by a trip but does not exists".to_owned())
    );
    assert_eq!(
        unknown_route_issue.related_objects,
        vec![RelatedObject {
            id: "trip_with_unknown_route".to_owned(),
            object_type: Some(ObjectType::Trip),
            name: Some("route id: unkown_route, service id: WE".to_owned())
        }]
    );
}