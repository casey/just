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

本指南同时也以 [书](https://just.systems/man/) 的形式在线阅读；

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

如果你不愿意安装 `sh`，也可以使用 `shell` 设置来指定你选择的 Shell。

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

然而，这并不排除错误修复，即使这样做可能会破坏依赖其行为的 `justfiles`。

永远不会有一个 `just` 2.0。任何理想的向后兼容的变化都是在每个 `justfile` 的基础上选择性加入的，所以用户可以在他们的闲暇时间进行迁移。

还没有准备好稳定化的功能将在 `--unstable` 标志后被选择性启用。由`--unstable`启用的功能可能会在任何时候以不兼容的方式发生变化。
