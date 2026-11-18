use std::{io, process::ExitStatus};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ObiError {
  // Input problems (the caller can fix these)
  #[error("invalid --line {input:?}: {reason}")]
  InvalidLine { input: String, reason: String },

  #[error("line range {start}-{end} is out of bounds: note has {total} lines")]
  LineOutOfRange {
    start: usize,
    end: usize,
    total: usize,
  },

  #[error("anchor not found in note: {0:?}")]
  AnchorNotFound(String),

  #[error("anchor {anchor:?} matches {count} times; use --all or a more specific anchor")]
  AnchorAmbiguous { anchor: String, count: usize },

  #[error("failed to read content from stdin: {0}")]
  Stdin(#[source] io::Error),

  // Obsidian CLI problems (environment or the other tool)
  #[error("could not launch the obsidian CLI (is Obsidian installed and running?): {0}")]
  ObsidianLaunch(#[source] io::Error),

  #[error("obsidian CLI failed ({status}): {message}")]
  ObsidianFailed { status: ExitStatus, message: String },
}

impl ObiError {
  /// Distinct exit codes let scripts and agents tell the categories apart.
  pub fn exit_code(&self) -> u8 {
    match self {
      Self::InvalidLine { .. }
      | Self::LineOutOfRange { .. }
      | Self::AnchorNotFound(_)
      | Self::AnchorAmbiguous { .. }
      | Self::Stdin(_) => 2,
      Self::ObsidianLaunch(_) | Self::ObsidianFailed { .. } => 3,
    }
  }
}
