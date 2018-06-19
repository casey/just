# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/en/1.0.0/)
and this project adheres to [Semantic Versioning](http://semver.org/spec/v2.0.0.html).

## [0.3.12] - 2018-06-19
### Added
- Implemented invocation_directory function

## [0.3.11] - 2018-05-6
### Fixed
- Fixed colors on windows (#317)

## [0.3.10] - 2018-3-19
## Added
- Make .env vars available in env_var functions (#310)

## [0.3.8] - 2018-3-5
## Added
- Add dotenv integration (#306)

## [0.3.7] - 2017-12-11
### Fixed
- Fix error if ! appears in comment (#296)

## [0.3.6] - 2017-12-11
### Fixed
- Lex CRLF line endings properly (#292)

## [0.3.5] - 2017-12-11
### Added
- Align doc-comments in `--list` output (#273)
- Add `arch()`, `os()`, and `os_family()` functions (#277)
- Add `env_var(key)` and `env_var_or_default(key, default)` functions (#280)

## [0.3.4] - 2017-10-06
### Added
- Do not evaluate backticks in assignments during dry runs (#253)

### Changed
- Change license to CC0 going forward (#270)

## [0.3.1] - 2017-10-06
### Added
- Started keeping a changelog in CHANGELOG.md (#220)
- Recipes whose names begin with an underscore will not appear in `--list` or `--summary` (#229)
