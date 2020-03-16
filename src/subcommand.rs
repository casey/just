use crate::common::*;

#[derive(PartialEq, Clone, Debug)]
pub(crate) enum Subcommand {
  Completions {
    shell: String,
  },
  Dump,
  Edit,
  Evaluate {
    overrides: BTreeMap<String, String>,
  },
  Init,
  List,
  Run {
    overrides: BTreeMap<String, String>,
    arguments: Vec<String>,
  },
  Show {
    name: String,
  },
  Summary,
  Variables,
}

// TODO: make this a lookup table
const COMPLETION_REPLACEMENTS: &[(&str, &str)] = &[
  (
    r#"    _arguments "${_arguments_options[@]}" \"#,
    r#"    local common=("#,
  ),
  (
    r#"'*--set=[Override <VARIABLE> with <VALUE>]' \"#,
    r#"'*--set[Override <VARIABLE> with <VALUE>]: :_just_variables' \"#,
  ),
  (
    r#"'-s+[Show information about <RECIPE>]' \
'--show=[Show information about <RECIPE>]' \"#,
    r#"'-s+[Show information about <RECIPE>]: :_just_commands' \
'--show=[Show information about <RECIPE>]: :_just_commands' \"#,
  ),
  (
    "    local commands; commands=(
\x20\x20\x20\x20\x20\x20\x20\x20
    )",
    r#"    local commands; commands=(
        ${${${(M)"${(f)$(_call_program commands just --list)}":#    *}/ ##/}/ ##/:Args: }
    )"#,
  ),
  (
    r#"_just "$@""#,
    r#"(( $+functions[_just_variables] )) ||
_just_variables() {
    local variables; variables=(
        ${(s: :)$(_call_program commands just --variables)}
    )

    _describe -t variables 'variables' variables
}

_just "$@""#,
  ),
];

impl Subcommand {
  pub(crate) fn completions(shell: &str) -> Result<(), i32> {
    fn replace(haystack: &mut String, needle: &str, replacement: &str) -> Result<(), i32> {
      if let Some(index) = haystack.find(needle) {
        haystack.replace_range(index..index + needle.len(), replacement);
        Ok(())
      } else {
        eprintln!("Failed to find text:");
        eprintln!("{}", needle);
        eprintln!("…in completion script:");
        eprintln!("{}", haystack);
        Err(EXIT_FAILURE)
      }
    }

    let shell = shell
      .parse::<clap::Shell>()
      .expect("Invalid value for clap::Shell");

    let buffer = Vec::new();
    let mut cursor = Cursor::new(buffer);
    Config::app().gen_completions_to(env!("CARGO_PKG_NAME"), shell, &mut cursor);
    let buffer = cursor.into_inner();
    let mut script = String::from_utf8(buffer).expect("Clap completion not UTF-8");

    if let clap::Shell::Zsh = shell {
      for (needle, replacement) in COMPLETION_REPLACEMENTS {
        replace(&mut script, needle, replacement)?;
      }
    }

    println!("{}", script.trim());

    Ok(())
  }
}
