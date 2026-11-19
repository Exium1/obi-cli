use std::ops::RangeInclusive;

use crate::{
  commands::obsidian::{obsidian_create, obsidian_file, obsidian_read},
  errors::ObiError,
  util::{parse_line_range, read_stdin_content},
};
use clap::{ArgGroup, Parser};
use similar::{TextDiff};

#[derive(Parser, Debug)]
#[command(
  disable_help_flag = true,
  group(
    ArgGroup::new("position")
      .required(true)
      .args(["line", "anchor"]),
  ),
  group(
    ArgGroup::new("open_location")
      .required(false)
      .args(["open", "newtab"]),
  ),
  
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

  #[arg(long)]
  dryrun: bool,
}

impl WriteCommand {
  pub fn execute(self) -> Result<(), ObiError> {
    println!("========================");
    println!("Path:\t\t{:?}", self.path);
    println!("Name:\t\t{:?}", self.name);
    println!("Content:\t{:?}", self.content);
    println!("Line:\t\t{:?}", self.line);
    println!("Anchor:\t\t{:?}", self.anchor);
    println!("Newtab:\t\t{:?}", self.newtab);
    println!("Open:\t\t{:?}", self.open);
    println!("Dryrun:\t\t{:?}", self.dryrun);
    println!("========================");

    // Get content from stdin if '-'
    let content = if self.content == "-" {
      read_stdin_content()?
    } else {
      self.content.clone()
    };

    // Get target path from cmd or `obsidian file`
    let target_path = match self.path {
      Some(path) => path,
      None => obsidian_file()?.path,
    };

    // Calculate line range for `line` arg
    let line_range = parse_line_range(self.line);

    // Get note contents using target path
    let read_output = obsidian_read(self.name.as_ref(), Some(&target_path))?;

    // Calculate modified content
    let modified_content =
      WriteCommand::update_content(&read_output, &content, self.anchor, line_range)?;

    println!("");

    if self.dryrun {
      let diff = TextDiff::from_lines(read_output, modified_content);

      let output = diff
        .unified_diff()
        .context_radius(0) // Hides all unchanged lines
        .to_string();

      println!("{}", output);
    } else {
      // Update obsidian note with new content and target path
      obsidian_create(
        &target_path,
        &self.name,
        &modified_content,
        self.open,
        self.newtab,
      )?;
    }

    Ok(())
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
