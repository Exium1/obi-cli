use std::process::{Command, Output};
mod commands;

fn main() {
  let mut args: Vec<String> = std::env::args().collect();
  args.remove(0); // don't need bin, just args now

  let cmd: &String = &(args[0]);

  match cmd.as_str() {
    "write" => {
      commands::write::execute(args);
      return ();
    }
    _ => {
      obsidian(args, false);
      return ();
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
