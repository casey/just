### Functions

`just` provides a few built-in functions that might be useful when writing recipes.

#### System Information

* `arch()` — Instruction set architecture. Possible values are: `"aarch64"`, `"arm"`, `"asmjs"`, `"hexagon"`, `"mips"`, `"msp430"`, `"powerpc"`, `"powerpc64"`, `"s390x"`, `"sparc"`, `"wasm32"`, `"x86"`, `"x86_64"`, and `"xcore"`.

* `os()` — Operating system. Possible values are: `"android"`, `"bitrig"`, `"dragonfly"`, `"emscripten"`, `"freebsd"`, `"haiku"`, `"ios"`, `"linux"`, `"macos"`, `"netbsd"`, `"openbsd"`, `"solaris"`, and `"windows"`.

* `os_family()` — Operating system family; possible values are: `"unix"` and `"windows"`.

For example:

````make
system-info:
  @echo "This is an {{arch()}} machine".
````

````sh
$ just system-info
This is an x86_64 machine
````

The `os_family()` function can be used to create cross-platform `justfile`s that work on various operating systems. For an example, see [cross-platform.just](examples/cross-platform.just) file.

#### Environment Variables

* `env_var(key)` — Retrieves the environment variable with name `key`, aborting if it is not present.

````make
home_dir := env_var('HOME')

test:
  echo "{{home_dir}}"
````

````sh
$ just
/home/user1
````

* `env_var_or_default(key, default)` — Retrieves the environment variable with name `key`, returning `default` if it is not present.

#### Invocation Directory

* `invocation_directory()` - Retrieves the path of the current working directory, before `just` changed it (chdir’d) prior to executing commands.

For example, to call `rustfmt` on files just under the “current directory” (from the user/invoker’s perspective), use the following rule:

````make
rustfmt:
  find {{invocation_directory()}} -name \*.rs -exec rustfmt {} \;
````

Alternatively, if your command needs to be run from the current directory, you could use (e.g.):

````make
build:
  cd {{invocation_directory()}}; ./some_script_that_needs_to_be_run_from_here
````

#### Justfile and Justfile Directory

* `justfile()` - Retrieves the path of the current `justfile`.

* `justfile_directory()` - Retrieves the path of the parent directory of the current `justfile`.

For example, to run a command relative to the location of the current `justfile`:

````make
script:
  ./{{justfile_directory()}}/scripts/some_script
````

#### Just Executable

* `just_executable()` - Absolute path to the `just` executable.

For example:

````make
executable:
  @echo The executable is at: {{just_executable()}}
````

````sh
$ just
The executable is at: /bin/just
````

#### String Manipulation

* `lowercase(s)` - Convert `s` to lowercase.

* `quote(s)` - Replace all single quotes with `'\''` and prepend and append single quotes to `s`. This is sufficient to escape special characters for many shells, including most Bourne shell descendants.

* `replace(s, from, to)` - Replace all occurrences of `from` in `s` to `to`.

* `trim(s)` - Remove leading and trailing whitespace from `s`.

* `trim_end(s)` - Remove trailing whitespace from `s`.

* `trim_end_match(s, pat)` - Remove suffix of `s` matching `pat`.

* `trim_end_matches(s, pat)` - Repeatedly remove suffixes of `s` matching `pat`.

* `trim_start(s)` - Remove leading whitespace from `s`.

* `trim_start_match(s, pat)` - Remove prefix of `s` matching `pat`.

* `trim_start_matches(s, pat)` - Repeatedly remove prefixes of `s` matching `pat`.

* `uppercase(s)` - Convert `s` to uppercase.

#### Path Manipulation

##### Fallible

* `absolute_path(path)` - Absolute path to relative `path` in the invocation directory. `absolute_path("./bar.txt")` in directory `/foo` is `/foo/bar.txt`.

* `extension(path)` - Extension of `path`. `extension("/foo/bar.txt")` is `txt`.

* `file_name(path)` - File name of `path` with any leading directory components removed. `file_name("/foo/bar.txt")` is `bar.txt`.

* `file_stem(path)` - File name of `path` without extension. `file_stem("/foo/bar.txt")` is `bar`.

* `parent_directory(path)` - Parent directory of `path`. `parent_directory("/foo/bar.txt")` is `/foo`.

* `without_extension(path)` - `path` without extension. `without_extension("/foo/bar.txt")` is `/foo/bar`.

These functions can fail, for example if a path does not have an extension, which will halt execution.

##### Infallible

* `join(a, b…)` - Join path `a` with path `b`. `join("foo/bar", "baz")` is `foo/bar/baz`. Accepts two or more arguments.

* `clean(path)` - Simplify `path` by removing extra path separators, intermediate `.` components, and `..` where possible. `clean("foo//bar")` is `foo/bar`, `clean("foo/..")` is `.`, `clean("foo/./bar")` is `foo/bar`.

#### Filesystem Access

* `path_exists(path)` - Returns `true` if the path points at an existing entity and `false` otherwise. Traverses symbolic links, and returns `false` if the path is inaccessible or points to a broken symlink.

##### Error Reporting

* `error(message)` - Abort execution and report error `message` to user.