use std::io::prelude::*;

fn can(command: &str) -> bool {
  if let Ok(paths) = std::env::var("PATH") {
    for path in paths.split(":") {
      let candidate = format!("{}/{}", path, command);
      if isfile(&candidate) {
        return true;
      }
    }
  }
  false
}

fn isfile(path: &str) -> bool {
  if let Ok(metadata) = std::fs::metadata(path) {
    metadata.is_file()
  } else {
    false
  }
}

fn cwd() -> String {
  match std::env::current_dir() {
    Ok(pathbuf) => pathbuf.to_str().unwrap_or_else(|| panic!("cwd: cwd was not a valid utf8 string")).to_string(),
    Err(err) => panic!("cwd: {}", err),
  }
}

fn cd(path: &str) {
  if let Err(err) = std::env::set_current_dir(path) {
    panic!("cd: {}", err)
  }
}

fn say(s: &str) {
  println!("{}", s)
}

fn warn(s: &str) {
  if let Err(err) = std::io::stderr().write(s.as_bytes()) {
    panic!("warn: could not write to stderr: {}", err);
  }
  if let Err(err) = std::io::stderr().write("\n".as_bytes()) {
    panic!("warn: could not write to stderr: {}", err);
  }
}

fn die(s: &str) {
  warn(s);
  std::process::exit(-1);
}

fn main() {
  let can_make = can("make");
  let can_gmake = can("gmake");
  if !(can_make || can_gmake) {
    die("cannot find \"make\" or \"gmake\" in $PATH");
  }

  println!("can make: {}", can_make);
  println!("can gmake: {}", can_gmake);

  loop {
    if isfile("justfile") {
      break;
    }
    if cwd() == "/" {
      die("No justfile found.")
    }
    cd("..");
  }

  
  let recipes: Vec<String> = std::env::args().skip(1).take_while(|arg| arg != "--").collect();
  let arguments: Vec<String> = std::env::args().skip(1 + recipes.len() + 1).collect();

  print!("{:?}", recipes);
  print!("{:?}", arguments);

  // export as $ARG0 -> ARGX
  // start at 0 or 1?
  // exec $MAKE MAKEFLAGS='' --always-make --no-print-directory -f justfile ${RECIPES[*]}
}
