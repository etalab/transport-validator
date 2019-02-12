mod agency;
mod check_id;
mod check_name;
mod coordinates;
mod duration_distance;
pub mod issues;
mod metadatas;
mod route_type;
mod shapes;
mod unused_stop;

use std::collections::BTreeMap;

#[derive(Serialize, Debug)]
pub struct Response {
    pub metadata: Option<metadatas::Metadata>,
    pub validations: BTreeMap<issues::IssueType, Vec<issues::Issue>>,
}

pub fn validate_and_metada(gtfs: &gtfs_structures::Gtfs, max_issues: usize) -> Response {
    Response {
        metadata: Some(metadatas::extract_metadata(gtfs)),
        validations: validate_gtfs(gtfs, max_issues),
    }
}

pub fn validate_gtfs(
    gtfs: &gtfs_structures::Gtfs,
    max_issues: usize,
) -> BTreeMap<issues::IssueType, Vec<issues::Issue>> {
    let mut validations = BTreeMap::new();
    let issues = unused_stop::validate(gtfs)
        .into_iter()
        .chain(duration_distance::validate(gtfs))
        .chain(check_name::validate(gtfs))
        .chain(check_id::validate(gtfs))
        .chain(coordinates::validate(gtfs))
        .chain(route_type::validate(gtfs))
        .chain(shapes::validate(gtfs))
        .chain(agency::validate(gtfs));
    for issue in issues {
        validations
            .entry(issue.issue_type.clone())
            .or_insert_with(Vec::new)
            .push(issue);
    }

    for issues in validations.values_mut() {
        issues.truncate(max_issues)
    }

    validations
}

pub fn create_issues(input: &str, max_issues: usize) -> Response {
    log::info!("Starting validation: {}", input);
    let gtfs = if input.starts_with("http") {
        log::info!("Starting download of {}", input);
        let result = gtfs_structures::Gtfs::from_url(input);
        log::info!("Download done of {}", input);
        result
    } else if input.to_lowercase().ends_with(".zip") {
        gtfs_structures::Gtfs::from_zip(input)
    } else {
        gtfs_structures::Gtfs::new(input)
    };

    match gtfs {
        Ok(gtfs) => self::validate_and_metada(&gtfs, max_issues),
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

pub fn validate(input: &str, max_issues: usize) -> Result<String, failure::Error> {
    Ok(serde_json::to_string(&create_issues(input, max_issues))?)
}
