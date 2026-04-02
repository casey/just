use {super::*, std::os::unix::ffi::OsStrExt};

#[test]
fn warn_for_non_unicode_invocation_directory() {
  let tempdir = tempdir();

  let non_unicode_dir = tempdir.path().join(std::ffi::OsStr::from_bytes(b"foo\xff"));

  fs::create_dir(&non_unicode_dir).unwrap();
  fs::write(non_unicode_dir.join("justfile"), "default:\n\ttrue\n").unwrap();

  let output = Command::new(JUST)
    .current_dir(&non_unicode_dir)
    .arg("--color")
    .arg("never")
    .output()
    .unwrap();

  let stderr = str::from_utf8(&output.stderr).unwrap();

  let expected = format!(
    "The invocation directory path `{}` is not Unicode. Just is considering phasing-out support \
        for non-Unicode paths. If you see this warning, please leave a comment on\n\
        https://github.com/casey/just/issues/3229. Thank you!\n",
    non_unicode_dir.display(),
  );

  assert!(stderr.contains(&expected), "stderr: {stderr}");
}

#[test]
fn warn_for_non_unicode_justfile_path() {
  let tempdir = tempdir();

  let non_unicode_dir = tempdir.path().join(std::ffi::OsStr::from_bytes(b"bar\xff"));

  fs::create_dir(&non_unicode_dir).unwrap();
  fs::write(non_unicode_dir.join("justfile"), "default:\n\ttrue\n").unwrap();

  let output = Command::new(JUST)
    .current_dir(&non_unicode_dir)
    .arg("--color")
    .arg("never")
    .output()
    .unwrap();

  let stderr = str::from_utf8(&output.stderr).unwrap();

  let expected = format!(
    "The justfile path `{}` is not Unicode. Just is considering phasing-out support for \
        non-Unicode paths. If you see this warning, please leave a comment on\n\
        https://github.com/casey/just/issues/3229. Thank you!\n",
    non_unicode_dir.join("justfile").display(),
  );

  assert!(stderr.contains(&expected), "stderr: {stderr}");
}
