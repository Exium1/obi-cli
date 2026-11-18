mod commands;
mod errors;
mod util;

use crate::{
  commands::{obsidian::obsidian, write::WriteCommand},
};
use clap::{Parser, Subcommand};
use std::{
  process::{ExitCode},
};

#[derive(Parser, Debug)]
#[command(
  // disable_help_flag = true,
  // disable_help_subcommand = true,
  // disable_version_flag = true,
  allow_external_subcommands = true
)]
struct Cli {
  #[command(subcommand)]
  command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
  Write(WriteCommand),

  #[command(external_subcommand)]
  External(Vec<String>),
}

fn main() -> std::process::ExitCode {
  let raw_args: Vec<String> = std::env::args().collect();
  let cli: Cli = Cli::parse_from(&raw_args);

  match cli.command {
    Commands::Write(cmd) => {
      println!("> obi");
      match cmd.execute() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
          eprintln!("error: {e}");
          ExitCode::from(e.exit_code())
        }
      }
    }
    Commands::External(args) => match obsidian(args, true) {
      Ok(_) => ExitCode::SUCCESS,
      Err(e) => {
        eprintln!("error: {e}");
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
