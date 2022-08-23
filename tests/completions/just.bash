#!/usr/bin/env bash

source ./completions/just.bash

exit_code='0'

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

test_complete_all_recipes() {
  COMP_WORDS=(just)
  COMP_CWORD=1 _just just
  reply_equals 'declare -a COMPREPLY=([0]="build" [1]="build-book" [2]="changes" [3]="check" [4]="ci" [5]="clippy" [6]="done" [7]="filter" [8]="fmt" [9]="forbid" [10]="fuzz" [11]="generate-completions" [12]="install" [13]="install-dev-deps" [14]="install-dev-deps-homebrew" [15]="man" [16]="polyglot" [17]="pr" [18]="publish" [19]="push" [20]="pwd" [21]="quine" [22]="render-readme" [23]="replace" [24]="run" [25]="sloc" [26]="test" [27]="test-quine" [28]="view-man" [29]="watch" [30]="watch-readme")'
}

test_complete_recipes_starting_with_i() {
  COMP_WORDS=(just i)
  COMP_CWORD=1 _just just
  reply_equals 'declare -a COMPREPLY=([0]="install" [1]="install-dev-deps" [2]="install-dev-deps-homebrew")'
}

test_complete_recipes_starting_with_r() {
  COMP_WORDS=(just r)
  COMP_CWORD=1 _just just
  reply_equals 'declare -a COMPREPLY=([0]="render-readme" [1]="replace" [2]="run")'
}

PATH="./target/debug:$PATH"
test_just_is_accessible

cleanup
test_complete_all_recipes

cleanup
test_complete_recipes_starting_with_i

cleanup
test_complete_recipes_starting_with_r

if [ "$exit_code" = '0' ]; then
  echo "All tests passed."
else
  echo "Some test[s] failed."
fi
exit "$exit_code"
