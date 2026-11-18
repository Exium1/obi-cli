use clap::{CommandFactory, Parser, Subcommand};
use std::{
  io::{Error, ErrorKind},
  process::{Command, Output},
};

// use crate::commands::write::WriteCommand;
use crate::commands::write_command::WriteCommand;
mod commands;

#[derive(Parser, Debug)]
#[command(
  // disable_help_flag = true,
  // disable_help_subcommand = true,
  // disable_version_flag = true
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
  let raw_args: Vec<String> = std::env::args().collect();
  let normalized = normalize_args(raw_args.clone().to_owned());
  println!("{:?}", &normalized);
  let cli = Cli::parse_from(normalized);

  match cli.command {
    Commands::Write(cmd) => {
      println!("Running custom write: {:#?}", cmd);
    }
    _ => {
      println!("> obsidian");
      Command::new("obsidian")
        .args(raw_args[1..].to_vec())
        .status()
        .expect("Failed to execute underlying CLI");
    }
  }

  Ok(())
}

fn normalize_args(raw_args: Vec<String>) -> Vec<String> {
  let subcmd = raw_args.get(1).map(|s| s.as_str()).unwrap_or("");

  // Ask clap itself if `subcmd` is one of your defined enum variants
  let is_custom_cmd = Cli::command()
    .get_subcommands()
    .any(|c| c.get_name() == subcmd);

  if is_custom_cmd {
    return raw_args;
  }

  raw_args
    .into_iter()
    .enumerate()
    .map(|(i, arg)| {
      if i <= 1 || arg.starts_with('-') {
        arg
      } else {
        format!("--{}", arg)
      }
    })
    .collect()
}
