use {
  super::*,
  Function::*,
  heck::{
    ToKebabCase, ToLowerCamelCase, ToShoutyKebabCase, ToShoutySnakeCase, ToSnakeCase, ToTitleCase,
    ToUpperCamelCase,
  },
  semver::{Version, VersionReq},
};

#[allow(clippy::arbitrary_source_item_ordering)]
pub(crate) enum Function {
  Nullary(fn(Context) -> StringResult),
  Unary(fn(Context, &str) -> StringResult),
  UnaryMap(fn(Context, &str) -> StringResult),
  UnaryPlus(fn(Context, &str, &[String]) -> StringResult),
  UnaryToValue(fn(Context, &str) -> ValueResult),
  Binary(fn(Context, &str, &str) -> StringResult),
  BinaryOptToValue(fn(Context, &str, Option<&str>) -> ValueResult),
  BinaryOptValueStr(fn(Context, &Value, Option<&str>) -> StringResult),
  BinaryOptValueStrToValue(fn(Context, &Value, Option<&str>) -> ValueResult),
  BinaryPlus(fn(Context, &str, &str, &[String]) -> StringResult),
  BinaryStrValue(fn(Context, &str, &Value) -> ValueResult),
  BinaryToValue(fn(Context, &str, &str) -> ValueResult),
  Ternary(fn(Context, &str, &str, &str) -> StringResult),
  ValueNullary(fn(Context) -> ValueResult),
  ValueUnary(fn(Context, &Value) -> ValueResult),
  ValueBinary(fn(Context, &Value, &Value) -> ValueResult),
  ValueBinaryOpt(fn(Context, &Value, Option<&Value>) -> ValueResult),
}

impl Function {
  pub(crate) fn expected_arguments(&self) -> RangeInclusive<usize> {
    match self {
      Nullary(_) | ValueNullary(_) => 0..=0,
      Unary(_) | ValueUnary(_) | UnaryMap(_) | UnaryToValue(_) => 1..=1,
      ValueBinaryOpt(_)
      | BinaryOptToValue(_)
      | BinaryOptValueStrToValue(_)
      | BinaryOptValueStr(_) => 1..=2,
      UnaryPlus(_) => 1..=usize::MAX,
      Binary(_) | BinaryStrValue(_) | ValueBinary(_) | BinaryToValue(_) => 2..=2,
      BinaryPlus(_) => 2..=usize::MAX,
      Ternary(_) => 3..=3,
    }
  }
}

pub(crate) struct Context<'src: 'run, 'run> {
  pub(crate) execution_context: &'run ExecutionContext<'src, 'run>,
  pub(crate) is_dependency: bool,
  pub(crate) name: Name<'src>,
  pub(crate) recipe: Option<Name<'src>>,
  pub(crate) scope: &'run Scope<'src, 'run>,
}

pub(crate) fn get(name: &str) -> Option<Function> {
  let name = if let Some(prefix) = name.strip_suffix("_dir") {
    format!("{prefix}_directory")
  } else if let Some(prefix) = name.strip_suffix("_dir_native") {
    format!("{prefix}_directory_native")
  } else {
    name.into()
  };

  let function = match name.as_str() {
    "absolute_path" => UnaryMap(absolute_path),
    "append" => BinaryStrValue(append),
    "arch" => Nullary(arch),
    "blake3" => Unary(blake3),
    "blake3_file" => Unary(blake3_file),
    "bool" => ValueUnary(bool),
    "cache_directory" => Nullary(|_| dir("cache", dirs::cache_dir)),
    "canonicalize" => Unary(canonicalize),
    "capitalize" => Unary(capitalize),
    "choose" => Binary(choose),
    "clean" => Unary(clean),
    "config_directory" => Nullary(|_| dir("config", dirs::config_dir)),
    "config_local_directory" => Nullary(|_| dir("local config", dirs::config_local_dir)),
    "data_directory" => Nullary(|_| dir("data", dirs::data_dir)),
    "data_local_directory" => Nullary(|_| dir("local data", dirs::data_local_dir)),
    "datetime" => Unary(datetime),
    "datetime_utc" => Unary(datetime_utc),
    "encode_uri_component" => Unary(encode_uri_component),
    "env" => ValueBinaryOpt(env),
    "env_var" => ValueUnary(env_var),
    "env_var_or_default" => ValueBinary(env_var_or_default),
    "error" => Unary(error),
    "executable_directory" => Nullary(|_| dir("executable", dirs::executable_dir)),
    "extension" => Unary(extension),
    "file_name" => Unary(file_name),
    "file_stem" => Unary(file_stem),
    "home_directory" => Nullary(|_| dir("home", dirs::home_dir)),
    "invocation_directory" => Nullary(invocation_directory),
    "invocation_directory_native" => Nullary(invocation_directory_native),
    "is_dependency" => ValueNullary(is_dependency),
    "join" => BinaryPlus(join),
    "join_list" => BinaryOptValueStrToValue(join_list),
    "just_executable" => Nullary(just_executable),
    "just_pid" => Nullary(just_pid),
    "just_version" => Nullary(just_version),
    "justfile" => Nullary(justfile),
    "justfile_directory" => Nullary(justfile_directory),
    "kebabcase" => Unary(kebabcase),
    "lowercamelcase" => Unary(lowercamelcase),
    "lowercase" => Unary(lowercase),
    "module_directory" => Nullary(module_directory),
    "module_file" => Nullary(module_file),
    "module_path" => Nullary(module_path),
    "num_cpus" => Nullary(num_cpus),
    "num_jobs" => ValueNullary(num_jobs),
    "os" => Nullary(os),
    "os_family" => Nullary(os_family),
    "parent_directory" => Unary(parent_directory),
    "path_exists" => UnaryToValue(path_exists),
    "prepend" => BinaryStrValue(prepend),
    "quote" => UnaryMap(quote),
    "read" => Unary(read),
    "recipe_name" => Nullary(recipe_name),
    "replace" => Ternary(replace),
    "replace_regex" => Ternary(replace_regex),
    "require" => Unary(require),
    "runtime_directory" => Nullary(|_| dir("runtime", dirs::runtime_dir)),
    "semver_matches" => BinaryToValue(semver_matches),
    "sha256" => Unary(sha256),
    "sha256_file" => Unary(sha256_file),
    "shell" => UnaryPlus(shell),
    "shoutykebabcase" => Unary(shoutykebabcase),
    "shoutysnakecase" => Unary(shoutysnakecase),
    "show" => ValueUnary(show),
    "snakecase" => Unary(snakecase),
    "source_directory" => Nullary(source_directory),
    "source_file" => Nullary(source_file),
    "split" => BinaryOptToValue(split),
    "style" => BinaryOptValueStr(style),
    "titlecase" => Unary(titlecase),
    "trim" => Unary(trim),
    "trim_end" => Unary(trim_end),
    "trim_end_match" => Binary(trim_end_match),
    "trim_end_matches" => Binary(trim_end_matches),
    "trim_start" => Unary(trim_start),
    "trim_start_match" => Binary(trim_start_match),
    "trim_start_matches" => Binary(trim_start_matches),
    "uppercamelcase" => Unary(uppercamelcase),
    "uppercase" => Unary(uppercase),
    "uuid" => Nullary(uuid),
    "which" => UnaryToValue(which),
    "without_extension" => Unary(without_extension),
    _ => return None,
  };
  Some(function)
}

fn boolean(context: &Context, condition: bool) -> Value {
  if context.execution_context.module.settings.lists {
    condition.into()
  } else {
    Value::from(condition.to_string())
  }
}

fn bool(context: Context, value: &Value) -> ValueResult {
  let condition = match value.elements() {
    [] => false,
    [element] => match element.as_str() {
      "" | "0" | "false" => false,
      "1" | "true" => true,
      _ => return Err(format!("`{element}` is not a valid boolean string")),
    },
    _ => {
      return Err("multi-element lists cannot be converted into booleans".into());
    }
  };
  Ok(boolean(&context, condition))
}

fn absolute_path(context: Context, path: &str) -> StringResult {
  let abs_path_unchecked = context
    .execution_context
    .working_directory()
    .join(path)
    .lexiclean();
  match abs_path_unchecked.to_str() {
    Some(absolute_path) => Ok(absolute_path.to_owned()),
    None => Err(format!(
      "working directory is not valid unicode: {}",
      context.execution_context.search.working_directory.display()
    )),
  }
}

fn append(context: Context, suffix: &str, s: &Value) -> ValueResult {
  Ok(if context.execution_context.module.settings.lists {
    s.elements()
      .iter()
      .map(|element| format!("{element}{suffix}"))
      .collect()
  } else {
    s.join()
      .split_whitespace()
      .map(|element| format!("{element}{suffix}"))
      .collect::<Vec<String>>()
      .join(" ")
      .into()
  })
}

fn arch(_context: Context) -> StringResult {
  Ok(env::consts::ARCH.to_owned())
}

fn blake3(_context: Context, s: &str) -> StringResult {
  Ok(blake3::hash(s.as_bytes()).to_string())
}

fn blake3_file(context: Context, path: &str) -> StringResult {
  let path = context.execution_context.working_directory().join(path);
  let mut hasher = blake3::Hasher::new();
  hasher
    .update_mmap_rayon(&path)
    .map_err(|err| format!("failed to hash `{}`: {err}", path.display()))?;
  Ok(hasher.finalize().to_string())
}

fn canonicalize(context: Context, path: &str) -> StringResult {
  let canonical = std::fs::canonicalize(context.execution_context.working_directory().join(path))
    .map_err(|err| format!("I/O error canonicalizing path: {err}"))?;

  canonical.to_str().map(str::to_string).ok_or_else(|| {
    format!(
      "canonical path is not valid unicode: {}",
      canonical.display(),
    )
  })
}

fn capitalize(_context: Context, s: &str) -> StringResult {
  let mut capitalized = String::new();
  for (i, c) in s.chars().enumerate() {
    if i == 0 {
      capitalized.extend(c.to_uppercase());
    } else {
      capitalized.extend(c.to_lowercase());
    }
  }
  Ok(capitalized)
}

fn choose(_context: Context, n: &str, alphabet: &str) -> StringResult {
  let mut chars = HashSet::<char>::with_capacity(alphabet.len());

  for c in alphabet.chars() {
    if !chars.insert(c) {
      return Err(format!("alphabet contains repeated character `{c}`"));
    }
  }

  let alphabet = alphabet.chars().collect::<Vec<char>>();

  let n = n
    .parse::<usize>()
    .map_err(|err| format!("failed to parse `{n}` as positive integer: {err}"))?;

  let mut rng = rand::rng();

  (0..n)
    .map(|_| {
      alphabet
        .choose(&mut rng)
        .ok_or_else(|| "empty alphabet".to_string())
    })
    .collect()
}

fn clean(_context: Context, path: &str) -> StringResult {
  Ok(Path::new(path).lexiclean().to_str().unwrap().to_owned())
}

fn dir(name: &'static str, f: fn() -> Option<PathBuf>) -> StringResult {
  match f() {
    Some(path) => path
      .as_os_str()
      .to_str()
      .map(str::to_string)
      .ok_or_else(|| {
        format!(
          "unable to convert {name} directory path to string: {}",
          path.display(),
        )
      }),
    None => Err(format!("{name} directory not found")),
  }
}

fn datetime(_context: Context, format: &str) -> StringResult {
  datetime_format(Local::now(), format).map_err(|err| err.to_string())
}

fn datetime_utc(_context: Context, format: &str) -> StringResult {
  datetime_format(Utc::now(), format).map_err(|err| err.to_string())
}

fn encode_uri_component(_context: Context, s: &str) -> StringResult {
  static PERCENT_ENCODE: percent_encoding::AsciiSet = percent_encoding::NON_ALPHANUMERIC
    .remove(b'-')
    .remove(b'_')
    .remove(b'.')
    .remove(b'!')
    .remove(b'~')
    .remove(b'*')
    .remove(b'\'')
    .remove(b'(')
    .remove(b')');
  Ok(percent_encoding::utf8_percent_encode(s, &PERCENT_ENCODE).to_string())
}

fn env(context: Context, keys: &Value, default: Option<&Value>) -> ValueResult {
  for key in keys {
    if let Some(value) = context.execution_context.dotenv.get(key) {
      return Ok(value.into());
    }

    match env::var(key) {
      Ok(value) => return Ok(value.into()),
      Err(VarError::NotPresent) => {}
      Err(VarError::NotUnicode(value)) => {
        return Err(format!(
          "environment variable `{key}` is not unicode: `{}`",
          value.to_string_lossy(),
        ));
      }
    }
  }

  if let Some(default) = default {
    return Ok(default.clone());
  }

  if keys.is_empty() {
    return Err("empty environment variable list with no default".into());
  }

  Err(format!(
    "{} {} not present",
    Count::unnumbered("environment variable", keys.elements().len()),
    List::and_ticked(keys.elements()),
  ))
}

fn env_var(context: Context, keys: &Value) -> ValueResult {
  env(context, keys, None)
}

fn env_var_or_default(context: Context, keys: &Value, default: &Value) -> ValueResult {
  env(context, keys, Some(default))
}

fn error(_context: Context, message: &str) -> StringResult {
  Err(message.to_owned())
}

fn extension(_context: Context, path: &str) -> StringResult {
  Utf8Path::new(path)
    .extension()
    .map(str::to_owned)
    .ok_or_else(|| format!("could not extract extension from `{path}`"))
}

fn file_name(_context: Context, path: &str) -> StringResult {
  Utf8Path::new(path)
    .file_name()
    .map(str::to_owned)
    .ok_or_else(|| format!("could not extract file name from `{path}`"))
}

fn file_stem(_context: Context, path: &str) -> StringResult {
  Utf8Path::new(path)
    .file_stem()
    .map(str::to_owned)
    .ok_or_else(|| format!("could not extract file stem from `{path}`"))
}

fn invocation_directory(context: Context) -> StringResult {
  Platform::convert_native_path(
    context.execution_context.config,
    &context.execution_context.search.working_directory,
    &context.execution_context.config.invocation_directory,
  )
  .map_err(|e| format!("error getting shell path: {e}"))
}

fn invocation_directory_native(context: Context) -> StringResult {
  context
    .execution_context
    .config
    .invocation_directory
    .to_str()
    .map(str::to_owned)
    .ok_or_else(|| {
      format!(
        "invocation directory is not valid unicode: {}",
        context
          .execution_context
          .config
          .invocation_directory
          .display()
      )
    })
}

fn is_dependency(context: Context) -> ValueResult {
  Ok(boolean(&context, context.is_dependency))
}

fn prepend(context: Context, prefix: &str, s: &Value) -> ValueResult {
  Ok(if context.execution_context.module.settings.lists {
    s.elements()
      .iter()
      .map(|element| format!("{prefix}{element}"))
      .collect()
  } else {
    s.join()
      .split_whitespace()
      .map(|element| format!("{prefix}{element}"))
      .collect::<Vec<String>>()
      .join(" ")
      .into()
  })
}

fn join(_context: Context, base: &str, with: &str, and: &[String]) -> StringResult {
  let mut result = Utf8Path::new(base).join(with);
  for arg in and {
    result.push(arg);
  }
  Ok(result.to_string())
}

fn join_list(_context: Context, value: &Value, separator: Option<&str>) -> ValueResult {
  Ok(value.elements().join(separator.unwrap_or(" ")).into())
}

fn just_executable(_context: Context) -> StringResult {
  let exe_path =
    env::current_exe().map_err(|e| format!("error getting current executable: {e}"))?;

  exe_path.to_str().map(str::to_owned).ok_or_else(|| {
    format!(
      "executable path is not valid unicode: {}",
      exe_path.display()
    )
  })
}

fn just_pid(_context: Context) -> StringResult {
  Ok(std::process::id().to_string())
}

fn just_version(_context: Context) -> StringResult {
  Ok(VERSION.into())
}

fn justfile(context: Context) -> StringResult {
  context
    .execution_context
    .search
    .justfile
    .to_str()
    .map(str::to_owned)
    .ok_or_else(|| {
      format!(
        "justfile path is not valid unicode: {}",
        context.execution_context.search.justfile.display()
      )
    })
}

fn justfile_directory(context: Context) -> StringResult {
  let justfile_directory = context
    .execution_context
    .search
    .justfile
    .parent()
    .ok_or_else(|| {
      format!(
        "could not resolve justfile directory, justfile `{}` had no parent",
        context.execution_context.search.justfile.display()
      )
    })?;

  justfile_directory
    .to_str()
    .map(str::to_owned)
    .ok_or_else(|| {
      format!(
        "justfile directory is not valid unicode: {}",
        justfile_directory.display()
      )
    })
}

fn kebabcase(_context: Context, s: &str) -> StringResult {
  Ok(s.to_kebab_case())
}

fn lowercamelcase(_context: Context, s: &str) -> StringResult {
  Ok(s.to_lower_camel_case())
}

fn lowercase(_context: Context, s: &str) -> StringResult {
  Ok(s.to_lowercase())
}

fn module_directory(context: Context) -> StringResult {
  let module_directory = context.execution_context.module.source.parent().unwrap();
  module_directory.to_str().map(str::to_owned).ok_or_else(|| {
    format!(
      "module directory is not valid unicode: {}",
      module_directory.display(),
    )
  })
}

fn module_file(context: Context) -> StringResult {
  let module_file = &context.execution_context.module.source;
  module_file.to_str().map(str::to_owned).ok_or_else(|| {
    format!(
      "module file path is not valid unicode: {}",
      module_file.display(),
    )
  })
}

fn module_path(context: Context) -> StringResult {
  Ok(context.execution_context.module.module_path.to_string())
}

fn num_cpus(_context: Context) -> StringResult {
  let num = num_cpus::get();
  Ok(num.to_string())
}

fn num_jobs(context: Context) -> ValueResult {
  Ok(
    context
      .execution_context
      .config
      .jobs
      .map(|jobs| Value::from(jobs.to_string()))
      .unwrap_or_default(),
  )
}

fn os(_context: Context) -> StringResult {
  Ok(env::consts::OS.to_owned())
}

fn os_family(_context: Context) -> StringResult {
  Ok(env::consts::FAMILY.to_owned())
}

fn parent_directory(_context: Context, path: &str) -> StringResult {
  let parent = Utf8Path::new(path)
    .parent()
    .map(Utf8Path::to_string)
    .ok_or_else(|| format!("could not extract parent directory from `{path}`"))?;

  if parent.is_empty() {
    Ok(".".into())
  } else {
    Ok(parent)
  }
}

fn path_exists(context: Context, path: &str) -> ValueResult {
  Ok(boolean(
    &context,
    context
      .execution_context
      .working_directory()
      .join(path)
      .exists(),
  ))
}

fn quote(_context: Context, s: &str) -> StringResult {
  Ok(format!("'{}'", s.replace('\'', "'\\''")))
}

fn read(context: Context, filename: &str) -> StringResult {
  fs::read_to_string(context.execution_context.working_directory().join(filename))
    .map_err(|err| format!("I/O error reading `{filename}`: {err}"))
}

fn recipe_name(context: Context) -> StringResult {
  context
    .recipe
    .map(|name| name.lexeme().into())
    .ok_or_else(|| "`recipe_name()` can only be used within a recipe".into())
}

fn replace(_context: Context, s: &str, from: &str, to: &str) -> StringResult {
  Ok(s.replace(from, to))
}

fn require(context: Context, name: &str) -> StringResult {
  crate::which(&context, name)?.ok_or_else(|| format!("could not find executable `{name}`"))
}

fn replace_regex(_context: Context, s: &str, regex: &str, replacement: &str) -> StringResult {
  Ok(
    Regex::new(regex)
      .map_err(|err| err.to_string())?
      .replace_all(s, replacement)
      .to_string(),
  )
}

fn sha256(_context: Context, s: &str) -> StringResult {
  let mut hasher = Sha256::new();
  hasher.update(s);
  Ok(hex::encode(hasher.finalize()))
}

fn sha256_file(context: Context, path: &str) -> StringResult {
  let path = context.execution_context.working_directory().join(path);
  let mut file =
    File::open(&path).map_err(|err| format!("failed to open `{}`: {err}", path.display()))?;
  let mut writer = HashWriter::<Sha256, Sink>::new(io::sink());
  io::copy(&mut file, &mut writer)
    .map_err(|err| format!("failed to read `{}`: {err}", path.display()))?;
  Ok(hex::encode(writer.finalize()))
}

fn shell(context: Context, command: &str, args: &[String]) -> StringResult {
  Evaluator::run_command(
    context.execution_context,
    &BTreeMap::new(),
    context.scope,
    command,
    Some(args),
  )
  .map_err(|output_error| output_error.to_string())
}

fn shoutykebabcase(_context: Context, s: &str) -> StringResult {
  Ok(s.to_shouty_kebab_case())
}

fn shoutysnakecase(_context: Context, s: &str) -> StringResult {
  Ok(s.to_shouty_snake_case())
}

fn show(_context: Context, value: &Value) -> ValueResult {
  Ok(value.color_display(Color::never()).to_string().into())
}

fn snakecase(_context: Context, s: &str) -> StringResult {
  Ok(s.to_snake_case())
}

fn source_directory(context: Context) -> StringResult {
  context
    .execution_context
    .search
    .justfile
    .parent()
    .unwrap()
    .join(context.name.token.path)
    .parent()
    .unwrap()
    .to_str()
    .map(str::to_owned)
    .ok_or_else(|| {
      format!(
        "source file path is not valid unicode: {}",
        context.name.token.path.display(),
      )
    })
}

fn source_file(context: Context) -> StringResult {
  context
    .execution_context
    .search
    .justfile
    .parent()
    .unwrap()
    .join(context.name.token.path)
    .to_str()
    .map(str::to_owned)
    .ok_or_else(|| {
      format!(
        "source file path is not valid unicode: {}",
        context.name.token.path.display(),
      )
    })
}

fn split(_context: Context, s: &str, separator: Option<&str>) -> ValueResult {
  Ok(if let Some(separator) = separator {
    s.split(separator).map(str::to_string).collect()
  } else {
    s.split_whitespace().map(str::to_string).collect()
  })
}

fn style(context: Context, styles: &Value, text: Option<&str>) -> StringResult {
  use nu_ansi_term::Color::*;

  static FIXED: LazyLock<Regex> =
    LazyLock::new(|| Regex::new("^(fg:|bg:)?(0|[1-9][0-9]{0,2})$").unwrap());

  static RGB_LONG: LazyLock<Regex> =
    LazyLock::new(|| Regex::new("^(fg:|bg:)?#([[:xdigit:]]{6})$").unwrap());

  static RGB_SHORT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new("^(fg:|bg:)?#([[:xdigit:]]{3})$").unwrap());

  fn layer(captures: regex::Captures) -> Layer {
    match captures.get(1).map(|capture| capture.as_str()) {
      Some("bg:") => Layer::Background,
      Some("fg:") | None => Layer::Foreground,
      _ => unreachable!(),
    }
  }

  let config = context.execution_context.config;

  let mut style = Style::new();
  let mut active = true;

  for token in styles {
    let error = || format!("invalid style: `{token}`");

    if let Some(captures) = FIXED.captures(token) {
      let Ok(color) = captures[2].parse::<u8>() else {
        return Err(error());
      };
      style.color(Fixed(color), layer(captures));
    } else if let Some(captures) = RGB_LONG.captures(token) {
      let [_, r, g, b] = u32::from_str_radix(&captures[2], 16).unwrap().to_be_bytes();
      style.color(Rgb(r, g, b), layer(captures));
    } else if let Some(captures) = RGB_SHORT.captures(token) {
      let n = u16::from_str_radix(&captures[2], 16).unwrap();
      let r = u8::try_from((n >> 8) & 0xf).unwrap() * 0x11;
      let g = u8::try_from((n >> 4) & 0xf).unwrap() * 0x11;
      let b = u8::try_from(n & 0xf).unwrap() * 0x11;
      style.color(Rgb(r, g, b), layer(captures));
    } else {
      match token.as_str() {
        // foreground
        "fg:black" | "black" => style.fg(Black),
        "fg:blue" | "blue" => style.fg(Blue),
        "fg:cyan" | "cyan" => style.fg(Cyan),
        "fg:green" | "green" => style.fg(Green),
        "fg:magenta" | "magenta" => style.fg(Magenta),
        "fg:red" | "red" => style.fg(Red),
        "fg:white" | "white" => style.fg(White),
        "fg:yellow" | "yellow" => style.fg(Yellow),
        // background
        "bg:black" => style.bg(Black),
        "bg:blue" => style.bg(Blue),
        "bg:cyan" => style.bg(Cyan),
        "bg:green" => style.bg(Green),
        "bg:magenta" => style.bg(Magenta),
        "bg:red" => style.bg(Red),
        "bg:white" => style.bg(White),
        "bg:yellow" => style.bg(Yellow),
        // properties
        "blink" => style.blink(),
        "bold" => style.bold(),
        "dim" => style.dim(),
        "hidden" => style.hidden(),
        "italic" => style.italic(),
        "reverse" => style.reverse(),
        "strikethrough" => style.strikethrough(),
        "underline" => style.underline(),
        // streams
        "stdout" => active = config.color.stdout().active(),
        "stderr" => active = config.color.stderr().active(),
        // roles
        "command" => {
          if let Some(color) = config.command_color {
            style.fg(color);
          }
          style.bold();
        }
        "error" => {
          style.fg(Red);
          style.bold();
        }
        "warning" => {
          style.fg(Yellow);
          style.bold();
        }
        _ => return Err(error()),
      }
    }
  }

  Ok(if active {
    match text {
      Some(text) => style.paint(text),
      None => style.prefix(),
    }
  } else {
    match text {
      Some(text) => text.into(),
      None => String::new(),
    }
  })
}

fn titlecase(_context: Context, s: &str) -> StringResult {
  Ok(s.to_title_case())
}

fn trim(_context: Context, s: &str) -> StringResult {
  Ok(s.trim().to_owned())
}

fn trim_end(_context: Context, s: &str) -> StringResult {
  Ok(s.trim_end().to_owned())
}

fn trim_end_match(_context: Context, s: &str, pat: &str) -> StringResult {
  Ok(s.strip_suffix(pat).unwrap_or(s).to_owned())
}

fn trim_end_matches(_context: Context, s: &str, pat: &str) -> StringResult {
  Ok(s.trim_end_matches(pat).to_owned())
}

fn trim_start(_context: Context, s: &str) -> StringResult {
  Ok(s.trim_start().to_owned())
}

fn trim_start_match(_context: Context, s: &str, pat: &str) -> StringResult {
  Ok(s.strip_prefix(pat).unwrap_or(s).to_owned())
}

fn trim_start_matches(_context: Context, s: &str, pat: &str) -> StringResult {
  Ok(s.trim_start_matches(pat).to_owned())
}

fn uppercamelcase(_context: Context, s: &str) -> StringResult {
  Ok(s.to_upper_camel_case())
}

fn uppercase(_context: Context, s: &str) -> StringResult {
  Ok(s.to_uppercase())
}

fn uuid(_context: Context) -> StringResult {
  Ok(uuid::Uuid::new_v4().to_string())
}

fn which(context: Context, name: &str) -> ValueResult {
  Ok(match crate::which(&context, name)? {
    Some(path) => Value::from(path),
    None => Value::new(),
  })
}

fn without_extension(_context: Context, path: &str) -> StringResult {
  let parent = Utf8Path::new(path)
    .parent()
    .ok_or_else(|| format!("could not extract parent from `{path}`"))?;

  let file_stem = Utf8Path::new(path)
    .file_stem()
    .ok_or_else(|| format!("could not extract file stem from `{path}`"))?;

  Ok(parent.join(file_stem).to_string())
}

/// Check whether a string processes properly as semver (e.x. "0.1.0")
/// and matches a given semver requirement (e.x. ">=0.1.0")
fn semver_matches(context: Context, version: &str, requirement: &str) -> ValueResult {
  Ok(boolean(
    &context,
    requirement
      .parse::<VersionReq>()
      .map_err(|err| format!("invalid semver requirement: {err}"))?
      .matches(
        &version
          .parse::<Version>()
          .map_err(|err| format!("invalid semver version: {err}"))?,
      ),
  ))
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn dir_not_found() {
    assert_eq!(dir("foo", || None).unwrap_err(), "foo directory not found");
  }

  #[cfg(unix)]
  #[test]
  fn dir_not_unicode() {
    use std::os::unix::ffi::OsStrExt;
    assert_eq!(
      dir("foo", || Some(
        std::ffi::OsStr::from_bytes(b"\xe0\x80\x80").into()
      ))
      .unwrap_err(),
      "unable to convert foo directory path to string: ���",
    );
  }
}
