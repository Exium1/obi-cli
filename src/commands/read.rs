use std::ops::RangeInclusive;

use clap::{ArgAction, Args, Parser};

use crate::{Runnable, commands::{obsidian::obsidian_read, replace::NoteArgs}, errors::ObiError, formatting::{SuccessReadOutput, Format, FormatType}, util::{get_target_path, hash_content, parse_line_range}};

#[derive(Parser, Debug)]
#[command(
  about = "Print a note, or part of one",
  disable_help_flag = true,
  disable_version_flag = true,
)]
pub struct ReadCommand {

  #[command(flatten)]
  note: NoteArgs,

  #[command(flatten)]
  part: PartArgs,

  #[command(flatten)]
  matching: MatchingArgs,

  /// Print hash, lines, range, content as format
  #[arg(short, long, value_enum, value_name = "F", help_heading = "Output")]
  format: Option<FormatType>,

  /// Prefix each line with its line number
  #[arg(long, help_heading = "Output")]
  numbered: bool,

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
#[group(multiple = false)]
#[command(next_help_heading = "Section (only one)")]
struct PartArgs {
  /// Only this section, heading included
  #[arg(short = 'H', long, value_name = "H")]
  heading: Option<String>,

  /// Only this line or inclusive range (1-based) (Ex: "14" or "2-7")
  #[arg(short, long, value_name = "N|N-M")]
  line: Option<String>,
}

#[derive(Args, Debug)]
#[command(next_help_heading = "Matching")]
struct MatchingArgs {
  /// Nth section when >1 headings
  #[arg(long, default_value_t = 1, value_name = "N")]
  occurrence: usize
}


// #[derive(Args, Debug)]
// #[group(multiple = false)]
// #[command(next_help_heading = "Output")]
// struct OutputArgs {
//   /// Print as JSON with path, hash, lines, range, content
//   #[arg(long)]
//   json: bool,

//   /// Prefix each line with its line number
//   #[arg(long)]
//   numbered: bool
// }

impl Runnable for ReadCommand {
  fn run(self) -> Result<(), ObiError> {

    // Fetch content
    let raw_output = obsidian_read(self.note.name.as_ref(), self.note.path.as_ref())?;
    let output: String;

    // part args
    if let Some(heading) = self.part.heading {
      let section = extract_section(&raw_output, &heading, self.matching.occurrence)
        .unwrap_or("".into());
      output = section;
    } else if let Some(line) = self.part.line {
      // Calculate line range for `line` arg
      if let Some(lr) = parse_line_range(&line) {
        let total_lines = raw_output.lines().count();
        if *lr.end() >= total_lines {
          return Err(ObiError::LineOutOfRange { 
            start: *lr.start() + 1, 
            end: *lr.end() + 1, 
            total: total_lines
          })
        }

        output = get_line_slice(&raw_output, lr);

      } else {
        return Err(ObiError::InvalidLine { input: line, reason: " line or line range could not be parsed".into() })
      }
    } else {
      output = raw_output.clone();
    }

    // format output (json vs numbered vs raw)
    // match self.format {
    //   Some(FormatType::Json) => println!("{:}", add_line_numbers(&output)),
    //   _ => println!("{:}", output),
    // }
    
    if let Some(f) = self.format {
      // Get some additional info for formatting
      let target_path = get_target_path(self.note.path)?;
      let hash = hash_content(&raw_output);
      let read_output = SuccessReadOutput::new(
        "ok".to_string(),
        target_path,
        hash,
        output.clone(),
        output.lines().count(),
        Some(raw_output.lines().count()),
        None,
        None
      );

      if let Some(output_formatted) = read_output.format_to(f) {
        println!("{:}", output_formatted);
      } else {
        println!("Unable to format the output.")
      }
    } else {
      if self.numbered {
        println!("{:}", add_line_numbers(&output))
      } else {
        println!("{:}", &output);
      }
    }

    Ok(())
  }
}

pub fn extract_section(content: &String, heading: &String, nth: usize) -> Option<String> {

  let mut capturing = false;
  let mut result = Vec::new();
  let mut target_level = 0;
  let mut matches_seen = 1;

  for line in content.lines() {
    let trimmed = line.trim_start();
    let hashes = trimmed.chars().take_while(|&c| c == '#').count();
    let is_heading = hashes > 0 && trimmed.chars().nth(hashes) == Some(' ');

    if is_heading {
      if capturing {
        if hashes <= target_level {
          break;
        }
      } else if trimmed == heading {
        if matches_seen == nth {
          capturing = true;
          target_level = hashes;
        }
        matches_seen += 1;
      }
    }

    if capturing {
      result.push(line);
    }
  }

  if result.is_empty() {
    None
  } else {
    Some(result.join(new_line_type(content)))
  }
}

fn get_line_slice(text: &str, range: RangeInclusive<usize>) -> String {

  let start = *range.start();
  let end = *range.end();
  let count = end.saturating_sub(start) + 1;

  text.lines()
    .skip(start)
    .take(count)
    .collect::<Vec<_>>()
    .join(new_line_type(text))
}

pub fn new_line_type(text: &str) -> &str {
  return if text.contains("\r\n") {
    "\r\n"
  } else {
    "\n"
  };
}

pub fn add_line_numbers(input: &str) -> String {
  input
    .lines()
    .enumerate()
    .map(|(i, line)| format!("{:>2}: {}", i + 1, line))
    .collect::<Vec<_>>()
    .join("\n")
}