#[derive(Serialize)]
pub enum Severity {
    Fatal,
    Error,
    Warning,
    Information,
}

#[derive(Serialize)]
pub enum IssueType {
    UnusedStop,
}

#[derive(Serialize)]
pub struct Issue {
    pub severity: Severity,
    pub issue_type: IssueType,
    pub object_id: String,
    pub object_name: Option<String>,
    pub related_object_id: Option<String>,
}
