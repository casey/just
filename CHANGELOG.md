Changelog
=========

[1.7.0](https://github.com/casey/just/releases/tag/1.7.0) - 2022-10-26
----------------------------------------------------------------------

### Breaking Changes to Unstable Features
- Make `fallback` setting default to true (#1384)

### Added
- Add more case-conversion functions (#1383 by @gVirtu)
- Add `tempdir` setting (#1369 by @dmatos2012)
- Add [no-exit-message] recipe annotation (#1354 by @gokhanettin)
- Add `capitalize(s)` function (#1375 by @femnad)

### Misc
- Credit contributors in changelog (#1385)
- Update asdf just plugin repository (#1380 by @kachick)
- Prepend commit messages with `- ` in changelog (#1379)
- Fail publish if `<sup>master</sup>` is found in README.md (#1378)
- Use for loop in capitalize implementation (#1377)

[1.6.0](https://github.com/casey/just/releases/tag/1.6.0) - 2022-10-19
----------------------------------------------------------------------

### Breaking Changes to Unstable Features
- Require `set fallback := true` to enable recipe fallback (#1368)

### Changed
- Allow fallback with search directory (#1348)

### Added
- Don't evaluate comments (#1358)
- Add skip-comments setting (#1333 by @neunenak)
- Allow bash completion to complete tasks in other directories (#1303 by @jpbochi)

### Misc
- Restore www/CNAME (#1364)
- Improve book config (#1363)
- Add kitchen sink justfile to test syntax highlighting (#1362 by @nk9)
- Note version in which absolute path construction was added (#1361)
- Inline setup and cleanup functions in completion script test (#1352)

[1.5.0](https://github.com/casey/just/releases/tag/1.5.0) - 2022-9-11
---------------------------------------------------------------------

### Changed
- Allow constructing absolute paths with `/` operator (#1320 by @erikkrieg)

### Misc
- Allow fewer lints (#1340)
- Fix issues reported by nightly clippy (#1336 by @neunenak)
- Refactor run.rs (#1335 by @neunenak)
- Allow comments on same line as settings (#1339)
- Fix justfile env shebang on Linux (#1330)
- Update Chinese translation of README.md (#1325 by @hustcer)
- Add additional settings to grammar
- Add an example of using a variable in a recipe parameter (#1311 by @papertigers)

[1.4.0](https://github.com/casey/just/releases/tag/1.4.0) - 2022-8-08
---------------------------------------------------------------------

### Fixed
- Fix shell setting precedence (#1306)

### Misc
- Don't hardcode homebrew prefix (#1295)
- Exclude files from cargo package (#1283)
- Add usage note to default list recipe (#1296 by @jpbochi)
- Add MPR/Prebuilt-MPR installation instructions to README.md (#1280 by @hwittenborn)
- Add make and makesure to readme (#1299)
- Document how to configure zsh completions on MacOS (#1285 by @nk9)
- Convert package table to HTML (#1291)

[1.3.0](https://github.com/casey/just/releases/tag/1.3.0) - 2022-7-25
---------------------------------------------------------------------

### Added
- Add `/` operator (#1237)

### Fixed
- Fix multibyte codepoint crash (#1243)

### Misc
- Update just-install reference on README.md (#1275 by @0xradical)
- Split Recipe::run into Recipe::{run_shebang,run_linewise} (#1270)
- Add asdf package to readme(#1264 by @jaacko-torus)
- Add mdbook deps for build-book recipe (#1258 by @) (#1259 by @TopherIsSwell)
- Fix typo: argumant -> argument (#1257 by @kianmeng)
- Improve error message if `if` is missing the `else` (#1252 by @nk9)
- Explain how to pass arguments of a command to a dependency (#1254 by @heavelock)
- Update Chinese translation of README.md (#1253 by @hustcer)
- Improvements to Sublime syntax file (#1250 by @nk9)
- Prevent unbounded recursion when parsing expressions (#1248 by @evanrichter)
- Publish to snap store (#1245)
- Restore fuzz test harness (#1246 by @evanrichter)
- Add just-install to README file (#1241 by @brombal)
- Fix dead readme link (#1240 by @wdroz)
- Do `use super::*;` instead of `use crate::common::*;` (#1239)
- Fix readme punctuation (#1235)
- Add argument splitting section to readme (#1230)
- Add notes about environment variables to readme (#1229)
- Fix book links (#1227)
- Add nushell README.md (#1224 by @hustcer)
- Use absolute links in readme (#1223)
- Copy changelog into manual (#1222)
- Translate Chinese manual introduction and title (#1220 by @hustcer)
- Build Chinese language user manual (#1219)
- Update Chinese translation of README.md (#1218 by @hustcer)
- Translate all of README.md into Chinese (#1217 by @hustcer)
- Translate all of features in README into Chinese (#1215 by @hustcer)
- Make link to examples directory absolute (#1213)
- Translate part of features in README into Chinese (#1211 by @hustcer)
- Add JetBrains IDE plugin to readme (#1209 by @linux-china)
- Translate features chapter of readme to Chinese (#1208 by @hustcer)

[1.2.0](https://github.com/casey/just/releases/tag/1.2.0) - 2022-5-31
---------------------------------------------------------------------

### Added
- Add `windows-shell` setting (#1198)
- SHA-256 and UUID functions (#1170 by @mbodmer)

### Misc
- Translate editor support and quick start to Chinese (#1206 by @hustcer)
- Translate first section of readme into Chinese (#1205 by @hustcer)
- Fix a bunch of typos (#1204)
- Remove cargo-limit usage from justfile (#1199)
- Add nix package manager install instructions (#1194 by @risingBirdSong)
- Fix broken link in readme (#1183 by @Vlad-Shcherbina)
- Add screenshot to manual (#1181)
- Style homepage (#1180)
- Center readme (#1178)
- Style and add links to homepage (#1177)
- Fix readme badge links (#1176)
- Generate book from readme (#1155)

[1.1.3](https://github.com/casey/just/releases/tag/1.1.3) - 2022-5-3
--------------------------------------------------------------------

### Fixed
- Skip duplicate recipe arguments (#1174)

### Misc
- Fix install script (#1172)
- Document that `invocation_directory()` returns an absolute path (#1162)
- Fix absolute_path documentation (#1160)
- Add cross-platform justfile example (#1152 by @presidento)

[1.1.2](https://github.com/casey/just/releases/tag/1.1.2) - 2022-3-30
---------------------------------------------------------------------

### Misc
- Document indentation rules (#1142)
- Remove stale link from readme (#1141)

### Unstable
- Search for missing recipes in parent directory justfiles (#1149)

[1.1.1](https://github.com/casey/just/releases/tag/1.1.1) - 2022-3-22
---------------------------------------------------------------------

### Misc
- Build MacOS ARM release binaries (#1138)
- Upgrade Windows Actions runners to windows-latest (#1137)

[1.1.0](https://github.com/casey/just/releases/tag/1.1.0) - 2022-3-10
---------------------------------------------------------------------

### Added
- Add `error()` function (#1118 by @chamons)
- Add `absolute_path` function (#1121 by @Laura7089)

[1.0.1](https://github.com/casey/just/releases/tag/1.0.1) - 2022-2-28
---------------------------------------------------------------------

### Fixed
- Make path_exists() relative to current directory (#1122)

### Misc
- Detail environment variable usage in readme (#1086 by @kenden)
- Format --init justfile (#1116 by @TheLocehiliosan)
- Add hint for Node.js script compatibility (#1113)

[1.0.0](https://github.com/casey/just/releases/tag/1.0.0) - 2022-2-22
---------------------------------------------------------------------

### Added
- Add path_exists() function (#1106 by @heavelock)

### Misc
- Note that `pipefail` isn't normally set (#1108)

[0.11.2](https://github.com/casey/just/releases/tag/0.11.2) - 2022-2-15
-----------------------------------------------------------------------

### Misc
- Fix dotenv-load documentation (#1104)
- Fixup broken release package script (#1100 by @lutostag)

[0.11.1](https://github.com/casey/just/releases/tag/0.11.1) - 2022-2-14
-----------------------------------------------------------------------

### Added
- Allow duplicate recipes (#1095 by @lutostag)

### Misc
- Add arrow pointing to table of contents button (#1096)
- Improve readme (#1093 by @halostatue)
- Remove asciidoc readme (#1092)
- Convert README.adoc to markdown (#1091)
- Add choco package to README (#1090 by @michidk)

[0.11.0](https://github.com/casey/just/releases/tag/0.11.0) - 2022-2-3
----------------------------------------------------------------------

### Breaking
- Change dotenv-load default to false (#1082)

[0.10.7](https://github.com/casey/just/releases/tag/0.10.7) - 2022-1-30
-----------------------------------------------------------------------

### Misc
- Don't run tests in release workflow (#1080)
- Fix windows chooser invocation error message test (#1079)
- Remove call to sed in justfile (#1078)

[0.10.6](https://github.com/casey/just/releases/tag/0.10.6) - 2022-1-29
-----------------------------------------------------------------------

### Added
- Add windows-powershell setting (#1057 by @michidk)

### Changed
- Allow using `-` and `@` in any order (#1063)

### Misc
- Use `Context` suffix for snafu error contexts (#1068)
- Upgrade snafu to 0.7 (#1067 by @shepmaster)
- Mention "$@" in the README (#1064 by @mpdude)
- Note how to use PowerShell with CLI in readme (#1056 by @michidk)
- Link to cheatsheet from readme (#1053)
- Link to Homebrew installation docs in readme (#1049 by @michidk)
- Workflow tweaks (#1045)
- Push to correct origin in publish recipe (#1044)

[0.10.5](https://github.com/casey/just/releases/tag/0.10.5) - 2021-12-4
-----------------------------------------------------------------------

### Changed
- Use musl libc for ARM binaries (#1037)

### Misc
- Make completions work with Bash alias (#1035 by @kurtbuilds)
- Run tests on PRs (#1040)
- Improve GitHub Actions workflow triggers (#1033)
- Publish from GitHub master branch instead of local master (#1032)

[0.10.4](https://github.com/casey/just/releases/tag/0.10.4) - 2021-11-21
------------------------------------------------------------------------

### Added
- Add `--dump-format json` (#992)
- Add `quote(s)` function for escaping strings (#1022)
- fmt: check formatting with `--check` (#1001 by @hdhoang)

### Misc
- Refactor github actions (#1028)
- Fix readme formatting (#1030 by @soenkehahn)
- Use ps1 extension for pwsh shebangs (#1027 by @dmringo)
- Ignore leading byte order mark in source files (#1021)
- Add color to `just --fmt --check` diff (#1015)

[0.10.3](https://github.com/casey/just/releases/tag/0.10.3) - 2021-10-30
------------------------------------------------------------------------

### Added
- Add `trim_end(s)` and `trim_start(s)` functions (#999)
- Add more string manipulation functions (#998)

### Changed
- Make `join` accept two or more arguments (#1000)

### Misc
- Add alternatives and prior art section to readme (#1008)
- Fix readme `make`'s not correctly displayed (#1007 by @peter50216)
- Document the default recipe (#1006)
- Document creating user justfile recipe aliases (#1005)
- Fix readme typo (#1004 by @0xflotus)
- Add packaging status table to readme (#1003)
- Reword `sh` not found error messages (#1002 by @hdhoang)
- Only pass +crt-static to cargo build (#997)
- Stop using tabs in justfile in editorconfig (#996)
- Use consistent rustflags formatting (#994)
- Use `cargo build` instead of `cargo rustc` (#993)
- Don't skip variables in variable iterator (#991)
- Remove deprecated equals error (#985)

[0.10.2](https://github.com/casey/just/releases/tag/0.10.2) - 2021-9-26
-----------------------------------------------------------------------

### Added
- Implement regular expression match conditionals (#970)

### Misc
- Add detailed instructions for installing prebuilt binaries (#978)
- Improve readme package table formatting (#977)
- Add conda package to README (#976 by @kellpossible)
- Change MSRV to 1.46.0 (#968)
- Use stable rustfmt instead of nightly (#967)
- Fix readme typo: FOO → WORLD (#964)
- Reword Emacs section in readme (#962)
- Mention justl mode for Emacs (#961 by @psibi)

[0.10.1](https://github.com/casey/just/releases/tag/0.10.1) - 2021-8-27
-----------------------------------------------------------------------

### Added
- Add flags for specifying name and path to environment file (#941 by @Celeo)

### Misc
- Fix error message tests for Alpine Linux (#956)
- Bump `target` version to 2.0 (#957)
- Mention `tree-sitter-just` in readme (#951)
- Document release RSS feed in readme (#950)
- Add installation instructions for Gentoo Linux (#946 by @dm9pZCAq)
- Make GitHub Actions instructions more prominent (#944)
- Wrap `--help` text to terminal width (#940)
- Add `.justfile` to sublime syntax file_extensions (#938)
- Suggest using `~/.global.justfile` instead of `~/.justfile` (#937)
- Update man page (#935)

[0.10.0](https://github.com/casey/just/releases/tag/0.10.0) - 2021-8-2
----------------------------------------------------------------------

### Changed
- Warn if `.env` file is loaded in `dotenv-load` isn't explicitly set (#925)

### Added
- Add `--changelog` subcommand (#932)
- Support `.justfile` as an alternative to `justfile` (#931)

### Misc
- Use cargo-limit for all recipes (#928)
- Fix colors (#927)
- Use ColorDisplay trait to print objects to the terminal (#926)
- Deduplicate recipe parsing (#923)
- Move subcommand functions into Subcommand (#918)
- Check GitHub Actions workflow with actionlint (#921)
- Add loader and refactor errors (#917)
- Rename: Module → Ast (#915)

[0.9.9](https://github.com/casey/just/releases/tag/0.9.9) - 2021-7-22
---------------------------------------------------------------------

### Added
- Add subsequent dependencies (#820)
- Implement `else if` chaining (#910)

### Fixed
- Fix circular variable dependency error message (#909)

### Misc
- Improve readme (#904 by @mtsknn)
- Add screenshot to readme (#911)
- Add install instructions for Fedora Linux (#898 by @olivierlemasle)
- Fix readme typos (#903 by @rokf)
- Actually fix release tagging and publish changelog with releases (#901)
- Fix broken prerelease tagging (#900)
- Use string value for ref-type check (#897)

[0.9.8](https://github.com/casey/just/releases/tag/0.9.8) - 2021-7-3
--------------------------------------------------------------------

### Misc
- Fix changelog formatting (#894)
- Only run install script on CI for non-releases (#895)

[0.9.7](https://github.com/casey/just/releases/tag/0.9.7) - 2021-7-3
--------------------------------------------------------------------

### Added
- Add string manipulation functions (#888 by @terror)

### Misc
- Remove test-utilities crate (#892)
- Remove outdated note in `Cargo.toml` (#891)
- Link to GitHub release pages in changelog (#886)

[0.9.6](https://github.com/casey/just/releases/tag/0.9.6) - 2021-6-24
---------------------------------------------------------------------

### Added
- Add `clean` function for simplifying paths (#883)
- Add `join` function for joining paths (#882)
- Add path manipulation functions (#872 by @TonioGela)

### Misc
- Add `file_extensions` to Sublime syntax file (#878 by @Frederick888)
- Document path manipulation functions in readme (#877)

[0.9.5](https://github.com/casey/just/releases/tag/0.9.5) - 2021-6-12
---------------------------------------------------------------------

### Added
- Add `--unstable` flag (#869)
- Add Sublime Text syntax file (#864)
- Add `--fmt` subcommand (#837 by @vglfr)

### Misc
- Mention doniogela.dev/just/ in readme (#866)
- Mention that vim-just is now available from vim-polyglot (#865)
- Mention `--list-heading` newline behavior (#860)
- Check for `rg` in `bin/forbid` (#859)
- Document that variables are not exported to backticks in the same scope (#856)
- Remove `dotenv_load` from tests (#853)
- Remove `v` prefix from version (#850)
- Improve install script (#847)
- Move pages assets back to `docs` (#846)
- Move pages assets to `www` (#845)

[0.9.4](https://github.com/casey/just/releases/tag/v0.9.4) - 2021-5-27
----------------------------------------------------------------------

### Misc
- Release `aarch64-unknown-linux-gnu` binaries (#843)
- Add `$` to non-default parameter grammar (#839)
- Add `$` to parameter grammar (#838 by @NoahTheDuke)
- Fix readme links (#836)
- Add `vim-just` installation instructions to readme (#835)
- Refactor shebang handling (#833)

[0.9.3](https://github.com/casey/just/releases/tag/v0.9.3) - 2021-5-16
----------------------------------------------------------------------

### Added
- Add shebang support for 'cmd.exe' (#828 by @pansila)
- Add `.exe` to powershell scripts (#826 by @sigoden)
- Add the `--command` subcommand (#824)

### Fixed
- Fix bang lexing and placate clippy (#821)

### Misc
- Fixed missing close apostrophe in GRAMMAR.md (#830 by @SOF3)
- Make 'else' keyword in grammar (#829 by @SOF3)
- Add forbid script (#827)
- Remove `summary` feature (#823)
- Document that just is now in Arch official repo (#814 by @svenstaro)
- Fix changelog years (#813)

[0.9.2](https://github.com/casey/just/releases/tag/v0.9.2) - 2021-5-02
----------------------------------------------------------------------

### Fixed
- Pass evaluated arguments as positional arguments (#810)

[0.9.1](https://github.com/casey/just/releases/tag/v0.9.1) - 2021-4-24
----------------------------------------------------------------------

### Added
- Change `--eval` to print variable value only (#806)
- Add `positional-arguments` setting (#804)
- Allow filtering variables to evaluate (#795)

### Changed
- Reform and improve string literals (#793)
- Allow evaluating justfiles with no recipes (#794)
- Unify string lexing (#790)

### Misc
- Test multi-line strings in interpolation (#789)
- Add shell setting examples to README (#787)
- Disable .env warning for now
- Warn if `.env` file loaded and `dotenv-load` unset (#784)

[0.9.0](https://github.com/casey/just/releases/tag/v0.9.0) - 2021-3-28
----------------------------------------------------------------------

### Changed
- Turn `=` deprecation warning into a hard error (#780)

[0.8.7](https://github.com/casey/just/releases/tag/v0.8.7) - 2021-3-28
----------------------------------------------------------------------

### Added
- Add `dotenv-load` setting (#778)

### Misc
- Change publish recipe to use stable rust (#777)

[0.8.6](https://github.com/casey/just/releases/tag/v0.8.6) - 2021-3-28
----------------------------------------------------------------------

### Added
- Add just_executable() function (#775 by @bew)
- Prefix parameters with `$` to export to environment (#773)
- Add `set export` to export all variables as environment variables (#767)

### Changed
- Suppress all output to stderr when `--quiet` (#771)

### Misc
- Improve chooser invocation error message (#772)
- De-emphasize cmd.exe in readme (#768)
- Fix warnings (#770)

[0.8.5](https://github.com/casey/just/releases/tag/v0.8.5) - 2021-3-24
----------------------------------------------------------------------

### Added
- Allow escaping double braces with `{{{{` (#765)

### Misc
- Reorganize readme to highlight editor support (#764)
- Add categories and keywords to Cargo manifest (#763)
- Fix command output in readme (#760 by @vvv)
- Note Emacs package `just-mode` in readme (#759 by @leon-barrett)
- Note shebang line splitting inconsistency in readme (#757)

[0.8.4](https://github.com/casey/just/releases/tag/v0.8.4) - 2021-2-9
---------------------------------------------------------------------

### Added
- Add options to control list formatting (#753)

### Misc
- Document how to change the working directory in a recipe (#752)
- Implement `Default` for `Table` (#748)
- Add Alpine Linux package to readme (#736 by @jirutka)
- Update to actions/cache@v2 (#742 by @zyctree)
- Add link in readme to GitHub Action (#729 by @rossmacarthur)
- Add docs for justfile() and justfile_directory() (#726 by @rminderhoud)
- Fix CI (#727)
- Improve readme (#725)
- Replace saythanks.io link with malto: link (#723)
- Update man page to v0.8.3 (#720)

[0.8.3](https://github.com/casey/just/releases/tag/v0.8.3) - 2020-10-27
-----------------------------------------------------------------------

### Added
- Allow ignoring line endings inside delimiters (#717)

[0.8.2](https://github.com/casey/just/releases/tag/v0.8.2) - 2020-10-26
-----------------------------------------------------------------------

### Added
- Add conditional expressions (#714)

### Fixed
- Allow completing variables and recipes after `--set` in zsh completion script (#697 by @heyrict)

### Misc
- Add Parser::forbid (#712)
- Automatically track expected tokens while parsing (#711)
- Document feature flags in Cargo.toml (#709)

[0.8.1](https://github.com/casey/just/releases/tag/v0.8.1) - 2020-10-15
-----------------------------------------------------------------------

### Changed
- Allow choosing multiple recipes to run (#700)
- Complete recipes in bash completion script (#685 by @vikesh-raj)
- Complete recipes names in PowerShell completion script (#651 by @Insomniak47)

### Misc
- Add FreeBSD port to readme (#705)
- Placate clippy (#698)
- Fix build fix (#693)
- Fix readme documentation for ignoring errors (#692 by @kenden)

[0.8.0](https://github.com/casey/just/releases/tag/v0.8.0) - 2020-10-3
----------------------------------------------------------------------

### Breaking
- Allow suppressing failures with `-` prefix (#687 by @iwillspeak)

### Misc
- Document how to ignore errors with `-` in readme (#690)
- Install BSD Tar on GitHub Actions to fix CI errors (#689)
- Move separate quiet config value to verbosity (#686 by @Celeo)

[0.7.3](https://github.com/casey/just/releases/tag/v0.7.3) - 2020-9-17
----------------------------------------------------------------------

### Added
- Add the `--choose` subcommand (#680)

### Misc
- Combine integration tests into single binary (#679)
- Document `--unsorted` flag in readme (#672)

[0.7.2](https://github.com/casey/just/releases/tag/v0.7.2) - 2020-8-23
----------------------------------------------------------------------

### Added
- Add option to print recipes in source order (#669)

### Misc
- Mention Linux, MacOS and Windows support in readme (#666)
- Add list highlighting nice features to readme (#664)

[0.7.1](https://github.com/casey/just/releases/tag/v0.7.1) - 2020-7-19
----------------------------------------------------------------------

### Fixed
- Search for `.env` file from working directory (#661)

### Misc
- Move link-time optimization config into `Cargo.toml` (#658)

[0.7.0](https://github.com/casey/just/releases/tag/v0.7.0) - 2020-7-16
----------------------------------------------------------------------

### Breaking
- Skip `.env` items which are set in environment (#656)

### Misc
- Mark tags that start with `v` as releases (#654)

[0.6.1](https://github.com/casey/just/releases/tag/v0.6.1) - 2020-6-28
----------------------------------------------------------------------

### Changed
- Only use `cygpath` on shebang if it contains `/` (#652)

[0.6.0](https://github.com/casey/just/releases/tag/v0.6.0) - 2020-6-18
----------------------------------------------------------------------

### Changed
- Ignore '@' returned from interpolation evaluation (#636 by @rjsberry)
- Strip leading spaces after line continuation (#635)

### Added
- Add variadic parameters that accept zero or more arguments (#645 by @rjsberry)

### Misc
- Clarify variadic parameter default values (#646 by @rjsberry)
- Add keybase example justfile  (#640 by @blaggacao)
- Strip trailing whitespace in `examples/pre-commit.just` (#644)
- Test that example justfiles successfully parse (#643)
- Link example justfiles in readme (#641)
- Add example justfile (#639 by @blaggacao)
- Document how to run recipes after another recipe (#630)

[0.5.11](https://github.com/casey/just/releases/tag/v0.5.11) - 2020-5-23
------------------------------------------------------------------------

### Added
- Don't load `.env` file when `--no-dotenv` is passed (#627)

### Changed
- Complete recipe names in fish completion script (#625 by @tyehle)
- Suggest aliases for unknown recipes (#624 by @Celeo)

[0.5.10](https://github.com/casey/just/releases/tag/v0.5.10) - 2020-3-18
------------------------------------------------------------------------

[0.5.9](https://github.com/casey/just/releases/tag/v0.5.9) - 2020-3-18
----------------------------------------------------------------------

### Added
- Update zsh completion file (#606 by @heyrict)
- Add `--variables` subcommand that prints variable names (#608)
- Add github pages site with improved install script (#597)

### Fixed
- Don't require justfile to print completions (#596)

### Misc
- Only build for linux on docs.rs (#611)
- Trim completions and ensure final newline (#609)
- Trigger build on pushes and pull requests (#607)
- Document behavior of `@` on shebang recipes (#602)
- Add `.nojekyll` file to github pages site (#599)
- Add `:` favicon (#598)
- Delete old CI configuration and update build badge (#595)
- Add download count badge to readme (#594)
- Wrap comments at 80 characters (#593)
- Use unstable rustfmt configuration options (#592)

[0.5.8](https://github.com/casey/just/releases/tag/v0.5.8) - 2020-1-28
----------------------------------------------------------------------

### Changed
- Only use `cygpath` on windows if present (#586)

### Misc
- Improve comments in justfile (#588)
- Remove unused dependencies (#587)

[0.5.7](https://github.com/casey/just/releases/tag/v0.5.7) - 2020-1-28
----------------------------------------------------------------------

### Misc
- Don't include directories in release archive (#583)

[0.5.6](https://github.com/casey/just/releases/tag/v0.5.6) - 2020-1-28
----------------------------------------------------------------------

### Misc
- Build and upload release artifacts from GitHub Actions (#581)
- List solus package in readme (#579)
- Expand use of Github Actions (#580)
- Fix readme typo: interpetation -> interpretation (#578 by @Plommonsorbet)

[0.5.5](https://github.com/casey/just/releases/tag/v0.5.5) - 2020-1-15
----------------------------------------------------------------------

### Added
- Generate shell completion scripts with `--completions` (#572)

### Misc
- Check long lines and FIXME/TODO on CI (#575)
- Add additional continuous integration checks (#574)

[0.5.4](https://github.com/casey/just/releases/tag/v0.5.4) - 2019-12-25
-----------------------------------------------------------------------

### Added
- Add `justfile_directory()` and `justfile()` (#569)

### Misc
- Add table of package managers that include just to readme (#568)
- Remove yaourt AUR helper from readme (#567 by @ky0n)
- Fix regression in error message color printing (#566)
- Reform indentation handling (#565)
- Update Cargo.lock with new version (#564)

[0.5.3](https://github.com/casey/just/releases/tag/v0.5.3) - 2019-12-11
-----------------------------------------------------------------------

### Misc
- Assert that lexer advances over entire input (#560)
- Fix typo: `chracter` -> `character` (#561)
- Improve pre-publish check (#562)

[0.5.2](https://github.com/casey/just/releases/tag/v0.5.2) - 2019-12-7
----------------------------------------------------------------------

### Added
- Add flags to set and clear shell arguments (#551)
- Allow passing arguments to dependencies (#555)

### Misc
- Un-implement Deref for Table (#546)
- Resolve recipe dependencies (#547)
- Resolve alias targets (#548)
- Remove unnecessary type argument to Alias (#549)
- Resolve functions (#550)
- Reform scope and binding (#556)

[0.5.1](https://github.com/casey/just/releases/tag/v0.5.1) - 2019-11-20
-----------------------------------------------------------------------

### Added
- Add `--init` subcommand (#541)

### Changed
- Avoid fs::canonicalize (#539)

### Misc
- Mention `set shell` as alternative to installing `sh` (#533)
- Refactor Compilation error to contain a Token (#535)
- Move lexer comment (#536)
- Add missing `--init` test (#543)

[0.5.0](https://github.com/casey/just/releases/tag/v0.5.0) - 2019-11-12
-----------------------------------------------------------------------

### Added

- Add `set shell := [...]` to grammar (#526)
- Add `shell` setting (#525)
- Document settings in readme (#527)

### Changed
- Reform positional argument parsing (#523)
- Highlight echoed recipe lines in bold by default (#512)

### Misc

- Gargantuan refactor (#522)
- Move subcommand execution into Subcommand (#514)
- Move `cd` out of Config::from_matches (#513)
- Remove now-unnecessary borrow checker appeasement (#511)
- Reform Parser (#509)
- Note need to publish with nightly cargo (#506)

[0.4.5](https://github.com/casey/just/releases/tag/v0.4.5) - 2019-10-31
-----------------------------------------------------------------------

### User-visible

### Changed
- Display alias with `--show NAME` if one exists

### Documented
- Document multi-line constructs (for/if/while) (#453)
- Generate man page with help2man (#463)
- Add context to deprecation warnings (#473)
- Improve messages for alias error messages (#500)

### Misc

### Cleanup
- Update deprecated rust range patterns and clippy config (#450 by @light4)
- Make comments in common.rs lowercase (#470)
- Use `pub(crate)` instead of `pub` (#471)
- Hide summary functionality behind feature flag (#472)
- Fix `summary` feature conditional compilation (#475)
- Allow integration test cases to omit common values (#480)
- Add `unindent()` for nicer integration test strings (#481)
- Start pulling argument parsing out of run::run() (#483)
- Add explicit `Subcommand` enum (#484)
- Avoid using error code `1` in integration tests (#486)
- Use more indented strings in integration tests (#489)
- Refactor `run::run` and Config (#490)
- Remove `misc.rs` (#491)
- Remove unused `use` statements (#497)
- Refactor lexer tests (#498)
- Use constants instead of literals in arg parser (#504)

### Infrastructure
- Add repository attribute to Cargo.toml (#493 by @SOF3)
- Check minimal version compatibility before publishing (#487)

### Continuous Integration
- Disable FreeBSD builds (#474)
- Use `bash` as shell for all integration tests (#479)
- Don't install `dash` on Travis (#482)

### Dependencies
- Use `tempfile` crate instead of `tempdir` (#455 by @NickeZ)
- Bump clap dependency to 2.33.0 (#458 by @NickeZ)
- Minimize dependency version requirements (#461)
- Remove dependency on brev (#462)
- Update dependencies (#501)

[0.4.4](https://github.com/casey/just/releases/tag/v0.4.4) - 2019-06-02
-----------------------------------------------------------------------

### Changed
- Ignore file name case while searching for justfile (#436 by @shevtsiv)

### Added
- Display alias target with `--show` (#443)

[0.4.3](https://github.com/casey/just/releases/tag/v0.4.3) - 2019-05-07
-----------------------------------------------------------------------

### Changed
- Deprecate `=` in assignments, aliases, and exports in favor of `:=` (#413)

### Added
- Pass stdin handle to backtick process (#409)

### Documented
- Fix readme command line (#411)
- Typo: "command equivelant" -> "command equivalent" (#418)
- Mention Make’s “phony target” workaround in the comparison (#421 by @roryokane)
- Add Void Linux install instructions to readme (#423)

### Cleaned up or Refactored
- Remove stray source files (#408)
- Replace some calls to brev crate (#410)
- Lexer code deduplication and refactoring (#414)
- Refactor and rename test macros (#415)
- Move CompilationErrorKind into separate module (#416)
- Remove `write_token_error_context` (#417)

[0.4.2](https://github.com/casey/just/releases/tag/v0.4.2) - 2019-04-12
-----------------------------------------------------------------------

### Changed
- Regex-based lexer replaced with much nicer character-at-a-time lexer (#406)

[0.4.1](https://github.com/casey/just/releases/tag/v0.4.1) - 2019-04-12
-----------------------------------------------------------------------

### Changed
- Make summary function non-generic (#404)

[0.4.0](https://github.com/casey/just/releases/tag/v0.4.0) - 2019-04-12
-----------------------------------------------------------------------

### Added
- Add recipe aliases (#390 by @ryloric)
- Allow arbitrary expressions as default arguments (#400)
- Add justfile summaries (#399)
- Allow outer shebang lines so justfiles can be used as scripts (#393)
- Allow `--justfile` without `--working-directory` (#392 by @smonami)
- Add link to Chinese translation of readme by @chinanf-boy (#377)

### Changed
- Upgrade to Rust 2018 (#394)
- Format the codebase with rustfmt (#346)

[0.3.13](https://github.com/casey/just/releases/tag/v0.3.13) - 2018-11-06
-------------------------------------------------------------------------

### Added
- Print recipe signature if missing arguments (#369 by @ladysamantha)
- Add grandiloquent verbosity level that echos shebang recipes (#348)
- Wait for child processes to finish (#345)
- Improve invalid escape sequence error messages (#328)

### Fixed
- Use PutBackN instead of PutBack in parser (#364)

[0.3.12](https://github.com/casey/just/releases/tag/v0.3.12) - 2018-06-19
-------------------------------------------------------------------------

### Added
- Implemented invocation_directory function

[0.3.11](https://github.com/casey/just/releases/tag/v0.3.11) - 2018-05-6
------------------------------------------------------------------------

### Fixed
- Fixed colors on windows (#317)

[0.3.10](https://github.com/casey/just/releases/tag/v0.3.10) - 2018-3-19
------------------------------------------------------------------------

### Added
- Make .env vars available in env_var functions (#310)

[0.3.8](https://github.com/casey/just/releases/tag/v0.3.8) - 2018-3-5
---------------------------------------------------------------------

### Added
- Add dotenv integration (#306)

[0.3.7](https://github.com/casey/just/releases/tag/v0.3.7) - 2017-12-11
-----------------------------------------------------------------------

### Fixed
- Fix error if ! appears in comment (#296)

[0.3.6](https://github.com/casey/just/releases/tag/v0.3.6) - 2017-12-11
-----------------------------------------------------------------------

### Fixed
- Lex CRLF line endings properly (#292)

[0.3.5](https://github.com/casey/just/releases/tag/v0.3.5) - 2017-12-11
-----------------------------------------------------------------------

### Added
- Align doc-comments in `--list` output (#273)
- Add `arch()`, `os()`, and `os_family()` functions (#277)
- Add `env_var(key)` and `env_var_or_default(key, default)` functions (#280)

[0.3.4](https://github.com/casey/just/releases/tag/v0.3.4) - 2017-10-06
-----------------------------------------------------------------------

### Added
- Do not evaluate backticks in assignments during dry runs (#253 by @aoeu)

### Changed
- Change license to CC0 going forward (#270)

[0.3.1](https://github.com/casey/just/releases/tag/v0.3.1) - 2017-10-06
-----------------------------------------------------------------------

### Added
- Started keeping a changelog in CHANGELOG.md (#220)
- Recipes whose names begin with an underscore will not appear in `--list` or `--summary` (#229)
