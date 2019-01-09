#[derive(Serialize, Debug, Eq, PartialEq)]
pub enum Severity {
    Fatal,
    Error,
    Warning,
    Information,
}

#[derive(Serialize, Debug, Eq, PartialEq)]
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
