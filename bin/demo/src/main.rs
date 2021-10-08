use std::{
  error::Error,
  io::{stdout, Write},
  process::Command,
  thread::{sleep, spawn},
  time::Duration,
};

type Result<T, E = Box<dyn Error>> = std::result::Result<T, E>;

const SCRIPT: &str = "
  just --list
  just build
  ./main
";

const BACKGROUND_COMMANDS: [&str; 10] = [
  "nvr --remote-send 'i'",
  "nvr --remote-send 'alias b := build<enter><enter>'",
  "nvr --remote-send 'host := `uname -a`<enter><enter>'",
  "nvr --remote-send '# build main<enter>'",
  "nvr --remote-send 'build:<enter>    cc *.c -o main<enter><esc>i<enter>'",
  "nvr --remote-send '# test everything<enter>'",
  "nvr --remote-send 'test-all: build<enter>    ./test --all<enter><esc>i<enter>'",
  "nvr --remote-send '# run a specific test<enter>'",
  "nvr --remote-send 'test TEST: build<enter>    ./test --test {{TEST}}<enter><esc>'",
  "nvr --remote-send ':x<enter>'",
];

const PROMPT: &str = "\x1b[0;34m$\x1b[0m ";

const CPM: u64 = 1000;

fn commands(block: &'static str) -> Vec<Vec<&'static str>> {
  block
    .lines()
    .map(|line| line.trim())
    .filter(|line| !line.is_empty())
    .map(|line| line.split(' ').collect())
    .collect()
}

fn print(text: &str) -> Result<()> {
  stdout().write_all(text.as_bytes())?;
  stdout().flush()?;
  Ok(())
}

fn run(command: &[&str]) -> Result<()> {
  Command::new(command[0])
    .args(&command[1..])
    .current_dir("tmp")
    .status()?;
  Ok(())
}

fn run_background_commands(line_delay: Duration) {
  spawn(move || {
    sleep(line_delay);
    for &background_command in BACKGROUND_COMMANDS.iter() {
      let parts = shlex::split(background_command).expect("Could not parse shell command");
      let mut parts_iter = parts.iter();
      Command::new(parts_iter.next().unwrap())
        .args(parts_iter.collect::<Vec<_>>())
        .current_dir("tmp")
        .status()
        .expect("Command failed");
      sleep(line_delay * 3);
    }
  });
}

fn main() -> Result<()> {
  let char_delay = Duration::from_millis(1000 * 60 / CPM);
  let line_delay = char_delay * 7;
  let enter_delay = char_delay * 5;

  run_background_commands(line_delay);
  run(&["nvr", "-s", "justfile"])?;

  for (i, command) in commands(SCRIPT).iter().enumerate() {
    print(PROMPT)?;

    if i > 0 {
      sleep(line_delay);
    }

    let line = command.join(" ");
    for c in line.chars() {
      sleep(char_delay);
      print(&c.to_string())?;
    }

    sleep(enter_delay);
    print("\n")?;

    run(&command)?;
  }

  Ok(())
}
