use super::*;

#[derive(ValueEnum, Debug, Clone, Copy, PartialEq)]
pub(crate) enum Shell {
  Bash,
  Elvish,
  Fish,
  #[value(alias = "nu")]
  Nushell,
  Powershell,
  Zsh,
}

impl Shell {
  pub(crate) fn script(self) -> &'static str {
    match self {
      Self::Bash => include_str!("../completions/just.bash"),
      Self::Elvish => include_str!("../completions/just.elvish"),
      Self::Fish => include_str!("../completions/just.fish"),
      Self::Nushell => include_str!("../completions/just.nu"),
      Self::Powershell => include_str!("../completions/just.powershell"),
      Self::Zsh => include_str!("../completions/just.zsh"),
    }
  }
}

#[cfg(test)]
mod tests {
  use {
    super::*,
    pretty_assertions::assert_eq,
    std::io::{Read, Seek},
    tempfile::tempfile,
  };

  #[test]
  fn scripts() {
    fs::create_dir_all("tmp/completions").unwrap();

    let bash = clap(clap_complete::Shell::Bash);
    fs::write("tmp/completions/just.bash", &bash).unwrap();

    let elvish = clap(clap_complete::Shell::Elvish);
    fs::write("tmp/completions/just.elvish", &elvish).unwrap();

    let fish = clap(clap_complete::Shell::Fish);
    fs::write("tmp/completions/just.fish", &fish).unwrap();

    let powershell = clap(clap_complete::Shell::PowerShell);
    fs::write("tmp/completions/just.powershell", &powershell).unwrap();

    let zsh = clap(clap_complete::Shell::Zsh);
    fs::write("tmp/completions/just.zsh", &zsh).unwrap();

    assert_eq!(Shell::Bash.script(), bash);
    assert_eq!(Shell::Elvish.script(), elvish);
    assert_eq!(Shell::Fish.script(), fish);
    assert_eq!(Shell::Powershell.script(), powershell);
    assert_eq!(Shell::Zsh.script(), zsh);
  }

  fn clap(shell: clap_complete::Shell) -> String {
    fn replace(haystack: &mut String, needle: &str, replacement: &str) {
      if let Some(index) = haystack.find(needle) {
        haystack.replace_range(index..index + needle.len(), replacement);
      } else {
        panic!("Failed to find text:\n{needle}\n…in completion script:\n{haystack}")
      }
    }

    let mut script = {
      let mut tempfile = tempfile().unwrap();

      clap_complete::generate(
        shell,
        &mut crate::config::Config::app(),
        env!("CARGO_PKG_NAME"),
        &mut tempfile,
      );

      tempfile.rewind().unwrap();

      let mut buffer = String::new();

      tempfile.read_to_string(&mut buffer).unwrap();

      buffer
    };

    match shell {
      clap_complete::Shell::Bash => {
        for (needle, replacement) in BASH_COMPLETION_REPLACEMENTS {
          replace(&mut script, needle, replacement);
        }
      }
      clap_complete::Shell::Fish => {
        script.insert_str(0, FISH_RECIPE_COMPLETIONS);
      }
      clap_complete::Shell::PowerShell => {
        for (needle, replacement) in POWERSHELL_COMPLETION_REPLACEMENTS {
          replace(&mut script, needle, replacement);
        }
      }
      clap_complete::Shell::Zsh => {
        for (needle, replacement) in ZSH_COMPLETION_REPLACEMENTS {
          replace(&mut script, needle, replacement);
        }
      }
      _ => {}
    }

    let mut script = script.trim().to_string();
    script.push('\n');
    script
  }

  const FISH_RECIPE_COMPLETIONS: &str = r#"function __fish_just_command_names --argument module
        set -l output
        if test -n "$module"
          set output (just --list "$module" 2>/dev/null | string split \n)
        else
          set output (just --list 2>/dev/null | string split \n)
        end

        for line in $output
          if string match -qr '^    ' -- $line
            set line (string replace -r '^    ' '' -- $line)
            echo (string split -f1 ' ' -- $line)
          end
        end
end

function __fish_just_module_path
        set -l words (commandline -opc)
        set -l module

        for word in $words[2..-1]
          if test -z "$word"
            continue
          end

          if string match -qr '^-|=' -- $word
            continue
          end

          if test -n "$module"
            set module "$module::$word"
          else
            set module "$word"
          end
        end

        echo $module
end

function __fish_just_recipe_path
        set -l words (commandline -opc)
        set -l recipes (string split ' ' -- (just --summary 2>/dev/null))
        set -l candidate

        for word in $words[2..-1]
          if test -z "$word"
            continue
          end

          if string match -qr '^-|=' -- $word
            continue
          end

          if test -n "$candidate"
            set candidate "$candidate::$word"
          else
            set candidate "$word"
          end

          if contains -- "$candidate" $recipes
            echo $candidate
            return 0
          end
        end

        return 1
end

function __fish_just_complete_recipes
        if string match -rq '(-f|--justfile)\s*=?(?<justfile>[^\s]+)' -- (string split -- ' -- ' (commandline -pc))[1]
          set -fx JUST_JUSTFILE "$justfile"
        end
        if test -z (__fish_just_recipe_path)
          __fish_just_command_names (__fish_just_module_path)
        end
end

# don't suggest files right off
complete -c just -n "__fish_is_first_arg" --no-files

# complete recipes
complete -c just -f -a '(__fish_just_complete_recipes)'

# autogenerated completions
"#;

  const ZSH_COMPLETION_REPLACEMENTS: &[(&str, &str)] = &[
    (
      r#"    _arguments "${_arguments_options[@]}" : \"#,
      r"    local common=(",
    ),
    (
      r"'*--set=[Override <VARIABLE> with <VALUE>]:VARIABLE:_default:VARIABLE:_default' \",
      r"'*--set=[Override <VARIABLE> with <VALUE>]: :(_just_variables)' \",
    ),
    (
      r"'()-s+[Show recipe at <PATH>]:PATH:_default' \
'()--show=[Show recipe at <PATH>]:PATH:_default' \",
      r"'-s+[Show recipe at <PATH>]: :(_just_commands)' \
'--show=[Show recipe at <PATH>]: :(_just_commands)' \",
    ),
    (
      "'*::ARGUMENTS -- Overrides and recipe(s) to run, defaulting to the first recipe in the \
     justfile:_default' \\
&& ret=0",
      r#")

    _arguments "${_arguments_options[@]}" $common \
        '1: :_just_commands' \
        '*: :->args' \
        && ret=0

    case $state in
        args)
            curcontext="${curcontext%:*}-${words[2]}:"

            local lastarg=${words[${#words}]}
            local recipe=$(_just_recipe_path)

            if [[ $lastarg = */* ]]; then
                # Arguments contain slash would be recognised as a file
                _arguments -s -S $common '*:: :_files'
            elif [[ $lastarg = *=* ]]; then
                # Arguments contain equal would be recognised as a variable
                _message "value"
            elif [[ $recipe ]]; then
                # Show usage message
                _message "`just --show $recipe`"
            else
                _just_commands
            fi
        ;;
    esac

    return ret
"#,
    ),
    (
      "    local commands; commands=()",
      r#"    [[ $PREFIX = -* ]] && return 1
    integer ret=1
    local variables; variables=(
        ${(s: :)$(_call_program commands just --variables)}
    )
    local module=$(_just_module_path)
    local commands; commands=(
        ${(@f)$(_just_command_names "$module")}
    )
"#,
    ),
    (
      r#"    _describe -t commands 'just commands' commands "$@""#,
      r#"    if compset -P '*='; then
        case "${${words[-1]%=*}#*=}" in
            *) _message 'value' && ret=0 ;;
        esac
    else
        _describe -t variables 'variables' variables -qS "=" && ret=0
        _describe -t commands 'just commands' commands "$@"
    fi
"#,
    ),
    (
      r#"if [ "$funcstack[1]" = "_just" ]; then"#,
      r#"(( $+functions[_just_command_names] )) ||
_just_command_names() {
    local module=$1
    local output

    if [[ -n $module ]]; then
        output=$(_call_program commands just --list "$module") || return 1
    else
        output=$(_call_program commands just --list) || return 1
    fi

    local -a commands=()
    local line

    for line in "${(@f)output}"; do
        [[ $line == "    "* ]] || continue
        line=${line#"    "}
        commands+=("${line%%[[:space:]]*}")
    done

    print -rl -- "${commands[@]}"
}

(( $+functions[_just_module_path] )) ||
_just_module_path() {
    local -a modules=()
    local module=""
    local i

    for ((i = 2; i < CURRENT; i++)); do
        [[ -z ${words[i]} || ${words[i]} == -* || ${words[i]} == *=* ]] && continue
        modules+=("${words[i]}")
    done

    for i in "${modules[@]}"; do
        if [[ -n $module ]]; then
            module="${module}::${i}"
        else
            module="${i}"
        fi
    done

    print -r -- "$module"
}

(( $+functions[_just_recipe_path] )) ||
_just_recipe_path() {
    local -a recipes=(
        ${(s: :)$(_call_program commands just --summary)}
    )
    local -a path=()
    local candidate=""
    local i

    for ((i = 2; i < CURRENT; i++)); do
        [[ -z ${words[i]} || ${words[i]} == -* || ${words[i]} == *=* ]] && continue
        path+=("${words[i]}")
        if [[ -n $candidate ]]; then
            candidate="${candidate}::${words[i]}"
        else
            candidate="${words[i]}"
        fi
        if [[ ${recipes[(I)$candidate]} -gt 0 ]]; then
            print -r -- "$candidate"
            return 0
        fi
    done

    return 1
}

if [ "$funcstack[1]" = "_just" ]; then"#,
    ),
    (
      r#"_just "$@""#,
      r#"(( $+functions[_just_variables] )) ||
_just_variables() {
    [[ $PREFIX = -* ]] && return 1
    integer ret=1
    local variables; variables=(
        ${(s: :)$(_call_program commands just --variables)}
    )

    if compset -P '*='; then
        case "${${words[-1]%=*}#*=}" in
            *) _message 'value' && ret=0 ;;
        esac
    else
        _describe -t variables 'variables' variables && ret=0
    fi

    return ret
}

_just "$@""#,
    ),
  ];

  const POWERSHELL_COMPLETION_REPLACEMENTS: &[(&str, &str)] = &[(
    r#"$completions.Where{ $_.CompletionText -like "$wordToComplete*" } |
        Sort-Object -Property ListItemText"#,
    r#"function Get-JustBaseArgs([string[]]$CommandElements) {
        $justArgs = @()

        foreach ($flag in @("--justfile", "-f")) {
            $justFileIndex = $CommandElements.IndexOf($flag)

            if ($justFileIndex -ne -1 -and $justFileIndex + 1 -lt $CommandElements.Length) {
                $justFileLocation = $CommandElements[$justFileIndex + 1]

                if ($justFileLocation -and (Test-Path $justFileLocation)) {
                    $justArgs += @("--justfile", $justFileLocation)
                    break
                }
            }
        }

        return $justArgs
    }

    function Get-JustCommandWords([string[]]$CommandElements, [string]$WordToComplete) {
        $words = @("just")

        for ($i = 1; $i -lt $CommandElements.Length; $i++) {
            $value = $CommandElements[$i]

            if (-not $value -or $value.StartsWith('-') -or $value -eq $WordToComplete) {
                break
            }

            $words += $value
        }

        return $words
    }

    function Get-JustModulePath([string[]]$CommandWords) {
        if ($CommandWords.Length -le 1) {
            return $null
        }

        return [string]::Join("::", $CommandWords[1..($CommandWords.Length - 1)])
    }

    function Get-JustRecipePath([string[]]$CommandWords, [string[]]$CommandElements) {
        $justArgs = Get-JustBaseArgs -CommandElements $CommandElements
        $recipes = $(just @justArgs --summary) -split ' '
        $candidateParts = @()

        foreach ($word in $CommandWords[1..($CommandWords.Length - 1)]) {
            $candidateParts += $word
            $candidate = [string]::Join("::", $candidateParts)

            if ($recipes -contains $candidate) {
                return $candidate
            }
        }

        return $null
    }

    function Get-JustCommandNames([string[]]$CommandWords, [string[]]$CommandElements) {
        $justArgs = Get-JustBaseArgs -CommandElements $CommandElements
        $module = Get-JustModulePath -CommandWords $CommandWords
        $listArgs = @("--list")

        if ($module) {
            $listArgs += $module
        }

        $lines = just @justArgs @listArgs

        foreach ($line in $lines) {
            if ($line -like "    *") {
                $trimmed = $line.TrimStart()
                $name = ($trimmed -split '\s+', 2)[0]
                [CompletionResult]::new($name)
            }
        }
    }

    $elementValues = $commandElements | Select-Object -ExpandProperty Value
    $commandWords = Get-JustCommandWords -CommandElements $elementValues -WordToComplete $wordToComplete
    $recipe = Get-JustRecipePath -CommandWords $commandWords -CommandElements $elementValues

    if (-not $recipe) {
        $completions += Get-JustCommandNames -CommandWords $commandWords -CommandElements $elementValues
    }

    $completions.Where{ $_.CompletionText -like "$wordToComplete*" } |
        Sort-Object -Property ListItemText"#,
  )];

  const BASH_COMPLETION_REPLACEMENTS: &[(&str, &str)] = &[
    (
      r#"            if [[ ${cur} == -* || ${COMP_CWORD} -eq 1 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi"#,
      r#"                if [[ ${cur} == -* ]] ; then
                    COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                    return 0
                else
                    local recipe=$(__just_recipe_path)
                    local recipes=$(just --summary 2> /dev/null)

                    if echo "${cur}" | \grep -qF '/'; then
                        local path_prefix=$(echo "${cur}" | sed 's/[/][^/]*$/\//')
                        local recipes=$(just --summary 2> /dev/null -- "${path_prefix}")
                        local recipes=$(printf "${path_prefix}%s\t" $recipes)
                    elif [[ -z ${recipe} ]]; then
                        local module=$(__just_module_path)
                        local recipes=$(__just_command_names "${module}")
                    fi

                    if [[ $? -eq 0 ]]; then
                        COMPREPLY=( $(compgen -W "${recipes}" -- "${cur}") )
                        return 0
                    fi
                fi"#,
    ),
    (
      r"local i cur prev opts cmd",
      r"local i cur prev words cword opts cmd",
    ),
    (
      r#"    cur="${COMP_WORDS[COMP_CWORD]}"
    prev="${COMP_WORDS[COMP_CWORD-1]}""#,
      r#"
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
"#,
    ),
    (r"for i in ${COMP_WORDS[@]}", r"for i in ${words[@]}"),
    (
      r#"COMPREPLY=( $(compgen -W "${recipes}" -- "${cur}") )"#,
      r#"COMPREPLY=( $(compgen -W "${recipes}" -- "${cur}") )
                        if type __ltrim_colon_completions &>/dev/null; then
                            __ltrim_colon_completions "$cur"
                        fi"#,
    ),
    (
      r#"if [[ "${BASH_VERSINFO[0]}" -eq 4 && "${BASH_VERSINFO[1]}" -ge 4 || "${BASH_VERSINFO[0]}" -gt 4 ]]; then"#,
      r#"__just_command_names() {
    local module="$1"
    local output

    if [[ -n "${module}" ]]; then
        output=$(just --list "${module}" 2> /dev/null) || return 1
    else
        output=$(just --list 2> /dev/null) || return 1
    fi

    while IFS= read -r line; do
        [[ ${line} == "    "* ]] || continue
        line="${line#"    "}"
        printf '%s\n' "${line%%[[:space:]]*}"
    done <<< "${output}"
}

__just_module_path() {
    local module=""
    local i

    for ((i = 1; i < cword; i++)); do
        [[ -z ${words[i]} || ${words[i]} == -* || ${words[i]} == *=* ]] && continue

        if [[ -n ${module} ]]; then
            module="${module}::${words[i]}"
        else
            module="${words[i]}"
        fi
    done

    printf '%s\n' "${module}"
}

__just_recipe_path() {
    local candidate=""
    local i
    local recipes=$(just --summary 2> /dev/null) || return 1

    for ((i = 1; i < cword; i++)); do
        [[ -z ${words[i]} || ${words[i]} == -* || ${words[i]} == *=* ]] && continue

        if [[ -n ${candidate} ]]; then
            candidate="${candidate}::${words[i]}"
        else
            candidate="${words[i]}"
        fi

        if printf '%s\n' "${recipes}" | \grep -qxF "${candidate}"; then
            printf '%s\n' "${candidate}"
            return 0
        fi
    done

    return 1
}

if [[ "${BASH_VERSINFO[0]}" -eq 4 && "${BASH_VERSINFO[1]}" -ge 4 || "${BASH_VERSINFO[0]}" -gt 4 ]]; then"#,
    ),
  ];
}
