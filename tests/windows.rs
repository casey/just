use super::*;

#[test]
fn bare_bash_in_shebang() {
  Test::new()
    .justfile(
      "
        default:
            #!bash
            echo FOO
      ",
    )
    .stdout("FOO\n")
    .run();
}

#[test]
fn spaces_in_windows_shell_arg() {
  Test::new()
    .justfile(
      "
        set windows-shell := [\"cmd.exe\", \"/c\"]
        default:
            echo FOO && \"echo FOO\"
      ",
    )
    .shell(false)
    .stdout("FOO \n")
    .stderr("echo FOO && \"echo FOO\"\n'\"echo FOO\"' is not recognized as an internal or external command,\noperable program or batch file.\nerror: Recipe `default` failed on line 3 with exit code 1\n")
    .status(1)
    .run();
}
