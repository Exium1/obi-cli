use std::io::{Read, stdin};

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
