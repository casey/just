#compdef just

autoload -U is-at-least

_just() {
    typeset -A opt_args
    typeset -a _arguments_options
    local ret=1

    if is-at-least 5.2; then
        _arguments_options=(-s -S -C)
    else
        _arguments_options=(-s -C)
    fi

    local context curcontext="$curcontext" state line
    _arguments "${_arguments_options[@]}" \
'--color=[Print colorful output]: :(auto always never)' \
'-f+[Use <JUSTFILE> as justfile.]' \
'--justfile=[Use <JUSTFILE> as justfile.]' \
'*--set=[Override <VARIABLE> with <VALUE>]' \
'--shell=[Invoke <SHELL> to run recipes]' \
'*--shell-arg=[Invoke shell with <SHELL-ARG> as an argument]' \
'-d+[Use <WORKING-DIRECTORY> as working directory. --justfile must also be set]' \
'--working-directory=[Use <WORKING-DIRECTORY> as working directory. --justfile must also be set]' \
'--completions=[Print shell completion script for <SHELL>]: :(zsh bash fish powershell elvish)' \
'-s+[Show information about <RECIPE>]' \
'--show=[Show information about <RECIPE>]' \
'(-q --quiet)--dry-run[Print what just would do without doing it]' \
'--highlight[Highlight echoed recipe lines in bold]' \
'--no-highlight[Don'\''t highlight echoed recipe lines in bold]' \
'(--dry-run)-q[Suppress all output]' \
'(--dry-run)--quiet[Suppress all output]' \
'--clear-shell-args[Clear shell arguments]' \
'*-v[Use verbose output]' \
'*--verbose[Use verbose output]' \
'--dump[Print entire justfile]' \
'-e[Edit justfile with editor given by $VISUAL or $EDITOR, falling back to `vim`]' \
'--edit[Edit justfile with editor given by $VISUAL or $EDITOR, falling back to `vim`]' \
'--evaluate[Print evaluated variables]' \
'--init[Initialize new justfile in project root]' \
'-l[List available recipes and their arguments]' \
'--list[List available recipes and their arguments]' \
'--summary[List names of available recipes]' \
'-h[Print help information]' \
'--help[Print help information]' \
'-V[Print version information]' \
'--version[Print version information]' \
'::ARGUMENTS -- Overrides and recipe(s) to run, defaulting to the first recipe in the justfile:_files' \
&& ret=0
    
}

(( $+functions[_just_commands] )) ||
_just_commands() {
    local commands; commands=(
        
    )
    _describe -t commands 'just commands' commands "$@"
}

_just "$@"