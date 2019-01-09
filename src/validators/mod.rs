mod duration_distance;
pub mod issues;
mod metadatas;
mod unused_stop;
mod route_name;

#[derive(Serialize, Debug)]
pub struct Response {
    pub metadata: metadatas::Metadata,
    pub validations: Vec<issues::Issue>,
}

#[derive(Serialize, Debug)]
pub struct Fatal {
    pub fatal_error: issues::Issue,
}

pub fn validate_and_metada(gtfs: &gtfs_structures::Gtfs) -> Response {
    Response {
        metadata: metadatas::extract_metadata(gtfs),
        validations: validate_gtfs(gtfs),
    }
}

pub fn validate_gtfs(gtfs: &gtfs_structures::Gtfs) -> Vec<issues::Issue> {
    unused_stop::validate(gtfs)
        .into_iter()
        .chain(duration_distance::validate(gtfs))
        .chain(route_name::validate(gtfs))
        .collect()
}

pub fn validate(input: &str) -> Result<String, failure::Error> {
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

    gtfs.map(|gtfs| self::validate_and_metada(&gtfs))
        .and_then(|validation| Ok(serde_json::to_string(&validation)?))
        .or_else(|err| {
            Ok(serde_json::to_string(&Fatal {
                fatal_error: issues::Issue {
                    severity: issues::Severity::Fatal,
                    issue_type: issues::IssueType::InvalidArchive,
                    object_id: format!("{}", err),
                    object_name: None,
                    related_object_id: None,
                },
            })?)
        })
}
