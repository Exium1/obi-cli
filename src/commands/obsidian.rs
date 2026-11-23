use std::{
  io::{Write, stderr, stdout},
  process::{Command, Output},
};

use log::{debug, info};

use crate::errors::ObiError::{self};

#[cfg(windows)]
const OBSIDIAN_BIN: &str = "obsidian.com";
#[cfg(not(windows))]
const OBSIDIAN_BIN: &str = "obsidian";

/*
 * Converts clap's GNU style args into Obsidian CLI's bare args.
 * Ex: `--key=val` / `--key val` / `--flag` into `key=val` / `flag`
 */
fn to_obsidian_args(args: Vec<String>) -> Vec<String> {
  debug!("to_obsidian_args: {:?}", args);

  let mut out = Vec::new();
  let mut iter = args.iter().peekable();

  while let Some(arg) = iter.next() {
    if let Some(stripped) = arg.strip_prefix("--") {
      if stripped.contains('=') {
        // --key=value -> key=value
        out.push(stripped.to_string())
      } else if let Some(next) = iter.peek() {
        // --key value -> key=value
        if !next.starts_with('-') {
          out.push(format!("{}={}", stripped, iter.next().unwrap()))
        } else {
          // --flag -> flag
          out.push(stripped.to_string());
        }
      } else {
        // trailing --flag -> flag
        out.push(stripped.to_string());
      }
    } else {
      // subcommand name or positional arg
      out.push(arg.clone());
    }
  }

  debug!("formatted: {:?}", out);

  out
}

/// Runs the Obsidian CLI and always captures its output.
/// When `hide_output` is false, the captured output is also echoed to the terminal.
pub fn obsidian(args: Vec<String>, show_output: bool) -> Result<Output, ObiError> {
  info!("> obsidian (show_output: {})", show_output);

  let output = match Command::new(OBSIDIAN_BIN)
    .args(to_obsidian_args(args))
    .output()
  {
    Ok(output) => output,
    Err(e) => return Err(ObiError::ObsidianLaunch(e)),
  };

  if !output.status.success() {
    return Err(ObiError::ObsidianFailed {
      status: output.status,
      message: String::from_utf8_lossy(&output.stderr).trim().to_owned(),
    });
  }

  if show_output {
    let _ = stdout().write_all(&output.stdout);
    let _ = stderr().write_all(&output.stderr);
  }

  Ok(output)
}

pub fn obsidian_read(name: Option<&String>, path: Option<&String>) -> Result<String, ObiError> {
  let mut args = vec!["read".to_owned()];

  if let Some(path) = path {
    args.push(format!("path={path}"));
  }

  if let Some(name) = name {
    args.push(format!("name={name}"));
  }

  let output = obsidian(args, false)?;
  let text = String::from_utf8_lossy(&output.stdout);
  let text = text
    .strip_suffix("\r\n")
    .or_else(|| text.strip_suffix('\n'))
    .unwrap_or(&text);

  if text.starts_with("Error: File \"") {
    return Err(ObiError::ObsidianMissingFile(match path {
      Some(path) => path.into(),
      None => "".into(),
    }));
  }

  Ok(text.to_owned())
}

pub fn obsidian_create(
  path: &String,
  name: &Option<String>,
  content: &String,
  open: bool,
  newtab: bool,
) -> Result<String, ObiError> {
  let mut args = vec![
    "create".to_owned(),
    format!("path={path}"),
    format!("content={content}"),
    "overwrite".to_owned(),
  ];

  if let Some(name) = name {
    args.push(format!("name={name}"));
  }

  if open {
    args.push(format!("open"));
  }

  if newtab {
    args.push(format!("newtab"));
  }

  let output = obsidian(args, true)?;
  Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

#[derive(Debug)]
pub struct NoteMeta {
  pub path: String,
  pub name: String,
  pub extension: String,
  pub size: u64,
  pub created: u64,
  pub modified: u64,
}

fn parse_obsidian(output: &str) -> Result<NoteMeta, ObiError> {
  let mut path = None;
  let mut name = None;
  let mut extension = None;
  let mut size = None;
  let mut created = None;
  let mut modified = None;

  if output.starts_with("Error: No active file.") {
    return Err(ObiError::ObsidianNoActiveFile());
  }

  for line in output.lines() {
    let Some((key, val)) = line.trim().split_once(char::is_whitespace) else {
      continue;
    };
    let val = val.trim_start();

    match key {
      "path" => path = Some(val.to_owned()),
      "name" => name = Some(val.to_owned()),
      "extension" => extension = Some(val.to_owned()),
      "size" => size = val.parse().ok(),
      "created" => created = val.parse().ok(),
      "modified" => modified = val.parse().ok(),
      _ => {}
    }
  }

  let err = || ObiError::ObsidianMismatch("missing or invalid field".into());

  Ok(NoteMeta {
    path: path.ok_or_else(err)?,
    name: name.ok_or_else(err)?,
    extension: extension.ok_or_else(err)?,
    size: size.ok_or_else(err)?,
    created: created.ok_or_else(err)?,
    modified: modified.ok_or_else(err)?,
  })
}

pub fn obsidian_file() -> Result<NoteMeta, ObiError> {
  let args = vec!["file".to_owned()];
  let output = obsidian(args, false)?;

  parse_obsidian(&String::from_utf8_lossy(&output.stdout))
}
