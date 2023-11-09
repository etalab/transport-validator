use crate::{Issue, IssueType, Severity};

pub fn validate(raw_gtfs: &gtfs_structures::RawGtfs) -> Vec<Issue> {
    let mut issues = vec![];
    dbg!(&raw_gtfs.files);
    if let Some(parent_folder) = raw_gtfs
        .files
        .iter()
        .filter(|f| dbg!(f).ends_with("stops.txt"))
        .find_map(|f| {
            dbg!(f);
            dbg!(std::path::Path::new(f));
            dbg!(std::path::Path::new(f).parent());
            let parent = std::path::Path::new(f).parent();
            // Note: the parent of a file can be Some(""), so we filter this explicitly
            if parent.map(|p| p.as_os_str().is_empty()).unwrap_or(true) {
                None
            } else {
                parent
            }
        })
    {
        let parent = parent_folder.to_str().unwrap_or("invalid_parent_folder");
        issues.push(
            Issue::new(Severity::Error, IssueType::SubFolder, parent)
                .details(&format!("Data is contained in sub solder: {}", parent)),
        );
    }

    issues
}

#[cfg(test)]
mod tests {
    use std::{fs::File, io::Cursor, path::Path};

    use super::validate;
    use crate::{Issue, IssueType, Severity};
    use std::io::Write;
    use tempfile;
    use walkdir::WalkDir;
    use zip::{write::FileOptions, ZipWriter};

    #[test]
    fn valid_layout() {
        let gtfs = gtfs_structures::RawGtfs::new("test_data/stops").unwrap();
        let issues = dbg!(validate(&gtfs));

        assert_eq!(Vec::<Issue>::new(), issues);
    }

    // zip a directory
    fn zip_directory(dir_path: &str) -> Result<File, anyhow::Error> {
        let mut data = Vec::new();
        {
            let buff = Cursor::new(&mut data);
            let mut zw = ZipWriter::new(buff);
            let path = std::path::Path::new(dir_path);

            for entry in WalkDir::new(path) {
                let entry = entry?;
                let path = entry.path();
                let name = path
                    .strip_prefix(Path::new(dir_path))?
                    .as_os_str()
                    .to_str()
                    .unwrap();
                if path.is_dir() {
                    zw.add_directory(name, FileOptions::default())?;
                } else {
                    zw.start_file(name, FileOptions::default())?;
                    let file_data = std::fs::read(path)?;
                    zw.write_all(file_data.as_slice())?;
                }
            }
            zw.finish()?;
        }
        let mut file = tempfile::tempfile()?;
        file.write_all(data.as_slice())?;
        Ok(file)
    }

    #[test]
    fn valid_zip_layout() {
        let ziped_directory =
            zip_directory("test_data/stops").expect("impossible to zip directory");
        let gtfs = gtfs_structures::RawGtfs::from_reader(ziped_directory).unwrap();
        let issues = dbg!(validate(&gtfs));

        assert_eq!(Vec::<Issue>::new(), issues);
    }
    #[test]
    fn invalid_zip_layout() {
        let ziped_directory =
            zip_directory("test_data/sub_folder").expect("impossible to zip directory");
        let gtfs = gtfs_structures::RawGtfs::from_reader(ziped_directory).unwrap();

        // the GTFS should have been read without problem
        assert!(gtfs.stops.is_ok());
        assert!(gtfs.agencies.is_ok());
        assert!(gtfs.routes.is_ok());

        let issues = dbg!(validate(&gtfs));

        assert_eq!(1, issues.len());
        let first_issue = &issues[0];
        assert_eq!(IssueType::SubFolder, first_issue.issue_type);
        assert_eq!(Severity::Error, first_issue.severity);
        assert_eq!("gtfs", first_issue.object_id);
        assert_eq!(
            Some("Data is contained in sub solder: gtfs".to_string()),
            first_issue.details
        );

        let j = serde_json::to_string_pretty(first_issue).unwrap();

        assert_eq!(
            r#"{
  "severity": "Error",
  "issue_type": "SubFolder",
  "object_id": "gtfs",
  "related_objects": [],
  "details": "Data is contained in sub solder: gtfs"
}"#,
            j
        );
    }
}
