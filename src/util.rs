use std::io::{Read, stdin};
use std::ops::RangeInclusive;

use crate::errors::ObiError;

pub fn read_stdin_content() -> Result<String, ObiError> {
  let mut buf = String::new();

  match stdin().read_to_string(&mut buf) {
    Ok(_) => (),
    Err(e) => return Err(ObiError::Stdin(e)),
  }

  // Drop a UTF-8 BOM if present (PowerShell and some Windows tools add one)
  if let Some(rest) = buf.strip_prefix('\u{feff}') {
    buf = rest.to_owned();
  }

  // Drop exactly one trailing line ending
  if buf.ends_with('\n') {
    buf.pop();
    if buf.ends_with('\r') {
      buf.pop();
    }
  }

  Ok(buf)
}

pub fn parse_line_range(line: Option<String>) -> Option<RangeInclusive<usize>> {
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
