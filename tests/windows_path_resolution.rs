use super::*;

#[cfg(windows)]
#[test]
fn windows_path_resolution() {

  // Goal: Confirm that $PATH entries are respected *before* C:\Windows\System32 when
  // locating shell executable. This can happen when PATH is configured to prefer
  // Git-for-Windows' bash.exe, and just must not call C:\Windows\System32\bash.exe
  // https://github.com/casey/just/issues/2947

  // Copy echoargs.exe to temp directory as where.exe, to intentionally match
  // the name of an executable in C:\Windows\System32
  let mut echoargs_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
  echoargs_path.push("target");
  echoargs_path.push("debug");
  echoargs_path.push("echoargs.exe");
  let tmp = tempdir();
  let tmp_subdir= tmp.path().join("subdir");
  let tmp_subdir_path = tmp_subdir.as_path();
  let exe_path = tmp_subdir_path.join("where.exe");
  std::fs::create_dir(tmp_subdir_path).expect("Failed to create temp subdirectory");
  std::fs::copy(&echoargs_path, &exe_path)
    .expect("Failed to copy exe to temp directory");

  // Prepend temp directory to PATH
  let new_path = tmp_subdir_path.to_str().unwrap().to_owned() + ";" + &env::var("PATH").unwrap();

  Test::with_tempdir(tmp)
    .shell(false)
    .env("Path", &new_path)
    .justfile(
      r#"
        set shell := ['where.exe']
        @default:
          test_marker
      "#,
    )
    .stdout("test_marker\n")
    .run();
}
