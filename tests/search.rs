use super::*;

#[test]
fn test_justfile_search() {
  Test::new()
    .justfile("default:\n\techo ok")
    .create_dir("a/b/c/d")
    .current_dir("a/b/c/d")
    .stderr("echo ok\n")
    .stdout("ok\n")
    .success();
}

#[test]
fn test_capitalized_justfile_search() {
  Test::new()
    .write("Justfile", "default:\n\techo ok")
    .create_dir("a/b/c/d")
    .current_dir("a/b/c/d")
    .stderr("echo ok\n")
    .stdout("ok\n")
    .success();
}

#[test]
fn test_upwards_path_argument() {
  #[track_caller]
  fn case(args: &[&str]) {
    Test::new()
      .justfile("default:\n\techo ok")
      .write("a/justfile", "default:\n\techo bad")
      .current_dir("a")
      .args(args)
      .stderr("echo ok\n")
      .stdout("ok\n")
      .success();
  }

  case(&["../"]);
  case(&["../default"]);
}

#[test]
fn test_downwards_path_argument() {
  #[track_caller]
  fn case(args: &[&str]) {
    Test::new()
      .justfile("default:\n\techo bad")
      .write("a/justfile", "default:\n\techo ok")
      .args(args)
      .stderr("echo ok\n")
      .stdout("ok\n")
      .success();
  }

  case(&["a/"]);
  case(&["a/default"]);
  case(&["./a/"]);
  case(&["./a/default"]);
  case(&["./a/"]);
  case(&["./a/default"]);
}

#[test]
fn test_upwards_multiple_path_argument() {
  #[track_caller]
  fn case(args: &[&str]) {
    Test::new()
      .justfile("default:\n\techo ok")
      .write("a/b/justfile", "default:\n\techo bad")
      .current_dir("a/b")
      .args(args)
      .stderr("echo ok\n")
      .stdout("ok\n")
      .success();
  }

  case(&["../../"]);
  case(&["../../default"]);
}

#[test]
fn test_downwards_multiple_path_argument() {
  #[track_caller]
  fn case(args: &[&str]) {
    Test::new()
      .justfile("default:\n\techo bad")
      .write("a/b/justfile", "default:\n\techo ok")
      .args(args)
      .stderr("echo ok\n")
      .stdout("ok\n")
      .success();
  }

  case(&["a/b/"]);
  case(&["a/b/default"]);
  case(&["./a/b/"]);
  case(&["./a/b/default"]);
  case(&["./a/b/"]);
  case(&["./a/b/default"]);
}

#[test]
fn single_downwards() {
  Test::new()
    .justfile("default:\n\techo ok")
    .create_dir("child")
    .args(["child/"])
    .stderr("echo ok\n")
    .stdout("ok\n")
    .success();
}

#[test]
fn single_upwards() {
  Test::new()
    .justfile("default:\n\techo ok")
    .create_dir("child")
    .current_dir("child")
    .args(["../"])
    .stderr("echo ok\n")
    .stdout("ok\n")
    .success();
}

#[test]
fn double_upwards() {
  Test::new()
    .justfile("default:\n\techo ok")
    .write("foo/bar/justfile", "default:\n\techo foo")
    .current_dir("foo/bar")
    .args(["../default"])
    .stderr("echo ok\n")
    .stdout("ok\n")
    .success();
}

#[test]
fn find_dot_justfile() {
  Test::new()
    .justfile(
      "
        foo:
          echo bad
      ",
    )
    .write(
      "dir/.justfile",
      "
        foo:
          echo ok
      ",
    )
    .current_dir("dir")
    .stderr("echo ok\n")
    .stdout("ok\n")
    .success();
}

#[test]
fn dot_justfile_conflicts_with_justfile() {
  Test::new()
    .justfile(
      "
        foo:
    ",
    )
    .write(
      ".justfile",
      "
        foo:
      ",
    )
    .stderr_regex("error: multiple candidate justfiles found in `.*`: `.justfile` and `justfile`\n")
    .failure();
}

#[test]
fn not_found() {
  Test::new()
    .stderr_regex("error: no justfile found\n")
    .failure();
}

#[test]
fn found_spongebob_case() {
  Test::new()
    .write("JuStFiLe", "default:\n\techo ok")
    .create_dir("a")
    .current_dir("a")
    .stderr("echo ok\n")
    .stdout("ok\n")
    .success();
}

#[test]
fn search_stops_at_closest_justfile() {
  Test::new()
    .justfile("default:\n\techo bad")
    .write("a/justfile", "default:\n\techo ok")
    .create_dir("a/b")
    .current_dir("a/b")
    .stderr("echo ok\n")
    .stdout("ok\n")
    .success();
}

#[test]
fn justfile_name_not_found() {
  Test::new()
    .justfile("default:\n\techo ok")
    .args(["--justfile-name", "foo"])
    .stderr_regex("error: no justfile found\n")
    .failure();
}

#[test]
fn justfile_name_skips_default_justfile() {
  Test::new()
    .write(
      "foo",
      "
        default:
        \techo ok
      ",
    )
    .create_dir("subdir")
    .write(
      "subdir/justfile",
      "
        default:
        \techo bad
      ",
    )
    .current_dir("subdir")
    .args(["--justfile-name", "foo"])
    .stderr("echo ok\n")
    .stdout("ok\n")
    .success();
}

#[test]
fn justfile_symlink_parent() {
  Test::new()
    .write(
      "src",
      "
        foo:
        \techo bar
      ",
    )
    .create_dir("sub")
    .symlink("src", "sub/justfile")
    .current_dir("sub")
    .stderr("echo bar\n")
    .stdout("bar\n")
    .success();
}

#[test]
fn justfile_without_parent_and_working_directory_panics() {
  Test::new()
    .args(["--justfile", "/", "--working-directory", ".", "--list"])
    .stderr("error: justfile path had no parent: /\n")
    .failure();
}
