macro_rules! warn {
  ($($arg:tt)*) => {{
    extern crate std;
    use std::io::prelude::*;
    let _ = writeln!(&mut std::io::stderr(), $($arg)*);
  }};
}
macro_rules! die {
  ($($arg:tt)*) => {{
    extern crate std;
    warn!($($arg)*);
    std::process::exit(-1)
  }};
}

#[derive(PartialEq, Clone, Copy)]
enum Make {
  GNU,         // GNU Make installed as `gmake`
  GNUStealthy, // GNU Make installed as `make`
  Other,       // Another make installed as `make`
}

impl Make {
  fn command(self) -> &'static str {
    if self == Make::GNU {
      "gmake"
    } else {
      "make"
    }
  }

  fn gnu(self) -> bool {
    self != Make::Other
  }
}

fn status(command: &mut std::process::Command) -> std::io::Result<std::process::ExitStatus> {
  command
    .stdin(std::process::Stdio::null())
    .stdout(std::process::Stdio::null())
    .stderr(std::process::Stdio::null())
    .status()
}

fn which_make() -> Option<Make> {
  // check `gmake`
  let result = status(std::process::Command::new("gmake").arg("-v"));

  if let Ok(exit_status) = result {
    if exit_status.success() {
      return Some(Make::GNU);
    }
  }

  // check `make`. pass gmake specific flags to see if it's actually gmake
  let result = status(std::process::Command::new("make").arg("-v").arg("--always-make"));

  if let Ok(exit_status) = result {
    return if exit_status.success() {
      Some(Make::GNUStealthy)
    } else {
      Some(Make::Other)
    };
  }

  return None;
}

fn main() {
  let make = match which_make() {
    None => die!("Could not execute `make` or `gmake`."),
    Some(make) => make,
  };

  let mut justfile = "justfile";

  loop {
    match std::fs::metadata("justfile") {
      Ok(metadata) => if metadata.is_file() { break; },
      Err(error) => die!("Error fetching justfile metadata: {}", error),
    }

    match std::fs::metadata("Justfile") {
      Ok(metadata) => if metadata.is_file() {
        justfile = "Justfile";
        break; 
      },
      Err(error) => die!("Error fetching justfile metadata: {}", error),
    }

    match std::env::current_dir() {
      Ok(pathbuf) => if pathbuf.as_os_str() == "/" { die!("No justfile found"); },
      Err(error) => die!("Error getting current dir: {}", error),
    }

    if let Err(error) = std::env::set_current_dir("..") {
      die!("Error changing directory: {}", error);
    }
  }
  
  let recipes: Vec<String> = std::env::args().skip(1).take_while(|arg| arg != "--").collect();
  let arguments: Vec<String> = std::env::args().skip(1 + recipes.len() + 1).collect();

  for (i, argument) in arguments.into_iter().enumerate() {
    std::env::set_var(format!("ARG{}", i), argument);
  }

  let mut command = std::process::Command::new(make.command());

  command.arg("MAKEFLAGS=");

  if make.gnu() {
    command.arg("--always-make").arg("--no-print-directory");
  }

  command.arg("-f").arg(justfile);

  for recipe in recipes {
    command.arg(recipe);
  }

  match command.status() {
    Err(error) => die!("Failed to execute `{:?}`: {}", command, error),
    Ok(exit_status) => match exit_status.code() {
      Some(code) => std::process::exit(code),
      None => std::process::exit(-1),
    }
  }
}
