_just() {
    local i cur prev words cword opts cmd
    COMPREPLY=()

    # Modules use "::" as the separator, which is considered a wordbreak character in bash.
    # The _get_comp_words_by_ref function is a hack to allow for exceptions to this rule without
    # modifying the global COMP_WORDBREAKS environment variable.
    if type _get_comp_words_by_ref &>/dev/null; then
        _get_comp_words_by_ref -n : cur prev words cword
    else
        cur="${COMP_WORDS[COMP_CWORD]}"
        prev="${COMP_WORDS[COMP_CWORD-1]}"
        words=$COMP_WORDS
        cword=$COMP_CWORD
    fi

    cmd=""
    opts=""

    for i in ${words[@]}
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
            opts="-E -n -g -f -q -u -v -d -c -e -l -s -h -V --alias-style --ceiling --check --chooser --clear-shell-args --color --command-color --cygpath --dotenv-filename --dotenv-path --dry-run --dump-format --explain --global-justfile --highlight --justfile --list-heading --list-prefix --list-submodules --no-aliases --no-deps --no-dotenv --no-highlight --one --quiet --allow-missing --set --shell --shell-arg --shell-command --tempdir --timestamp --timestamp-format --unsorted --unstable --verbose --working-directory --yes --changelog --choose --command --completions --dump --edit --evaluate --fmt --groups --init --list --man --request --show --summary --variables --help --version [ARGUMENTS]..."
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
                        if type __ltrim_colon_completions &>/dev/null; then
                            __ltrim_colon_completions "$cur"
                        fi
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
                        if type __ltrim_colon_completions &>/dev/null; then
                            __ltrim_colon_completions "$cur"
                        fi
                        return 0
                    fi
                elif [[ ${COMP_CWORD} -eq 2 ]]; then
                    # Handle submodule completion: just <module> <recipe>
                    local module="${COMP_WORDS[1]}"
                    
                    # Try to find the module's justfile and get recipes + aliases from it
                    # First, find the project root (where the main justfile is)
                    local project_root=""
                    local current_dir="${PWD}"
                    while [[ -n "$current_dir" && "$current_dir" != "/" ]]; do
                        if [[ -f "$current_dir/justfile" ]]; then
                            project_root="$current_dir"
                            break
                        fi
                        current_dir=$(dirname "$current_dir")
                    done
                    
                    # Try to find module justfile in common locations
                    local module_justfile=""
                    if [[ -n "$project_root" ]]; then
                        # Check common module locations
                        for dir in "$project_root/$module" "$project_root/$module/justfile" "$project_root/modules/$module" "$project_root/modules/$module/justfile"; do
                            if [[ -f "$dir/justfile" ]]; then
                                module_justfile="$dir/justfile"
                                break
                            elif [[ -f "$dir" && "$dir" == *justfile ]]; then
                                module_justfile="$dir"
                                break
                            fi
                        done
                    fi
                    
                    # If we found the module's justfile, get recipes and aliases from it
                    if [[ -n "$module_justfile" && -f "$module_justfile" ]]; then
                        local list_output=$(just --list --justfile "$module_justfile" 2> /dev/null)
                        if [[ $? -eq 0 ]]; then
                            # Extract recipe names (first word after leading spaces, before any * or #)
                            local recipes=$(echo "$list_output" | sed -n 's/^[[:space:]]*\([a-zA-Z0-9_-]*\).*/\1/p' | grep -v '^$' | grep -v '^Available$')
                            
                            # Extract aliases from [alias: ...] or [aliases: ...] patterns
                            local aliases=$(echo "$list_output" | sed -n 's/.*\[alias:[[:space:]]*\([^]]*\)\].*/\1/p' | tr ',' '\n')
                            aliases="$aliases"$(echo "$list_output" | sed -n 's/.*\[aliases:[[:space:]]*\([^]]*\)\].*/\1/p' | tr ',' '\n')
                            aliases=$(echo "$aliases" | sed 's/^[[:space:]]*//;s/[[:space:]]*$//' | grep -v '^$')
                            
                            # Combine recipes and aliases
                            local all_completions=$(printf "%s\n%s\n" "$recipes" "$aliases" | sort -u | tr '\n' ' ')
                            
                            if [[ -n "$all_completions" ]]; then
                                COMPREPLY=( $(compgen -W "${all_completions}" -- "${cur}") )
                                return 0
                            fi
                        fi
                    fi
                    
                    # Fallback: use --summary with module:: prefix (won't include aliases)
                    local all_recipes=$(just --summary 2> /dev/null)
                    if [[ $? -eq 0 ]]; then
                        # Filter recipes that start with module:: and strip the prefix
                        # just --summary returns space-separated recipes, so convert to lines first
                        local module_recipes=$(echo "$all_recipes" | tr ' ' '\n' | grep -E "^${module}::" | sed "s/^${module}:://")
                        
                        if [[ -n "$module_recipes" ]]; then
                            COMPREPLY=( $(compgen -W "${module_recipes}" -- "${cur}") )
                            return 0
                        fi
                    fi
                fi
            case "${prev}" in
                --alias-style)
                    COMPREPLY=($(compgen -W "left right separate" -- "${cur}"))
                    return 0
                    ;;
                --ceiling)
                    COMPREPLY=($(compgen -f "${cur}"))
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
                --cygpath)
                    COMPREPLY=($(compgen -f "${cur}"))
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
                --tempdir)
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
