use {super::*, std::os::unix::ffi::OsStrExt};

fn non_unicode_dir_name() -> &'static std::ffi::OsStr {
  std::ffi::OsStr::from_bytes(b"foo\xff")
}

#[test]
fn warn_for_non_unicode_invocation_directory() {
  let tempdir = tempdir();
  let dir = tempdir.path().join(non_unicode_dir_name());
  fs::create_dir(&dir).unwrap();
  fs::write(dir.join("justfile"), "default:\n\ttrue\n").unwrap();

  Test::with_tempdir(tempdir)
    .no_justfile()
    .current_dir(non_unicode_dir_name())
    .test_round_trip(false)
    .stderr_regex(
      ".*The invocation directory path `[^`]+` is not Unicode\\. Just is considering phasing-out \
      support for non-Unicode paths\\. If you see this warning, please leave a comment on \
      https://github\\.com/casey/just/issues/3229\\. Thank you!.*",
    )
    .success();
}

#[test]
fn warn_for_non_unicode_justfile_path() {
  let tempdir = tempdir();
  let dir = tempdir.path().join(non_unicode_dir_name());
  fs::create_dir(&dir).unwrap();
  fs::write(dir.join("justfile"), "default:\n\ttrue\n").unwrap();

  Test::with_tempdir(tempdir)
    .no_justfile()
    .current_dir(non_unicode_dir_name())
    .test_round_trip(false)
    .stderr_regex(
      ".*The justfile path `[^`]+` is not Unicode\\. Just is considering phasing-out support for \
      non-Unicode paths\\. If you see this warning, please leave a comment on \
      https://github\\.com/casey/just/issues/3229\\. Thank you!.*",
    )
    .success();
}
