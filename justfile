test: build
	cargo test --lib

# only run tests matching PATTERN
filter PATTERN: build
	cargo test --lib {{PATTERN}}

# test with backtrace
backtrace:
	RUST_BACKTRACE=1 cargo test --lib

build:
	cargo build

check:
	cargo check

@banner:
	clear
	echo
	echo
	echo
	echo
	echo
	echo
	echo
	echo
	echo
	echo
	echo
	just test

watch COMMAND='test':
	cargo watch {{COMMAND}}

version = `sed -En 's/version[[:space:]]*=[[:space:]]*"([^"]+)"/v\1/p' Cargo.toml`

# publish to crates.io
publish: lint clippy test
	git branch | grep '* master'
	git diff --no-ext-diff --quiet --exit-code
	cargo publish
	git tag -a {{version}} -m 'Release {{version}}'
	git push github {{version}}
	git push github

build-binary-mac VERSION:
	just build-binary {{VERSION}} x86_64-apple-darwin

build-binary-linux VERSION:
	just build-binary {{VERSION}} x86_64-unknown-linux-musl

build-and-fetch-linux-binary VERSION:
	vagrant up
	vagrant ssh -- 'bash -lc "cd just && git checkout master && git pull && just build-binary-linux {{VERSION}}"'
	rm -rf tmp/linux
	mkdir -p tmp/linux
	scp \
	  -P 2222 \
	  -i .vagrant/machines/default/virtualbox/private_key \
	  'vagrant@127.0.0.1:just/tmp/*-x86_64-unknown-linux-musl.tar.gz' \
	  tmp/linux

build-binary VERSION TARGET:
	git diff --no-ext-diff --quiet --exit-code
	git checkout {{VERSION}}
	cargo build --release --target={{TARGET}}
	rm -rf tmp/just-{{VERSION}}-{{TARGET}}
	rm -rf tmp/just-{{VERSION}}-{{TARGET}}.tar.gz
	mkdir -p tmp/just-{{VERSION}}-{{TARGET}}
	cp \
	  GRAMMAR.md \
	  LICENSE.md \
	  README.md \
	  target/{{TARGET}}/release/just \
	  tmp/just-{{VERSION}}-{{TARGET}}
	cd tmp && tar cvfz \
	  just-{{VERSION}}-{{TARGET}}.tar.gz \
	  just-{{VERSION}}-{{TARGET}}

# clean up feature branch BRANCH
done BRANCH:
	git checkout {{BRANCH}}
	git pull --rebase github master
	git checkout master
	git pull --rebase github master
	git branch -d {{BRANCH}}

# push master to github as branch GITHUB-BRANCH
push GITHUB-BRANCH +FLAGS='':
	git branch | grep '* master'
	git diff --no-ext-diff --quiet --exit-code
	git push {{FLAGS}} github master:refs/heads/{{GITHUB-BRANCH}}

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

# everyone's favorite animate paper clip
clippy: lint
	rustup run nightly cargo clippy -- -D clippy

# count non-empty lines of code
sloc:
	@cat src/*.rs | sed '/^\s*$/d' | wc -l

@lint:
	echo Checking for FIXME/TODO...
	! grep --color -En 'FIXME|TODO' src/*.rs
	echo Checking for long lines...
	! grep --color -En '.{101}' src/*.rs

nop:

fail:
	exit 1

backtick-fail:
	echo {{`exit 1`}}

test-quine:
	cargo run -- quine

# make a quine, compile it, and verify it
quine:
	@echo '{{quine-text}}' > tmp/gen0.c
	cc tmp/gen0.c -o tmp/gen0
	./tmp/gen0 > tmp/gen1.c
	cc tmp/gen1.c -o tmp/gen1
	./tmp/gen1 > tmp/gen2.c
	diff tmp/gen1.c tmp/gen2.c
	@echo 'It was a quine!'

quine-text = '
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

# run all polyglot recipes
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

ruby:
	#!/usr/bin/env ruby
	puts "Hello from ruby!"

# Local Variables:
# mode: makefile
# End:
# vim: set ft=make :
