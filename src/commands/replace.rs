use std::{convert::Infallible, str::FromStr};
use clap::{ArgAction, ArgGroup, Args, Parser};
use serde::{Deserialize, Serialize};
use similar::TextDiff;

use crate::{Runnable, commands::obsidian::{obsidian_create, obsidian_file, obsidian_read}, errors::ObiError, util::{hash_content, read_stdin_content}};

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
  about = "Replace text in a note",
  group(
    ArgGroup::new("open_location")
      .required(false)
      .args(["open", "new_tab"]),
  ),
  group(
    ArgGroup::new("matching")
      .required(false)
      .args(["occurrence", "all"]),
  ),
  disable_help_flag = true,
  disable_version_flag = true,
)]
pub struct ReplaceCommand {
  #[command(flatten)]
  note: NoteArgs,

  #[command(flatten)]
  text: TextArgs,

  #[command(flatten)]
  matching: MatchingArgs,

  #[command(flatten)]
  safety: SafetyArgs,

  #[command(flatten)]
  output: OutputArgs,

  #[arg(
    short,
    long,
    action = ArgAction::Help,
    help = "Print help",
    help_heading = "Other",
  )]
  help: Option<bool>,
}

#[derive(Args, Debug)]
#[command(next_help_heading = "Note")]
pub struct NoteArgs {
  /// Name of note [default: active file]
  #[arg(short, long)]
  pub name: Option<String>,

  /// Path of note [default: active file]
  #[arg(short, long)]
  pub path: Option<String>,
}

#[derive(Args, Debug)]
#[command(next_help_heading = "Text")]
struct TextArgs {
  /// Old text to be replace. ('-' to use stdin)
  #[arg(long, value_name="OLD|-")]
  old: InputValue,

  /// New text to replace. ('-' to use stdin)
  #[arg(long, value_name="NEW|-")]
  new: InputValue,

  /// Read {"old": "...", "new": "..."} from stdin.
  #[arg(long = "json-input")]
  json_input: bool,
}

#[derive(Args, Debug)]
#[group(multiple = false)]
#[command(next_help_heading = "Matching")]
struct MatchingArgs {
  /// Select which occurrence of old string to replace
  #[arg(long, value_name = "N", default_value_t = 1)]
  occurrence: usize,

  /// Select all occurrences of the old string to replace
  #[arg(long)]
  all: bool,
}

#[derive(Args, Debug)]
#[command(next_help_heading = "Safety")]
struct SafetyArgs {
  /// Abort unless the note's current hash is H
  #[arg(long = "expected-hash", value_name = "H")]
  expected_hash: Option<String>,

  /// Allow an empty 'new' string (deleting)
  #[arg(long = "allow-empty")]
  allow_empty: bool,

  /// Show diff without writing to note
  #[arg(long = "dry-run")]
  dry_run: bool,

  /// Context lines in diff
  #[arg(long, default_value_t = 3, value_name = "N")]
  context: usize,
}

#[derive(Args, Debug)]
#[command(next_help_heading = "Output")]
struct OutputArgs {
  /// Open this note after replacing
  #[arg(long)]
  open: bool,

  /// Open this note in a new tab after replacing
  #[arg(long = "newtab")]
  new_tab: bool,

  /// Print the result as JSON
  #[arg(long)]
  json: bool,
}


#[derive(Debug, Deserialize)]
struct JSONInput {
  old: String,
  new: String
}

#[derive(Debug, Serialize)]
struct JSONOutputLines {
  before: usize,
  after: usize
}

#[derive(Debug, Serialize)]
struct JSONOutput {
  path: String,
  hash: String,
  lines: JSONOutputLines
}

impl Runnable for ReplaceCommand {
  fn run(self) -> Result<(), ObiError> {

    // Read stdin if required
    let stdin: String = if self.text.old.eq(&InputValue::Dash) || self.text.new.eq(&InputValue::Dash) {
      read_stdin_content()?
    } else {
      "-".into()
    };

    // Check if json input is required
    let json_input: Option<JSONInput> = if self.text.json_input {
      match serde_json::from_str(stdin.as_str()) {
        Ok(json) => Some(json),
        Err(_) => return Err(ObiError::JSONInputInvalid())
      }
    } else  {
      None
    };
    
    let old: &String = match &self.text.old {
      InputValue::Dash => {
        match &json_input {
          Some(i) => &i.old,
          None => &stdin
        }
      },
      InputValue::Text(s) => &s
    };
    
    let new: &String = match &self.text.new {
      InputValue::Dash => {
        match &json_input {
          Some(i) => &i.new,
          None => &stdin
        }
      },
      InputValue::Text(s) => &s
    };

    if old.is_empty() {
      return Err(ObiError::ContentIsEmpty())
    }

    // Throw error if new string is empty without allow-empty flag
    if new.is_empty() && !self.safety.allow_empty {
      return Err(ObiError::ContentIsEmpty())
    }

    // Get active file if path isn't specified (needed for empty replace cmd)
    let target_path = match self.note.path {
      Some(path) => path,
      None => obsidian_file()?.path
    };

    // Get note contents using target path
    let read_output = obsidian_read(self.note.name.as_ref(), Some(&target_path))?;
    
    // Expected hash check
    if let Some(expected_hash) = self.safety.expected_hash {
      let actual_hash = hash_content(&read_output);
      if expected_hash != actual_hash {
        return Err(ObiError::ContentHashUnexpected(actual_hash));
      }
    }

    // Replacement (occurrence & all flags)
    let updated_content = replace_content(
      &read_output,
      &old,
      &new,
      Some(self.matching.occurrence),
      self.matching.all
    )?;

    if self.safety.dry_run {
      // Dry run
      let diff = TextDiff::from_lines(&read_output,&updated_content);

      let output = diff
        .unified_diff()
        .context_radius(self.safety.context) // Hides all unchanged lines
        .to_string();

      println!("{}", output);
    } else {
      // Write
      obsidian_create(
        &target_path,
        &self.note.name,
        &updated_content,
        self.output.open,
        self.output.new_tab
      )?;
    }

    if self.output.json {
      let json_output = JSONOutput {
        path: target_path,
        hash: hash_content(&updated_content),
        lines: JSONOutputLines {
          before: read_output.lines().count(),
          after: updated_content.lines().count()
        }
      };

      match serde_json::to_string(&json_output) {
        Ok(o) => println!("{}", o),
        Err(_) => return Err(ObiError::ContentIsEmpty())
      };
    }

    Ok(())
  }
}

fn replace_content(
  original: &String,
  old: &String,
  new: &String,
  occurrence: Option<usize>,
  all: bool
) -> Result<String, ObiError> {

  let mut result = original.to_string();

  if let Some(n) = occurrence {
    if let Some(nth) = n.checked_sub(1) {
      if let Some((idx, matched)) = result.match_indices(old).nth(nth) {
        result.replace_range(idx..idx + matched.len(), new);
      }
    }
  } else if all {
    result = result.replace(old, new);
  } else {
    let count = result.matches(old).count();

    if count > 1 {
      return Err(ObiError::AnchorAmbiguous { anchor: old.into(), count });
    }

    result = result.replacen(old, new, 1);
  }
  
  Ok(result)
}