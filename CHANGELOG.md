Changelog
=========


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
