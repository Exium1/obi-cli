use std::process::{Command, Output};

use crate::commands::write::WriteCommand;
mod commands;

fn main() {
  let full_args: Vec<String> = std::env::args().collect();
  let args = full_args[1..].to_vec();
  let cmd: &String = &(args[0]);

  match cmd.as_str() {
    "write" => {
      let write_cmd: WriteCommand = WriteCommand::from_args(&args);
      write_cmd.execute();
    }
    _ => {
      obsidian(args, false);
    }
  }
}

fn obsidian(args: Vec<String>, output: bool) -> Option<Output> {
  println!("obsidian-cli...");
  println!("{:?}", args);

  let mut binding = Command::new("obsidian");
  let cmd = binding.args(args);

  if output {
    return Some(cmd.output().expect("obsidian cli should execute"));
  } else {
    cmd.spawn().expect("obsidian cli should execute");
    return None;
  }
}
