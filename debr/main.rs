use clap::{Parser, Subcommand};
use std::path::Path;

mod bash;
mod lb;
mod post_cfg;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Subcommands for the executable
    #[command(subcommand)]
    command: Option<Commands>,

    /// Path to the configuration file (default: config.json)
    #[arg(short = 'c', long = "config", default_value_t = String::from("config.json"), help = "Path to the configuration file")]
    config: String,

    /// Path for the live-debian-build to use (default: out/live)
    #[arg(short = 'o', long = "out-dir", default_value_t = String::from("out"), help = "Path for the live-debian-build to use")]
    out_dir: String,
}

#[derive(Subcommand, Debug)]
enum Commands {
    #[command(about = "Install dependencies")]
    Deps,

    #[command(about = "Initialize build")]
    Config,

    #[command(about = "Clean build")]
    Clean,

    #[command(about = "Build live")]
    Build,
}

fn main() {
    let args = Args::parse();

    let live_dir = Path::new(&format!("{}/live", args.out_dir)).to_path_buf();

    match args.command {
        Some(Commands::Deps) => {
            bash::install();
            std::process::exit(0);
        }

        Some(Commands::Config) => {
            if let Err(e) = lb::config(Some(live_dir.as_path())) {
                eprintln!("Error during lb config: {}", e);
                std::process::exit(1);
            }
            post_cfg::apply(&args, live_dir.as_path());
        }

        Some(Commands::Clean) => {
            if let Err(e) = lb::clean(Some(live_dir.as_path())) {
                eprintln!("Error during lb clean: {}", e);
                std::process::exit(1);
            }
        }

        Some(Commands::Build) => {
            if let Err(e) = lb::build(Some(live_dir.as_path())) {
                eprintln!("Error during lb build: {}", e);
                std::process::exit(1);
            }
        }

        None => {
            // No subcommand was provided, process as usual
        }
    }
}
