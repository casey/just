↖️ Table of Contents

<h1 align="center"><code>just</code></h1>

<div align="center">
  <a href="https://crates.io/crates/just">
    <img src="https://img.shields.io/crates/v/just.svg" alt="crates.io version">
  </a>
  <a href="https://github.com/casey/just/actions">
    <img src="https://github.com/casey/just/workflows/Build/badge.svg" alt="build status">
  </a>
  <a href="https://github.com/casey/just/releases">
    <img src="https://img.shields.io/github/downloads/casey/just/total.svg" alt="downloads">
  </a>
  <a href="https://discord.gg/ezYScXR">
    <img src="https://img.shields.io/discord/695580069837406228?logo=discord" alt="chat on discord">
  </a>
  <a href="mailto:casey@rodarmor.com?subject=Thanks%20for%20Just!">
    <img src="https://img.shields.io/badge/Say%20Thanks-!-1EAEDB.svg" alt="say thanks">
  </a>
</div>
<br>

`just` is a handy way to save and run project-specific commands.

This readme is also available as a [book](https://just.systems/man/en/).

(中文文档在 [这里](https://github.com/casey/just/blob/master/README.中文.md), 快看过来!)

Commands, called recipes, are stored in a file called `justfile` with syntax inspired by `make`:

![screenshot](https://raw.githubusercontent.com/casey/just/master/screenshot.png)

You can then run them with `just RECIPE`:

```sh
$ just test-all
cc *.c -o main
./test --all
Yay, all your tests passed!
```

`just` has a ton of useful features, and many improvements over `make`:

- `just` is a command runner, not a build system, so it avoids much of [`make`'s complexity and idiosyncrasies](#what-are-the-idiosyncrasies-of-make-that-just-avoids). No need for `.PHONY` recipes!

- Linux, MacOS, and Windows are supported with no additional dependencies. (Although if your system doesn't have an `sh`, you'll need to [choose a different shell](#shell).)

- Errors are specific and informative, and syntax errors are reported along with their source context.

- Recipes can accept [command line arguments](#recipe-parameters).

- Wherever possible, errors are resolved statically. Unknown recipes and circular dependencies are reported before anything runs.

- `just` [loads `.env` files](#dotenv-integration), making it easy to populate environment variables.

- Recipes can be [listed from the command line](#listing-available-recipes).

- Command line completion scripts are [available for most popular shells](#shell-completion-scripts).

- Recipes can be written in [arbitrary languages](#writing-recipes-in-other-languages), like Python or NodeJS.

- `just` can be invoked from any subdirectory, not just the directory that contains the `justfile`.

- And [much more](https://just.systems/man/en/)!

If you need help with `just` please feel free to open an issue or ping me on [Discord](https://discord.gg/ezYScXR). Feature requests and bug reports are always welcome!

Installation
------------

### Prerequisites

`just` should run on any system with a reasonable `sh`, including Linux, MacOS, and the BSDs.

On Windows, `just` works with the `sh` provided by [Git for Windows](https://git-scm.com), [GitHub Desktop](https://desktop.github.com), or [Cygwin](http://www.cygwin.com).

If you'd rather not install `sh`, you can use the `shell` setting to use the shell of your choice.

Like PowerShell:

```make
# use PowerShell instead of sh:
set shell := ["powershell.exe", "-c"]

hello:
  Write-Host "Hello, world!"
```

…or `cmd.exe`:

```make
# use cmd.exe instead of sh:
set shell := ["cmd.exe", "/c"]

list:
  dir
```

You can also set the shell using command-line arguments. For example, to use PowerShell, launch `just` with `--shell powershell.exe --shell-arg -c`.

(PowerShell is installed by default on Windows 7 SP1 and Windows Server 2008 R2 S1 and later, and `cmd.exe` is quite fiddly, so PowerShell is recommended for most Windows users.)

### Packages

<table>
  <thead>
    <tr>
      <th>Operating System</th>
      <th>Package Manager</th>
      <th>Package</th>
      <th>Command</th>
    </tr>
  </thead>
  <tbody>
  <tr>
    <td><a href="https://forge.rust-lang.org/release/platform-support.html">Various</a></td>
    <td><a href="https://www.rust-lang.org">Cargo</a></td>
    <td><a href="https://crates.io/crates/just">just</a></td>
    <td><code>cargo install just</code></td>
  </tr>
  <tr>
    <td><a href="https://en.wikipedia.org/wiki/Microsoft_Windows">Microsoft Windows</a></td>
    <td><a href="https://scoop.sh">Scoop</a></td>
    <td><a href="https://github.com/ScoopInstaller/Main/blob/master/bucket/just.json">just</a></td>
    <td><code>scoop install just</code></td>
  </tr>
  <tr>
    <td><a href="https://docs.brew.sh/Installation">Various</a></td>
    <td><a href="https://brew.sh">Homebrew</a></td>
    <td><a href="https://formulae.brew.sh/formula/just">just</a></td>
    <td><code>brew install just</code></td>
  </tr>
  <tr>
    <td><a href="https://en.wikipedia.org/wiki/MacOS">macOS</a></td>
    <td><a href="https://www.macports.org">MacPorts</a></td>
    <td><a href="https://ports.macports.org/port/just/summary">just</a></td>
    <td><code>port install just</code></td>
  </tr>
  <tr>
    <td><a href="https://www.archlinux.org">Arch Linux</a></td>
    <td><a href="https://wiki.archlinux.org/title/Pacman">pacman</a></td>
    <td><a href="https://archlinux.org/packages/community/x86_64/just/">just</a></td>
    <td><code>pacman -S just</code></td>
  </tr>
  <tr>
    <td><a href="https://nixos.org/download.html#download-nix">Various</a></td>
    <td><a href="https://nixos.org/nix/">Nix</a></td>
    <td><a href="https://github.com/NixOS/nixpkgs/blob/master/pkgs/development/tools/just/default.nix">just</a></td>
    <td><code>nix-env -iA nixpkgs.just</code></td>
  </tr>
  <tr>
    <td><a href="https://nixos.org/nixos/">NixOS</a></td>
    <td><a href="https://nixos.org/nix/">Nix</a></td>
    <td><a href="https://github.com/NixOS/nixpkgs/blob/master/pkgs/development/tools/just/default.nix">just</a></td>
    <td><code>nix-env -iA nixos.just</code></td>
  </tr>
  <tr>
    <td><a href="https://getsol.us">Solus</a></td>
    <td><a href="https://getsol.us/articles/package-management/basics/en">eopkg</a></td>
    <td><a href="https://dev.getsol.us/source/just/">just</a></td>
    <td><code>eopkg install just</code></td>
  </tr>
  <tr>
    <td><a href="https://voidlinux.org">Void Linux</a></td>
    <td><a href="https://wiki.voidlinux.org/XBPS">XBPS</a></td>
    <td><a href="https://github.com/void-linux/void-packages/blob/master/srcpkgs/just/template">just</a></td>
    <td><code>xbps-install -S just</code></td>
  </tr>
  <tr>
    <td><a href="https://www.freebsd.org">FreeBSD</a></td>
    <td><a href="https://www.freebsd.org/doc/handbook/pkgng-intro.html">pkg</a></td>
    <td><a href="https://www.freshports.org/deskutils/just/">just</a></td>
    <td><code>pkg install just</code></td>
  </tr>
  <tr>
    <td><a href="https://alpinelinux.org">Alpine Linux</a></td>
    <td><a href="https://wiki.alpinelinux.org/wiki/Alpine_Linux_package_management">apk-tools</a></td>
    <td><a href="https://pkgs.alpinelinux.org/package/edge/community/x86_64/just">just</a></td>
    <td><code>apk add just</code></td>
  </tr>
  <tr>
    <td><a href="https://getfedora.org">Fedora Linux</a></td>
    <td><a href="https://dnf.readthedocs.io/en/latest/">DNF</a></td>
    <td><a href="https://src.fedoraproject.org/rpms/rust-just">just</a></td>
    <td><code>dnf install just</code></td>
  </tr>
  <tr>
    <td><a href="https://www.gentoo.org">Gentoo Linux</a></td>
    <td><a href="https://wiki.gentoo.org/wiki/Portage">Portage</a></td>
    <td><a href="https://github.com/gentoo-mirror/dm9pZCAq/tree/master/sys-devel/just">dm9pZCAq/sys-devel/just</a></td>
    <td>
      <code>eselect repository enable dm9pZCAq</code><br>
      <code>emerge --sync dm9pZCAq</code><br>
      <code>emerge sys-devel/just</code>
    </td>
  </tr>
  <tr>
    <td><a href="https://docs.conda.io/en/latest/miniconda.html#system-requirements">Various</a></td>
    <td><a href="https://docs.conda.io/projects/conda/en/latest/index.html">Conda</a></td>
    <td><a href="https://anaconda.org/conda-forge/just">just</a></td>
    <td><code>conda install -c conda-forge just</code></td>
  </tr>
  <tr>
    <td><a href="https://en.wikipedia.org/wiki/Microsoft_Windows">Microsoft Windows</a></td>
    <td><a href="https://chocolatey.org">Chocolatey</a></td>
    <td><a href="https://github.com/michidk/just-choco">just</a></td>
    <td><code>choco install just</code></td>
  </tr>
  <tr>
    <td><a href="https://snapcraft.io/docs/installing-snapd">Various</a></td>
    <td><a href="https://snapcraft.io">Snap</a></td>
    <td><a href="https://snapcraft.io/just">just</a></td>
    <td><code>snap install --edge --classic just</code></td>
  </tr>
  <tr>
    <td><a href="https://github.com/casey/just/releases">Various</a></td>
    <td><a href="https://asdf-vm.com">asdf</a></td>
    <td><a href="https://github.com/ggilmore/asdf-just">just</a></td>
    <td>
      <code>asdf plugin add just</code><br>
      <code>asdf install just &lt;version&gt;</code>
    </td>
  </tr>
  <tr>
    <td><a href="https://debian.org">Debian derivatives</td>
    <td><a href="https://mpr.makedeb.org">MPR</a></td>
    <td><a href="https://mpr.makedeb.org/packages/just">just</a></td>
    <td>
      <code>git clone 'https://mpr.makedeb.org/just'</code><br>
      <code>cd just</code><br>
      <code>makedeb -si</code>
    </td>
  </tr>
  <tr>
    <td><a href="https://debian.org">Debian derivatives</a></td>
    <td><a href="https://docs.makedeb.org/prebuilt-mpr">Prebuilt-MPR</a></td>
    <td><a href="https://mpr.makedeb.org/packages/just">just</a></td>
    <td>
      <sup><b>You must have the <a href="https://docs.makedeb.org/prebuilt-mpr/getting-started/#setting-up-the-repository">Prebuilt-MPR set up</a> on your system in order to run this command.</b></sup><br>
      <code>sudo apt install just</code>
    </td>
  </tr>
  </tbody>
</table>

![package version table](https://repology.org/badge/vertical-allrepos/just.svg)

### Pre-Built Binaries

Pre-built binaries for Linux, MacOS, and Windows can be found on [the releases page](https://github.com/casey/just/releases).

You can use the following command on Linux, MacOS, or Windows to download the latest release, just replace `DEST` with the directory where you'd like to put `just`:

```sh
curl --proto '=https' --tlsv1.2 -sSf https://just.systems/install.sh | bash -s -- --to DEST
```

For example, to install `just` to `~/bin`:

```sh
# create ~/bin
mkdir -p ~/bin

# download and extract just to ~/bin/just
curl --proto '=https' --tlsv1.2 -sSf https://just.systems/install.sh | bash -s -- --to ~/bin

# add `~/bin` to the paths that your shell searches for executables
# this line should be added to your shells initialization file,
# e.g. `~/.bashrc` or `~/.zshrc`
export PATH="$PATH:$HOME/bin"

# just should now be executable
just --help
```

### GitHub Actions

[extractions/setup-just](https://github.com/extractions/setup-just) can be used to install `just` in a GitHub Actions workflow.

Example usage:

```yaml
- uses: extractions/setup-just@v1
  with:
    just-version: 0.8 # optional semver specification, otherwise latest
```

### Release RSS Feed

An [RSS feed](https://en.wikipedia.org/wiki/RSS) of `just` releases is available [here](https://github.com/casey/just/releases.atom).

### Node.js Installation

[just-install](https://npmjs.com/package/just-install) can be used to automate installation of `just` in Node.js applications.

`just` is a great, more robust alternative to npm scripts. If you want to include `just` in the dependencies of a Node.js application, `just-install` will install a local, platform-specific binary as part of the `npm install` command. This removes the need for every developer to install `just` independently using one of the processes mentioned above. After installation, the `just` command will work in npm scripts or with npx. It's great for teams who want to make the set up process for their project as easy as possible.

For more information, see the [just-install README file](https://github.com/brombal/just-install#readme).

Backwards Compatibility
-----------------------

With the release of version 1.0, `just` features a strong commitment to backwards compatibility and stability.

Future releases will not introduce backwards incompatible changes that make existing `justfile`s stop working, or break working invocations of the command-line interface.

This does not, however, preclude fixing outright bugs, even if doing so might break `justfiles` that rely on their behavior.

There will never be a `just` 2.0. Any desirable backwards-incompatible changes will be opt-in on a per-`justfile` basis, so users may migrate at their leisure.

Features that aren't yet ready for stabilization are gated behind the `--unstable` flag. Features enabled by `--unstable` may change in backwards incompatible ways at any time.

Editor Support
--------------

`justfile` syntax is close enough to `make` that you may want to tell your editor to use `make` syntax highlighting for `just`.

### Vim and Neovim

#### `vim-just`

The [vim-just](https://github.com/NoahTheDuke/vim-just) plugin provides syntax highlighting for `justfile`s.

Install it with your favorite package manager, like [Plug](https://github.com/junegunn/vim-plug):

```vim
call plug#begin()

Plug 'NoahTheDuke/vim-just'

call plug#end()
```

Or with Vim's built-in package support:

```sh
mkdir -p ~/.vim/pack/vendor/start
cd ~/.vim/pack/vendor/start
git clone https://github.com/NoahTheDuke/vim-just.git
```

`vim-just` is also available from [vim-polyglot](https://github.com/sheerun/vim-polyglot), a multi-language Vim plugin.

#### `tree-sitter-just`

[tree-sitter-just](https://github.com/IndianBoy42/tree-sitter-just) is an [Nvim Treesitter](https://github.com/nvim-treesitter/nvim-treesitter) plugin for Neovim.

#### Makefile Syntax Highlighting

Vim's built-in makefile syntax highlighting isn't perfect for `justfile`s, but it's better than nothing. You can put the following in `~/.vim/filetype.vim`:

```vimscript
if exists("did_load_filetypes")
  finish
endif

augroup filetypedetect
  au BufNewFile,BufRead justfile setf make
augroup END
```

Or add the following to an individual `justfile` to enable `make` mode on a per-file basis:

```text
# vim: set ft=make :
```

### Emacs

[just-mode](https://github.com/leon-barrett/just-mode.el) provides syntax highlighting and automatic indentation of `justfile`s. It is available on [MELPA](https://melpa.org/) as [just-mode](https://melpa.org/#/just-mode).

[justl](https://github.com/psibi/justl.el) provides commands for executing and listing recipes.

You can add the following to an individual `justfile` to enable `make` mode on a per-file basis:

```text
# Local Variables:
# mode: makefile
# End:
```

### Visual Studio Code

An extension for VS Code by [skellock](https://github.com/skellock) is [available here](https://marketplace.visualstudio.com/items?itemName=skellock.just) ([repository](https://github.com/skellock/vscode-just)).

You can install it from the command line by running:

```sh
code --install-extension skellock.just
```

### JetBrains IDEs

A plugin for JetBrains IDEs by [linux_china](https://github.com/linux-china) is [available here](https://plugins.jetbrains.com/plugin/18658-just).

### Kakoune

Kakoune supports `justfile` syntax highlighting out of the box, thanks to TeddyDD.

### Sublime Text

A syntax file for Sublime Text written by TonioGela is available in [extras/just.sublime-syntax](https://github.com/casey/just/blob/master/extras/just.sublime-syntax).

### Other Editors

Feel free to send me the commands necessary to get syntax highlighting working in your editor of choice so that I may include them here.

Quick Start
-----------

See [the installation section](#installation) for how to install `just` on your computer. Try running `just --version` to make sure that it's installed correctly.

For an overview of the syntax, check out [this cheatsheet](https://cheatography.com/linux-china/cheat-sheets/justfile/).

Once `just` is installed and working, create a file named `justfile` in the root of your project with the following contents:

```make
recipe-name:
  echo 'This is a recipe!'

# this is a comment
another-recipe:
  @echo 'This is another recipe.'
```

When you invoke `just` it looks for file `justfile` in the current directory and upwards, so you can invoke it from any subdirectory of your project.

The search for a `justfile` is case insensitive, so any case, like `Justfile`, `JUSTFILE`, or `JuStFiLe`, will work. `just` will also look for files with the name `.justfile`, in case you'd like to hide a `justfile`.

Running `just` with no arguments runs the first recipe in the `justfile`:

```sh
$ just
echo 'This is a recipe!'
This is a recipe!
```

One or more arguments specify the recipe(s) to run:

```sh
$ just another-recipe
This is another recipe.
```

`just` prints each command to standard error before running it, which is why `echo 'This is a recipe!'` was printed. This is suppressed for lines starting with `@`, which is why `echo 'This is another recipe.'` was not printed.

Recipes stop running if a command fails. Here `cargo publish` will only run if `cargo test` succeeds:

```make
publish:
  cargo test
  # tests passed, time to publish!
  cargo publish
```

Recipes can depend on other recipes. Here the `test` recipe depends on the `build` recipe, so `build` will run before `test`:

```make
build:
  cc main.c foo.c bar.c -o main

test: build
  ./test

sloc:
  @echo "`wc -l *.c` lines of code"
```

```sh
$ just test
cc main.c foo.c bar.c -o main
./test
testing… all tests passed!
```

Recipes without dependencies will run in the order they're given on the command line:

```sh
$ just build sloc
cc main.c foo.c bar.c -o main
1337 lines of code
```

Dependencies will always run first, even if they are passed after a recipe that depends on them:

```sh
$ just test build
cc main.c foo.c bar.c -o main
./test
testing… all tests passed!
```

Examples
--------

A variety of example `justfile`s can be found in the [examples directory](https://github.com/casey/just/tree/master/examples).

Features
--------

### The Default Recipe

When `just` is invoked without a recipe, it runs the first recipe in the `justfile`. This recipe might be the most frequently run command in the project, like running the tests:

```make
test:
  cargo test
```

You can also use dependencies to run multiple recipes by default:

```make
default: lint build test

build:
  echo Building…

test:
  echo Testing…

lint:
  echo Linting…
```

If no recipe makes sense as the default recipe, you can add a recipe to the beginning of your `justfile` that lists the available recipes:

```make
default:
  just --list
```

### Listing Available Recipes

Recipes can be listed in alphabetical order with `just --list`:

```sh
$ just --list
Available recipes:
    build
    test
    deploy
    lint
```

`just --summary` is more concise:

```sh
$ just --summary
build test deploy lint
```

Pass `--unsorted` to print recipes in the order they appear in the `justfile`:

```make
test:
  echo 'Testing!'

build:
  echo 'Building!'
```

```sh
$ just --list --unsorted
Available recipes:
    test
    build
```

```sh
$ just --summary --unsorted
test build
```

If you'd like `just` to default to listing the recipes in the `justfile`, you can use this as your default recipe:

```make
default:
  @just --list
```

The heading text can be customized with `--list-heading`:

```sh
$ just --list --list-heading $'Cool stuff…\n'
Cool stuff…
    test
    build
```

And the indentation can be customized with `--list-prefix`:

```sh
$ just --list --list-prefix ····
Available recipes:
····test
····build
```

The argument to `--list-heading` replaces both the heading and the newline following it, so it should contain a newline if non-empty. It works this way so you can suppress the heading line entirely by passing the empty string:

```sh
$ just --list --list-heading ''
    test
    build
```

### Aliases

Aliases allow recipes to be invoked with alternative names:

```make
alias b := build

build:
  echo 'Building!'
```

```sh
$ just b
build
echo 'Building!'
Building!
```

### Settings

Settings control interpretation and execution. Each setting may be specified at most once, anywhere in the `justfile`.

For example:

```make
set shell := ["zsh", "-cu"]

foo:
  # this line will be run as `zsh -cu 'ls **/*.txt'`
  ls **/*.txt
```

#### Table of Settings

| Name                      | Value              | Description                                                                                   |
| ------------------------- | ------------------ | --------------------------------------------------------------------------------------------- |
| `allow-duplicate-recipes` | boolean            | Allow recipes appearing later in a `justfile` to override earlier recipes with the same name. |
| `dotenv-load`             | boolean            | Load a `.env` file, if present.                                                               |
| `export`                  | boolean            | Export all variables as environment variables.                                                |
| `positional-arguments`    | boolean            | Pass positional arguments.                                                                    |
| `shell`                   | `[COMMAND, ARGS…]` | Set the command used to invoke recipes and evaluate backticks.                                |
| `windows-powershell`      | boolean            | Use PowerShell on Windows as default shell.                                                   |

Boolean settings can be written as:

```mf
set NAME
```

Which is equivalent to:

```mf
set NAME := true
```

#### Allow Duplicate Recipes

If `allow-duplicate-recipes` is set to `true`, defining multiple recipes with the same name is not an error and the last definition is used. Defaults to `false`.

```make
set allow-duplicate-recipes

@foo:
  echo foo

@foo:
  echo bar
```

```sh
$ just foo
bar
```

#### Dotenv Load

If `dotenv-load` is `true`, a `.env` file will be loaded if present. Defaults to `false`.

#### Export

The `export` setting causes all `just` variables to be exported as environment variables. Defaults to `false`.

```make
set export

a := "hello"

@foo b:
  echo $a
  echo $b
```

```sh
$ just foo goodbye
hello
goodbye
```

#### Positional Arguments

If `positional-arguments` is `true`, recipe arguments will be passed as positional arguments to commands. For linewise recipes, argument `$0` will be the name of the recipe.

For example, running this recipe:

```make
set positional-arguments

@foo bar:
  echo $0
  echo $1
```

Will produce the following output:

```sh
$ just foo hello
foo
hello
```

When using an `sh`-compatible shell, such as `bash` or `zsh`, `$@` expands to the positional arguments given to the recipe, starting from one. When used within double quotes as `"$@"`, arguments including whitespace will be passed on as if they were double-quoted. That is, `"$@"` is equivalent to `"$1" "$2"`… When there are no positional parameters, `"$@"` and `$@` expand to nothing (i.e., they are removed).

This example recipe will print arguments one by one on separate lines:

```make
set positional-arguments

@test *args='':
  bash -c 'while (( "$#" )); do echo - $1; shift; done' -- "$@"
```

Running it with _two_ arguments:

```sh
$ just test foo "bar baz"
- foo
- bar baz
```

#### Shell

The `shell` setting controls the command used to invoke recipe lines and backticks. Shebang recipes are unaffected.

```make
# use python3 to execute recipe lines and backticks
set shell := ["python3", "-c"]

# use print to capture result of evaluation
foos := `print("foo" * 4)`

foo:
  print("Snake snake snake snake.")
  print("{{foos}}")
```

`just` passes the command to be executed as an argument. Many shells will need an additional flag, often `-c`, to make them evaluate the first argument.

##### Windows Shell

`just` uses `sh` on Windows by default. To use a different shell on Windows, use `windows-shell`:

```make
set windows-shell := ["pwsh.exe", "-NoLogo", "-Command"]

hello:
  Write-Host "Hello, world!"
```

##### Windows PowerShell

*`set windows-powershell` uses the legacy `powershell.exe` binary, and is no longer recommended. See the `windows-shell` setting above for a more flexible way to control which shell is used on Windows.*

`just` uses `sh` on Windows by default. To use `powershell.exe` instead, set `windows-powershell` to true.

```make
set windows-powershell := true

hello:
  Write-Host "Hello, world!"
```

##### Python 3

```make
set shell := ["python3", "-c"]
```

##### Bash

```make
set shell := ["bash", "-uc"]
```

##### Z Shell

```make
set shell := ["zsh", "-uc"]
```

##### Fish

```make
set shell := ["fish", "-c"]
```

##### Nushell

```make
set shell := ["nu", "-c"]
```

If you want to change the default table mode to `light`:

```make
set shell := ['nu', '-m', 'light', '-c']
```

*[Nushell](https://github.com/nushell/nushell) was written in Rust, and **has cross-platform support for Windows / macOS and Linux**.*

### Documentation Comments

Comments immediately preceding a recipe will appear in `just --list`:

```make
# build stuff
build:
  ./bin/build

# test stuff
test:
  ./bin/test
```

```sh
$ just --list
Available recipes:
    build # build stuff
    test # test stuff
```

### Dotenv Integration

If [`dotenv-load`](#dotenv-load) is set, `just` will load environment variables from a file named `.env`. This file can be located in the same directory as your `justfile` or in a parent directory. These variables are environment variables, not `just` variables, and so must be accessed using `$VARIABLE_NAME` in recipes and backticks.

For example, if your `.env` file contains:

```sh
# a comment, will be ignored
DATABASE_ADDRESS=localhost:6379
SERVER_PORT=1337
```

And your `justfile` contains:

```make
set dotenv-load

serve:
  @echo "Starting server with database $DATABASE_ADDRESS on port $SERVER_PORT…"
  ./server --database $DATABASE_ADDRESS --port $SERVER_PORT
```

`just serve` will output:

```sh
$ just serve
Starting server with database localhost:6379 on port 1337…
./server --database $DATABASE_ADDRESS --port $SERVER_PORT
```

### Variables and Substitution

Variables, strings, concatenation, path joining, and substitution using `{{…}}` are supported:

```make
tmpdir  := `mktemp`
version := "0.2.7"
tardir  := tmpdir / "awesomesauce-" + version
tarball := tardir + ".tar.gz"

publish:
  rm -f {{tarball}}
  mkdir {{tardir}}
  cp README.md *.c {{tardir}}
  tar zcvf {{tarball}} {{tardir}}
  scp {{tarball}} me@server.com:release/
  rm -rf {{tarball}} {{tardir}}
```

#### Joining Paths

The `/` operator can be used to join two strings with a slash:

```make
foo := "a" / "b"
```

```
$ just --evaluate foo
a/b
```

Note that a `/` is added even if one is already present:

```make
foo := "a/"
bar := foo / "b"
```

```
$ just --evaluate bar
a//b
```

The `/` operator uses the `/` character, even on Windows. Thus, using the `/` operator should be avoided with paths that use universal naming convention (UNC), i.e., those that start with `\?`, since forward slashes are not supported with UNC paths.

#### Escaping `{{`

To write a recipe containing `{{`, use `{{{{`:

```make
braces:
  echo 'I {{{{LOVE}} curly braces!'
```

(An unmatched `}}` is ignored, so it doesn't need to be escaped.)

Another option is to put all the text you'd like to escape inside of an interpolation:

```make
braces:
  echo '{{'I {{LOVE}} curly braces!'}}'
```

Yet another option is to use `{{ "{{" }}`:

```make
braces:
  echo 'I {{ "{{" }}LOVE}} curly braces!'
```

### Strings

Double-quoted strings support escape sequences:

```make
string-with-tab             := "\t"
string-with-newline         := "\n"
string-with-carriage-return := "\r"
string-with-double-quote    := "\""
string-with-slash           := "\\"
string-with-no-newline      := "\
"
```

```sh
$ just --evaluate
"tring-with-carriage-return := "
string-with-double-quote    := """
string-with-newline         := "
"
string-with-no-newline      := ""
string-with-slash           := "\"
string-with-tab             := "     "
```

Strings may contain line breaks:

```make
single := '
hello
'

double := "
goodbye
"
```

Single-quoted strings do not recognize escape sequences:

```make
escapes := '\t\n\r\"\\'
```

```sh
$ just --evaluate
escapes := "\t\n\r\"\\"
```

Indented versions of both single- and double-quoted strings, delimited by triple single- or triple double-quotes, are supported. Indented string lines are stripped of leading whitespace common to all non-blank lines:

```make
# this string will evaluate to `foo\nbar\n`
x := '''
  foo
  bar
'''

# this string will evaluate to `abc\n  wuv\nbar\n`
y := """
  abc
    wuv
  xyz
"""
```

Similar to unindented strings, indented double-quoted strings process escape sequences, and indented single-quoted strings ignore escape sequences. Escape sequence processing takes place after unindentation. The unindention algorithm does not take escape-sequence produced whitespace or newlines into account.

### Ignoring Errors

Normally, if a command returns a non-zero exit status, execution will stop. To continue execution after a command, even if it fails, prefix the command with `-`:

```make
foo:
  -cat foo
  echo 'Done!'
```

```sh
$ just foo
cat foo
cat: foo: No such file or directory
echo 'Done!'
Done!
```

### Functions

`just` provides a few built-in functions that might be useful when writing recipes.

#### System Information

- `arch()` — Instruction set architecture. Possible values are: `"aarch64"`, `"arm"`, `"asmjs"`, `"hexagon"`, `"mips"`, `"msp430"`, `"powerpc"`, `"powerpc64"`, `"s390x"`, `"sparc"`, `"wasm32"`, `"x86"`, `"x86_64"`, and `"xcore"`.

- `os()` — Operating system. Possible values are: `"android"`, `"bitrig"`, `"dragonfly"`, `"emscripten"`, `"freebsd"`, `"haiku"`, `"ios"`, `"linux"`, `"macos"`, `"netbsd"`, `"openbsd"`, `"solaris"`, and `"windows"`.

- `os_family()` — Operating system family; possible values are: `"unix"` and `"windows"`.

For example:

```make
system-info:
  @echo "This is an {{arch()}} machine".
```

```sh
$ just system-info
This is an x86_64 machine
```

The `os_family()` function can be used to create cross-platform `justfile`s that work on various operating systems. For an example, see [cross-platform.just](https://github.com/casey/just/blob/master/examples/cross-platform.just) file.

#### Environment Variables

- `env_var(key)` — Retrieves the environment variable with name `key`, aborting if it is not present.

```make
home_dir := env_var('HOME')

test:
  echo "{{home_dir}}"
```

```sh
$ just
/home/user1
```

- `env_var_or_default(key, default)` — Retrieves the environment variable with name `key`, returning `default` if it is not present.

#### Invocation Directory

- `invocation_directory()` - Retrieves the absolute path to the current directory when `just` was invoked, before  `just` changed it (chdir'd) prior to executing commands.

For example, to call `rustfmt` on files just under the "current directory" (from the user/invoker's perspective), use the following rule:

```make
rustfmt:
  find {{invocation_directory()}} -name \*.rs -exec rustfmt {} \;
```

Alternatively, if your command needs to be run from the current directory, you could use (e.g.):

```make
build:
  cd {{invocation_directory()}}; ./some_script_that_needs_to_be_run_from_here
```

#### Justfile and Justfile Directory

- `justfile()` - Retrieves the path of the current `justfile`.

- `justfile_directory()` - Retrieves the path of the parent directory of the current `justfile`.

For example, to run a command relative to the location of the current `justfile`:

```make
script:
  ./{{justfile_directory()}}/scripts/some_script
```

#### Just Executable

- `just_executable()` - Absolute path to the `just` executable.

For example:

```make
executable:
  @echo The executable is at: {{just_executable()}}
```

```sh
$ just
The executable is at: /bin/just
```

#### String Manipulation

- `lowercase(s)` - Convert `s` to lowercase.

- `quote(s)` - Replace all single quotes with `'\''` and prepend and append single quotes to `s`. This is sufficient to escape special characters for many shells, including most Bourne shell descendants.

- `replace(s, from, to)` - Replace all occurrences of `from` in `s` to `to`.

- `trim(s)` - Remove leading and trailing whitespace from `s`.

- `trim_end(s)` - Remove trailing whitespace from `s`.

- `trim_end_match(s, pat)` - Remove suffix of `s` matching `pat`.

- `trim_end_matches(s, pat)` - Repeatedly remove suffixes of `s` matching `pat`.

- `trim_start(s)` - Remove leading whitespace from `s`.

- `trim_start_match(s, pat)` - Remove prefix of `s` matching `pat`.

- `trim_start_matches(s, pat)` - Repeatedly remove prefixes of `s` matching `pat`.

- `uppercase(s)` - Convert `s` to uppercase.

#### Path Manipulation

##### Fallible

- `absolute_path(path)` - Absolute path to relative `path` in the working directory. `absolute_path("./bar.txt")` in directory `/foo` is `/foo/bar.txt`.

- `extension(path)` - Extension of `path`. `extension("/foo/bar.txt")` is `txt`.

- `file_name(path)` - File name of `path` with any leading directory components removed. `file_name("/foo/bar.txt")` is `bar.txt`.

- `file_stem(path)` - File name of `path` without extension. `file_stem("/foo/bar.txt")` is `bar`.

- `parent_directory(path)` - Parent directory of `path`. `parent_directory("/foo/bar.txt")` is `/foo`.

- `without_extension(path)` - `path` without extension. `without_extension("/foo/bar.txt")` is `/foo/bar`.

These functions can fail, for example if a path does not have an extension, which will halt execution.

##### Infallible

- `join(a, b…)` - *This function uses `/` on Unix and `\` on Windows, which can be lead to unwanted behavior. The `/` operator, e.g., `a / b`, which always uses `/`, should be considered as a replacement unless `\`s are specifically desired on Windows.* Join path `a` with path `b`. `join("foo/bar", "baz")` is `foo/bar/baz`. Accepts two or more arguments.

- `clean(path)` - Simplify `path` by removing extra path separators, intermediate `.` components, and `..` where possible. `clean("foo//bar")` is `foo/bar`, `clean("foo/..")` is `.`, `clean("foo/./bar")` is `foo/bar`.

#### Filesystem Access

- `path_exists(path)` - Returns `true` if the path points at an existing entity and `false` otherwise. Traverses symbolic links, and returns `false` if the path is inaccessible or points to a broken symlink.

##### Error Reporting

- `error(message)` - Abort execution and report error `message` to user.

#### UUID and Hash Generation

- `sha256(string)` - Return the SHA-256 hash of `string` as a hexadecimal string.
- `sha256_file(path)` - Return the SHA-256 hash of the file at `path` as a hexadecimal string.
- `uuid()` - Return a randomly generated UUID.

### Command Evaluation Using Backticks

Backticks can be used to store the result of commands:

```make
localhost := `dumpinterfaces | cut -d: -f2 | sed 's/\/.*//' | sed 's/ //g'`

serve:
  ./serve {{localhost}} 8080
```

Indented backticks, delimited by three backticks, are de-indented in the same manner as indented strings:

````make
# This backtick evaluates the command `echo foo\necho bar\n`, which produces the value `foo\nbar\n`.
stuff := ```
    echo foo
    echo bar
  ```
````

See the [Strings](#strings) section for details on unindenting.

Backticks may not start with `#!`. This syntax is reserved for a future upgrade.

### Conditional Expressions

`if`/`else` expressions evaluate different branches depending on if two expressions evaluate to the same value:

```make
foo := if "2" == "2" { "Good!" } else { "1984" }

bar:
  @echo "{{foo}}"
```

```sh
$ just bar
Good!
```

It is also possible to test for inequality:

```make
foo := if "hello" != "goodbye" { "xyz" } else { "abc" }

bar:
  @echo {{foo}}
```

```sh
$ just bar
xyz
```

And match against regular expressions:

```make
foo := if "hello" =~ 'hel+o' { "match" } else { "mismatch" }

bar:
  @echo {{foo}}
```

```sh
$ just bar
match
```

Regular expressions are provided by the [regex crate](https://github.com/rust-lang/regex), whose syntax is documented on [docs.rs](https://docs.rs/regex/1.5.4/regex/#syntax). Since regular expressions commonly use backslash escape sequences, consider using single-quoted string literals, which will pass slashes to the regex parser unmolested.

Conditional expressions short-circuit, which means they only evaluate one of their branches. This can be used to make sure that backtick expressions don't run when they shouldn't.

```make
foo := if env_var("RELEASE") == "true" { `get-something-from-release-database` } else { "dummy-value" }
```

Conditionals can be used inside of recipes:

```make
bar foo:
  echo {{ if foo == "bar" { "hello" } else { "goodbye" } }}
```

Note the space after the final `}`! Without the space, the interpolation will be prematurely closed.

Multiple conditionals can be chained:

```make
foo := if "hello" == "goodbye" {
  "xyz"
} else if "a" == "a" {
  "abc"
} else {
  "123"
}

bar:
  @echo {{foo}}
```

```sh
$ just bar
abc
```

### Stopping execution with error

Execution can be halted with the `error` function. For example:

```
foo := if "hello" == "goodbye" {
  "xyz"
} else if "a" == "b" {
  "abc"
} else {
  error("123")
}
```

Which produce the following error when run:

```
error: Call to function `error` failed: 123
   |
16 |   error("123")
```

### Setting Variables from the Command Line

Variables can be overridden from the command line.

```make
os := "linux"

test: build
  ./test --test {{os}}

build:
  ./build {{os}}
```

```sh
$ just
./build linux
./test --test linux
```

Any number of arguments of the form `NAME=VALUE` can be passed before recipes:

```sh
$ just os=plan9
./build plan9
./test --test plan9
```

Or you can use the `--set` flag:

```sh
$ just --set os bsd
./build bsd
./test --test bsd
```

### Getting and Setting Environment Variables

#### Exporting `just` Variables

Assignments prefixed with the `export` keyword will be exported to recipes as environment variables:

```make
export RUST_BACKTRACE := "1"

test:
  # will print a stack trace if it crashes
  cargo test
```

Parameters prefixed with a `$` will be exported as environment variables:

```make
test $RUST_BACKTRACE="1":
  # will print a stack trace if it crashes
  cargo test
```

Exported variables and parameters are not exported to backticks in the same scope.

```make
export WORLD := "world"
# This backtick will fail with "WORLD: unbound variable"
BAR := `echo hello $WORLD`
```

```make
# Running `just a foo` will fail with "A: unbound variable"
a $A $B=`echo $A`:
  echo $A $B
```

When [export](#export) is set, all `just` variables are exported as environment variables.

#### Getting Environment Variables from the environment

Environment variables from the environment are passed automatically to the recipes.

```make
print_home_folder:
  echo "HOME is: '${HOME}'"
```

```sh
$ just
HOME is '/home/myuser'
```
#### Loading Environment Variables from a `.env` File

`just` will load environment variables from a `.env` file if [dotenv-load](#dotenv-load) is set. The variables in the file will be available as environment variables to the recipes. See [dotenv-integration](#dotenv-integration) for more information.

#### Setting `just` Variables from Environments Variables

Environment variables can be propagated to `just` variables using the functions `env_var()` and `env_var_or_default()`.
See [environment-variables](#environment-variables).

### Recipe Parameters

Recipes may have parameters. Here recipe `build` has a parameter called `target`:

```make
build target:
  @echo 'Building {{target}}…'
  cd {{target}} && make
```

To pass arguments on the command line, put them after the recipe name:

```sh
$ just build my-awesome-project
Building my-awesome-project…
cd my-awesome-project && make
```

To pass arguments to a dependency, put the dependency in parentheses along with the arguments:

```make
default: (build "main")

build target:
  @echo 'Building {{target}}…'
  cd {{target}} && make
```

A command's arguments can be passed to dependency by putting the dependency in parentheses along with the arguments:

```make
build target:
  @echo "Building {{target}}…"

push target: (build target)
  @echo 'Pushing {{target}}…'
```

Parameters may have default values:

```make
default := 'all'

test target tests=default:
  @echo 'Testing {{target}}:{{tests}}…'
  ./test --tests {{tests}} {{target}}
```

Parameters with default values may be omitted:

```sh
$ just test server
Testing server:all…
./test --tests all server
```

Or supplied:

```sh
$ just test server unit
Testing server:unit…
./test --tests unit server
```

Default values may be arbitrary expressions, but concatenations or path joins must be parenthesized:

```make
arch := "wasm"

test triple=(arch + "-unknown-unknown") input=(arch / "input.dat"):
  ./test {{triple}}
```

The last parameter of a recipe may be variadic, indicated with either a `+` or a `*` before the argument name:

```make
backup +FILES:
  scp {{FILES}} me@server.com:
```

Variadic parameters prefixed with `+` accept _one or more_ arguments and expand to a string containing those arguments separated by spaces:

```sh
$ just backup FAQ.md GRAMMAR.md
scp FAQ.md GRAMMAR.md me@server.com:
FAQ.md                  100% 1831     1.8KB/s   00:00
GRAMMAR.md              100% 1666     1.6KB/s   00:00
```

Variadic parameters prefixed with `*` accept _zero or more_ arguments and expand to a string containing those arguments separated by spaces, or an empty string if no arguments are present:

```make
commit MESSAGE *FLAGS:
  git commit {{FLAGS}} -m "{{MESSAGE}}"
```

Variadic parameters can be assigned default values. These are overridden by arguments passed on the command line:

```make
test +FLAGS='-q':
  cargo test {{FLAGS}}
```

`{{…}}` substitutions may need to be quoted if they contain spaces. For example, if you have the following recipe:

```make
search QUERY:
  lynx https://www.google.com/?q={{QUERY}}
```

And you type:

```sh
$ just search "cat toupee"
```

`just` will run the command `lynx https://www.google.com/?q=cat toupee`, which will get parsed by `sh` as `lynx`, `https://www.google.com/?q=cat`, and `toupee`, and not the intended `lynx` and `https://www.google.com/?q=cat toupee`.

You can fix this by adding quotes:

```make
search QUERY:
  lynx 'https://www.google.com/?q={{QUERY}}'
```

Parameters prefixed with a `$` will be exported as environment variables:

```make
foo $bar:
  echo $bar
```

### Running Recipes at the End of a Recipe

Normal dependencies of a recipes always run before a recipe starts. That is to say, the dependee always runs before the depender. These dependencies are called "prior dependencies".

A recipe can also have subsequent dependencies, which run after the recipe and are introduced with an `&&`:

```make
a:
  echo 'A!'

b: a && c d
  echo 'B!'

c:
  echo 'C!'

d:
  echo 'D!'
```

…running _b_ prints:

```sh
$ just b
echo 'A!'
A!
echo 'B!'
B!
echo 'C!'
C!
echo 'D!'
D!
```

### Running Recipes in the Middle of a Recipe

`just` doesn't support running recipes in the middle of another recipe, but you can call `just` recursively in the middle of a recipe. Given the following `justfile`:

```make
a:
  echo 'A!'

b: a
  echo 'B start!'
  just c
  echo 'B end!'

c:
  echo 'C!'
```

…running _b_ prints:

```sh
$ just b
echo 'A!'
A!
echo 'B start!'
B start!
echo 'C!'
C!
echo 'B end!'
B end!
```

This has limitations, since recipe `c` is run with an entirely new invocation of `just`: Assignments will be recalculated, dependencies might run twice, and command line arguments will not be propagated to the child `just` process.

### Writing Recipes in Other Languages

Recipes that start with a `#!` are executed as scripts, so you can write recipes in other languages:

```make
polyglot: python js perl sh ruby

python:
  #!/usr/bin/env python3
  print('Hello from python!')

js:
  #!/usr/bin/env node
  console.log('Greetings from JavaScript!')

perl:
  #!/usr/bin/env perl
  print "Larry Wall says Hi!\n";

sh:
  #!/usr/bin/env sh
  hello='Yo'
  echo "$hello from a shell script!"

nu:
  #!/usr/bin/env nu
  let hello = 'Yo'
  echo $"($hello) from a shell script!"

ruby:
  #!/usr/bin/env ruby
  puts "Hello from ruby!"
```

```sh
$ just polyglot
Hello from python!
Greetings from JavaScript!
Larry Wall says Hi!
Yo from a shell script!
Hello from ruby!
```

### Safer Bash Shebang Recipes

If you're writing a `bash` shebang recipe, consider adding `set -euxo pipefail`:

```make
foo:
  #!/usr/bin/env bash
  set -euxo pipefail
  hello='Yo'
  echo "$hello from Bash!"
```

It isn't strictly necessary, but `set -euxo pipefail` turns on a few useful features that make `bash` shebang recipes behave more like normal, linewise `just` recipe:

- `set -e` makes `bash` exit if a command fails.

- `set -u` makes `bash` exit if a variable is undefined.

- `set -x` makes `bash` print each script line before it's run.

- `set -o pipefail` makes `bash` exit if a command in a pipeline fails. This is `bash`-specific, so isn't turned on in normal linewise `just` recipes.

Together, these avoid a lot of shell scripting gotchas.

#### Shebang Recipe Execution on Windows

On Windows, shebang interpreter paths containing a `/` are translated from Unix-style paths to Windows-style paths using `cygpath`, a utility that ships with [Cygwin](http://www.cygwin.com).

For example, to execute this recipe on Windows:

```make
echo:
  #!/bin/sh
  echo "Hello!"
```

The interpreter path `/bin/sh` will be translated to a Windows-style path using `cygpath` before being executed.

If the interpreter path does not contain a `/` it will be executed without being translated. This is useful if `cygpath` is not available, or you wish to pass a Windows-style path to the interpreter.

### Setting Variables in a Recipe

Recipe lines are interpreted by the shell, not `just`, so it's not possible to set `just` variables in the middle of a recipe:

```mf
foo:
  x := "hello" # This doesn't work!
  echo {{x}}
```

It is possible to use shell variables, but there's another problem. Every recipe line is run by a new shell instance, so variables set in one line won't be set in the next:

```make
foo:
  x=hello && echo $x # This works!
  y=bye
  echo $y            # This doesn't, `y` is undefined here!
```

The best way to work around this is to use a shebang recipe. Shebang recipe bodies are extracted and run as scripts, so a single shell instance will run the whole thing:

```make
foo:
  #!/usr/bin/env bash
  set -euxo pipefail
  x=hello
  echo $x
```

### Sharing Environment Variables Between Recipes

Each line of each recipe is executed by a fresh shell, so it is not possible to share environment variables between recipes.

#### Using Python Virtual Environments

Some tools, like [Python's venv](https://docs.python.org/3/library/venv.html), require loading environment variables in order to work, making them challenging to use with `just`. As a workaround, you can execute the virtual environment binaries directly:

```make
venv:
  [ -d foo ] || python3 -m venv foo

run: venv
  ./foo/bin/python3 main.py
```

### Changing the Working Directory in a Recipe

Each recipe line is executed by a new shell, so if you change the working directory on one line, it won't have an effect on later lines:

```make
foo:
  pwd    # This `pwd` will print the same directory…
  cd bar
  pwd    # …as this `pwd`!
```

There are a couple ways around this. One is to call `cd` on the same line as the command you want to run:

```make
foo:
  cd bar && pwd
```

The other is to use a shebang recipe. Shebang recipe bodies are extracted and run as scripts, so a single shell instance will run the whole thing, and thus a `pwd` on one line will affect later lines, just like a shell script:

```make
foo:
  #!/usr/bin/env bash
  set -euxo pipefail
  cd bar
  pwd
```

### Indentation

Recipe lines can be indented with spaces or tabs, but not a mix of both. All of a recipe's lines must have the same indentation, but different recipes in the same `justfile` may use different indentation.

### Multi-Line Constructs

Recipes without an initial shebang are evaluated and run line-by-line, which means that multi-line constructs probably won't do what you want.

For example, with the following `justfile`:

```mf
conditional:
  if true; then
    echo 'True!'
  fi
```

The extra leading whitespace before the second line of the `conditional` recipe will produce a parse error:

```sh
$ just conditional
error: Recipe line has extra leading whitespace
  |
3 |         echo 'True!'
  |     ^^^^^^^^^^^^^^^^
```

To work around this, you can write conditionals on one line, escape newlines with slashes, or add a shebang to your recipe. Some examples of multi-line constructs are provided for reference.

#### `if` statements

```make
conditional:
  if true; then echo 'True!'; fi
```

```make
conditional:
  if true; then \
    echo 'True!'; \
  fi
```

```make
conditional:
  #!/usr/bin/env sh
  if true; then
    echo 'True!'
  fi
```

#### `for` loops

```make
for:
  for file in `ls .`; do echo $file; done
```

```make
for:
  for file in `ls .`; do \
    echo $file; \
  done
```

```make
for:
  #!/usr/bin/env sh
  for file in `ls .`; do
    echo $file
  done
```

#### `while` loops

```make
while:
  while `server-is-dead`; do ping -c 1 server; done
```

```make
while:
  while `server-is-dead`; do \
    ping -c 1 server; \
  done
```

```make
while:
  #!/usr/bin/env sh
  while `server-is-dead`; do
    ping -c 1 server
  done
```

### Command Line Options

`just` supports a number of useful command line options for listing, dumping, and debugging recipes and variable:

```sh
$ just --list
Available recipes:
  js
  perl
  polyglot
  python
  ruby
$ just --show perl
perl:
  #!/usr/bin/env perl
  print "Larry Wall says Hi!\n";
$ just --show polyglot
polyglot: python js perl sh ruby
```

Run `just --help` to see all the options.

### Private Recipes

Recipes and aliases whose name starts with a `_` are omitted from `just --list`:

```make
test: _test-helper
  ./bin/test

_test-helper:
  ./bin/super-secret-test-helper-stuff
```

```sh
$ just --list
Available recipes:
    test
```

And from `just --summary`:

```sh
$ just --summary
test
```

This is useful for helper recipes which are only meant to be used as dependencies of other recipes.

### Quiet Recipes

A recipe name may be prefixed with `@` to invert the meaning of `@` before each line:

```make
@quiet:
  echo hello
  echo goodbye
  @# all done!
```

Now only the lines starting with `@` will be echoed:

```sh
$ j quiet
hello
goodbye
# all done!
```

Shebang recipes are quiet by default:

```make
foo:
  #!/usr/bin/env bash
  echo 'Foo!'
```

```sh
$ just foo
Foo!
```

Adding `@` to a shebang recipe name makes `just` print the recipe before executing it:

```make
@bar:
  #!/usr/bin/env bash
  echo 'Bar!'
```

```sh
$ just bar
#!/usr/bin/env bash
echo 'Bar!'
Bar!
```

### Selecting Recipes to Run With an Interactive Chooser

The `--choose` subcommand makes `just` invoke a chooser to select which recipes to run. Choosers should read lines containing recipe names from standard input and print one or more of those names separated by spaces to standard output.

Because there is currently no way to run a recipe that requires arguments with `--choose`, such recipes will not be given to the chooser. Private recipes and aliases are also skipped.

The chooser can be overridden with the `--chooser` flag. If `--chooser` is not given, then `just` first checks if `$JUST_CHOOSER` is set. If it isn't, then the chooser defaults to `fzf`, a popular fuzzy finder.

Arguments can be included in the chooser, i.e. `fzf --exact`.

The chooser is invoked in the same way as recipe lines. For example, if the chooser is `fzf`, it will be invoked with `sh -cu 'fzf'`, and if the shell, or the shell arguments are overridden, the chooser invocation will respect those overrides.

If you'd like `just` to default to selecting recipes with a chooser, you can use this as your default recipe:

```make
default:
  @just --choose
```

### Invoking `justfile`s in Other Directories

If the first argument passed to `just` contains a `/`, then the following occurs:

1.  The argument is split at the last `/`.

2.  The part before the last `/` is treated as a directory. `just` will start its search for the `justfile` there, instead of in the current directory.

3.  The part after the last slash is treated as a normal argument, or ignored if it is empty.

This may seem a little strange, but it's useful if you wish to run a command in a `justfile` that is in a subdirectory.

For example, if you are in a directory which contains a subdirectory named `foo`, which contains a `justfile` with the recipe `build`, which is also the default recipe, the following are all equivalent:

```sh
$ (cd foo && just build)
$ just foo/build
$ just foo/
```

### Hiding `justfile`s

`just` looks for `justfile`s named `justfile` and `.justfile`, which can be used to keep a `justfile` hidden.

### Just Scripts

By adding a shebang line to the top of a `justfile` and making it executable, `just` can be used as an interpreter for scripts:

```sh
$ cat > script <<EOF
#!/usr/bin/env just --justfile

foo:
  echo foo
EOF
$ chmod +x script
$ ./script foo
echo foo
foo
```

When a script with a shebang is executed, the system supplies the path to the script as an argument to the command in the shebang. So, with a shebang of `#!/usr/bin/env just --justfile`, the command will be `/usr/bin/env just --justfile PATH_TO_SCRIPT`.

With the above shebang, `just` will change its working directory to the location of the script. If you'd rather leave the working directory unchanged, use `#!/usr/bin/env just --working-directory . --justfile`.

Note: Shebang line splitting is not consistent across operating systems. The previous examples have only been tested on macOS. On Linux, you may need to pass the `-S` flag to `env`:

```make
#!/usr/bin/env -S just --justfile

default:
  echo foo
```

### Dumping `justfile`s as JSON

The `--dump` command can be used with `--dump-format json` to print a JSON representation of a `justfile`. The JSON format is currently unstable, so the `--unstable` flag is required.

### Falling back to parent `justfile`s

If a recipe is not found, `just` will look for `justfile`s in the parent
directory and up, until it reaches the root directory.

This feature is currently unstable, and so must be enabled with the
`--unstable` flag.

As an example, suppose the current directory contains this `justfile`:

```make
foo:
  echo foo
```

And the parent directory contains this `justfile`:

```make
bar:
  echo bar
```

```sh
$ just --unstable bar
Trying ../justfile
echo bar
bar
```

### Avoiding Argument Splitting

Given this `justfile`:

```make
foo argument:
  touch {{argument}}
```

The following command will create two files, `some` and `argument.txt`:

```sh
$ just foo "some argument.txt"
```

The users shell will parse `"some argument.txt"` as a single argument, but when `just` replaces `touch {{argument}}` with `touch some argument.txt`, the quotes are not preserved, and `touch` will receive two arguments.

There are a few ways to avoid this: quoting, positional arguments, and exported arguments.

#### Quoting

Quotes can be added around the `{{argument}}` interpolation:

```make
foo argument:
  touch '{{argument}}'
```

This preserves `just`'s ability to catch variable name typos before running, for example if you were to write `{{argument}}`, but will not do what you want if the value of `argument` contains single quotes.

#### Positional Arguments

The `positional-arguments` setting causes all arguments to be passed as positional arguments, allowing them to be accessed with `$1`, `$2`, …, and `$@`, which can be then double-quoted to avoid further splitting by the shell:

```make
set positional-arguments

foo argument:
  touch "$1"
```

This defeats `just`'s ability to catch typos, for example if you type `$2`, but works for all possible values of `argument`, including those with double quotes.

#### Exported Arguments

All arguments are exported when the `export` setting is set:

```make
set export

foo argument:
  touch "$argument"
```

Or individual arguments may be exported by prefixing them with `$`:

```make
foo $argument:
  touch "$argument"
```

This defeats `just`'s ability to catch typos, for example if you type `$argumant`, but works for all possible values of `argument`, including those with double quotes.

Changelog
---------

A changelog for the latest release is available in [CHANGELOG.md](https://raw.githubusercontent.com/casey/just/master/CHANGELOG.md). Changelogs for previous releases are available on [the releases page](https://github.com/casey/just/releases). `just --changelog` can also be used to make a `just` binary print its changelog.

Miscellanea
-----------

### Companion Tools

Tools that pair nicely with `just` include:

- [`watchexec`](https://github.com/mattgreen/watchexec) — a simple tool that watches a path and runs a command whenever it detects modifications.

### Shell Alias

For lightning-fast command running, put `alias j=just` in your shell's configuration file.

In `bash`, the aliased command may not keep the shell completion functionality described in the next section. Add the following line to your `.bashrc` to use the same completion function as `just` for your aliased command:

```sh
complete -F _just -o bashdefault -o default j
```

### Shell Completion Scripts

Shell completion scripts for Bash, Zsh, Fish, PowerShell, and Elvish are available in the [completions](https://github.com/casey/just/tree/master/completions) directory. Please refer to your shell's documentation for how to install them.

The `just` binary can also generate the same completion scripts at runtime, using the `--completions` command:

```sh
$ just --completions zsh > just.zsh
```

*macOS Note:* Recent versions of macOS use zsh as the default shell. If you use Homebrew to install `just`, it will automatically install the most recent copy of the zsh completion script in the Homebrew zsh directory, which the built-in version of zsh doesn't know about by default. It's best to use this copy of the script if possible, since it will be updated whenever you update `just` via Homebrew. Also, many other Homebrew packages use the same location for completion scripts, and the built-in zsh doesn't know about those either. To take advantage of `just` completion in zsh in this scenario, you can set `fpath` to the Homebrew location before calling `compinit`. Note also that Oh My Zsh runs `compinit` by default. So your `.zshrc` file could look like this:

```zsh
# Init Homebrew, which adds environment variables
eval "$(/opt/homebrew/bin/brew shellenv)"

fpath=($HOMEBREW_PREFIX/share/zsh/site-functions $fpath)

# Then choose one of these options:
# 1. If you're using Oh My Zsh, you can initialize it here
# source $ZSH/oh-my-zsh.sh

# 2. Otherwise, run compinit yourself
# autoload -U compinit
# compinit
```

### Grammar

A non-normative grammar of `justfile`s can be found in [GRAMMAR.md](https://github.com/casey/just/blob/master/GRAMMAR.md).

### just.sh

Before `just` was a fancy Rust program it was a tiny shell script that called `make`. You can find the old version in [extras/just.sh](https://github.com/casey/just/blob/master/extras/just.sh).

### User `justfile`s

If you want some recipes to be available everywhere, you have a few options.

First, create a `justfile` in `~/.user.justfile` with some recipes.

#### Recipe Aliases

If you want to call the recipes in `~/.user.justfile` by name, and don't mind creating an alias for every recipe, add the following to your shell's initialization script:

```sh
for recipe in `just --justfile ~/.user.justfile --summary`; do
  alias $recipe="just --justfile ~/.user.justfile --working-directory . $recipe"
done
```

Now, if you have a recipe called `foo` in `~/.user.justfile`, you can just type `foo` at the command line to run it.

It took me way too long to realize that you could create recipe aliases like this. Notwithstanding my tardiness, I am very pleased to bring you this major advance in `justfile` technology.

#### Forwarding Alias

If you'd rather not create aliases for every recipe, you can create a single alias:

```sh
alias .j='just --justfile ~/.user.justfile --working-directory .'
```

Now, if you have a recipe called `foo` in `~/.user.justfile`, you can just type `.j foo` at the command line to run it.

I'm pretty sure that nobody actually uses this feature, but it's there.

¯\\\_(ツ)\_/¯

#### Customization

You can customize the above aliases with additional options. For example, if you'd prefer to have the recipes in your `justfile` run in your home directory, instead of the current directory:

```sh
alias .j='just --justfile ~/.user.justfile --working-directory ~'
```

### Node.js `package.json` Script Compatibility

The following export statement gives `just` recipes access to local Node module binaries, and makes `just` recipe commands behave more like `script` entries in Node.js `package.json` files:

```make
export PATH := "./node_modules/.bin:" + env_var('PATH')
```

### Alternatives and Prior Art

There is no shortage of command runners out there! Some more or less similar alternatives to `just` include:

- [make](https://en.wikipedia.org/wiki/Make_(software)): The Unix build tool that inspired `just`.
- [makesure](https://github.com/xonixx/makesure): A simple and portable command runner written in AWK and shell.
- [mmake](https://github.com/tj/mmake): A wrapper around `make` with a number of improvements, including remote includes.
- [robo](https://github.com/tj/robo): A YAML-based command runner written in Go.

Contributing
------------

`just` welcomes your contributions! `just` is released under the maximally permissive [CC0](https://creativecommons.org/publicdomain/zero/1.0/legalcode.txt) public domain dedication and fallback license, so your changes must also be released under this license.

### Janus

[Janus](https://github.com/casey/janus) is a tool that collects and analyzes `justfile`s, and can determine if a new version of `just` breaks or changes the interpretation of existing `justfile`s.

Before merging a particularly large or gruesome change, Janus should be run to make sure that nothing breaks. Don't worry about running Janus yourself, Casey will happily run it for you on changes that need it.

### Minimum Supported Rust Version

The minimum supported Rust version, or MSRV, is Rust 1.47.0.

Frequently Asked Questions
--------------------------

### What are the idiosyncrasies of Make that Just avoids?

`make` has some behaviors which are confusing, complicated, or make it unsuitable for use as a general command runner.

One example is that under some circumstances, `make` won't actually run the commands in a recipe. For example, if you have a file called `test` and the following makefile:

```make
test:
  ./test
```

`make` will refuse to run your tests:

```sh
$ make test
make: `test' is up to date.
```

`make` assumes that the `test` recipe produces a file called `test`. Since this file exists and the recipe has no other dependencies, `make` thinks that it doesn't have anything to do and exits.

To be fair, this behavior is desirable when using `make` as a build system, but not when using it as a command runner. You can disable this behavior for specific targets using `make`'s built-in [`.PHONY` target name](https://www.gnu.org/software/make/manual/html_node/Phony-Targets.html), but the syntax is verbose and can be hard to remember. The explicit list of phony targets, written separately from the recipe definitions, also introduces the risk of accidentally defining a new non-phony target. In `just`, all recipes are treated as if they were phony.

Other examples of `make`'s idiosyncrasies include the difference between `=` and `:=` in assignments, the confusing error messages that are produced if you mess up your makefile, needing `$$` to use environment variables in recipes, and incompatibilities between different flavors of `make`.

### What's the relationship between Just and Cargo build scripts?

[`cargo` build scripts](http://doc.crates.io/build-script.html) have a pretty specific use, which is to control how `cargo` builds your Rust project. This might include adding flags to `rustc` invocations, building an external dependency, or running some kind of codegen step.

`just`, on the other hand, is for all the other miscellaneous commands you might run as part of development. Things like running tests in different configurations, linting your code, pushing build artifacts to a server, removing temporary files, and the like.

Also, although `just` is written in Rust, it can be used regardless of the language or build system your project uses.

Further Ramblings
-----------------

I personally find it very useful to write a `justfile` for almost every project, big or small.

On a big project with multiple contributors, it's very useful to have a file with all the commands needed to work on the project close at hand.

There are probably different commands to test, build, lint, deploy, and the like, and having them all in one place is useful and cuts down on the time you have to spend telling people which commands to run and how to type them.

And, with an easy place to put commands, it's likely that you'll come up with other useful things which are part of the project's collective wisdom, but which aren't written down anywhere, like the arcane commands needed for some part of your revision control workflow, install all your project's dependencies, or all the random flags you might need to pass to the build system.

Some ideas for recipes:

- Deploying/publishing the project

- Building in release mode vs debug mode

- Running in debug mode or with logging enabled

- Complex git workflows

- Updating dependencies

- Running different sets of tests, for example fast tests vs slow tests, or running them with verbose output

- Any complex set of commands that you really should write down somewhere, if only to be able to remember them

Even for small, personal projects it's nice to be able to remember commands by name instead of ^Reverse searching your shell history, and it's a huge boon to be able to go into an old project written in a random language with a mysterious build system and know that all the commands you need to do whatever you need to do are in the `justfile`, and that if you type `just` something useful (or at least interesting!) will probably happen.

For ideas for recipes, check out [this project's `justfile`](https://github.com/casey/just/blob/master/justfile), or some of the `justfile`s [out in the wild](https://github.com/search?o=desc&q=filename%3Ajustfile&s=indexed&type=Code).

Anyways, I think that's about it for this incredibly long-winded README.

I hope you enjoy using `just` and find great success and satisfaction in all your computational endeavors!

😸
