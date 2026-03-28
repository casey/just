_clap_reassemble_words() {
    if [[ "$COMP_WORDBREAKS" != *:* ]]; then
        return
    fi
    local i j=0 line=$COMP_LINE
    words=()
    _CLAP_COMPLETE_INDEX=0
    for ((i = 0; i < ${#COMP_WORDS[@]}; i++)); do
        if ((i > 0 && j > 0)) && [[ "${COMP_WORDS[i]}" == :* || "${words[j-1]}" == *: ]] && [[ "$line" != [[:blank:]]* ]]; then
            words[j-1]="${words[j-1]}${COMP_WORDS[i]}"
        else
            words[j]="${COMP_WORDS[i]}"
            ((j++))
        fi
        if ((i == COMP_CWORD)); then
            _CLAP_COMPLETE_INDEX=$((j - 1))
        fi
        line=${line#*"${COMP_WORDS[i]}"}
    done
}

_clap_trim_completions() {
    local cur="${words[_CLAP_COMPLETE_INDEX]}"
    if [[ "$cur" != *:* || "$COMP_WORDBREAKS" != *:* ]]; then
        return
    fi
    local colon_word=${cur%"${cur##*:}"}
    local i=${#COMPREPLY[*]}
    while [[ $((--i)) -ge 0 ]]; do
        COMPREPLY[$i]=${COMPREPLY[$i]#"$colon_word"}
    done
}

_clap_complete_just() {
    local IFS=$'\013'
    local _CLAP_COMPLETE_INDEX=${COMP_CWORD}
    local _CLAP_COMPLETE_COMP_TYPE=${COMP_TYPE}
    if compopt +o nospace 2> /dev/null; then
        local _CLAP_COMPLETE_SPACE=false
    else
        local _CLAP_COMPLETE_SPACE=true
    fi
    local words=("${COMP_WORDS[@]}")
    _clap_reassemble_words
    COMPREPLY=( $( \
        _CLAP_IFS="$IFS" \
        _CLAP_COMPLETE_INDEX="$_CLAP_COMPLETE_INDEX" \
        _CLAP_COMPLETE_COMP_TYPE="$_CLAP_COMPLETE_COMP_TYPE" \
        _CLAP_COMPLETE_SPACE="$_CLAP_COMPLETE_SPACE" \
        JUST_COMPLETE="bash" \
        just -- "${words[@]}" \
    ) )
    if [[ $? != 0 ]]; then
        unset COMPREPLY
    elif [[ $_CLAP_COMPLETE_SPACE == false ]] && [[ "${COMPREPLY-}" =~ [=/:]$ ]]; then
        compopt -o nospace
    fi
    _clap_trim_completions
}

if [[ "${BASH_VERSINFO[0]}" -eq 4 && "${BASH_VERSINFO[1]}" -ge 4 || "${BASH_VERSINFO[0]}" -gt 4 ]]; then
    complete -o nospace -o bashdefault -o nosort -F _clap_complete_just just
else
    complete -o nospace -o bashdefault -F _clap_complete_just just
fi
