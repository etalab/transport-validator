#[derive(Serialize, Debug, Eq, PartialEq)]
pub enum Severity {
    Fatal,
    Error,
    Warning,
    Information,
}

#[derive(Serialize, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum IssueType {
    UnusedStop,
    Slow,
    ExcessiveSpeed,
    NegativeTravelTime,
    CloseStops,
    NullDuration,
    InvalidReference,
    InvalidArchive,
    MissingRouteName,
    MissingId,
    MissingCoordinates,
    InvalidCoordinates,
    InvalidRouteType,
}

#[derive(Serialize, Debug)]
pub struct Issue {
    pub severity: Severity,
    pub issue_type: IssueType,
    pub object_id: String,
    pub object_name: Option<String>,
    pub related_object_id: Option<String>,
    pub details: Option<String>,
}

impl Issue {
    pub fn new(severity: Severity, issue_type: IssueType, id: &str) -> Self {
        Issue {
            severity,
            issue_type,
            object_id: id.to_owned(),
            object_name: None,
            related_object_id: None,
            details: None,
        }
    }
    pub fn new_with_obj<T: gtfs_structures::Id + std::fmt::Display>(
        severity: Severity,
        issue_type: IssueType,
        o: &T,
    ) -> Self {
        Issue {
            severity,
            issue_type,
            object_id: o.id().to_owned(),
            object_name: Some(format!("{}", o)),
            related_object_id: None,
            details: None,
        }
    }

    pub fn details(mut self, d: &str) -> Self {
        self.details = Some(d.to_owned());
        self
    }
    pub fn related_object<T: gtfs_structures::Id>(mut self, o: &T) -> Self {
        self.related_object_id = Some(o.id().to_owned());
        self
    }
}
