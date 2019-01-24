mod agency_id;
mod check_id;
mod coordinates;
mod duration_distance;
pub mod issues;
mod metadatas;
mod route_name;
mod route_type;
mod shapes;
mod unused_stop;

#[derive(Serialize, Debug)]
pub struct Response {
    pub metadata: Option<metadatas::Metadata>,
    pub validations: Vec<issues::Issue>,
}

#[derive(Serialize, Debug)]
pub struct Fatal {
    pub fatal_error: issues::Issue,
}

pub fn validate_and_metada(gtfs: &gtfs_structures::Gtfs) -> Response {
    Response {
        metadata: Some(metadatas::extract_metadata(gtfs)),
        validations: validate_gtfs(gtfs),
    }
}

pub fn validate_gtfs(gtfs: &gtfs_structures::Gtfs) -> Vec<issues::Issue> {
    unused_stop::validate(gtfs)
        .into_iter()
        .chain(duration_distance::validate(gtfs))
        .chain(route_name::validate(gtfs))
        .chain(check_id::validate(gtfs))
        .chain(coordinates::validate(gtfs))
        .chain(route_type::validate(gtfs))
        .chain(shapes::validate(gtfs))
        .chain(agency_id::validate(gtfs))
        .collect()
}

pub fn create_issues(input: &str) -> Response {
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
        Ok(gtfs) => self::validate_and_metada(&gtfs),
        Err(e) => Response {
            metadata: None,
            validations: vec![issues::Issue {
                severity: issues::Severity::Fatal,
                issue_type: issues::IssueType::InvalidArchive,
                object_id: "".to_string(),
                object_name: None,
                related_objects: vec![],
                details: Some(format!("{}", e)),
            }],
        },
    }
}

pub fn validate(input: &str) -> Result<String, failure::Error> {
    Ok(serde_json::to_string(&create_issues(input))?)
}
