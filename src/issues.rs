//! A module for issues creation.
use crate::visualization;
use geojson::FeatureCollection;
use gtfs_structures::Gtfs;
use serde::Serialize;

/// Represents the severity of an [`Issue`].
///
/// [`Issue`]: struct.Issue.html
#[derive(Serialize, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Copy, Clone)]
pub enum Severity {
    /// Critical error, the GTFS archive couldn't be opened.
    Fatal,
    /// The file does not respect the GTFS specification.
    Error,
    /// Not a specification error, but something is most likely wrong in the data.
    Warning,
    /// Simple information.
    Information,
}

/// Represents the different types of issue.
#[derive(Serialize, Debug, Eq, PartialEq, Ord, PartialOrd, Clone, Hash, Copy)]
pub enum IssueType {
    /// A stop is not used.
    UnusedStop,
    /// The speed between two stops is too low.
    Slow,
    /// The speed between two stops is too high.
    ExcessiveSpeed,
    /// The travel duration between two stops is negative.
    NegativeTravelTime,
    /// Two stops very close to each other in the same trips
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
    DuplicateObjectId,
    /// A fatal error has occured by building the links in the model
    UnloadableModel,
    /// Mandatory file missing
    MissingMandatoryFile,
    /// The file does not belong to a GTFS archive
    ExtraFile,
    /// It's impossible to interpolate the departure/arrival of some stoptimes of the trip
    ImpossibleToInterpolateStopTimes,
    /// Invalid Stop Location type in trip.
    /// Only Stop Points are allowed to be used in a Trip
    InvalidStopLocationTypeInTrip,
    /// The parent station of this stop is not a valid one
    InvalidStopParent,
    /// The Id is not in ASCII encoding
    IdNotAscii,
    /// The shape id referenced in trips.txt does not exist
    InvalidShapeId,
    /// A shape id defined in shapes.txt is not used elsewhere
    UnusedShapeId,
}

/// Represents an object related to another object that is causing an issue.
#[derive(Serialize, Debug, Eq, PartialEq)]
pub struct RelatedObject {
    /// Related object's id.
    pub id: String,
    /// Related object's type.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub object_type: Option<gtfs_structures::ObjectType>,
    /// Related object's name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

/// Represent a line that is causing an issue
#[derive(Serialize, Debug, Eq, PartialEq)]
pub struct RelatedLine {
    /// line number
    pub line_number: u64,
    /// headers of the file
    pub headers: Vec<String>,
    /// line values
    pub values: Vec<String>,
}

/// Represent a file that is causing an issue
#[derive(Serialize, Debug, Eq, PartialEq)]
pub struct RelatedFile {
    /// File name.
    pub file_name: String,
    /// line causing a problem in the file
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<RelatedLine>,
}

/// Represents an issue.
#[derive(Serialize, Debug, PartialEq)]
pub struct Issue {
    /// Issue severity.
    pub severity: Severity,
    /// Issue type.
    pub issue_type: IssueType,
    /// Id of the object causing an issue.
    pub object_id: String,
    /// Type of the object causing an issue.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub object_type: Option<gtfs_structures::ObjectType>,
    /// Name of the object causing an issue.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub object_name: Option<String>,
    /// [Object(s) related] to an object causing an issue.
    ///
    /// [Object(s) related]: struct.RelatedObject.html
    pub related_objects: Vec<RelatedObject>,
    /// Optional details about the issue.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
    /// File causing an issue
    #[serde(skip_serializing_if = "Option::is_none")]
    pub related_file: Option<RelatedFile>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub geojson: Option<FeatureCollection>,
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
            related_file: None,
            geojson: None,
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
            related_file: None,
            geojson: None,
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
    pub fn add_related_object<
        T: gtfs_structures::Id + std::fmt::Display + gtfs_structures::Type,
    >(
        mut self,
        o: &T,
    ) -> Self {
        self.push_related_object(o);
        self
    }

    /// Adds a related object to a given issue.
    /// mutate the object without using the builder pattern
    pub fn push_related_object<
        T: gtfs_structures::Id + std::fmt::Display + gtfs_structures::Type,
    >(
        &mut self,
        o: &T,
    ) {
        self.related_objects.push(RelatedObject {
            id: o.id().to_owned(),
            name: Some(format!("{}", o)),
            object_type: Some(o.object_type()),
        });
    }

    pub fn push_related_geojson(&mut self, gtfs: &Gtfs) {
        self.geojson = visualization::generate_issue_visualization(self, gtfs);
    }
}
