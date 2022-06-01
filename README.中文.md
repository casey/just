↖️ 目录

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

`just` 为您提供一种保存和运行项目特有命令的便捷方式。

本指南同时也可以以 [书](https://just.systems/man/) 的形式在线阅读；

命令，在此也称为配方，存储在一个名为 `justfile` 的文件中，其语法受 `make` 启发：

![screenshot](screenshot.png)

然后你可以用 `just RECIPE` 运行它们：

```sh
$ just test-all
cc *.c -o main
./test --all
Yay, all your tests passed!
```

`just` 有很多很棒的特性，而且相比 `make` 有很多改进：

- `just` 是一个命令运行器，而不是一个构建系统，所以它避免了许多 [`make` 的复杂性和特异性](#what-are-theidiosyncrasies-of-make-that-just-avoids)。不需要 `.PHONY` 配方!

- 支持 Linux、MacOS 和 Windows，而且无需额外的依赖。(尽管如果你的系统没有 `sh`，你需要 [选择一个不同的 Shell](#shell))。

- 错误具体且富有参考价值，语法错误将会与产生它们的上下文一起被报告。

- 配方可以接受 [命令行参数](#配方参数)。

- 错误会尽可能被静态地解决。未知的配方和循环依赖关系会在运行之前被报告。

- `just` 可以 [加载`.env`文件](#dotenv-integration)，简化环境变量注入。

- 配方可以在 [命令行中列出](#listing-available-recipes)。

- 命令行自动补全脚本 [支持大多数流行的 Shell](#shell-completion-scripts)。

- 配方可以用 [任意语言](#writing-recipes-in-other-languages) 编写，如 Python 或 NodeJS。

- `just` 可以从任何子目录中调用，而不仅仅是包含 `justfile` 的目录。

- 不仅如此，还有 [更多](https://just.systems/man/)！

如果你在使用 `just` 方面需要帮助，请随时创建一个 Issue 或在[Discord](https://discord.gg/ezYScXR)上与我联系。我们随时欢迎功能请求和错误报告！

安装
------------

### 预备知识

`just` 应该可以在任何有合适的 `sh` 的系统上运行，包括Linux、MacOS 和 BSD。

在 Windows 上，`just` 可以使用 [Git for Windows](https://git-scm.com)、[GitHub Desktop](https://desktop.github.com) 或 [Cygwin](http://www.cygwin.com) 所提供的 `sh`。

如果你不愿意安装 `sh`，也可以使用 `shell` 设置来指定你要使用的 Shell。

比如 PowerShell：

```make
# 使用 PowerShell 替代 sh:
set shell := ["powershell.exe", "-c"]

hello:
  Write-Host "Hello, world!"
```

…或者 `cmd.exe`:

```make
# 使用 cmd.exe 替代 sh:
set shell := ["cmd.exe", "/c"]

list:
  dir
```

你也可以使用命令行参数来设置 Shell。例如，若要使用PowerShell 也可以用 `--shell powershell.exe --shell-arg -c` 启动`just`。

(PowerShell 默认安装在 Windows 7 SP1 和 Windows Server 2008 R2 S1 及更高版本上，而 `cmd.exe` 相当麻烦，所以PowerShell 被推荐给大多数 Windows 用户)

### 安装包

| 操作系统                                     | 包管理器           | 安装包                                          | 命令                                                                                 |
| ---------------------------------------------------- | ------------------------- | ------------------------------------------------ | --------------------------------------------------------------------------------------- |
| [Various][rust-platforms]                            | [Cargo][cargo]            | [just][just-crate]                               | `cargo install just`                                                                    |
| [Microsoft Windows][windows]                         | [Scoop][scoop]            | [just][just-scoop]                               | `scoop install just`                                                                    |
| [Various][homebrew-install]                          | [Homebrew][homebrew]      | [just][just-homebrew]                            | `brew install just`                                                                     |
| [macOS][macos]                                       | [MacPorts][macports]      | [just][just-macports]                            | `port install just`                                                                     |
| [Arch Linux][arch linux]                             | [pacman][pacman]          | [just][just-pacman]                              | `pacman -S just`                                                                        |
| [Various][nix-platforms]                             | [Nix][nix]                | [just][just-nixpkg]                              | `nix-env -iA nixpkgs.just`                                                              |
| [NixOS][nixos]                                       | [Nix][nix]                | [just][just-nixpkg]                              | `nix-env -iA nixos.just`                                                                |
| [Solus][solus]                                       | [eopkg][solus-eopkg]      | [just][just-solus]                               | `eopkg install just`                                                                    |
| [Void Linux][void linux]                             | [XBPS][xbps]              | [just][just-void]                                | `xbps-install -S just`                                                                  |
| [FreeBSD][freebsd]                                   | [pkg][freebsd-pkg]        | [just][just-freebsd]                             | `pkg install just`                                                                      |
| [Alpine Linux][alpine linux]                         | [apk-tools][apk-tools]    | [just][just-alpine]                              | `apk add just`                                                                          |
| [Fedora Linux][fedora linux]                         | [DNF][dnf]                | [just][just-fedora]                              | `dnf install just`                                                                      |
| [Gentoo Linux][gentoo linux]                         | [Portage][gentoo-portage] | [dm9pZCAq overlay: sys-devel/just][just-portage] | `eselect repository enable dm9pZCAq && emerge --sync dm9pZCAq && emerge sys-devel/just` |
| [Various][conda-platforms]                           | [Conda][conda]            | [just][just-conda]                               | `conda install -c conda-forge just`                                                     |
| [Microsoft Windows][windows]                         | [Chocolatey][chocolatey]  | [just][just-chocolatey]                          | `choco install just`                                                                    |

[rust-platforms]: https://forge.rust-lang.org/release/platform-support.html
[cargo]: https://www.rust-lang.org
[just-crate]: https://crates.io/crates/just
[windows]: https://en.wikipedia.org/wiki/Microsoft_Windows
[scoop]: https://scoop.sh
[just-scoop]: https://github.com/ScoopInstaller/Main/blob/master/bucket/just.json
[homebrew-install]: https://docs.brew.sh/Installation
[homebrew]: https://brew.sh
[just-homebrew]: https://formulae.brew.sh/formula/just
[macos]: https://en.wikipedia.org/wiki/MacOS
[macports]: https://www.macports.org
[just-macports]: https://ports.macports.org/port/just/summary
[arch linux]: https://www.archlinux.org
[nix-platforms]: https://nixos.org/download.html#download-nix
[pacman]: https://wiki.archlinux.org/title/Pacman
[just-pacman]: https://archlinux.org/packages/community/x86_64/just/
[nixos]: https://nixos.org/nixos/
[nix-plat]: https://nixos.org/nix/manual/#ch-supported-platforms
[nix]: https://nixos.org/nix/
[just-nixpkg]: https://github.com/NixOS/nixpkgs/blob/master/pkgs/development/tools/just/default.nix
[solus]: https://getsol.us/
[solus-eopkg]: https://getsol.us/articles/package-management/basics/en
[just-solus]: https://dev.getsol.us/source/just/
[void linux]: https://voidlinux.org
[xbps]: https://wiki.voidlinux.org/XBPS
[just-void]: https://github.com/void-linux/void-packages/blob/master/srcpkgs/just/template
[freebsd]: https://www.freebsd.org/
[freebsd-pkg]: https://www.freebsd.org/doc/handbook/pkgng-intro.html
[just-freebsd]: https://www.freshports.org/deskutils/just/
[alpine linux]: https://alpinelinux.org/
[apk-tools]: https://wiki.alpinelinux.org/wiki/Alpine_Linux_package_management
[just-alpine]: https://pkgs.alpinelinux.org/package/edge/community/x86_64/just
[fedora linux]: https://getfedora.org/
[dnf]: https://dnf.readthedocs.io/en/latest/
[just-fedora]: https://src.fedoraproject.org/rpms/rust-just
[gentoo linux]: https://www.gentoo.org/
[gentoo-portage]: https://wiki.gentoo.org/wiki/Portage
[just-portage]: https://github.com/gentoo-mirror/dm9pZCAq/tree/master/sys-devel/just
[conda-platforms]: https://docs.conda.io/en/latest/miniconda.html#system-requirements
[conda]: https://docs.conda.io/projects/conda/en/latest/index.html
[just-conda]: https://anaconda.org/conda-forge/just
[chocolatey]: https://chocolatey.org
[just-chocolatey]: https://github.com/michidk/just-choco

![package version table](https://repology.org/badge/vertical-allrepos/just.svg)

### 预制二进制文件

Linux、MacOS 和 Windows 的预制二进制文件可以在 [发布页](https://github.com/casey/just/releases) 上找到。

你也可以在 Linux、MacOS 或 Windows 上使用下面的命令来下载最新的版本，只需将 `DEST` 替换为你想存储 `just` 的目录：

```sh
curl --proto '=https' --tlsv1.2 -sSf https://just.systems/install.sh | bash -s -- --to DEST
```

例如，安装 `just` 到 `~/bin` 目录：

```sh
# 创建 ~/bin
mkdir -p ~/bin

# 下载并解压 just 到 ~/bin/just
curl --proto '=https' --tlsv1.2 -sSf https://just.systems/install.sh | bash -s -- --to ~/bin

# 在 Shell 搜索可执行文件的路径中添加`~/bin`
# 这一行应该被添加到你的 Shell 初始化文件中，e.g. `~/.bashrc` 或者 `~/.zshrc`：
export PATH="$PATH:$HOME/bin"

# 现在 just 应该就可以执行了
just --help
```

### GitHub Actions

[extractions/setup-just](https://github.com/extractions/setup-just) 可以用来在 GitHub Actions 的工作流程中安装 `just`。

使用举例：

```yaml
- uses: extractions/setup-just@v1
  with:
    just-version: 0.8 # optional semver specification, otherwise latest
```

### 发布 RSS 订阅

`just` 的发布 [RSS 订阅](https://en.wikipedia.org/wiki/RSS) 可以在 [这里](https://github.com/casey/just/releases.atom) 找到。

向后兼容性
-----------------------

随着1.0版本的发布，`just` 突出对向后兼容性和稳定性的强烈承诺。

未来的版本将不会引入向后不兼容的变化，不会使现有的 `justfile` 停止工作，或破坏命令行界面的正常调用。

然而，这并不排除修复全面的错误，即使这样做可能会破坏依赖其行为的 `justfiles`。

永远不会有一个 `just` 2.0。任何理想的向后兼容的变化都是在每个 `justfile` 的基础上选择性加入的，所以用户可以在他们的闲暇时间进行迁移。

还没有准备好稳定化的功能将在 `--unstable` 标志后被选择性启用。由`--unstable`启用的功能可能会在任何时候以不兼容的方式发生变化。

编辑器支持
--------------

`justfile` 的语法与 `make` 非常接近，你可以让你的编辑器对 `just` 使用 `make` 语法高亮。

### Vim 和 Neovim

#### `vim-just`

[vim-just](https://github.com/NoahTheDuke/vim-just) 插件可以为 vim 提供 `justfile` 语法高亮显示。

你可以用你喜欢的软件包管理器安装它，如 [Plug](https://github.com/junegunn/vim-plug)：

```vim
call plug#begin()

Plug 'NoahTheDuke/vim-just'

call plug#end()
```

或者使用 Vim 的内置包支持：

```sh
mkdir -p ~/.vim/pack/vendor/start
cd ~/.vim/pack/vendor/start
git clone https://github.com/NoahTheDuke/vim-just.git
```

`vim-just` 也可以从 [vim-polyglot](https://github.com/sheerun/vim-polyglot) 获得，这是一个多语言的 Vim 插件。

#### `tree-sitter-just`

[tree-sitter-just](https://github.com/IndianBoy42/tree-sitter-just) 是一个针对 Neovim 的 [Nvim Treesitter](https://github.com/nvim-treesitter/nvim-treesitter) 插件。

#### Makefile 语法高亮

Vim 内置的 makefile 语法高亮对 `justfile` 来说并不完美，但总比没有好。你可以把以下内容放在 `~/.vim/filetype.vim` 中：

```vimscript
if exists("did_load_filetypes")
  finish
endif

augroup filetypedetect
  au BufNewFile,BufRead justfile setf make
augroup END
```
或者在单个 `justfile` 中添加以下内容，以在每个文件的基础上启用 `make` 模式：

```text
# vim: set ft=make :
```

### Emacs

[just-mode](https://github.com/leon-barrett/just-mode.el) 可以为 `justfile` 提供语法高亮和自动缩进。它可以在 [MELPA](https://melpa.org/) 上通过 [just-mode](https://melpa.org/#/just-mode) 获得。

[justl](https://github.com/psibi/justl.el) 提供了执行和列出配方的命令。

你可以在一个单独的 `justfile` 中添加以下内容，以便对每个文件启用 `make` 模式：

```text
# Local Variables:
# mode: makefile
# End:
```

### Visual Studio Code

由 [skellock](https://github.com/skellock) 为 VS Code 提供的扩展 [可在此获得](https://marketplace.visualstudio.com/items?itemName=skellock.just)（[仓库](https://github.com/skellock/vscode-just)）。

你可以通过运行以下命令来安装它：

```sh
code --install-extension skellock.just
```

### Kakoune

Kakoune 已经内置支持 `justfile` 语法高亮，这要感谢 TeddyDD。

### Sublime Text

由 TonioGela 编写的 Sublime Text 的语法高亮文件在 [extras/just.sublim-syntax](extras/just.sublim-syntax) 中提供。

### 其它编辑器

欢迎给我发送必要的命令，以便在你选择的编辑器中实现语法高亮，这样我就可以把它们放在这里。

快速开始
-----------

参见 [安装部分](#安装) 了解如何在你的电脑上安装 `just`。试着运行 `just --version` 以确保它被正确安装。

关于语法的概述，请查看这个 [速查表](https://cheatography.com/linux-china/cheat-sheets/justfile/)。

一旦 `just` 安装完毕并开始工作，在你的项目根目录创建一个名为 `justfile` 的文件，内容如下：

```make
recipe-name:
  echo 'This is a recipe!'

# 这是一行注释
another-recipe:
  @echo 'This is another recipe.'
```

当你调用 `just` 时，它会在当前目录和父目录寻找文件 `justfile`，所以你可以从你项目的任何子目录中调用它。

搜索 `justfile` 是不分大小写的，所以任何大小写，如 `Justfile`、`JUSTFILE` 或 `JuStFiLe` 都可以工作。`just` 也会寻找名字为 `.justfile` 的文件，以便你打算隐藏一个 `justfile`。

运行 `just` 时未传参数，则运行 `justfile` 中的第一个配方：

```sh
$ just
echo 'This is a recipe!'
This is a recipe!
```

通过一个或多个参数指定要运行的配方：

```sh
$ just another-recipe
This is another recipe.
```

`just` 在运行每条命令前都会将其打印到标准错误中，这就是为什么 `echo 'This is a recipe!'` 被打印出来。对于以 `@` 开头的行，这将被抑制，这就是为什么 `echo 'This is another recipe.'` 没有被打印。

如果一个命令失败，配方就会停止运行。这里 `cargo publish` 只有在 `cargo test` 成功后才会运行：

```make
publish:
  cargo test
  # 前面的测试通过才会执行 publish!
  cargo publish
```

配方可以依赖其他配方。在这里，`test` 配方依赖于 `build` 配方，所以 `build` 将在 `test` 之前运行：

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

没有依赖关系的配方将按照命令行上给出的顺序运行：

```sh
$ just build sloc
cc main.c foo.c bar.c -o main
1337 lines of code
```

依赖项总是先运行，即使它们被放在依赖它们的配方之后：

```sh
$ just test build
cc main.c foo.c bar.c -o main
./test
testing… all tests passed!
```

示例
--------

在 [Examples 目录](examples) 中可以找到各种 `justfile` 的例子。

特性介绍
--------

### 默认配方

当 `just` 被调用而没有传入任何配方时，它会运行 `justfile` 中的第一个配方。这个配方可能是项目中最常运行的命令，比如运行测试：

```make
test:
  cargo test
```

你也可以使用依赖关系来默认运行多个配方：

```make
default: lint build test

build:
  echo Building…

test:
  echo Testing…

lint:
  echo Linting…
```

在没有合适配方作为默认配方的情况下，你也可以在 `justfile` 的开头添加一个配方，用于列出可用的配方：

```make
default:
  just --list
```

### 列出可用的配方

可以用 `just --list` 按字母顺序列出配方：

```sh
$ just --list
Available recipes:
    build
    test
    deploy
    lint
```

`just --summary` 以更简洁的形式列出配方：

```sh
$ just --summary
build test deploy lint
```

传入 `--unsorted` 选项可以按照它们在 `justfile` 中出现的顺序打印配方：

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

如果你想让 `just` 默认列出 `justfile` 中的配方，你可以使用这个作为默认配方：

```make
default:
  @just --list
```

标题文本可以用 `--list-heading` 来定制：

```sh
$ just --list --list-heading $'Cool stuff…\n'
Cool stuff…
    test
    build
```

而缩进可以用 `--list-prefix` 来定制：

```sh
$ just --list --list-prefix ····
Available recipes:
····test
····build
```

`--list-heading` 参数同时替换了标题和后面的换行，所以如果不是空的，应该包含一个换行。这样做是为了允许你通过传递空字符串来完全抑制标题行：

```sh
$ just --list --list-heading ''
    test
    build
```

### 别名

别名允许你用其他名称来调用配方：

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

### 设置

设置控制解释和执行。每个设置最多可以指定一次，可以出现在 `justfile` 的任何地方。

例如：

```make
set shell := ["zsh", "-cu"]

foo:
  # this line will be run as `zsh -cu 'ls **/*.txt'`
  ls **/*.txt
```

#### 设置一览表

| 名称                      | 值                 | 描述                                                   |
| ------------------------- | ------------------ | ------------------------------------------------------ |
| `allow-duplicate-recipes` | boolean            | 允许在 `justfile` 后面出现的配方覆盖之前的同名配方     |
| `dotenv-load`             | boolean            | 加载 `.env` 环境变量文件, 如果有的话                   |
| `export`                  | boolean            | 将所有变量导出为环境变量                               |
| `positional-arguments`    | boolean            | 传递位置参数                                           |
| `shell`                   | `[COMMAND, ARGS…]` | 设置用于调用配方和评估反引号内包裹内容的命令           |
| `windows-powershell`      | boolean            | 在 Windows 上使用 PowerShell 作为默认 Shell            |

Bool 类型设置可以写成：

```mf
set NAME
```

这就相当于：

```mf
set NAME := true
```

#### 允许重复的配方

如果 `allow-duplicate-recipes` 被设置为 `true`，那么定义多个同名的配方就不会出错，而会使用最后的定义。默认为 `false`。

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

#### 环境变量加载

如果将 `dotenv-load` 设置为 `true`，并且存在 `.env` 文件，则该环境配置文件将被加载。默认为 `false`。

#### 导出

`export` 设置使所有 `just` 变量作为环境变量被导出。默认值为 `false`。

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

#### 位置参数

如果 `positional-arguments` 为 `true`，配方参数将作为位置参数传递给命令。对于行式配方，参数 `$0` 将是配方的名称。

例如，运行这个配方：

```make
set positional-arguments

@foo bar:
  echo $0
  echo $1
```

将产生以下输出：

```sh
$ just foo hello
foo
hello
```

当使用 `sh` 兼容的 Shell，如 `bash` 或 `zsh` 时，`$@` 会展开为传给配方的位置参数，从1开始。当在双引号内使用 `"$@"` 时，包括空白的参数将被传递，就像它们是双引号一样。也就是说，`"$@"` 相当于 `"$1" "$2"`......当没有位置参数时，`"$@"` 和 `$@` 将展开为空（即，它们被删除）。

这个例子的配方将逐行打印参数：

```make
set positional-arguments

@test *args='':
  bash -c 'while (( "$#" )); do echo - $1; shift; done' -- "$@"
```

用 _两个_ 参数运行：

```sh
$ just test foo "bar baz"
- foo
- bar baz
```

#### Shell

`shell` 设置控制用于调用执行配方代码行和反引号内指令的命令。Shebang 配方不受影响。

```make
# use python3 to execute recipe lines and backticks
set shell := ["python3", "-c"]

# use print to capture result of evaluation
foos := `print("foo" * 4)`

foo:
  print("Snake snake snake snake.")
  print("{{foos}}")
```

`just` 把要执行的命令作为一个参数进行传递。许多 Shell 需要一个额外的标志，通常是 `-c`，以使它们评估执行第一个参数。

##### Windows PowerShell

`just` 在Windows上默认使用 `sh`。要使用 PowerShell，请将 `windows-powershell` 设置为 `true`。

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

### 文档注释

紧接着配方前面的注释将出现在 `just --list` 中：

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

### `.env` 集成

如果 [`dotenv-load`](#dotenv-load) 被设置，`just` 将从一个名为 `.env` 的文件中加载环境变量。这个文件可以和你的 `justfile` 位于同一目录下，或者位于其父目录下。这些变量是环境变量，而不是 `just` 的变量，因此必须使用 `$VARIABLE_NAME` 在配方和反引号中访问。

例如，假如你的 `.env` 文件包含：

```sh
# 注释，将被忽略
DATABASE_ADDRESS=localhost:6379
SERVER_PORT=1337
```

而你的 `justfile` 包含：

```make
set dotenv-load

serve:
  @echo "Starting server with database $DATABASE_ADDRESS on port $SERVER_PORT…"
  ./server --database $DATABASE_ADDRESS --port $SERVER_PORT
```

`just serve` 将会输出：

```sh
$ just serve
Starting server with database localhost:6379 on port 1337…
./server --database $DATABASE_ADDRESS --port $SERVER_PORT
```

### 变量和替换

支持在变量、字符串、拼接和替换中使用 `{{…}}` ：

```make
version := "0.2.7"
tardir  := "awesomesauce-" + version
tarball := tardir + ".tar.gz"

publish:
  rm -f {{tarball}}
  mkdir {{tardir}}
  cp README.md *.c {{tardir}}
  tar zcvf {{tarball}} {{tardir}}
  scp {{tarball}} me@server.com:release/
  rm -rf {{tarball}} {{tardir}}
```

#### 转义 `{{`

想要写一个包含  `{{` 的配方，可以使用 `{{{{`：

```make
braces:
  echo 'I {{{{LOVE}} curly braces!'
```

(未匹配的 `}}` 会被忽略，所以不需要转义)

另一个选择是把所有你想转义的文本都放在插值里面：

```make
braces:
  echo '{{'I {{LOVE}} curly braces!'}}'
```

然而，另一个选择是使用  `{{ "{{" }}`：

```make
braces:
  echo 'I {{ "{{" }}LOVE}} curly braces!'
```

### 字符串

双引号字符串支持转义序列：

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

字符串可以包含换行符：

```make
single := '
hello
'

double := "
goodbye
"
```

单引号字符串不支持转义序列：

```make
escapes := '\t\n\r\"\\'
```

```sh
$ just --evaluate
escapes := "\t\n\r\"\\"
```

支持单引号和双引号字符串的缩进版本，以三个单引号或三个双引号为界。缩进的字符串行被删除了所有非空行所共有的前导空白：

```make
# 这个字符串执行结果为 `foo\nbar\n`
x := '''
  foo
  bar
'''

# 这个字符串执行结果为 `abc\n  wuv\nbar\n`
y := """
  abc
    wuv
  xyz
"""
```

与未缩进的字符串类似，缩进的双引号字符串处理转义序列，而缩进的单引号字符串则忽略转义序列。转义序列的处理是在取消缩进后进行的。取消缩进的算法不考虑转义序列产生的空白或换行。

### 错误忽略

通常情况下，如果一个命令返回一个非零的退出状态，将停止执行。要想在一个命令之后继续执行，即使它失败了，需要在命令前加上 `-`：

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

### 函数

`just` 提供了一些内置函数，在编写配方时可能很有用。

#### 系统信息

- `arch()` — 指令集结构。可能的值是：`"aarch64"`, `"arm"`, `"asmjs"`, `"hexagon"`, `"mips"`, `"msp430"`, `"powerpc"`, `"powerpc64"`, `"s390x"`, `"sparc"`, `"wasm32"`, `"x86"`, `"x86_64"`, 和 `"xcore"`。

- `os()` — 操作系统，可能的值是: `"android"`, `"bitrig"`, `"dragonfly"`, `"emscripten"`, `"freebsd"`, `"haiku"`, `"ios"`, `"linux"`, `"macos"`, `"netbsd"`, `"openbsd"`, `"solaris"`, 和 `"windows"`。

- `os_family()` — 操作系统系列；可能的值是：`"unix"` 和 `"windows"`。

例如：

```make
system-info:
  @echo "This is an {{arch()}} machine".
```

```sh
$ just system-info
This is an x86_64 machine
```

`os_family()` 函数可以用来创建跨平台的 `justfile`，使其可以在不同的操作系统上工作。一个例子，见 [cross-platform.just](examples/cross-platform.just) 文件。

#### 环境变量

- `env_var(key)` — 检索名称为 `key` 的环境变量，如果不存在则终止。

```make
home_dir := env_var('HOME')

test:
  echo "{{home_dir}}"
```

```sh
$ just
/home/user1
```

- `env_var_or_default(key, default)` — 检索名称为 `key` 的环境变量，如果不存在则返回 `default`。
