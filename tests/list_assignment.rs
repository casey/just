use super::*;

test! {
  name: correct_parseage,
  justfile: "
  archs := 'arm' 'loongarch' 'x86_64'
  rec:
    echo {{ archs }}
  ",
  stdout: "
  arm loongarch x86_64
  ",
  stderr: "
  echo arm loongarch x86_64
  ",
  status: EXIT_SUCCESS,
}
