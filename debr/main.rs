use clap::{Parser, Subcommand};
use std::path::Path;
use std::{error::Error, process};

mod bash;
mod lb;
mod post_cfg;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Option<Commands>,
    #[arg(short = 'c', long = "config", default_value_t = String::from("config.json"), help = "Path to the configuration file")]
    config: String,
    #[arg(short = 'o', long = "out-dir", default_value_t = String::from("out"), help = "Path for the live-debian-build to use")]
    out_dir: String
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
    Lb {
        #[arg(help = "Aequivalent to lb")]
        lb_args: Option<Vec<String>>,
    },
}

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let mut raw_args: Vec<String> = std::env::args().collect();

    preprocess(&mut raw_args);

    let args = Args::parse_from(raw_args);
    let live_dir = &Path::new(&args.out_dir).join("live/");

    match args.command {
        Some(Commands::Deps) => {
            bash::install()?;
        }

        Some(Commands::Config) => {
            lb::config(Some(live_dir))?;
            post_cfg::apply(&args, live_dir)?;
        }

        Some(Commands::Clean) => {
            lb::clean(Some(live_dir))?;
        }

        Some(Commands::Build) => {
            lb::build(Some(live_dir))?;
        }

        Some(Commands::Lb { lb_args }) => {
            // Handle Lb command
            let lb_args = lb_args.unwrap_or_default(); // If args is None, use an empty Vec<String>
            let lb_args_ref: Vec<&str> = lb_args.iter().map(|s| s.as_str()).collect();
            // Assuming lb::lb expects a slice of &str
            lb::lb(&lb_args_ref, Some(Path::new(&args.out_dir)))?;
        }

        None => {
            eprintln!("No command provided. Use --help for usage information.");
        }
    }

    Ok(())
}

fn preprocess(args: &mut Vec<String>) {
    // ensure that all commands after lb are treated as options
    // => escape with --
    let mut lb_found = false;
    let mut i = 0;

    while i < args.len() {
        if lb_found {
            if args[i].starts_with("-") {
                // Insert "--" before the argument that needs escaping
                args.insert(i, "--".to_string());
                // Skip the next index to avoid double insertion
                i += 1;
            }
        } else if args[i] == "lb" {
            lb_found = true;
        }
        // Move to the next argument
        i += 1;
    }
}
