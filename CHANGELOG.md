Changelog
=========

[v0.8.5] - 2020-3-24
--------------------

### Added
- Allow escaping double braces with `{{{{` (#765)

### Misc
- Reorganize readme to highlight editor support (#764)
- Add categories and keywords to Cargo manifest (#763)
- Fix command output in readme (#760)
- Note Emacs package `just-mode` in readme (#759)
- Note shebang line splitting inconsistency in readme (#757)


[v0.8.4] - 2020-2-9
-------------------

### Added
- Add options to control list formatting (#753)

### Misc
- Document how to change the working directory in a recipe (#752)
- Implement `Default` for `Table` (#748)
- Add Alpine Linux package to readme (#736)
- Update to actions/cache@v2 (#742)
- Add link in readme to GitHub Action (#729)
- Add docs for justfile() and justfile_directory() (#726)
- Fix CI (#727)
- Improve readme (#725)
- Replace saythanks.io link with malto: link (#723)
- Update man page to v0.8.3 (#720)


[v0.8.3] - 2020-10-27
---------------------

### Added
- Allow ignoring line endings inside delimiters (#717)


[v0.8.2] - 2020-10-26
---------------------

### Added
- Add conditional expressions (#714)

### Fixed
- Allow completing variables and recipes after `--set` in zsh completion script (#697)

### Misc
- Add Parser::forbid (#712)
- Automatically track expected tokens while parsing (#711)
- Document feature flags in Cargo.toml (#709)


[v0.8.1] - 2020-10-15
---------------------

### Changed
- Allow choosing multiple recipes to run (#700)
- Complete recipes in bash completion script (#685)
- Complete recipes names in PowerShell completion script (#651)

### Misc
- Add FreeBSD port to readme (#705)
- Placate clippy (#698)
- Fix build fix (#693)
- Fix readme documentation for ignoring errors (#692)


[v0.8.0] - 2020-10-3
--------------------

### Breaking
- Allow suppressing failures with `-` prefix (#687)

### Misc
- Document how to ignore errors with `-` in readme (#690)
- Install BSD Tar on GitHub Actions to fix CI errors (#689)
- Move separate quiet config value to verbosity (#686)


[v0.7.3] - 2020-9-17
--------------------

### Added
- Add the `--choose` subcommand (#680)

### Misc
- Combine integration tests into single binary (#679)
- Document `--unsorted` flag in readme (#672)


[v0.7.2] - 2020-8-23
--------------------

### Added
- Add option to print recipes in source order (#669)

### Misc
- Mention Linux, MacOS and Windows support in readme (#666)
- Add list highlighting nice features to readme (#664)


[v0.7.1] - 2020-7-19
--------------------

### Fixed
- Search for `.env` file from working directory (#661)

### Misc
- Move link-time optimization config into `Cargo.toml` (#658)


[v0.7.0] - 2020-7-16
--------------------

### Breaking
- Skip `.env` items which are set in environment (#656)

### Misc
- Mark tags that start with `v` as releases (#654)


[v0.6.1] - 2020-6-28
--------------------

### Changed
- Only use `cygpath` on shebang if it contains `/` (#652)


[v0.6.0] - 2020-6-18
--------------------

### Changed
- Ignore '@' returned from interpolation evaluation (#636)
- Strip leading spaces after line continuation (#635)

### Added
- Add variadic parameters that accept zero or more arguments (#645)

### Misc
- Clarify variadic parameter default values (#646)
- Add keybase example justfile  (#640)
- Strip trailing whitespace in `examples/pre-commit.just` (#644)
- Test that example justfiles successfully parse (#643)
- Link example justfiles in readme (#641)
- Add example justfile (#639)
- Document how to run recipes after another recipe (#630)


[v0.5.11] - 2020-5-23
---------------------

### Added
- Don't load `.env` file when `--no-dotenv` is passed (#627)

### Changed
- Complete recipe names in fish completion script (#625)
- Suggest aliases for unknown recipes (#624)


[v0.5.10] - 2020-3-18
---------------------


[v0.5.9] - 2020-3-18
--------------------

### Added
- Update zsh completion file (#606)
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


[v0.5.8] - 2020-1-28
--------------------

## Changed
- Only use `cygpath` on windows if present (#586)

## Misc
- Improve comments in justfile (#588)
- Remove unused dependencies (#587)


[v0.5.7] - 2020-1-28
--------------------

## Misc
- Don't include directories in release archive (#583)


[v0.5.6] - 2020-1-28
--------------------

## Misc
- Build and upload release artifacts from GitHub Actions (#581)
- List solus package in readme (#579)
- Expand use of Github Actions (#580)
- Fix readme typo: interpetation -> interpretation (#578)


[v0.5.5] - 2020-1-15
--------------------

## Added
- Generate shell completion scripts with `--completions` (#572)

## Misc
- Check long lines and FIXME/TODO on CI (#575)
- Add additional continuous integration checks (#574)


[v0.5.4] - 2019-12-25
---------------------

# Added
- Add `justfile_directory()` and `justfile()` (#569)

## Misc
- Add table of package managers that include just to readme (#568)
- Remove yaourt AUR helper from readme (#567)
- Fix regression in error message color printing (#566)
- Reform indentation handling (#565)
- Update Cargo.lock with new version (#564)


[v0.5.3] - 2019-12-11
---------------------

## Misc
- Assert that lexer advances over entire input (#560)
- Fix typo: `chracter` -> `character` (#561)
- Improve pre-publish check (#562)


[v0.5.2] - 2019-12-7
--------------------

## Added
- Add flags to set and clear shell arguments (#551)
- Allow passing arguments to dependencies (#555)

## Misc
- Un-implement Deref for Table (#546)
- Resolve recipe dependencies (#547)
- Resolve alias targets (#548)
- Remove unnecessary type argument to Alias (#549)
- Resolve functions (#550)
- Reform scope and binding (#556)


[v0.5.1] - 2019-11-20
---------------------

## Added
- Add `--init` subcommand (#541)

## Changed
- Avoid fs::canonicalize (#539)

## Misc
- Mention `set shell` as altenative to installing `sh` (#533)
- Refactor Compilation error to contain a Token (#535)
- Move lexer comment (#536)
- Add missing `--init` test (#543)


[v0.5.0] - 2019-11-12
---------------------

## Added

- Add `set shell := [...]` to grammar (#526)
- Add `shell` setting (#525)
- Document settings in readme (#527)

## Changed
- Reform positional argument parsing (#523)
- Highlight echoed recipe lines in bold by default (#512)

## Misc

- Gargantuan refactor (#522)
- Move subcommand execution into Subcommand (#514)
- Move `cd` out of Config::from_matches (#513)
- Remove now-unnecessary borrow checker appeasement (#511)
- Reform Parser (#509)
- Note need to publish with nightly cargo (#506)


[v0.4.5] - 2019-10-31
---------------------

## User-visible

### Changed
- Display alias with `--show NAME` if one exists

### Documented
- Document multi-line constructs (for/if/while) (#453)
- Generate man page with help2man (#463)
- Add context to deprecation warnings (#473)
- Improve messages for alias error messages (#500)

## Misc

### Cleanup
- Update deprecated rust range patterns and clippy config (#450)
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
- Add repository attribute to Cargo.toml (#493)
- Check minimal version compatibility before publishing (#487)

### Continuous Integration
- Disable FreeBSD builds (#474)
- Use `bash` as shell for all integration tests (#479)
- Don't install `dash` on Travis (#482)

### Dependencies
- Use `tempfile` crate instead of `tempdir` (#455)
- Bump clap dependency to 2.33.0 (#458)
- Minimize dependency version requirements (#461)
- Remove dependency on brev (#462)
- Update dependencies (#501)


[v0.4.4] - 2019-06-02
---------------------

### Changed
- Ignore file name case while searching for justfile (#436)

### Added
- Display alias target with `--show` (#443)


[v0.4.3] - 2019-05-07
---------------------

### Changed
- Deprecate `=` in assignments, aliases, and exports in favor of `:=` (#413)

### Added
- Pass stdin handle to backtick process (#409)

### Documented
- Fix readme command line (#411)
- Typo: "command equivelant" -> "command equivalent" (#418)
- Mention Make’s “phony target” workaround in the comparison (#421)
- Add Void Linux install instructions to readme (#423)

### Cleaned up or Refactored
- Remove stray source files (#408)
- Replace some calls to brev crate (#410)
- Lexer code deduplication and refactoring (#414)
- Refactor and rename test macros (#415)
- Move CompilationErrorKind into separate module (#416)
- Remove `write_token_error_context` (#417)


[v0.4.2] - 2019-04-12
---------------------

### Changed
- Regex-based lexer replaced with much nicer character-at-a-time lexer (#406)


[v0.4.1] - 2019-04-12
---------------------

### Changed
- Make summary function non-generic (#404)


[v0.4.0] - 2019-04-12
---------------------

### Added
- Add recipe aliases by @ryloric (#390)
- Allow arbitrary expressions as default arguments (#400)
- Add justfile summaries (#399)
- Allow outer shebang lines so justfiles can be used as scripts (#393)
- Allow `--justfile` without `--working-directory` by @smonami (#392)
- Add link to Chinese translation of readme by @chinanf-boy (#377)

### Changed
- Upgrade to Rust 2018 (#394)
- Format the codebase with rustfmt (#346)


[v0.3.13] - 2018-11-06
----------------------

### Added
- Print recipe signature if missing arguments (#369)
- Add grandiloquent verbosity level that echos shebang recipes (#348)
- Wait for child processes to finish (#345)
- Improve invalid escape sequence error messages (#328)

### Fixed
- Use PutBackN instead of PutBack in parser (#364)


[v0.3.12] - 2018-06-19
----------------------

### Added
- Implemented invocation_directory function


[v0.3.11] - 2018-05-6
---------------------

### Fixed
- Fixed colors on windows (#317)


[v0.3.10] - 2018-3-19
---------------------

### Added
- Make .env vars available in env_var functions (#310)


[v0.3.8] - 2018-3-5
-------------------

## Added
- Add dotenv integration (#306)


[v0.3.7] - 2017-12-11
---------------------

### Fixed
- Fix error if ! appears in comment (#296)


[v0.3.6] - 2017-12-11
---------------------

### Fixed
- Lex CRLF line endings properly (#292)


[v0.3.5] - 2017-12-11
---------------------

### Added
- Align doc-comments in `--list` output (#273)
- Add `arch()`, `os()`, and `os_family()` functions (#277)
- Add `env_var(key)` and `env_var_or_default(key, default)` functions (#280)


[v0.3.4] - 2017-10-06
---------------------

### Added
- Do not evaluate backticks in assignments during dry runs (#253)

### Changed
- Change license to CC0 going forward (#270)


[v0.3.1] - 2017-10-06
---------------------

### Added
- Started keeping a changelog in CHANGELOG.md (#220)
- Recipes whose names begin with an underscore will not appear in `--list` or `--summary` (#229)
