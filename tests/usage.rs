use super::*;

#[test]
fn usage() {
  Test::new()
    .justfile("mod bar")
    .write(
      "bar.just",
      "
[arg('a', short='a')]
[arg('b', pattern='123|789', help='hello')]
[arg('d', short='d', long='delightful')]
[arg('e', short='e', pattern='abc|xyz')]
[arg('f', long='f', pattern='lucky')]
[arg('g', short='g', value='foo')]
foo a b c='abc' d e f='xyz' g='bar' *h:
",
    )
    .args(["--usage", "bar", "foo"])
    .stdout(
      "
        Usage: just bar foo [OPTIONS] b [c] [h...]

        Arguments:
          b hello [pattern: '123|789']
          [c] [default: 'abc']
          [h...]

        Options:
          -a a
          -d, --delightful d
          -e e [pattern: 'abc|xyz']
              --f f [default: 'xyz'] [pattern: 'lucky']
          -g
      ",
    )
    .success();
}
