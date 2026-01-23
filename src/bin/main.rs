use clap::{Parser, ValueEnum};
#[cfg(feature = "daemon")]
use validator::daemon;
use validator::{custom_rules, validate};

#[derive(Debug, ValueEnum, PartialEq, Eq, Clone, Copy)]
enum OutputFormat {
    Json,
    Yaml,
    PrettyJson,
}

impl std::fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.to_possible_value()
            .expect("no values are skipped")
            .get_name()
            .fmt(f)
    }
}

#[derive(Parser, Debug)]
#[command(name = "gtfs-validator", about = "Validates the gtfs file.")]
struct Opt {
    #[arg(
        short,
        long = "input",
        help = "Path to the gtfs file (can be a directory or a zip file) or HTTP URL of the file (will be downloaded)"
    )]
    input: Option<String>,
    #[arg(
        short,
        long = "max-issues",
        help = "The maximum number of issues per type",
        default_value = "1000"
    )]
    max_size: usize,
    #[arg(
        short,
        long = "output-format",
        help = "Output format (when using the validator in command line)",
        default_value_t = OutputFormat::Json,
        value_enum
    )]
    format: OutputFormat,
    #[arg(
        short,
        long = "custom-rules",
        help = "Provide a YAML file to customize some validation rules"
    )]
    custom_rules: Option<String>,
}

fn main() -> Result<(), anyhow::Error> {
    #[cfg(feature = "daemon")]
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let opt = Opt::parse();
    let custom_rules = custom_rules::custom_rules(opt.custom_rules);

    if let Some(input) = opt.input {
        let validations = &validate::generate_validation(&input, opt.max_size, &custom_rules);
        let serialized = match opt.format {
            OutputFormat::Yaml => serde_norway::to_string(validations)?,
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
