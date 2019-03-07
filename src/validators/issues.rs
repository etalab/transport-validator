//! A module for issues creation.

/// Represents the severity of an [`Issue`].
///
/// [`Issue`]: struct.Issue.html
#[derive(Serialize, Debug, Eq, PartialEq)]
pub enum Severity {
    /// Critical error, the GTFS archive couldn't be opened.
    Fatal,
    /// The file does not respect the GTFS specification.
    Error,
    /// Not a specification error, but something is most likely wrong in the data.
    Warning,
    #[allow(dead_code)]
    /// Simple information.
    Information,
}

/// Represents the different types of issue.
#[derive(Serialize, Debug, Eq, PartialEq, Ord, PartialOrd, Clone)]
pub enum IssueType {
    /// A stop is not used.
    UnusedStop,
    /// The speed between two stops is too low.
    Slow,
    /// The speed between two stops is too high.
    ExcessiveSpeed,
    /// The travel duration between two stops is negative.
    NegativeTravelTime,
    /// Two stops are too close to each other.
    CloseStops,
    /// The travel duration between two stops is null.
    NullDuration,
    /// Reference not valid.
    InvalidReference,
    /// Archive not valid.
    InvalidArchive,
    /// An agency, a route or a stop has its name missing.
    MissingName,
    /// An agency, a calendar, a route, a shape point, a stop or a trip has its Id missing.
    MissingId,
    /// A shape point or a stop is missing its coordinate(s).
    MissingCoordinates,
    /// The coordinates of a shape point or a stop are not valid.
    InvalidCoordinates,
    /// The type of a route is not valid.
    InvalidRouteType,
    /// An agency or a feed publisher is missing its URL.
    MissingUrl,
    /// The URL of an agency or a feed publisher is not valid.
    InvalidUrl,
    /// The TimeZone of an agency is not valid.
    InvalidTimezone,
    /// Two stop points or stop areas are identical.
    DuplicateStops,
    /// A fare is missing its price.
    MissingPrice,
    /// The currency of a fare is not valid
    InvalidCurrency,
    /// The number of transfers of a fare is not valid.
    InvalidTransfers,
    /// The transfer duration of a fare is not valid.
    InvalidTransferDuration,
    /// The publisher language code is missing.
    MissingLanguage,
    /// The publisher language code is not valid.
    InvalidLanguage,
    /// The object has at least one object with the same id.
    DupplicateObjectId,
}

/// Represents an object related to another object that is causing an issue.
#[derive(Serialize, Debug)]
pub struct RelatedObject {
    /// Related object's id.
    pub id: String,
    /// Related object's name.
    pub name: Option<String>,
}

/// Represents an issue.
#[derive(Serialize, Debug)]
pub struct Issue {
    /// Issue severity.
    pub severity: Severity,
    /// Issue type.
    pub issue_type: IssueType,
    /// Id of the object causing an issue.
    pub object_id: String,
    /// Type of the object causing an issue.
    pub object_type: Option<gtfs_structures::ObjectType>,
    /// Name of the object causing an issue.
    pub object_name: Option<String>,
    /// [Object(s) related] to an object causing an issue.
    ///
    /// [Object(s) related]: struct.RelatedObject.html
    pub related_objects: Vec<RelatedObject>,
    /// Optional details about the issue.
    pub details: Option<String>,
}

impl Issue {
    /// Creates a new issue with the [severity], the [type of issue] and the concerned object's id.
    ///
    /// [severity]: enum.Severity.html
    /// [type of issue]: enum.IssueType.html
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
    /// Creates a new issue with the [severity], the [type of issue], and the concerned object's id, type and name.
    ///
    /// [severity]: enum.Severity.html
    /// [type of issue]: enum.IssueType.html
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

    /// Adds details to a given issue.
    pub fn details(mut self, d: &str) -> Self {
        self.details = Some(d.to_owned());
        self
    }

    /// Adds an object name to a given issue.
    pub fn name(mut self, d: &str) -> Self {
        self.object_name = Some(d.to_owned());
        self
    }

    /// Adds an object type to a given issue.
    pub fn object_type(mut self, d: gtfs_structures::ObjectType) -> Self {
        self.object_type = Some(d);
        self
    }

    /// Adds a related object to a given issue.
    pub fn add_related_object<T: gtfs_structures::Id + std::fmt::Display>(mut self, o: &T) -> Self {
        self.related_objects.push(RelatedObject {
            id: o.id().to_owned(),
            name: Some(format!("{}", o)),
        });
        self
    }
}
