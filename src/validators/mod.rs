mod agency;
mod check_id;
mod check_name;
mod coordinates;
mod duplicate_stops;
mod duration_distance;
mod fare_attributes;
mod feed_info;
mod file_presence;
mod invalid_reference;
pub mod issues;
mod metadatas;
mod raw_gtfs;
mod route_type;
mod shapes;
mod unused_stop;
use itertools::Itertools;
use std::collections::BTreeMap;

fn pretty_print_error(error: failure::Error) -> String {
    error.iter_chain().map(|c| format!("{}", c)).join(": ")
}

#[derive(Serialize, Debug)]
/// Holds the issues and metadata about the GTFS.
pub struct Response {
    pub metadata: Option<metadatas::Metadata>,
    pub validations: BTreeMap<issues::IssueType, Vec<issues::Issue>>,
}

/// Validates the files of the GTFS and returns its metadata and issues.
pub fn validate_and_metadata(rgtfs: gtfs_structures::RawGtfs, max_issues: usize) -> Response {
    let mut validations = BTreeMap::new();
    let mut issues: Vec<_> = raw_gtfs::validate(&rgtfs)
        .into_iter()
        .chain(invalid_reference::validate(&rgtfs))
        .chain(file_presence::validate(&rgtfs))
        .collect();
    let mut metadata = metadatas::extract_metadata(&rgtfs);

    match gtfs_structures::Gtfs::try_from(rgtfs) {
        Ok(gtfs) => {
            issues.extend(
                unused_stop::validate(&gtfs)
                    .into_iter()
                    .chain(duration_distance::validate(&gtfs))
                    .chain(check_name::validate(&gtfs))
                    .chain(check_id::validate(&gtfs))
                    .chain(coordinates::validate(&gtfs))
                    .chain(route_type::validate(&gtfs))
                    .chain(shapes::validate(&gtfs))
                    .chain(agency::validate(&gtfs))
                    .chain(duplicate_stops::validate(&gtfs))
                    .chain(fare_attributes::validate(&gtfs))
                    .chain(feed_info::validate(&gtfs)),
            );
        }
        Err(e) => {
            issues.push(
                issues::Issue::new(
                    issues::Severity::Fatal,
                    issues::IssueType::UnloadableModel,
                    "A fatal error has occured while loading the model, many rules have not been checked",
                )
                .details(&pretty_print_error(e)),
            );
        }
    }

    for issue in issues {
        validations
            .entry(issue.issue_type)
            .or_insert_with(Vec::new)
            .push(issue);
    }

    for (issue_type, issues) in validations.iter_mut() {
        metadata
            .issues_count
            .insert(issue_type.clone(), issues.len());
        issues.truncate(max_issues);
    }

    Response {
        metadata: Some(metadata),
        validations,
    }
}

/// Returns a [Response] with every issue on the GTFS.
///
/// [Response]: struct.Response.html
pub fn create_issues(input: &str, max_issues: usize) -> Response {
    log::info!("Starting validation: {}", input);
    let raw_gtfs = if input.starts_with("http") {
        log::info!("Starting download of {}", input);
        let result = gtfs_structures::RawGtfs::from_url(input);
        log::info!("Download done of {}", input);
        result
    } else if input.to_lowercase().ends_with(".zip") {
        gtfs_structures::RawGtfs::from_zip(input)
    } else {
        gtfs_structures::RawGtfs::new(input)
    };
    process(raw_gtfs, max_issues)
}

pub fn process(
    raw_gtfs: Result<gtfs_structures::RawGtfs, failure::Error>,
    max_issues: usize,
) -> Response {
    match raw_gtfs {
        Ok(raw_gtfs) => self::validate_and_metadata(raw_gtfs, max_issues),
        Err(e) => {
            let mut validations = BTreeMap::new();
            validations.insert(
                issues::IssueType::InvalidArchive,
                vec![issues::Issue::new(
                    issues::Severity::Fatal,
                    issues::IssueType::InvalidArchive,
                    "",
                )
                .details(format!("{}", e).as_ref())],
            );
            Response {
                metadata: None,
                validations,
            }
        }
    }
}

pub fn create_issues_from_reader<T: std::io::Read + std::io::Seek>(
    reader: T,
    max_issues: usize,
) -> Response {
    let g = gtfs_structures::RawGtfs::from_reader(reader);
    process(g, max_issues)
}

/// Returns a JSON with all the issues on the GTFS. Either takes an URL, a directory path or a .zip file as parameter.
pub fn validate(input: &str, max_issues: usize) -> Result<String, failure::Error> {
    Ok(serde_json::to_string(&create_issues(input, max_issues))?)
}

// Test reading a GTFS with a broken stops.txt file.
// we should have the RawGTFS rules applied, and a `Fatal` error on the stops.txt file
#[test]
fn test_invalid_stop_points() {
    let issues = create_issues("test_data/invalid_stop_file", 1000);

    let unloadable_model_errors = &issues.validations[&issues::IssueType::UnloadableModel];

    assert_eq!(unloadable_model_errors.len(), 1);
    let unloadable_model_error = &unloadable_model_errors[0];

    assert_eq!(unloadable_model_error, &issues::Issue {
            severity: issues::Severity::Fatal,
            issue_type: issues::IssueType::UnloadableModel,
            object_id: "A fatal error has occured while loading the model, many rules have not been checked".to_string(),
            object_type: None,
            object_name: None,
            related_objects: vec![],
            details: Some(
                "error while reading stops.txt: CSV deserialize error: record 12 (line: 13, byte: 739): invalid float literal".to_string()
            )
        });

    // a nice feature is that even if the model was unloadable, we can check some rules
    // there we can check that a trip id is missing (but we don't check the stops's id, as they are all missing, since we can't read the stops.txt file)
    assert_eq!(
        issues.validations[&issues::IssueType::InvalidReference],
        vec![issues::Issue {
            severity: issues::Severity::Fatal,
            issue_type: issues::IssueType::InvalidReference,
            object_id: "AAMV".to_string(),
            object_type: Some(gtfs_structures::ObjectType::Route),
            object_name: None,
            related_objects: vec![issues::RelatedObject {
                id: "AAMV4".to_string(),
                object_type: Some(gtfs_structures::ObjectType::Trip),
                name: Some("route id: AAMV, service id: WE".to_string())
            }],
            details: Some("The route is referenced by a trip but does not exists".to_string())
        }]
    );
}
