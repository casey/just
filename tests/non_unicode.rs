use {super::*, std::os::unix::ffi::OsStrExt};

#[test]
fn non_unicode_invocation_directory_is_an_error() {
  let dir = std::ffi::OsStr::from_bytes(b"foo\xff");
  let tempdir = tempdir();
  fs::create_dir(tempdir.path().join(dir)).unwrap();

  Test::with_tempdir(tempdir)
    .current_dir(dir)
    .stderr_regex(
      "^error: current directory is not valid unicode: PathBuf contains invalid UTF-8: \
      .*/foo\u{FFFD}\n$",
    )
    .failure();
}
