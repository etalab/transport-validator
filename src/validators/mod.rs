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

use std::collections::BTreeMap;

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
                .details(format!("{}", e).as_ref()),
            );
        }
    }

    for issue in issues {
        validations
            .entry(issue.issue_type.clone())
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

/// Returns a JSON with all the issues on the GTFS. Either takes an URL, a directory path or a .zip file as parameter.
pub fn validate(input: &str, max_issues: usize) -> Result<String, failure::Error> {
    Ok(serde_json::to_string(&create_issues(input, max_issues))?)
}
