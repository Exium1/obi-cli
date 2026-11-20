use std::{convert::Infallible, str::FromStr};

use clap::{ArgGroup, Parser};

#[derive(Debug, Clone, PartialEq)]
enum InputValue {
  Text(String),
  Dash,
}

impl FromStr for InputValue {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "-" {
            Ok(InputValue::Dash)
        } else {
            Ok(InputValue::Text(s.to_string()))
        }
    }
}

#[derive(Parser, Debug)]
#[command(
  group(
    ArgGroup::new("open_location")
      .required(false)
      .args(["open", "newtab"]),
  ),
  
)]
#[command(about = "Replace text in a note")]
pub struct ReplaceCommand {
  /// Name of note (Defaults to active note)
  #[arg(short, long)]
  name: Option<String>,

  /// Path of note (Defaults to active note)
  #[arg(short, long)]
  path: Option<String>,

  /// Old text to be replace
  #[arg(long)]
  old: InputValue,

  /// New text to replace
  #[arg(long)]
  new: InputValue,

  /// Open this note after replacing
  #[arg(long)]
  open: bool,

  /// Open this note in a new tab after replacing
  #[arg(long)]
  newtab: bool,

  /// Show diff without writing to note
  #[arg(long = "dry-run")]
  dryrun: bool,
}
