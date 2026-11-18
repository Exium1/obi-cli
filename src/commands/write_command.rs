use clap::Parser;

#[derive(Parser, Debug)]
#[command(disable_help_flag = true)]
pub struct WriteCommand {
  #[arg(long)]
  name: Option<String>,

  #[arg(long)]
  path: Option<String>,

  #[arg(long)]
  content: Option<String>,

  #[arg(long)]
  line: Option<String>,

  #[arg(long)]
  anchor: Option<String>,

  #[arg(long)]
  open: bool,

  #[arg(long)]
  newtab: bool,
}
