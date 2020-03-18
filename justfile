#!/usr/bin/env just --justfile
# ^ A shebang isn't required, but allows a justfile to be executed
#   like a script, with `./justfile test`, for example.

alias t := test

alias c := check

bt := '0'

export RUST_BACKTRACE := bt

test:
	cargo test

fuzz:
	cargo +nightly fuzz run fuzz-compiler

@spam:
	{ \
		figlet test; \
		cargo build --color always 2>&1; \
		cargo test  --color always -- --color always 2>&1; \
	} | less

# only run tests matching PATTERN
filter PATTERN:
	cargo test {{PATTERN}}

build:
	cargo build

check:
	cargo check

fmt:
	cargo +nightly fmt --all

watch +COMMAND='test':
	cargo watch --clear --exec "{{COMMAND}}"

man:
	cargo build --features help4help2man
	help2man \
		--name 'save and run commands' \
		--manual 'JUST MANUAL' \
		--no-info \
		target/debug/just \
		> man/just.1

view-man: man
	man man/just.1

version := `sed -En 's/version[[:space:]]*=[[:space:]]*"([^"]+)"/v\1/p' Cargo.toml | head -1`

# add git log messages to changelog
changes:
	git log --pretty=format:%s >> CHANGELOG.md

# check run before publishing
publish-check: lint clippy test man
	cargo outdated --exit-code 1
	git branch | grep '* master'
	git diff --no-ext-diff --quiet --exit-code
	grep {{version}} CHANGELOG.md
	cargo build --features summary
	cargo +nightly generate-lockfile -Z minimal-versions
	cargo test
	git checkout Cargo.lock

# publish to crates.io and push release tag to github
publish: publish-check
	cargo +nightly publish
	git tag -a {{version}} -m 'Release {{version}}'
	git push github {{version}}

push: test
	! git branch | grep '* master'
	git push github

pr: push
	hub pull-request -o

# clean up feature branch BRANCH
done BRANCH:
	git checkout master
	git diff --no-ext-diff --quiet --exit-code
	git pull --rebase github master
	git diff --no-ext-diff --quiet --exit-code {{BRANCH}}
	git branch -D {{BRANCH}}

# install just from crates.io
install:
	cargo install -f just

# install development dependencies
install-dev-deps:
	rustup install nightly
	rustup update nightly
	rustup run nightly cargo install -f clippy
	cargo install -f cargo-watch
	cargo install -f cargo-check
	cargo +nightly install cargo-fuzz

# install system development dependencies with homebrew
install-dev-deps-homebrew:
	brew install help2man

# everyone's favorite animate paper clip
clippy:
	cargo clippy

# count non-empty lines of code
sloc:
	@cat src/*.rs | sed '/^\s*$/d' | wc -l

@lint:
	echo Checking for FIXME/TODO...
	! grep --color -En 'FIXME|TODO' src/*.rs
	echo Checking for long lines...
	! grep --color -En '.{101}' src/*.rs

replace FROM TO:
	sd '{{FROM}}' '{{TO}}' src/*.rs

test-quine:
	cargo run -- quine

# make a quine, compile it, and verify it
quine:
	mkdir -p tmp
	@echo '{{quine-text}}' > tmp/gen0.c
	cc tmp/gen0.c -o tmp/gen0
	./tmp/gen0 > tmp/gen1.c
	cc tmp/gen1.c -o tmp/gen1
	./tmp/gen1 > tmp/gen2.c
	diff tmp/gen1.c tmp/gen2.c
	rm -r tmp
	@echo 'It was a quine!'

quine-text := '
	int printf(const char*, ...);

	int main() {
		char *s =
			"int printf(const char*, ...);"
			"int main() {"
			"	 char *s = %c%s%c;"
			"  printf(s, 34, s, 34);"
			"  return 0;"
			"}";
		printf(s, 34, s, 34);
		return 0;
	}
'

render-readme:
	#!/usr/bin/env ruby
	require 'github/markup'
	$rendered = GitHub::Markup.render("README.adoc", File.read("README.adoc"))
	File.write('tmp/README.html', $rendered)

watch-readme:
	just render-readme
	fswatch -ro README.adoc | xargs -n1 -I{} just render-readme

generate-completions:
	./bin/generate-completions

# run all polyglot recipes
polyglot: _python _js _perl _sh _ruby
# (recipes that start with `_` are hidden from --list)

_python:
	#!/usr/bin/env python3
	print('Hello from python!')

_js:
	#!/usr/bin/env node
	console.log('Greetings from JavaScript!')

_perl:
	#!/usr/bin/env perl
	print "Larry Wall says Hi!\n";

_sh:
	#!/usr/bin/env sh
	hello='Yo'
	echo "$hello from a shell script!"

_ruby:
	#!/usr/bin/env ruby
	puts "Hello from ruby!"

# Print working directory, for demonstration purposes!
pwd:
	echo {{invocation_directory()}}

# Local Variables:
# mode: makefile
# End:
# vim: set ft=make :
