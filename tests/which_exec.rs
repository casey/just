use super::*;

fn make_path() -> TempDir {
  let tmp = temptree! {
    "hello.exe": "#!/usr/bin/env bash\necho hello\n",
  };

  #[cfg(not(windows))]
  {
    let exe = tmp.path().join("hello.exe");
    let perms = std::os::unix::fs::PermissionsExt::from_mode(0o755);
    fs::set_permissions(exe, perms).unwrap();
  }

  tmp
}

#[test]
fn finds_executable() {
  let tmp = make_path();
  let mut path = env::current_dir().unwrap();
  path.push("bin");
  Test::new()
    .justfile(r#"p := which("hello.exe")"#)
    .env("PATH", tmp.path().to_str().unwrap())
    .args(["--evaluate", "p"])
    .stdout(format!("{}", tmp.path().join("hello.exe").display()))
    .run();
}

#[test]
fn prints_empty_string_for_missing_executable() {
  let tmp = make_path();
  let mut path = env::current_dir().unwrap();
  path.push("bin");
  Test::new()
    .justfile(r#"p := which("goodbye.exe")"#)
    .env("PATH", tmp.path().to_str().unwrap())
    .args(["--evaluate", "p"])
    .stdout("")
    .run();
}
