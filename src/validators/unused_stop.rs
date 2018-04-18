extern crate gtfs_structures;
use validators::issues::*;
use std::collections::HashMap;

pub fn validate(gtfs: &gtfs_structures::Gtfs) -> Vec<Issue> {
    let mut use_count = HashMap::new();

    // A stop can be used for a stop time
    for stop_time in &gtfs.stop_times {
        let count = use_count.entry(stop_time.stop_id.to_owned()).or_insert(0);
        *count += 1;
    }

    // A stop can be the parent station
    for stop in &gtfs.stops {
        for parent in &stop.parent_station {
            let count = use_count.entry(parent.to_owned()).or_insert(0);
            *count += 1;
        }
    }

    gtfs.stops
        .iter()
        .filter(|stop| use_count.get(&stop.id).unwrap_or(&0) == &0)
        .map(|stop| Issue {
            severity: Severity::Error,
            issue_type: IssueType::UnusedStop,
            object_id: stop.id.to_owned(),
            object_name: Some(stop.stop_name.to_owned()),
            related_object_id: None,
        })
        .collect()
}

#[test]
fn test() {
    let gtfs = gtfs_structures::Gtfs::new("test_data/unused_stop").unwrap();
    let issues = validate(&gtfs);

    assert_eq!(1, issues.len());
    assert_eq!("BOGUS", issues[0].object_id);
}
