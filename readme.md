# Validate GTFS files

The General Transit Feed Specification ([GTFS](https://gtfs.org/)) defines a common format for public transportation schedules and associated geographic information.

This project is a validating tool for such files and can perform checks ranging from simple ones (the archive is not valid, a file is missing) to more complex ones (a vehicle is moving too fast).

## Online tool
transport-validator is the tool used by the [French National Access Point](https://transport.data.gouv.fr/) to validate GTFS files. If you want to use it online, you can validate your own files at [this address](https://transport.data.gouv.fr/validation?locale=en).


## Validation output
Validation output is twofold:
* it gives useful information about the validated file, under the `metadata` entry
* it lists a serie of validation items, with a corresponding severity, under the `validations` entry. When relevant, geographical data ([GeoJSON](https://geojson.org/)) related to the issue is given to ease file debugging.

The output is by default formatted in `json`, but `yaml` is also available. See [Options](https://github.com/etalab/transport-validator#options) for more information.

```json
{
    "metadata": {
        ...
    },
    "validations": {
       ...
    }
}
```

### Metadata
Give some useful informations about the validated file content:

| Entry                           | format | Description                                                                                     |
|---------------------------------|--------|----------------------------------------------------------------------------------------|
| start_date | "YYYY-MM-DD" | The starting date of the calendar information (both `calendar.txt` and `calendar_dates.txt` are taken into account). |
| end_date | "YYYY-MM-DD" | The ending date of the calendar information (both `calendar.txt` and `calendar_dates.txt` are taken into account). |
| stop_areas_count | integer | Numer of stop areas (`location_type` equal to `1` in `stops.txt`) found in the file|
| stop_points_count | integer | Numer of stops (`location_type` equal to `0` in `stops.txt`) found in the file|
| lines_count | integer | Numer of routes found in `routes.txt` |
| networks | list of strings | A list of unique agencies names, found in `agency.txt` | 
| modes | list of strings | A list of the `route_types` found in `routes.txt` | 
| issues_count| Object | A summary of the validation issues found in the `validations` section. Keys of the object are the issue name, values are the number of corresponding issues found. |
| has_fares | boolean | True if a `fare_attributes.txt` file exists and contains information |
| has_shapes | boolean | True if a `shapes.txt` file exists and contains information |
| has_pathways | boolean | True if a `pathways.txt` file exists and contains information |
| lines_with_custom_color_count | integer | Numer of routes found in `routes.txt` with a custom `route_color` or a custom `route_text_color`. Custom means different from the default values. |
| some_stops_need_phone_agency | boolean | Some stops have a `continuous_pickup` or a `continuous_drop_off` field equal to `2`. |
| some_stops_need_phone_driver | boolean | Some stops have a `continuous_pickup` or a `continuous_drop_off` field equal to `3`. |

#### Example

```json
    "metadata": {
        "start_date": "2020-11-02",
        "end_date": "2022-01-31",
        "stop_areas_count": 1215,
        "stop_points_count": 2122,
        "lines_count": 45,
        "networks": [
            "carSud"
        ],
        "modes": [
            "bus", "tramway"
        ],
        "issues_count": {
            "ExcessiveSpeed": 5,
            "CloseStops": 10,
            "NullDuration": 10,
            "MissingName": 1,
            "MissingCoordinates": 1215,
            "InvalidCoordinates": 1215,
            "DuplicateStops": 49,
            "IdNotAscii": 171
        },
        "has_fares": true,
        "has_shapes": true,
        "has_pathways": false,
        "lines_with_custom_color_count": 45,
        "some_stops_need_phone_agency": false,
        "some_stops_need_phone_driver": false
    }
```

## Validations
The `"validations"` key contains the actual validation results.
### Severity

Each check is associated with a severity level.

| Severity | Description |
|----------|-------------|
| Fatal | Critical error, the GTFS archive couldn't be opened |
| Error | The file does not respect the GTFS specification |
| Warning | Not a specification error, but something is most likely wrong in the data |
| Information | Simple information |

### List of checks
The validator performs a number of checks. The list of checks can be seen in the file [issues.rs](https://github.com/etalab/transport-validator/blob/master/src/issues.rs#L21-L83).

Here is a human friendly list of them :

| check name                      | Severity | Description                                                                                     |
|---------------------------------|----|-------------------------------------------------------------------------------------------------|
| UnusedStop | Information | A stop is not used. |
| Slow | Information |The speed between two stops is too low. |
| ExcessiveSpeed | Information | The speed between two stops is too high. |
| CloseStops | Information | Two stops very close to each other in the same trips |
| InvalidRouteType | Information | The type of a route is not valid. |
| DuplicateStops | Information | Two stop points or stop areas look identical. They share the same name, and are geographically very close. This check is not applied to station entrances (`location_type` equal to `2`) |
| ExtraFile | Information | The file does not belong to a GTFS archive |
|  |  |  |
| NegativeTravelTime | Warning | The travel duration between two stops is negative. |
| MissingName | Warning | An agency, a route or a stop has its name missing. |
| MissingCoordinates | Warning | A shape point or a stop is missing its coordinate(s). |
| NullDuration | Warning | The travel duration between two stops is null. |
| MissingUrl | Warning | An agency or a feed publisher is missing its URL. |
| InvalidUrl | Warning | The URL of an agency or a feed publisher is not valid. |
| MissingLanguage | Warning | The publisher language code is missing. |
| InvalidLanguage | Warning | The publisher language code is not valid. |
| DuplicateObjectId | Warning | The object has at least one object with the same id. |
| InvalidStopLocationTypeInTrip | Warning | Only Stop Points are allowed to be used in a Trip |
| InvalidStopParent | Warning | The parent station of this stop is not a valid one |
| IdNotAscii | Warning | The identifier is not only ASCII characters |
|  |  |  |
| MissingId | Error | An agency, a calendar, a route, a shape point, a stop or a trip has its Id missing. |
| InvalidCoordinates | Error | The coordinates of a shape point or a stop are not valid. |
| InvalidTimezone | Error | The TimeZone of an agency is not valid. |
| MissingPrice | Error | A fare is missing its price. |
| InvalidCurrency | Error | The currency of a fare is not valid |
| InvalidTransfers | Error | The number of transfers of a fare is not valid. |
| InvalidTransferDuration | Error | The transfer duration of a fare is not valid. |
| ImpossibleToInterpolateStopTimes | Error | It's impossible to interpolate the departure/arrival of some stoptimes of the trip |
|  |  |  |
| InvalidReference | Fatal | Reference not valid. For example a stop referenced by a stop time that does not exist |
| InvalidArchive | Fatal | .zip Archive not valid. |
| UnloadableModel | Fatal | A fatal error has occured by building the links in the model |
| MissingMandatoryFile | Fatal | Mandatory file missing |

### Geojson information
When relevant for the check, geojson information is added for each check output, making the GTFS debug process easier.

### Example

Here is a validation output containing one warning, triggered by a non Ascii Stop id:

```json
"validations": {
    "IdNotAscii" : [
        {
            "severity": "Warning",
            "issue_type": "IdNotAscii",
            "object_id": "AllBél",
            "object_type": "Stop",
            "object_name": "",
            "related_objects": [],
            "geojson": {
                "features": [
                    {
                        "geometry": null,
                        "properties": {
                            "id": "AllBél",
                            "name": ""
                        },
                        "type": "Feature"
                    }
                ],
                "type": "FeatureCollection"
            }
        }
    ]
}
```

Another example showing the geojson information for an information about two stops too close:

```json
    "validations": {
        "CloseStops": [
            {
                "severity": "Information",
                "issue_type": "CloseStops",
                "object_id": "PH00320P",
                "object_type": "Stop",
                "object_name": "Baril Les Hauts",
                "related_objects": [
                    {
                        "id": "PH00320C",
                        "object_type": "Stop",
                        "name": "Baril Les Hauts"
                    },
                    {
                        "id": "MAGM",
                        "object_type": "Route",
                        "name": "MAGM-MagmaBus Navette Centre Ville St Philippe"
                    }
                ],
                "details": "distance between the stops is 0 meter(s)",
                "geojson": {
                    "features": [
                        {
                            "geometry": {
                                "coordinates": [
                                    55.71866572404381,
                                    -21.356751531407003
                                ],
                                "type": "Point"
                            },
                            "properties": {
                                "id": "PH00320P",
                                "name": "Baril Les Hauts"
                            },
                            "type": "Feature"
                        },
                        {
                            "geometry": {
                                "coordinates": [
                                    55.71866572404381,
                                    -21.356751531407003
                                ],
                                "type": "Point"
                            },
                            "properties": {
                                "id": "PH00320C",
                                "name": "Baril Les Hauts"
                            },
                            "type": "Feature"
                        },
                        {
                            "geometry": null,
                            "properties": {
                                "details": "distance between the stops is 0 meter(s)"
                            },
                            "type": "Feature"
                        }
                    ],
                    "type": "FeatureCollection"
                }
            }
        ]
    }
```
## Installation

1. This project is written in Rust. You need to first [install Rust](https://rustup.rs/) on your machine.

2. Clone the project:

```
git clone https://github.com/etalab/transport-validator/
cd transport-validator
```
## Run the validator

### Run from a local directory
The development version can be run as:

`cargo run -- --input test_data/unused_stop`

The release version (significantly faster) can be run as:

`cargo run --release -- --input test_data/unused_stop`


The validator can read a zip file, or an url:

```
cargo run --release -- -i some_gtfs.zip
cargo run --release -- -i https://example.com/network.gfts
```

### Run as a dæmon

The validator can run as a HTTP dæmon to validate any file from a url.

For now the call is synchronous. Be aware that if the file is large, the time required to download the GTFS zip, the request might time out.

The command to launch the dæmon is:

`cargo run --release`

You can then ask for a validation:

`curl http://localhost:7878/validate?url=https://example.com/gtfs.zip`

## Options

* `--input` or `-i`: Path to the gtfs file. Can be a directory or a zip file
* `--max-issues` or `-m`: The maxium number of issues per type. Defaults to 1000.
* `--output-format` or `-f`: Output format (when using the validator in command line). Value by default is `json`, but `yaml` is also available.

## Lint

To lint our code we use [rustfmt](https://github.com/rust-lang-nursery/rustfmt)

Install it running:

```
rustup component add rustfmt-preview
```

Lint your code running:

```
cargo fmt --all -- --write-mode=diff
```

## Alternatives

* [Google Transitfeed](https://github.com/google/transitfeed)
* [Conveyal/Catalogue’s Datatool](https://github.com/catalogueglobal/datatools-server/)
* [Chouette](https://github.com/afimb/chouette)