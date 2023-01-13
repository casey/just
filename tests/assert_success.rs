use super::*;

pub(crate) fn assert_success(output: &std::process::Output) {
  if !output.status.success() {
    eprintln!("stderr: {}", String::from_utf8_lossy(&output.stderr));
    eprintln!("stdout: {}", String::from_utf8_lossy(&output.stdout));
    panic!("{}", output.status);
  }
}
