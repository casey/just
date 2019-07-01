# Functions

Just provides a few built-in functions that might be useful when writing recipes.

### System Information

- `arch()` – Instruction set architecture. Possible values are: `"aarch64"`, `"arm"`, `"asmjs"`, `"hexagon"`, `"mips"`, `"msp430"`, `"powerpc"`, `"powerpc64"`, `"s390x"`, `"sparc"`, `"wasm32"`, `"x86"`, `"x86_64"`, and `"xcore"`.

- `os()` – Operating system. Possible values are: `"android"`, `"bitrig"`, `"dragonfly"`, `"emscripten"`, `"freebsd"`, `"haiku"`, `"ios"`, `"linux"`, `"macos"`, `"netbsd"`, `"openbsd"`, `"solaris"`, and `"windows"`.

- `os_family()` – Operating system family; possible values are: `"unix"` and `"windows"`.

For example:

```make
system-info:
    @echo "This is an {{arch()}} machine".
```

```sh
$ just system-info
This is an x86_64 machine
```

### Environment Variables

- `env_var(key)` – Retrieves the environment variable with name `key`, aborting if it is not present.

- `env_var_or_default(key, default)` – Retrieves the environment variable with name `key`, returning `default` if it is not present.

### Invocation Directory

- `invocation_directory()` - Retrieves the path of the current working directory, before `just` changed it (chdir'd) prior to executing commands.

For example, to call `rustfmt` on files just under the "current directory" (from the user/invoker's perspective), use the following rule:

```sh
rustfmt:
    find {{invocation_directory()}} -name \*.rs -exec rustfmt {} \;
```

Alternatively, if your command needs to be run from the current directory, you could use (e.g.):

```sh
build:
    cd {{invocation_directory()}}; ./some_script_that_needs_to_be_run_from_here
```

### Dotenv Integration

`just` will load environment variables from a file named `.env`. This file can be located in the same directory as your justfile or in a parent directory. These variables are environment variables, not `just` variables, and so must be accessed using `$VARIABLE_NAME` in recipes and backticks.

For example, if your `.env` file contains:

```sh
# a comment, will be ignored
DATABASE_ADDRESS=localhost:6379
SERVER_PORT=1337
```

And your justfile contains:

```make
serve:
  @echo "Starting server with database $DATABASE_ADDRESS on port $SERVER_PORT..."
  ./server --database $DATABASE_ADDRESS --port $SERVER_PORT
```

`just serve` will output:

```sh
$ just serve
Starting server with database localhost:6379 on port 1337...
./server --database $DATABASE_ADDRESS --port $SERVER_PORT
```