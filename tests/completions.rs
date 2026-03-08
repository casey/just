use super::*;

#[test]
fn bash() {
  if cfg!(not(target_os = "linux")) {
    return;
  }
  let output = Command::new(JUST)
    .args(["--completions", "bash"])
    .output()
    .unwrap();

  assert!(output.status.success());

  let script = str::from_utf8(&output.stdout).unwrap();

  let tempdir = tempdir();

  let path = tempdir.path().join("just.bash");

  fs::write(&path, script).unwrap();

  let status = Command::new("./tests/completions/just.bash")
    .arg(path)
    .status()
    .unwrap();

  assert!(status.success());
}

#[test]
fn zsh() {
  if which("zsh").is_err() {
    return;
  }

  let output = Command::new(JUST)
    .args(["--completions", "zsh"])
    .output()
    .unwrap();

  assert!(output.status.success());

  let script = str::from_utf8(&output.stdout).unwrap();

  let tempdir = tempdir();

  let path = tempdir.path().join("just.zsh");

  fs::write(&path, script).unwrap();

  let status = Command::new("zsh")
    .arg("./tests/completions/just.zsh")
    .arg(path)
    .status()
    .unwrap();

  assert!(status.success());
}

#[test]
fn fish() {
  if which("fish").is_err() {
    return;
  }

  let output = Command::new(JUST)
    .args(["--completions", "fish"])
    .output()
    .unwrap();

  assert!(output.status.success());

  let script = str::from_utf8(&output.stdout).unwrap();

  let tempdir = tempdir();

  let path = tempdir.path().join("just.fish");

  fs::write(&path, script).unwrap();

  let status = Command::new("fish")
    .arg("./tests/completions/just.fish")
    .arg(path)
    .status()
    .unwrap();

  assert!(status.success());
}

#[test]
fn powershell() {
  if which("pwsh").is_err() {
    return;
  }

  let output = Command::new(JUST)
    .args(["--completions", "powershell"])
    .output()
    .unwrap();

  assert!(output.status.success());

  let script = str::from_utf8(&output.stdout).unwrap();

  let tempdir = tempdir();

  let path = tempdir.path().join("just.ps1");

  fs::write(&path, script).unwrap();

  let status = Command::new("pwsh")
    .arg("-NoProfile")
    .arg("-File")
    .arg("./tests/completions/just.powershell")
    .arg(path)
    .status()
    .unwrap();

  assert!(status.success());
}

#[test]
fn nushell() {
  if which("nu").is_err() {
    return;
  }

  let output = Command::new(JUST)
    .args(["--completions", "nushell"])
    .output()
    .unwrap();

  assert!(output.status.success());

  let script = str::from_utf8(&output.stdout).unwrap();

  let tempdir = tempdir();

  let path = tempdir.path().join("just.nu");

  fs::write(&path, script).unwrap();

  let command = format!(
    "source '{}'; cd tests/completions; let root = (nu-complete just 'just p' | get value | str join ' '); if $root != 'publish push' {{ error make {{msg: $'unexpected root: ($root)'}} }}; let nested = (nu-complete just 'just repo o' | get value | str join ' '); if $nested != 'open' {{ error make {{msg: $'unexpected nested: ($nested)'}} }}; let recipes = (nu-complete just 'just repo open c' | get value | str join ' '); if $recipes != 'codex' {{ error make {{msg: $'unexpected recipe: ($recipes)'}} }}",
    path.display()
  );

  let status = Command::new("nu")
    .arg("-c")
    .arg(command)
    .status()
    .unwrap();

  assert!(status.success());
}

#[test]
fn replacements() {
  for shell in ["bash", "elvish", "fish", "nushell", "powershell", "zsh"] {
    let output = Command::new(JUST)
      .args(["--completions", shell])
      .output()
      .unwrap();
    assert!(
      output.status.success(),
      "shell completion generation for {shell} failed: {}\n{}",
      output.status,
      String::from_utf8_lossy(&output.stderr),
    );
  }
}
