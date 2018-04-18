# Validate GTFS files

_General Transit Files Specification_ is a file format defining of transit timetables.

This project allows to validate the format and semantics of the timetable.

## Alternatives

* [Google Transitfeed](https://github.com/google/transitfeed)
* [Conveyal/Catalogue’s Datatool](https://github.com/catalogueglobal/datatools-server/)
* [Chouette](https://github.com/afimb/chouette)

## Build and run

1. This project is written in Rust. [Install it](https://rustup.rs/).

2. Clone the project:

```
git clone https://github.com/etalab/transport-validator-rust/
cd transport-validator-rust
```

3. Run it as standalone

The release version (significantly faster) can be run as:

`cargo run --release -- --input test_data/unused_stop`

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
