use std::{
  io::{Error, IsTerminal, Read, stdin}, process::{Command, Output, Stdio},
};

pub struct WriteCommand {
  name: Option<String>,
  path: Option<String>,
  content: String,
  line: Option<String>,
  anchor: Option<String>,
  open: bool,
  newtab: bool,
}

impl WriteCommand {
  fn new(
    name: Option<String>,
    path: Option<String>,
    content_raw: Option<String>,
    line: Option<String>,
    anchor: Option<String>,
    open: bool,
    newtab: bool,
  ) -> Self {

    // Get input from stdin if content isn't set
    let content: String = match content_raw {
      Some(text) => text,
      None => {

        let mut stdin = stdin();
        let mut buffer = String::new();

        if !stdin.is_terminal() {
          let _ = stdin.read_to_string(&mut buffer);
        }

        buffer
      }
    };

    Self {
      name,
      path,
      content,
      line,
      anchor,
      open,
      newtab,
    }
  }

  pub fn from_args(
    args: &Vec<String>
  ) -> Self {

    let mut name: Option<String> = None;
    let mut path: Option<String> = None;
    let mut content: Option<String> = None;
    let mut line: Option<String> = None;
    let mut anchor: Option<String> = None;
    let mut open = false;
    let mut newtab = false;

    // Extract all args into command
    for arg in args {
      if arg.contains("=") {
        let split: Option<(&str, &str)> = arg.split_once("=");

        match split {
          Some((p, s)) => match p {
            "name" => name = Some(String::from(s)),
            "path" => path = Some(String::from(s)),
            "content" => content = Some(String::from(s)),
            "line" => line = Some(String::from(s)),
            "anchor" => anchor = Some(String::from(s)),
            _ => continue,
          },
          None => continue,
        }
      } else if arg.eq("newtab") {
        newtab = true;
      } else if arg.eq("open") {
        open = true;
      }
    }

    WriteCommand::new(name, path, content, line, anchor, open, newtab)
  }

  pub fn execute(self) {
    // Read from obsidian
    let mut args_read = vec!["read".to_string()];

    match self.path {
      Some(path) => args_read.push(format!("path={}", path)),
      None => (),
    }
    
    match self.name {
      Some(name) => args_read.push(format!("name={}", name)),
      None => (),
    }

    let read_output = obsidian_cli(args_read).expect("Failed to get obsidian read output");
    // let read_text = String::from_utf8_lossy(&(read_output.stdout));
    // let target_line = line.map(|l| l.saturating_sub(1)); // Convert 1-based to 0-based index

    println!("output: {:?}", String::from_utf8(read_output.stdout));

    // Modify lines
    println!("content: {}", self.content);

    // Write to obsidian
  }
}

fn obsidian_cli(args: Vec<String>) -> Result<Output, Error> {
  Command::new("obsidian")
    .args(args)
    .stdout(Stdio::piped())
    .output()
}

/* 
pub fn execute(args: Vec<String>) {
  println!("Write command...");

  let mut cmd = WriteCommand {
    name: None,
    path: None,
    content: None,
    line: None,
    anchor: None,
    open: false,
    newtab: false,
  };

  // Extract all args into command
  for arg in &args {
    if arg.contains("=") {
      let split: Option<(&str, &str)> = arg.split_once("=");

      match split {
        Some((p, s)) => match p {
          "name" => cmd.name = Some(String::from(s)),
          "path" => cmd.path = Some(String::from(s)),
          "content" => cmd.content = Some(String::from(s)),
          "line" => cmd.line = Some(String::from(s)),
          "anchor" => cmd.anchor = Some(String::from(s)),
          _ => continue,
        },
        None => continue,
      }
    } else if arg.eq("newtab") {
      cmd.newtab = true;
    } else if arg.eq("open") {
      cmd.open = true;
    }
  }

  println!("Path: {:?}", cmd.path);
  println!("Name: {:?}", cmd.name);
  println!("Content: {:?}", cmd.content);
  println!("Line: {:?}", cmd.line);
  println!("Anchor: {:?}", cmd.anchor);
  println!("Newtab: {:?}", cmd.newtab);
  println!("Open: {:?}", cmd.open);

  // Require `content` field
  if cmd.content.is_none() {
    println!("Content value is needed.");
    exit(1);
  }

  // Require either line or anchor
  if cmd.anchor.is_none() && cmd.line.is_none() {
    println!("A line number or anchor is needed.");
    exit(1);
  }

  // Get the target file to write to
  let mut target_file: Option<String> = cmd.path;

  if target_file.is_none() {
    target_file = cmd.name
  }

  if target_file.is_none() {
    let output = obsidian(vec!["file".to_string()], true);

    if output.is_some() {
      let out_string = String::from_utf8(output.expect("Get value of output").stdout);
      let stdout = out_string.expect("msg");
      let re = Regex::new(r"path\t(?<path>.+?.md)\n").unwrap();

      if let Some(caps) = re.captures(stdout.as_str()) {
        target_file = Some((&caps["path"]).to_string());
      } else {
        println!("No match found.");
      }
    }
  } else {
    let temp = target_file.clone().expect("Is not none");
    if !temp.ends_with(".md") {
      target_file = Some(temp + ".md");
    }
  }

  if target_file.is_none() {
    exit(1)
  }

  // println!("Target file is: {:?}", target_file);

  // let out: Result<(), std::io::Error> = piped_file_read(vec![
  //   "read".to_string(),
  //   "path=".to_string() + &(target_file.expect("target file should exist")),
  // ]);

  let out = piped_replace(
    target_file.expect("target file should exist"),
    cmd.content.expect("Content is not null"),
    cmd.anchor,
    cmd.line,
  );

  // println!("{:}", out);

  // Validate cmd struct
}

fn piped_file_read(args: Vec<String>) -> std::io::Result<()> {
  let mut obsidian = Command::new("obsidian")
    .args(args)
    .stdout(Stdio::piped())
    .spawn()?;

  let obsidian_out = obsidian.stdout.take().expect("Failed to open echo stdout");
  let wc = Command::new("wc")
    .arg("-l")
    .stdin(Stdio::from(obsidian_out))
    .output()?;

  obsidian.wait()?;

  println!("Word count: {}", String::from_utf8_lossy(&wc.stdout).trim());

  return Ok(());
}

fn piped_replace(
  path: String,
  content: String,
  anchor: Option<String>,
  line: Option<String>,
) -> std::io::Result<()> {
  if anchor.is_none() && line.is_none() {
    exit(1);
  }

  let output = Command::new("obsidian")
    .args(["read", &format!("path={}", path)])
    .stdout(Stdio::piped())
    .output()?;

  let text = String::from_utf8_lossy(&output.stdout);
  let target_range = parse_line_range(line);

  let modified: Vec<String> = text
    .lines()
    .enumerate()
    .map(|(idx, current_line)| {
      // 3. Transform each line
      if let Some(range) = &target_range {
        if !range.contains(&idx) {
          return current_line.to_string();
        }
      }

      match &anchor {
        Some(a) => current_line.replacen(a, &content, 1),
        None => content.clone(),
      }
    })
    .collect();

  println!("output:\n{}", modified.join("\n"));

  Ok(())
}

fn parse_line_range(line: Option<String>) -> Option<RangeInclusive<usize>> {
  let l = line?; // Returns None if line is None

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

fn obsidian_read(args: Vec<String>) -> std::io::Result<String> {
  let output = Command::new("obsidian").args(args).output()?;

  let text = String::from_utf8_lossy(&output.stdout).to_string();
  Ok(text)
}

fn obsidian_read_stdout(args: Vec<String>) -> Result<Child, Error> {
  let read_stdout = Command::new("obsidian")
    .args(args)
    .stdout(Stdio::piped())
    .spawn();

  return read_stdout;
}
*/