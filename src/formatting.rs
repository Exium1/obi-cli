use std::ops::RangeInclusive;

use clap::{ArgAction, Args, Parser, ValueEnum, ValueHint::Unknown};
use serde::Serialize;
use serde_with::skip_serializing_none;
use derive_new::new;

#[derive(Debug, Serialize)]

pub struct HeadingOutput {
  level: u8,
  text: String,
  range: RangeInclusive<usize>,
}

#[derive(Debug, Serialize)]

pub struct OutlineOutput {
  frontmatter: String,
  headings: Vec<HeadingOutput>
}

#[skip_serializing_none]
#[derive(Debug, Serialize, new)]
pub struct SuccessReadOutput {
  status: String,
  path: String,
  hash: String,
  content: String,
  lines: usize,
  range: Option<RangeInclusive<usize>>,
  outline: Option<OutlineOutput>
}

#[derive(Debug, Serialize)]

pub struct LinesOutput {
  before: RangeInclusive<usize>,
  after: RangeInclusive<usize>,
}

#[derive(Debug, Serialize)]
pub struct SuccessWriteOutput {
  status: String,
  path: String,
  hash: String,
  command: String,
  lines: Option<LinesOutput>,
  dry_run: bool
}

#[derive(Debug, Serialize)]

pub struct Matches {
  matches: Vec<String>
}

#[derive(Debug, Serialize)]
pub struct ErrorOutput {
  status: String,
  code: usize,
  error: String,
  matches: Option<Matches>
}

#[derive(ValueEnum, Clone, Debug)]
pub enum FormatType {
  Json,
  Yaml,
  Toml
}

pub trait Format {
  fn format_to(&self, f: FormatType) -> Option<String>;
}

impl Format for SuccessReadOutput {
  fn format_to(&self, f: FormatType) -> Option<String> {

    match f {
      FormatType::Json => 
        serde_json::to_string_pretty(&self).ok(),
      FormatType::Yaml => 
        serde_yaml_ng::to_string(&self).ok(),
      FormatType::Toml =>
        toml::to_string_pretty(&self).ok(),
    }
  }
}