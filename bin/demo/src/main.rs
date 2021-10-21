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

const JUSTFILE: &str = "alias b := build

host := `uname -a`

# build main
build:
    cc *.c -o main

# test everything
test-all: build
    ./test --all

# run a specific test
test TEST: build
    ./test --test {{TEST}}
";

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

fn send_char_to_nvr(c: char) {
  let to_send = if c == '\n' {
    String::from("<enter>")
  } else if c == '\\' {
    String::from("<esc>")
  } else {
    c.to_string()
  };
  let parts = shlex::split(&format!("nvr --remote-send '{}'", to_send))
    .expect("Could not parse shell command");
  let mut parts_iter = parts.iter();

  Command::new(parts_iter.next().unwrap())
    .args(parts_iter.collect::<Vec<_>>())
    .current_dir("tmp")
    .status()
    .expect("Command failed");
}

fn run_background_commands(line_delay: Duration) {
  spawn(move || {
    sleep(line_delay);
    send_char_to_nvr('i');
    sleep(line_delay);

    for c in JUSTFILE.chars() {
      send_char_to_nvr(c);
    }

    for c in "\\:x\n".chars() {
      send_char_to_nvr(c);
      sleep(line_delay);
    }
  });
}

fn main() -> Result<()> {
  let char_delay = Duration::from_millis(1000 * 60 / CPM);
  let line_delay = char_delay * 7;
  let enter_delay = char_delay * 15;

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
