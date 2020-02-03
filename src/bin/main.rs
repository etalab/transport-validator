use validator::{daemon, validate};

use structopt::StructOpt;

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
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let opt = Opt::from_args();

    if let Some(input) = opt.input {
        match validate::validate(&input, opt.max_size) {
            Ok(json) => println!("{}", json),
            Err(err) => println!("Error: {}", err),
        }
    } else {
        log::info!("Starting the validator as a dæmon");
        daemon::run_server().expect("server failed")
    }
}
