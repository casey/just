use super::*;

#[test]
fn usage() {
  Test::new()
    .justfile(
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
    .args(["--usage", "foo"])
    .stdout(
      "
        Usage: just foo [OPTIONS] b [c] [h...]

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
    .run();
}
