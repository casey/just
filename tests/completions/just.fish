#!/usr/bin/env fish

function reply_equals
    set -l actual $argv[1]
    set -l expected $argv[2]

    if test "$actual" = "$expected"
        echo (status current-function)": ok"
    else
        set -g exit_code 1
        echo (status current-function)": failed!" >&2
        echo "expected: $expected" >&2
        echo "actual:   $actual" >&2
    end
end

complete -c just -e
source $argv[1]
cd tests/completions
cargo build >/dev/null
set -gx PATH (git rev-parse --show-toplevel)/target/debug $PATH
set -g exit_code 0

function test_complete_root_recipes
    set -l actual (string join ' ' (complete -C "just p"))
    reply_equals "$actual" "publish push"
end
test_complete_root_recipes

function test_complete_nested_modules
    set -l actual (string join ' ' (complete -C "just repo o"))
    reply_equals "$actual" "open"
end
test_complete_nested_modules

function test_complete_nested_module_recipes
    set -l actual (string join ' ' (complete -C "just repo open c"))
    reply_equals "$actual" "codex"
end
test_complete_nested_module_recipes

if test $exit_code -eq 0
    echo "All tests passed."
else
    echo "Some test[s] failed."
end

exit $exit_code
