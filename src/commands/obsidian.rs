use std::{
  io::{Write, stderr, stdout},
  process::{Command, Output},
};

use crate::errors::ObiError;

#[cfg(windows)]
const OBSIDIAN_BIN: &str = "obsidian.com";
#[cfg(not(windows))]
const OBSIDIAN_BIN: &str = "obsidian";

/*
 * Converts clap's GNU style args into Obsidian CLI's bare args.
 * Ex: `--key=val` / `--key val` / `--flag` into `key=val` / `flag`
 */
fn to_obsidian_args(args: Vec<String>) -> Vec<String> {
  println!("to_obsidian_args: {:?}", args);

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

  println!("formatted: {:?}", out);

  out
}

/// Runs the Obsidian CLI and always captures its output.
/// When `hide_output` is false, the captured output is also echoed to the terminal.
pub fn obsidian(args: Vec<String>, show_output: bool) -> Result<Output, ObiError> {
  println!("> obsidian (show_output: {})", show_output);

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

pub fn obsidian_read(name: Option<String>, path: Option<String>) -> Result<String, ObiError> {
  let mut args = vec!["read".to_owned()];

  if let Some(name) = name {
    args.push(format!("name={name}"));
  }
  if let Some(path) = path {
    args.push(format!("path={path}"));
  }

  let output = obsidian(args, false)?;
  Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}
