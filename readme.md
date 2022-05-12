# Validate GTFS files

_General Transit Files Specification_ is a file format defining of transit timetables.

This project allows to validate the format and semantics of the timetable.

## Online tool
transport-validator is the tool used by the [French National Access Point](https://transport.data.gouv.fr/) to validate GTFS files. If you want to try it, you can validate your own files at [this address](https://transport.data.gouv.fr/validation).

## List of checks done by the validator

The checks performed by the validator can be seen in the file [issues.rs](https://github.com/etalab/transport-validator/blob/master/src/issues.rs#L21-L83).

Here is a human friendly list of them :

| check name                      | Description                                                                                     |
|---------------------------------|-------------------------------------------------------------------------------------------------|
| UnusedStop | A stop is not used. |
| Slow | The speed between two stops is too low. |
| ExcessiveSpeed | The speed between two stops is too high. |
| NegativeTravelTime | The travel duration between two stops is negative. |
| CloseStops | Two stops very close to each other in the same trips |
| NullDuration | The travel duration between two stops is null. |
| InvalidReference | Reference not valid. |
| InvalidArchive | Archive not valid. |
| MissingName | An agency, a route or a stop has its name missing. |
| MissingId | An agency, a calendar, a route, a shape point, a stop or a trip has its Id missing. |
| MissingCoordinates | A shape point or a stop is missing its coordinate(s). |
| InvalidCoordinates | The coordinates of a shape point or a stop are not valid. |
| InvalidRouteType | The type of a route is not valid. |
| MissingUrl | An agency or a feed publisher is missing its URL. |
| InvalidUrl | The URL of an agency or a feed publisher is not valid. |
| InvalidTimezone | The TimeZone of an agency is not valid. |
| DuplicateStops | Two stop points or stop areas are identical. |
| MissingPrice | A fare is missing its price. |
| InvalidCurrency | The currency of a fare is not valid |
| InvalidTransfers | The number of transfers of a fare is not valid. |
| InvalidTransferDuration | The transfer duration of a fare is not valid. |
| MissingLanguage | The publisher language code is missing. |
| InvalidLanguage | The publisher language code is not valid. |
| DuplicateObjectId | The object has at least one object with the same id. |
| UnloadableModel | A fatal error has occured by building the links in the model |
| MissingMandatoryFile | Mandatory file missing |
| ExtraFile | The file does not belong to a GTFS archive |
| ImpossibleToInterpolateStopTimes | It's impossible to interpolate the departure/arrival of some stoptimes of the trip |
| InvalidStopLocationTypeInTrip | Only Stop Points are allowed to be used in a Trip |
| InvalidStopParent | The parent station of this stop is not a valid one |
| IdNotAscii | The identifier is not only ASCII characters |


## Alternatives

* [Google Transitfeed](https://github.com/google/transitfeed)
* [Conveyal/Catalogue’s Datatool](https://github.com/catalogueglobal/datatools-server/)
* [Chouette](https://github.com/afimb/chouette)



## Build and run

1. This project is written in Rust. You need to first [install it](https://rustup.rs/) on your machine.

2. Clone the project:

```
git clone https://github.com/etalab/transport-validator/
cd transport-validator
```

3. Run it as standalone

The release version (significantly faster) can be run as:

`cargo run --release -- --input test_data/unused_stop`

If you prefer formating the data in yaml or in pretty printed json, use the --output-format. For example:

`cargo  run --release -- --output-format yaml --input test_data/unused_stop

The development version can be run as:

`cargo run -- --input test_data/unused_stop`

The validator can also read a zip file, or an url:

```
cargo run --release -- -i some_gtfs.zip
cargo run --release -- -i https://example.com/network.gfts
```

4. Lint it

To lint our code we use [rustfmt](https://github.com/rust-lang-nursery/rustfmt)

Install it running:

```
rustup component add rustfmt-preview
```

Lint your code running:

```
cargo fmt --all -- --write-mode=diff
```

5. Run it as a dæmon

The validator can run as a HTTP dæmon to validate any file from a url.

For now the call is synchronous. Be aware that if the file is large, the time required to download the GTFS zip, the request might time out.

`cargo run --release`

You can then ask for a validation:

`curl http://localhost:7878/validate?url=https://example.com/gtfs.zip`
