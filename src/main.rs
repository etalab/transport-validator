mod daemon;
mod validators;

extern crate failure;
extern crate gotham;
#[macro_use]
extern crate gotham_derive;
extern crate gtfs_structures;
extern crate hyper;
extern crate structopt;
extern crate structopt_derive;
use structopt::StructOpt;
extern crate env_logger;
#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;

#[derive(StructOpt, Debug)]
#[structopt(name = "gtfs-validator", about = "Validates the gtfs file.")]
struct Opt {
    #[structopt(
        short = "i",
        long = "input",
        help = "Path to the gtfs file. Can be a directory or a zip file"
    )]
    input: Option<String>,
}

fn main() {
    env_logger::init();

    let opt = Opt::from_args();

    if let Some(input) = opt.input {
        match validators::validate(&input) {
            Ok(json) => println!("{}", json),
            Err(err) => println!("Error: {}", err),
        }
    } else {
        info!("Starting the validator as a dæmon");
        daemon::run_server()
    }
}
