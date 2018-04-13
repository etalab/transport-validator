mod validators;

extern crate gtfs_structures;
extern crate structopt;
#[macro_use]
extern crate structopt_derive;
use structopt::StructOpt;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

#[derive(StructOpt, Debug)]
#[structopt(name = "gtfs-validator", about = "Validates the gtfs file.")]
struct Opt {
    #[structopt(short = "i", long = "input", help = "Path to the gtfs file. Can be a directory or a zip file")]
    input: String,
}


fn main() {
    let opt = Opt::from_args();
    let gtfs = if opt.input.to_lowercase().ends_with(".zip"){
        gtfs_structures::Gtfs::from_zip(&opt.input)
    }
    else{
        gtfs_structures::Gtfs::new(&opt.input)
    };
    let validation = gtfs
        .map(|gtfs| validators::validate(&gtfs))
        .and_then(|validation| Ok(serde_json::to_string(&validation)?));
    match validation {
        Ok(json) => println!("Validation: {}", json),
        Err(err) => println!("Error: {}", err)
    }
}
