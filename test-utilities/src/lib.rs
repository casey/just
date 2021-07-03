use std::process::Output;

pub fn assert_success(output: &Output) {
  if !output.status.success() {
    eprintln!("stderr: {}", String::from_utf8_lossy(&output.stderr));
    eprintln!("stdout: {}", String::from_utf8_lossy(&output.stdout));
    panic!("{}", output.status);
  }
}

pub fn assert_stdout(output: &Output, stdout: &str) {
  assert_success(output);
  assert_eq!(String::from_utf8_lossy(&output.stdout), stdout);
}
