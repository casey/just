#!/usr/bin/env zsh

reply_equals() {
  local actual=$1
  local expected=$2

  if [[ "$actual" == "$expected" ]]; then
    echo "${funcstack[2]}: ok"
  else
    exit_code=1
    echo >&2 "${funcstack[2]}: failed!"
    echo >&2 "expected: $expected"
    echo >&2 "actual:   $actual"
  fi
}

compdef() { :; }
_call_program() {
  shift
  "$@"
}

source "$1"
cargo build >/dev/null
PATH="$(git rev-parse --show-toplevel)/target/debug:$PATH"
tmpdir=$(mktemp -d)
cd "$tmpdir"
cat <<'EOF' > justfile
mod repo "repo.just"
EOF
cat <<'EOF' > repo.just
mod open "open.just"
EOF
cat <<'EOF' > open.just
codex:
  @:

vscode:
  @:
EOF
exit_code=0

test_nested_module_names() {
  local actual
  actual=$(_just_command_names repo::open | tr '\n' ' ' | sed 's/ $//')
  reply_equals "$actual" "codex vscode"
}
test_nested_module_names

test_module_path() {
  words=(just repo open co)
  CURRENT=4
  local actual
  actual=$(_just_module_path)
  reply_equals "$actual" "repo::open"
}
test_module_path

test_recipe_path() {
  words=(just repo open codex "")
  CURRENT=5
  local actual
  actual=$(_just_recipe_path)
  reply_equals "$actual" "repo::open::codex"
}
test_recipe_path

if [[ $exit_code -eq 0 ]]; then
  echo "All tests passed."
else
  echo "Some test[s] failed."
fi

exit $exit_code
