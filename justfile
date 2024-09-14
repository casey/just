#!/usr/bin/env -S just --justfile
# ^ A shebang isn't required, but allows a justfile to be executed
#   like a script, with `./justfile test`, for example.

alias t := test

log := "warn"

export JUST_LOG := log

watch +args='test':
  cargo watch --clear --exec '{{ args }}'

test:
  cargo test --all

ci: forbid test build-book clippy
  cargo fmt --all -- --check
  cargo update --locked --package just

fuzz:
  cargo +nightly fuzz run fuzz-compiler

run:
  cargo run

# only run tests matching PATTERN
filter PATTERN:
  cargo test {{PATTERN}}

build:
  cargo build

fmt:
  cargo fmt --all

shellcheck:
  shellcheck www/install.sh

man:
  mkdir -p man
  cargo run -- --man > man/just.1

view-man: man
  man man/just.1

# add git log messages to changelog
update-changelog:
  echo >> CHANGELOG.md
  git log --pretty='format:- %s' >> CHANGELOG.md

update-contributors:
  cargo run --release --package update-contributors

outdated:
  cargo outdated -R

# publish current GitHub master branch
publish:
  #!/usr/bin/env bash
  set -euxo pipefail
  rm -rf tmp/release
  git clone git@github.com:casey/just.git tmp/release
  cd tmp/release
  ! grep '<sup>master</sup>' README.md
  VERSION=`sed -En 's/version[[:space:]]*=[[:space:]]*"([^"]+)"/\1/p' Cargo.toml | head -1`
  git tag -a $VERSION -m "Release $VERSION"
  git push origin $VERSION
  cargo publish
  cd ../..
  rm -rf tmp/release

readme-version-notes:
  grep '<sup>master</sup>' README.md

# clean up feature branch BRANCH
done BRANCH=`git rev-parse --abbrev-ref HEAD`:
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
  cargo +nightly install cargo-fuzz
  cargo install cargo-check
  cargo install cargo-watch
  cargo install mdbook mdbook-linkcheck

# everyone's favorite animate paper clip
clippy:
  cargo clippy --all --all-targets --all-features -- --deny warnings

forbid:
  ./bin/forbid

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
      "   char *s = %c%s%c;"
      "  printf(s, 34, s, 34);"
      "  return 0;"
      "}";
    printf(s, 34, s, 34);
    return 0;
  }
'

test-completions:
  ./tests/completions/just.bash

build-book:
  cargo run --package generate-book
  mdbook build book/en
  mdbook build book/zh

# run all polyglot recipes
polyglot: _python _js _perl _sh _ruby

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

_nu:
  #!/usr/bin/env nu
  let hellos = ["Greetings", "Yo", "Howdy"]
  $hellos | each {|el| print $"($el) from a nushell script!" }

_ruby:
  #!/usr/bin/env ruby
  puts "Hello from ruby!"

# Print working directory, for demonstration purposes!
pwd:
  echo {{invocation_directory()}}

test-bash-completions:
  rm -rf tmp
  mkdir -p tmp/bin
  cargo build
  cp target/debug/just tmp/bin
  ./tmp/bin/just --completions bash > tmp/just.bash
  echo 'mod foo' > tmp/justfile
  echo 'bar:' > tmp/foo.just
  cd tmp && PATH="`realpath bin`:$PATH" bash --init-file just.bash

test-release-workflow:
  -git tag -d test-release
  -git push origin :test-release
  git tag test-release
  git push origin test-release

# Local Variables:
# mode: makefile
# End:
# vim: set ft=make :
