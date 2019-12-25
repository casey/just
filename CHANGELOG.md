Changelog
=========


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
