mod daemon;
mod validators;

#[macro_use]
extern crate gotham_derive;

use structopt::StructOpt;

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
    #[structopt(
        short = "m",
        long = "max-issues",
        help = "The maxium number of issues per type",
        default_value = "1000"
    )]
    max_size: usize,
}

fn main() {
    env_logger::init();

    let opt = Opt::from_args();

    if let Some(input) = opt.input {
        match validators::validate(&input, opt.max_size) {
            Ok(json) => println!("{}", json),
            Err(err) => println!("Error: {}", err),
        }
    } else {
        log::info!("Starting the validator as a dæmon");
        daemon::run_server()
    }
}
