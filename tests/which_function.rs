use super::*;

trait TempDirExt {
  fn executable(self, file: impl AsRef<Path>) -> Self;
}

impl TempDirExt for TempDir {
  fn executable(self, file: impl AsRef<Path>) -> Self {
    let file = self.path().join(file.as_ref());

    // Make sure it exists first, as a sanity check.
    assert!(
      file.exists(),
      "executable file does not exist: {}",
      file.display()
    );

    // Windows uses file extensions to determine whether a file is executable.
    // Other systems don't care. To keep these tests cross-platform, just make
    // sure all executables end with ".exe" suffix.
    assert!(
      file.extension() == Some("exe".as_ref()),
      "executable file does not end with .exe: {}",
      file.display()
    );

    #[cfg(not(windows))]
    {
      let perms = std::os::unix::fs::PermissionsExt::from_mode(0o755);
      fs::set_permissions(file, perms).unwrap();
    }

    self
  }
}

#[test]
fn finds_executable() {
  let tmp = temptree! {
    "hello.exe": "#!/usr/bin/env bash\necho hello\n",
  }
  .executable("hello.exe");

  Test::new()
    .justfile("p := which('hello.exe')")
    .env("PATH", tmp.path().to_str().unwrap())
    .args(["--evaluate", "p"])
    .stdout(format!("{}", tmp.path().join("hello.exe").display()))
    .run();
}

#[test]
fn prints_empty_string_for_missing_executable() {
  let tmp = temptree! {
    "hello.exe": "#!/usr/bin/env bash\necho hello\n",
  }
  .executable("hello.exe");

  Test::new()
    .justfile("p := which('goodbye.exe')")
    .env("PATH", tmp.path().to_str().unwrap())
    .args(["--evaluate", "p"])
    .stdout("")
    .run();
}

#[test]
fn skips_non_executable_files() {
  let tmp = temptree! {
    "hello.exe": "#!/usr/bin/env bash\necho hello\n",
    "hi": "just some regular file",
  }
  .executable("hello.exe");

  Test::new()
    .justfile("p := which('hi')")
    .env("PATH", tmp.path().to_str().unwrap())
    .args(["--evaluate", "p"])
    .stdout("")
    .run();
}

#[test]
fn supports_multiple_paths() {
  let tmp1 = temptree! {
    "hello1.exe": "#!/usr/bin/env bash\necho hello\n",
  }
  .executable("hello1.exe");

  let tmp2 = temptree! {
    "hello2.exe": "#!/usr/bin/env bash\necho hello\n",
  }
  .executable("hello2.exe");

  let path =
    env::join_paths([tmp1.path().to_str().unwrap(), tmp2.path().to_str().unwrap()]).unwrap();

  Test::new()
    .justfile("p := which('hello1.exe')")
    .env("PATH", path.to_str().unwrap())
    .args(["--evaluate", "p"])
    .stdout(format!("{}", tmp1.path().join("hello1.exe").display()))
    .run();

  Test::new()
    .justfile("p := which('hello2.exe')")
    .env("PATH", path.to_str().unwrap())
    .args(["--evaluate", "p"])
    .stdout(format!("{}", tmp2.path().join("hello2.exe").display()))
    .run();
}

#[test]
fn supports_shadowed_executables() {
  let tmp1 = temptree! {
    "shadowed.exe": "#!/usr/bin/env bash\necho hello\n",
  }
  .executable("shadowed.exe");

  let tmp2 = temptree! {
    "shadowed.exe": "#!/usr/bin/env bash\necho hello\n",
  }
  .executable("shadowed.exe");

  // which should never resolve to this directory, no matter where or how many
  // times it appears in PATH, because the "shadowed" file is not executable.
  let dummy = if cfg!(windows) {
    temptree! {
      "shadowed": "#!/usr/bin/env bash\necho hello\n",
    }
  } else {
    temptree! {
      "shadowed.exe": "#!/usr/bin/env bash\necho hello\n",
    }
  };

  // This PATH should give priority to tmp1/shadowed.exe
  let tmp1_path = env::join_paths([
    dummy.path().to_str().unwrap(),
    tmp1.path().to_str().unwrap(),
    dummy.path().to_str().unwrap(),
    tmp2.path().to_str().unwrap(),
    dummy.path().to_str().unwrap(),
  ])
  .unwrap();

  // This PATH should give priority to tmp2/shadowed.exe
  let tmp2_path = env::join_paths([
    dummy.path().to_str().unwrap(),
    tmp2.path().to_str().unwrap(),
    dummy.path().to_str().unwrap(),
    tmp1.path().to_str().unwrap(),
    dummy.path().to_str().unwrap(),
  ])
  .unwrap();

  Test::new()
    .justfile("p := which('shadowed.exe')")
    .env("PATH", tmp1_path.to_str().unwrap())
    .args(["--evaluate", "p"])
    .stdout(format!("{}", tmp1.path().join("shadowed.exe").display()))
    .run();

  Test::new()
    .justfile("p := which('shadowed.exe')")
    .env("PATH", tmp2_path.to_str().unwrap())
    .args(["--evaluate", "p"])
    .stdout(format!("{}", tmp2.path().join("shadowed.exe").display()))
    .run();
}
