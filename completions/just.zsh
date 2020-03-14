#compdef just

autoload -U is-at-least

_just() {
    typeset -A opt_args
    typeset -a _arguments_options
    local context curcontext="$curcontext" state line ret=1

    if is-at-least 5.2; then
        _arguments_options=(-s -S -C)
    else
        _arguments_options=(-s -C)
    fi

<<<<<<< HEAD
    local common=(
        '--color=[Print colorful output]: :(auto always never)' 
        '(-f --justfile)'{-f+,--justfile=}'[Use <JUSTFILE> as justfile.]' 
        '*--set[Override <VARIABLE> with <VALUE>]: :_just_variables' 
        '--shell=[Invoke <SHELL> to run recipes]' 
        '*--shell-arg=[Invoke shell with <SHELL-ARG> as an argument]' 
        '(-d --working-directory)'{-d+,--working-directory}'[Use <WORKING-DIRECTORY> as working directory. --justfile must also be set]' 
        '--completions=[Print shell completion script for <SHELL>]: :(zsh bash fish powershell elvish)' 
        '(-s --show)'{-s+,--show=}'[Show information about <RECIPE>]: :_just_commands' 
        '(-q --quiet)--dry-run[Print what just would do without doing it]' 
        '(--dry-run)'{-q,--quiet}'[Suppress all output]' 
        '(--no-highlight)--highlight[Highlight echoed recipe lines in bold]' 
        '(--highlight)--no-highlight[Don'\''t highlight echoed recipe lines in bold]' 
        '--clear-shell-args[Clear shell arguments]' 
        '*'{-v,--verbose}'[Use verbose output]' 
        '--dump[Print entire justfile]' 
        '(- 1 *)'{-e,--edit}'[Edit justfile with editor given by $VISUAL or $EDITOR, falling back to `vim`]' 
        '--evaluate[Print evaluated variables]' 
        '--init[Initialize new justfile in project root]' 
        '(- 1 *)'{-l,--list}'[List available recipes and their arguments]' 
        '(- 1 *)--summary[List names of available recipes]' 
        '(- 1 *)'{-h,--help}'[Print help information]' 
        '(- 1 *)'{-V,--version}'[Print version information]' 
    )

    _arguments "${_arguments_options[@]}" $common \
        '1: :_just_commands' \
        '*: :->args' \
        && ret=0

    case $state in
        args)
            curcontext="${curcontext%:*}-${words[2]}:"

            # Display usage
            local -a args_str args
            args_str="`just --show ${words[2]}`"

            echo $args_str >> /tmp/debug

            lastarg=${words[${#words}]}

            if [[ ${lastarg} = */* ]]; then
                # Arguments contain slash would be recognised as a file
                _arguments -s -S $common '::ARGUMENTS -- Overrides and recipe(s) to run, defaulting to the first recipe in the justfile:_files' 
            else
                # Show usage message
                _message $args_str
                # Or complete with other commands
                #_arguments -s -S $common '*:: :_just_commands'
            fi
        ;;
    esac

    return ret
}

(( $+functions[_just_variables] )) ||
_just_variables() {
    local -a variables

    variables=( ${(s: :)$(_call_program commands just --variables)} )
    _describe -t variables 'variables' variables
}

(( $+functions[_just_commands] )) ||
_just_commands() {
    local -a commands

    commands=( ${${${(M)"${(f)$(_call_program commands just --list)}":#    *}/ ##/}/ ##/:Args: } )
    _describe -t commands 'just commands' commands "$@"
}

_just "$@"
