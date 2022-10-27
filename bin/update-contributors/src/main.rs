use {
  regex::{Captures, Regex},
  std::{fs, process::Command, str},
};

fn author(pr: u64) -> String {
  eprintln!("#{}", pr);
  let output = Command::new("sh")
    .args(&[
      "-c",
      &format!("gh pr view {} --json author | jq -r .author.login", pr),
    ])
    .output()
    .unwrap();

  assert!(
    output.status.success(),
    "{}",
    String::from_utf8_lossy(&output.stderr)
  );

  str::from_utf8(&output.stdout).unwrap().trim().to_owned()
}

fn main() {
  fs::write(
    "CHANGELOG.md",
    &*Regex::new(r"\(#(\d+)\)").unwrap().replace_all(
      &fs::read_to_string("CHANGELOG.md").unwrap(),
      |captures: &Captures| {
        let pr = captures[1].parse().unwrap();
        match author(pr).as_str() {
          "casey" => format!("(#{})", pr),
          contributor => format!("(#{} by @{})", pr, contributor),
        }
      },
    ),
  )
  .unwrap();
}
