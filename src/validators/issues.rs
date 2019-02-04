#[derive(Serialize, Debug, Eq, PartialEq)]
pub enum Severity {
    Fatal,
    Error,
    Warning,
    #[allow(dead_code)]
    Information,
}

#[derive(Serialize, Debug, Eq, PartialEq, Ord, PartialOrd, Clone)]
pub enum IssueType {
    UnusedStop,
    Slow,
    ExcessiveSpeed,
    NegativeTravelTime,
    CloseStops,
    NullDuration,
    InvalidReference,
    InvalidArchive,
    MissingName,
    MissingId,
    MissingCoordinates,
    InvalidCoordinates,
    InvalidRouteType,
    MissingUrl,
    InvalidUrl,
    InvalidTimezone,
}

#[derive(Serialize, Debug)]
pub struct RelatedObject {
    pub id: String,
    pub name: Option<String>,
}

#[derive(Serialize, Debug)]
pub struct Issue {
    pub severity: Severity,
    pub issue_type: IssueType,
    pub object_id: String,
    pub object_type: Option<gtfs_structures::ObjectType>,
    pub object_name: Option<String>,
    pub related_objects: Vec<RelatedObject>,
    pub details: Option<String>,
}

impl Issue {
    pub fn new(severity: Severity, issue_type: IssueType, id: &str) -> Self {
        Issue {
            severity,
            issue_type,
            object_id: id.to_owned(),
            object_type: None,
            object_name: None,
            related_objects: vec![],
            details: None,
        }
    }
    pub fn new_with_obj<T: gtfs_structures::Id + gtfs_structures::Type + std::fmt::Display>(
        severity: Severity,
        issue_type: IssueType,
        o: &T,
    ) -> Self {
        Issue {
            severity,
            issue_type,
            object_id: o.id().to_owned(),
            object_type: Some(o.object_type()),
            object_name: Some(format!("{}", o)),
            related_objects: vec![],
            details: None,
        }
    }

    pub fn details(mut self, d: &str) -> Self {
        self.details = Some(d.to_owned());
        self
    }

    pub fn object_type(mut self, d: gtfs_structures::ObjectType) -> Self {
        self.object_type = Some(d);
        self
    }

    pub fn add_related_object<T: gtfs_structures::Id + std::fmt::Display>(mut self, o: &T) -> Self {
        self.related_objects.push(RelatedObject {
            id: o.id().to_owned(),
            name: Some(format!("{}", o)),
        });
        self
    }
}
