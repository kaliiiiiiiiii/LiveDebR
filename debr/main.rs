use clap::Parser;

mod config_utils;
mod bash;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Flag to install dependencies
    #[arg(long, help = "Install dependencies for LiveDebR")]
    deps: bool,

    #[arg(
        short = 'c',
        long,
        default_value_t = String::from("config.json"),
        help = "Path to the configuration file"
    )]
    config: String,
}

fn main() {
    let args = Args::parse();

    if args.deps {
        bash::install();
    }

    let config_path = config_utils::find_config_path(&args.config).unwrap_or_else(|| {
        eprintln!("Error: Configuration file '{}' not found", args.config);
        std::process::exit(1);
    });

    let config: config_utils::Config = config_utils::read_config(&config_path).unwrap_or_else(|err| {
        eprintln!(
            "Error: Failed to read configuration file '{}': {}",
            config_path.display(),
            err
        );
        std::process::exit(1);
    });

    println!("Chrome enabled: {}", config.chrome);
}
