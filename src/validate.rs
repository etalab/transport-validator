use crate::{custom_rules, issues, metadatas, validators};
use serde::Serialize;
use std::collections::BTreeMap;
use std::convert::TryFrom;
use std::error::Error;

fn create_unloadable_model_error(error: gtfs_structures::Error) -> issues::Issue {
    let msg = if let Some(inner) = error.source() {
        format!("{}: {}", error, inner)
    } else {
        format!("{}", error)
    };
    let mut issue = issues::Issue::new(
        issues::Severity::Fatal,
        issues::IssueType::UnloadableModel,
        "A fatal error has occured while loading the model, many rules have not been checked",
    )
    .details(&msg);

    if let gtfs_structures::Error::CSVError {
        file_name,
        source,
        line_in_error,
    } = error
    {
        issue.related_file = Some(issues::RelatedFile {
            file_name,
            line: source
                .position()
                .and_then(|p| line_in_error.map(|l| (p.line(), l)))
                .map(|(line_number, line_in_error)| issues::RelatedLine {
                    line_number,
                    headers: line_in_error.headers,
                    values: line_in_error.values,
                }),
        });
    }
    issue
}

#[derive(Serialize, Debug)]
/// Holds the issues and metadata about the GTFS.
pub struct Response {
    pub metadata: Option<metadatas::Metadata>,
    pub validations: BTreeMap<issues::IssueType, Vec<issues::Issue>>,
}

/// Validates the files of the GTFS and returns its metadata and issues.
pub fn validate_and_metadata(
    rgtfs: gtfs_structures::RawGtfs,
    max_issues: usize,
    custom_rules: &custom_rules::CustomRules,
) -> Response {
    let mut validations = BTreeMap::new();
    let mut issues: Vec<_> = validators::raw_gtfs::validate(&rgtfs)
        .into_iter()
        .chain(validators::invalid_reference::validate(&rgtfs))
        .chain(validators::file_presence::validate(&rgtfs))
        .chain(validators::sub_folder::validate(&rgtfs))
        .collect();
    let mut metadata = metadatas::extract_metadata(&rgtfs);

    match gtfs_structures::Gtfs::try_from(rgtfs) {
        Ok(ref gtfs) => {
            issues.extend(
                validators::unused_stop::validate(gtfs)
                    .into_iter()
                    .chain(validators::duration_distance::validate(gtfs, custom_rules))
                    .chain(validators::check_name::validate(gtfs))
                    .chain(validators::check_id::validate(gtfs))
                    .chain(validators::stops::validate(gtfs))
                    .chain(validators::route_type::validate(gtfs))
                    .chain(validators::shapes::validate(gtfs))
                    .chain(validators::agency::validate(gtfs))
                    .chain(validators::calendar::validate(gtfs))
                    .chain(validators::duplicate_stops::validate(gtfs))
                    .chain(validators::fare_attributes::validate(gtfs))
                    .chain(validators::feed_info::validate(gtfs))
                    .chain(validators::stop_times::validate(gtfs))
                    .chain(validators::interpolated_stoptimes::validate(gtfs))
                    .chain(validators::unusable_trip::validate(gtfs)),
            );
            issues
                .iter_mut()
                .for_each(|issue| issue.push_related_geojson(gtfs));

            // advanced_metadata::enrich_advanced_metadata(&mut metadata, gtfs);
            metadata.enrich_with_advanced_infos(gtfs);
        }
        Err(e) => {
            issues.push(create_unloadable_model_error(e));
        }
    }

    for issue in issues {
        validations
            .entry(issue.issue_type)
            .or_insert_with(Vec::new)
            .push(issue);
    }

    for (issue_type, issues) in validations.iter_mut() {
        metadata.issues_count.insert(*issue_type, issues.len());
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
pub fn generate_validation(
    input: &str,
    max_issues: usize,
    custom_rules: &custom_rules::CustomRules,
) -> Response {
    log::info!("Starting validation: {}", input);
    let raw_gtfs = gtfs_structures::RawGtfs::new(input);
    process(raw_gtfs, max_issues, custom_rules)
}

pub fn process(
    raw_gtfs: Result<gtfs_structures::RawGtfs, gtfs_structures::Error>,
    max_issues: usize,
    custom_rules: &custom_rules::CustomRules,
) -> Response {
    match raw_gtfs {
        Ok(raw_gtfs) => self::validate_and_metadata(raw_gtfs, max_issues, custom_rules),
        Err(e) => {
            let mut validations = BTreeMap::new();
            validations.insert(
                issues::IssueType::InvalidArchive,
                vec![issues::Issue::new(
                    issues::Severity::Fatal,
                    issues::IssueType::InvalidArchive,
                    "",
                )
                .details(&format!("{}", e))],
            );
            Response {
                metadata: None,
                validations,
            }
        }
    }
}

pub fn generate_validation_from_reader<T: std::io::Read + std::io::Seek>(
    reader: T,
    max_issues: usize,
    custom_rules: &custom_rules::CustomRules,
) -> Response {
    let g = gtfs_structures::RawGtfs::from_reader(reader);
    process(g, max_issues, custom_rules)
}

/// Returns a JSON with all the issues on the GTFS. Either takes an URL, a directory path or a .zip file as parameter.
pub fn validate(
    input: &str,
    max_issues: usize,
    custom_rules: &custom_rules::CustomRules,
) -> Result<String, anyhow::Error> {
    Ok(serde_json::to_string(&generate_validation(
        input,
        max_issues,
        custom_rules,
    ))?)
}

// Test reading a GTFS with a broken stops.txt file.
// we should have the RawGTFS rules applied, and a `Fatal` error on the stops.txt file
#[test]
fn test_invalid_stop_points() {
    let custom_rules = custom_rules::CustomRules {
        ..Default::default()
    };
    let issues = generate_validation("test_data/invalid_stop_file", 1000, &custom_rules);

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
                "impossible to read csv file \'stops.txt\': CSV deserialize error: record 12 (line: 13, byte: 739): invalid float literal".to_string()
            ),
            related_file: Some(issues::RelatedFile {
                file_name: "stops.txt".to_owned(),
                line: Some(issues::RelatedLine {
                        line_number: 13,
                        headers: vec!["stop_id", "stop_name", "stop_desc", "stop_lat", "stop_lon", "zone_id", "stop_url", "location_type", "parent_station"].into_iter().map(|s| s.to_owned()).collect(),
                        values: vec!["stop_with_bad_coord", "Moo", "", "baaaaaad_coord", "-116.40094", "", "", "", "1"].into_iter().map(|s| s.to_owned()).collect()
                    }),
            }),
            geojson: None
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
            related_file: None,
            related_objects: vec![issues::RelatedObject {
                id: "AAMV4".to_string(),
                object_type: Some(gtfs_structures::ObjectType::Trip),
                name: Some("route id: AAMV, service id: WE".to_string())
            }],
            details: Some("The route is referenced by a trip but does not exist".to_string()),
            geojson: None
        }]
    );
}
