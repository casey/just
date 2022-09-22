#!/usr/bin/env bash

# --- Shared functions ---

cdroot() {
  cd "$(git rev-parse --show-toplevel)" > /dev/null
}

setup() {
  cdroot
  cd tests/completions > /dev/null
}

cleanup() {
  unset COMP_WORDS
  unset COMP_CWORD
  unset COMPREPLY
}

reply_equals() {
  local reply=$(declare -p COMPREPLY)
  local expected="$1"

  if [ "$reply" = "$expected" ]; then
    echo "${FUNCNAME[1]}: ok"
  else
    exit_code='1'
    echo >&2 "${FUNCNAME[1]}: failed! Completion for \`${COMP_WORDS[*]}\` does not match."

    echo
    diff -U3 --label expected <(echo "$expected") --label actual <(echo "$reply") >&2
    echo
  fi
}

# --- Initial Setup ---
cdroot
source ./completions/just.bash
PATH="$(git rev-parse --show-toplevel)/target/debug:$PATH"
exit_code='0'

# --- Tests ---

test_just_is_accessible() {
  if just --version > /dev/null; then
    echo "${FUNCNAME[0]}: ok"
  else
    echo "${FUNCNAME[0]}: failed! Can't find just binary."
    echo "  PATH=$PATH"
    echo
    exit_code='1'
  fi
}
setup
test_just_is_accessible

test_complete_all_recipes() {
  COMP_WORDS=(just)
  COMP_CWORD=1 _just just
  reply_equals 'declare -a COMPREPLY=([0]="deploy" [1]="install" [2]="publish" [3]="push" [4]="test")'
}
cleanup
setup
test_complete_all_recipes

test_complete_recipes_starting_with_i() {
  COMP_WORDS=(just i)
  COMP_CWORD=1 _just just
  reply_equals 'declare -a COMPREPLY=([0]="install")'
}
cleanup
setup
test_complete_recipes_starting_with_i

test_complete_recipes_starting_with_p() {
  setup
  COMP_WORDS=(just p)
  COMP_CWORD=1 _just just
  reply_equals 'declare -a COMPREPLY=([0]="publish" [1]="push")'
}
cleanup
setup
test_complete_recipes_starting_with_p

test_complete_recipes_from_subdirs() {
  COMP_WORDS=(just subdir/)
  COMP_CWORD=1 _just just
  reply_equals 'declare -a COMPREPLY=([0]="subdir/special" [1]="subdir/surprise")'
}
cleanup
setup
test_complete_recipes_from_subdirs
cleanup

# --- Conclusion ---

if [ "$exit_code" = '0' ]; then
  echo "All tests passed."
else
  echo "Some test[s] failed."
fi
exit "$exit_code"
