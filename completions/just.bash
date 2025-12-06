_just() {
    local i cur prev words cword opts cmd
    COMPREPLY=()
    if [[ "${BASH_VERSINFO[0]}" -ge 4 ]]; then
        cur="$2"
    else
        cur="${COMP_WORDS[COMP_CWORD]}"
    fi
    prev="$3"
    cmd=""
    opts=""

    for i in "${COMP_WORDS[@]:0:COMP_CWORD}"
    do
        case "${cmd},${i}" in
            ",$1")
                cmd="just"
                ;;
            *)
                ;;
        esac
    done

    case "${cmd}" in
        just)
            opts="-E -n -g -f -q -u -v -d -c -e -l -s -h -V --alias-style --check --chooser --clear-shell-args --color --command-color --dotenv-filename --dotenv-path --dry-run --dump-format --explain --global-justfile --highlight --justfile --list-heading --list-prefix --list-submodules --no-aliases --no-deps --no-dotenv --no-highlight --one --quiet --allow-missing --set --shell --shell-arg --shell-command --timestamp --timestamp-format --unsorted --unstable --verbose --working-directory --yes --changelog --choose --command --completions --dump --edit --evaluate --fmt --groups --init --list --man --request --show --summary --variables --help --version [ARGUMENTS]..."
                if [[ ${cur} == -* ]] ; then
                    COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                    return 0
                elif [[ ${COMP_CWORD} -eq 1 ]]; then
                    # Get recipes and aliases from just --list
                    local list_output=$(just --list 2> /dev/null)
                    if [[ $? -eq 0 ]]; then
                        # Extract recipe names (first word after leading spaces, before any * or #)
                        local recipes=$(echo "$list_output" | sed -n 's/^[[:space:]]*\([a-zA-Z0-9_-]*\).*/\1/p' | grep -v '^$' | grep -v '^Available$')

                        # Extract aliases from [alias: ...] or [aliases: ...] patterns
                        local aliases=$(echo "$list_output" | sed -n 's/.*\[alias:[[:space:]]*\([^]]*\)\].*/\1/p' | tr ',' '\n')
                        aliases="$aliases"$(echo "$list_output" | sed -n 's/.*\[aliases:[[:space:]]*\([^]]*\)\].*/\1/p' | tr ',' '\n')
                        aliases=$(echo "$aliases" | sed 's/^[[:space:]]*//;s/[[:space:]]*$//' | grep -v '^$')

                        # Combine recipes and aliases
                        local all_completions=$(printf "%s\n%s\n" "$recipes" "$aliases" | sort -u | tr '\n' ' ')

                        if echo "${cur}" | \grep -qF '/'; then
                            local path_prefix=$(echo "${cur}" | sed 's/[/][^/]*$/\//')
                            local path_recipes=$(just --summary 2> /dev/null -- "${path_prefix}")
                            if [[ $? -eq 0 ]]; then
                                all_completions=$(printf "${path_prefix}%s\t" $path_recipes)
                            fi
                        fi

                        COMPREPLY=( $(compgen -W "${all_completions}" -- "${cur}") )
                        return 0
                    fi

                    # Fallback to --summary if --list fails
                    local recipes=$(just --summary 2> /dev/null)
                    if echo "${cur}" | \grep -qF '/'; then
                        local path_prefix=$(echo "${cur}" | sed 's/[/][^/]*$/\//')
                        local recipes=$(just --summary 2> /dev/null -- "${path_prefix}")
                        local recipes=$(printf "${path_prefix}%s\t" $recipes)
                    fi
                    if [[ $? -eq 0 ]]; then
                        COMPREPLY=( $(compgen -W "${recipes}" -- "${cur}") )
                        return 0
                    fi
                elif [[ ${COMP_CWORD} -ge 2 ]]; then
                    # Recursive hierarchical completion for any depth:
                    # just a <TAB> -> lists children of `a`
                    # just a b <TAB> -> lists children of `a::b`
                    # just a b c <TAB> -> lists children of `a::b::c`

                    # Build the current prefix from the already-typed words after `just`
                    local prefix_str=""
                    local i
                    for ((i = 1; i < COMP_CWORD; i++)); do
                        if [[ -z "$prefix_str" ]]; then
                            prefix_str="${COMP_WORDS[i]}"
                        else
                            prefix_str="${prefix_str}::${COMP_WORDS[i]}"
                        fi
                    done

                    # Try to find the justfile for the current prefix by converting :: to /
                    # e.g., dev-env::docker -> dev-env/docker/justfile
                    local prefix_path=$(echo "$prefix_str" | sed 's/::/\//g')
                    local project_root=""
                    local current_dir="${PWD}"
                    
                    # Find the project root (where the main justfile is)
                    while [[ -n "$current_dir" && "$current_dir" != "/" ]]; do
                        if [[ -f "$current_dir/justfile" ]]; then
                            project_root="$current_dir"
                            break
                        fi
                        current_dir=$(dirname "$current_dir")
                    done
                    
                    # Search for the justfile in the prefix path relative to project root
                    local prefix_justfile=""
                    if [[ -n "$project_root" ]]; then
                        local test_path="$project_root/$prefix_path/justfile"
                        if [[ -f "$test_path" ]]; then
                            prefix_justfile="$test_path"
                        fi
                    fi

                    # If we found a justfile for the prefix, use --list to get recipes and aliases
                    if [[ -n "$prefix_justfile" ]]; then
                        local list_output=$(just --list --justfile "$prefix_justfile" 2> /dev/null)
                        if [[ $? -eq 0 ]]; then
                            # Extract recipe names (first word after leading spaces, before any * or #)
                            local recipes=$(echo "$list_output" | sed -n 's/^[[:space:]]*\([a-zA-Z0-9_-]*\).*/\1/p' | grep -v '^$' | grep -v '^Available$')

                            # Extract aliases from [alias: ...] or [aliases: ...] patterns
                            local aliases=$(echo "$list_output" | sed -n 's/.*\[alias:[[:space:]]*\([^]]*\)\].*/\1/p' | tr ',' '\n')
                            aliases="$aliases"$(echo "$list_output" | sed -n 's/.*\[aliases:[[:space:]]*\([^]]*\)\].*/\1/p' | tr ',' '\n')
                            aliases=$(echo "$aliases" | sed 's/^[[:space:]]*//;s/[[:space:]]*$//' | grep -v '^$')

                            # Combine recipes and aliases, extract only the next segment
                            local candidates=$(printf "%s\n%s\n" "$recipes" "$aliases" | sort -u | tr '\n' ' ')

                            if [[ -n "$candidates" ]]; then
                                COMPREPLY=( $(compgen -W "${candidates}" -- "${cur}") )
                                return 0
                            fi
                        fi
                    fi

                    # Fallback: Get all recipes from just --summary (space-separated)
                    # This doesn't include aliases, but works for recipes
                    local all_recipes
                    all_recipes=$(just --summary 2> /dev/null)
                    if [[ $? -ne 0 || -z "$all_recipes" ]]; then
                        return 0
                    fi

                    # From recipes that start with prefix_str::, take only the next segment
                    # after the prefix, so completion is hierarchical.
                    local candidates
                    candidates=$(
                        printf "%s\n" $all_recipes \
                        | tr ' ' '\n' \
                        | grep -E "^${prefix_str}::" \
                        | sed "s/^${prefix_str}:://" \
                        | sed 's/::.*$//' \
                        | sort -u
                    )

                    if [[ -n "$candidates" ]]; then
                        COMPREPLY=( $(compgen -W "${candidates}" -- "${cur}") )
                        return 0
                    fi
                fi
            case "${prev}" in
                --alias-style)
                    COMPREPLY=($(compgen -W "left right separate" -- "${cur}"))
                    return 0
                    ;;
                --chooser)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --color)
                    COMPREPLY=($(compgen -W "always auto never" -- "${cur}"))
                    return 0
                    ;;
                --command-color)
                    COMPREPLY=($(compgen -W "black blue cyan green purple red yellow" -- "${cur}"))
                    return 0
                    ;;
                --dotenv-filename)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --dotenv-path)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -E)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --dump-format)
                    COMPREPLY=($(compgen -W "json just" -- "${cur}"))
                    return 0
                    ;;
                --justfile)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -f)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --list-heading)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --list-prefix)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --set)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --shell)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --shell-arg)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --timestamp-format)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --working-directory)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -d)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --command)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -c)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --completions)
                    COMPREPLY=($(compgen -W "bash elvish fish nushell powershell zsh" -- "${cur}"))
                    return 0
                    ;;
                --list)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -l)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --request)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --show)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -s)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
    esac
}

if [[ "${BASH_VERSINFO[0]}" -eq 4 && "${BASH_VERSINFO[1]}" -ge 4 || "${BASH_VERSINFO[0]}" -gt 4 ]]; then
    complete -F _just -o nosort -o bashdefault -o default just
else
    complete -F _just -o bashdefault -o default just
fi
