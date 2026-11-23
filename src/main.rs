mod commands;
mod errors;
mod util;
mod formatting;

use crate::{commands::{obsidian::obsidian, read::ReadCommand, replace::ReplaceCommand}, errors::ObiError};
use clap::{ArgAction::{self, Version}, Parser, Subcommand};
use clap_verbosity_flag::Verbosity;
use log::{error, info};
use std::process::ExitCode;

#[derive(Parser, Debug)]
#[command(version, 
  allow_external_subcommands = true,
  disable_help_flag = true,
  disable_version_flag = true,
)]
struct Cli {
  #[command(subcommand)]
  command: Commands,

  #[command(flatten, next_help_heading = "Other")]
  verbose: Verbosity,

  #[arg(
    short,
    long,
    action = ArgAction::Help,
    help = "Print help",
    help_heading = "Other",
  )]
  help: Option<bool>,

  #[arg(
    short = 'V',
    long,
    action = Version,
    help = "Print version",
    help_heading = "Other",
  )]
  version: Option<bool>,
}

pub trait Runnable {
  fn run(self) -> Result<(), ObiError>;
}

#[derive(Subcommand, Debug)]
enum Commands {
  Read(ReadCommand),
  Replace(ReplaceCommand),

  #[command(external_subcommand, after_help_long = "yo this is some long external text")]
  External(Vec<String>),
}

fn main() -> std::process::ExitCode {
  let cli: Cli = Cli::parse();
  env_logger::Builder::new()
    .filter_level(cli.verbose.log_level_filter())
    .init();

  match cli.command {
    Commands::Read(cmd) => {
      info!("> obi");
      match cmd.run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
          error!("Error: {e}");
          ExitCode::from(e.exit_code())
        }
      }      
    }
    Commands::Replace(cmd) => {
      info!("> obi");
      match cmd.run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
          error!("Error: {e}");
          ExitCode::from(e.exit_code())
        }
      }      
    }
    Commands::External(args) => match obsidian(args, true) {
      Ok(_) => ExitCode::SUCCESS,
      Err(e) => {
        error!("error: {e}");
        ExitCode::from(e.exit_code())
      }
    },
  }

  // println!("TEST A:");
  // let out = Command::new("obsidian").arg("read").output().unwrap();
  // println!("========================\n{}\n========================", String::from_utf8_lossy_owned(out.stdout.clone()));
  // println!("{:?}", &out.stdout);

  // println!("TEST B:");
  // let outB = Command::new("obsidian.com").arg("read").output().unwrap();
  // println!("========================\n{}\n========================", String::from_utf8_lossy_owned(outB.stdout.clone()));
  // println!("{:?}", &outB.stdout);
}
