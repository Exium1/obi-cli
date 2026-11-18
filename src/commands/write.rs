use std::{
  error::Error,
  io::{Read, stdin},
  ops::RangeInclusive,
};

use crate::{commands::obsidian::obsidian_read, errors::ObiError, util::read_stdin_content};
use clap::{ArgGroup, Parser};

#[derive(Parser, Debug)]
#[command(
    disable_help_flag = true,
    group(
        ArgGroup::new("position")
            .required(true)
            .args(["line", "anchor"]),
    )
)]
pub struct WriteCommand {
  #[arg(long)]
  name: Option<String>,

  #[arg(long)]
  path: Option<String>,

  #[arg(long)]
  content: String, // "-" means read from stdin

  #[arg(long)]
  line: Option<String>,

  #[arg(long)]
  anchor: Option<String>,

  #[arg(long)]
  open: bool,

  #[arg(long)]
  newtab: bool,
}

impl WriteCommand {
  pub fn execute(self) -> Result<(), ObiError> {
    println!("Path: {:?}", self.path);
    println!("Name: {:?}", self.name);
    println!("Content: {:?}", self.content);
    println!("Line: {:?}", self.line);
    println!("Anchor: {:?}", self.anchor);
    println!("Newtab: {:?}", self.newtab);
    println!("Open: {:?}", self.open);

    let content = if self.content == "-" {
      read_stdin_content()?
    } else {
      self.content.clone()
    };

    let line_range = WriteCommand::parse_line_range(self.line);
    let read_output =
      obsidian_read(self.name, self.path).expect("Failed to read with obsidian cli.");

    println!(
      "\nOLD NOTE:\n========================\n{}\n========================",
      &read_output
    );

    let modified_content =
      WriteCommand::update_content(&read_output, &content, self.anchor, line_range)?;

    println!(
      "\nNEW NOTE:\n========================\n{}\n========================",
      &modified_content
    );

    Ok(())
  }

  fn parse_line_range(line: Option<String>) -> Option<RangeInclusive<usize>> {
    let l: String = line?; // Returns None if line is None

    // Split on '-' (or ',' if you want standard sed syntax)
    if let Some((start_str, end_str)) = l.split_once('-') {
      let start = start_str.trim().parse::<usize>().ok()?.saturating_sub(1);
      let end = end_str.trim().parse::<usize>().ok()?.saturating_sub(1);
      Some(start..=end)
    } else {
      let single = l.trim().parse::<usize>().ok()?.saturating_sub(1);
      Some(single..=single) // A single line is just a range from N to N
    }
  }

  fn update_content(
    original: &String,
    content: &String,
    anchor: Option<String>,
    line_range: Option<RangeInclusive<usize>>,
  ) -> Result<String, ObiError> {
    // Substitute anchor
    if let Some(a) = anchor {
      let count = original.matches(a.as_str()).count();
      return match count {
        0 => Err(ObiError::AnchorNotFound(a)),
        1 => Ok(original.replacen(&a, content, 1)),
        _ => Err(ObiError::AnchorAmbiguous { anchor: a, count }),
      };
    }

    if let Some(lr) = &line_range {
      let total = original.lines().count();
      if *lr.end() >= total {
        return Err(ObiError::LineOutOfRange {
          start: *lr.start() + 1, // report 1-based to match what the user typed
          end: *lr.end() + 1,
          total,
        });
      }
      let mut modified_lines: Vec<String> = original
        .clone()
        .lines()
        .enumerate()
        .filter_map(|(index, line)| {
          if index == *lr.start() {
            Some(content.to_string()) // replace the whole range with one copy
          } else if lr.contains(&index) {
            None // rest of the range: drop
          } else {
            Some(line.to_string()) // outside the range: keep
          }
        })
        .collect();

      // lines() strips the trailing newline, so restore it if the original had one
      if original.ends_with('\n') {
        modified_lines.push(String::new());
      }

      let eol = if original.contains("\r\n") {
        "\r\n"
      } else {
        "\n"
      };

      return Ok(modified_lines.join(eol));
    }

    return Ok(original.to_owned());
  }
}
