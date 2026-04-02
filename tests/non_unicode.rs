use {super::*, std::os::unix::ffi::OsStrExt};

#[test]
fn warn_for_non_unicode_invocation_directory() {
  let tempdir = tempdir();

  let non_unicode_dir = tempdir
    .path()
    .join(std::ffi::OsStr::from_bytes(b"foo\xff"));

  fs::create_dir(&non_unicode_dir).unwrap();
  fs::write(non_unicode_dir.join("justfile"), "default:\n\ttrue\n").unwrap();

  let output = Command::new(JUST)
    .current_dir(&non_unicode_dir)
    .arg("--color")
    .arg("never")
    .output()
    .unwrap();

  let stderr = str::from_utf8(&output.stderr).unwrap();

  assert!(
    stderr.contains("invocation directory"),
    "expected warning about invocation directory in stderr: {stderr}",
  );

  assert!(
    stderr.contains("not Unicode"),
    "expected 'not Unicode' in stderr: {stderr}",
  );
}

#[test]
fn warn_for_non_unicode_justfile_path() {
  let tempdir = tempdir();

  let non_unicode_dir = tempdir
    .path()
    .join(std::ffi::OsStr::from_bytes(b"bar\xff"));

  fs::create_dir(&non_unicode_dir).unwrap();
  fs::write(non_unicode_dir.join("justfile"), "default:\n\ttrue\n").unwrap();

  let output = Command::new(JUST)
    .current_dir(&non_unicode_dir)
    .arg("--color")
    .arg("never")
    .output()
    .unwrap();

  let stderr = str::from_utf8(&output.stderr).unwrap();

  assert!(
    stderr.contains("justfile"),
    "expected warning about justfile in stderr: {stderr}",
  );
}
