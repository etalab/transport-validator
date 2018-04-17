mod daemon;
mod validators;

extern crate failure;
extern crate gotham;
#[macro_use]
extern crate gotham_derive;
extern crate gtfs_structures;
extern crate hyper;
extern crate structopt;
#[macro_use]
extern crate structopt_derive;
use structopt::StructOpt;
#[macro_use]
extern crate serde_derive;

#[derive(StructOpt, Debug)]
#[structopt(name = "gtfs-validator", about = "Validates the gtfs file.")]
struct Opt {
    #[structopt(short = "i", long = "input",
                help = "Path to the gtfs file. Can be a directory or a zip file")]
    input: Option<String>,
}

fn main() {
    let opt = Opt::from_args();

    if let Some(input) = opt.input {
        match validators::validate(&input) {
            Ok(json) => println!("Validation: {}", json),
            Err(err) => println!("Error: {}", err),
        }
    } else {
        println!("Starting the validator as a dæmon");
        daemon::run_server()
    }
}
