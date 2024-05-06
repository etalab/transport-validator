#[cfg(feature = "daemon")]
use validator::daemon;
use validator::{custom_rules, validate};

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
        help = "Path to the gtfs file (can be a directory or a zip file) or HTTP URL of the file (will be downloaded)"
    )]
    input: Option<String>,
    #[structopt(
        short = "m",
        long = "max-issues",
        help = "The maximum number of issues per type",
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
    #[structopt(
        short = "c",
        long = "custom-rules",
        help = "Provide a YAML file to customize some validation rules"
    )]
    custom_rules: Option<String>,
}

fn main() -> Result<(), anyhow::Error> {
    #[cfg(feature = "daemon")]
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let opt = Opt::from_args();
    let custom_rules = custom_rules::custom_rules(opt.custom_rules);

    if let Some(input) = opt.input {
        let validations = &validate::generate_validation(&input, opt.max_size, &custom_rules);
        let serialized = match opt.format {
            OutputFormat::Yaml => serde_yaml::to_string(validations)?,
            OutputFormat::Json => serde_json::to_string(validations)?,
            OutputFormat::PrettyJson => serde_json::to_string_pretty(validations)?,
        };
        println!("{}", serialized);
    } else {
        #[cfg(feature = "daemon")]
        {
            log::info!("Starting the validator as a dæmon");
            daemon::run_server()?;
        }
        #[cfg(not(feature = "daemon"))]
        {
            eprintln!("transport-validator was compiled without support for running as daemon.");
            eprintln!("use -i to supply a local file to test instead.");
            std::process::exit(1);
        }
    }
    Ok(())
}
