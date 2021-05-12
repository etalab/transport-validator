use validator::{daemon, validate};

use structopt::clap::arg_enum;
use structopt::StructOpt;
arg_enum! {
    #[derive(Debug)]
    enum OutputFormat {
        Json,
        Yaml,
        PrettyJson
    }
}

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
    #[structopt(
        short = "f",
        long = "output-format",
        help = "Output format (when using the validator in command line)",
        default_value = "json",
        possible_values = &OutputFormat::variants(),
        case_insensitive = true
    )]
    format: OutputFormat,
}

fn main() -> Result<(), anyhow::Error> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let opt = Opt::from_args();

    if let Some(input) = opt.input {
        let validations = &validate::create_issues(&input, opt.max_size);
        let serialized = match opt.format {
            OutputFormat::Yaml => serde_yaml::to_string(validations)?,
            OutputFormat::Json => serde_json::to_string(validations)?,
            OutputFormat::PrettyJson => serde_json::to_string_pretty(validations)?,
        };
        println!("{}", serialized);
    } else {
        log::info!("Starting the validator as a dæmon");
        daemon::run_server()?;
    }
    Ok(())
}
