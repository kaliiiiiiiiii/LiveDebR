use clap::{Parser, Subcommand};
use std::path::{Path, PathBuf};
use std::{error::Error, process};

mod bash;
mod lb;
mod post_cfg;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[command(subcommand)]
    command: Option<Commands>,
    #[arg(short = 'c', long = "config", default_value_t = String::from("config.json"), help = "Path to the configuration file")]
    config: String,
    #[arg(short = 'o', long = "out-dir", default_value_t = String::from("out"), help = "Path for the live-debian-build to use")]
    out_dir: String
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    #[command(about = "Install dependencies")]
    Deps,
    #[command(about = "Initialize build")]
    Config,
    
    #[command(about = "Build live debian")]
    Build,
    #[command(about = "Drop-in replacement for the lb command")]
    Lb {
        #[arg(help = "drop-in replacement for lb command")]
        lb_args: Option<Vec<String>>,
    },
    #[command(about = "Clean all live-build files except of cache, including the config")]
    Clean,
    #[command(about = "Clean all build files except of cache", name="clean-build")]
    CleanBuild,
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
            config(live_dir, &args)?;
        }

        Some(Commands::Clean) => {
            lb::clean(Some(live_dir), None)?;
        }

        Some(Commands::CleanBuild) => {
            lb::clean(Some(live_dir),Some(true))?;
        }

        Some(Commands::Build) => {
            if !live_dir.join("config/").exists(){
                println!("Using Default config");
                config(live_dir, &args)?;
            }
            lb::build(Some(live_dir))?;
        }

        Some(Commands::Lb { lb_args }) => {
            let lb_args = lb_args.unwrap_or_default();
            let lb_args_ref: Vec<&str> = lb_args.iter().map(|s| s.as_str()).collect();
            lb::lb(&lb_args_ref, Some(live_dir))?;
        }

        None => {
            eprintln!("No command provided. Use --help for usage information.");
        }
    }

    Ok(())
}

fn preprocess(args: &mut Vec<String>) {
    let mut lb_found = false;
    let mut i = 0;

    while i < args.len() {
        if lb_found {
            if args[i].starts_with("-") {
                args.insert(i, "--".to_string());
                i += 1;
            }
        } else if vec!["lb"].contains(&args[i].as_str()) {
            lb_found = true;
        }
        i += 1;
    }
}

pub fn config(live_dir:&PathBuf, args:&Args) -> Result<(), Box<dyn Error>> {
    lb::config(Some(live_dir))?;
    post_cfg::apply(&args, live_dir)?;
    Ok(())
}
