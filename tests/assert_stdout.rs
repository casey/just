use super::*;

pub(crate) fn assert_stdout(output: &std::process::Output, stdout: &str) {
  assert_success(output);
  assert_eq!(String::from_utf8_lossy(&output.stdout), stdout);
}
