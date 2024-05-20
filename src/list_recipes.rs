use super::*;
const MAX_LINE_WIDTH: usize = 30;

pub(crate) fn list_groups(config: &Config, justfile: &Justfile) {
  println!("Recipe groups:");

  for group in justfile.public_groups() {
    println!("{}{group}", config.list_prefix);
  }
}

fn get_recipe_aliases<'a>(
  config: &Config,
  justfile: &'a Justfile,
) -> BTreeMap<&'a str, Vec<&'a str>> {
  let mut recipe_aliases: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
  if config.no_aliases {
    return recipe_aliases;
  }

  for alias in justfile.aliases.values() {
    if alias.is_private() {
      continue;
    }
    recipe_aliases
      .entry(alias.target.name.lexeme())
      .and_modify(|e| e.push(alias.name.lexeme()))
      .or_insert(vec![alias.name.lexeme()]);
  }
  recipe_aliases
}

fn get_line_widths<'a>(
  config: &Config,
  justfile: &'a Justfile,
  recipe_aliases: &BTreeMap<&'a str, Vec<&'a str>>,
) -> BTreeMap<&'a str, usize> {
  let mut line_widths: BTreeMap<&str, usize> = BTreeMap::new();

  for recipe in &justfile.public_recipes(config.unsorted) {
    let name = recipe.name.lexeme();

    for name in iter::once(&name).chain(recipe_aliases.get(name).unwrap_or(&Vec::new())) {
      let mut line_width = UnicodeWidthStr::width(*name);

      for parameter in &recipe.parameters {
        line_width +=
          UnicodeWidthStr::width(format!(" {}", parameter.color_display(Color::never())).as_str());
      }

      if line_width <= MAX_LINE_WIDTH {
        line_widths.insert(name, line_width);
      }
    }
  }

  line_widths
}

fn print_recipe(
  recipe: &Recipe,
  aliases: &[&str],
  level: usize,
  config: &Config,
  line_widths: &BTreeMap<&str, usize>,
  max_line_width: usize,
) {
  let name = recipe.name();
  let doc_color = config.color.stdout().doc();

  for (i, name) in iter::once(&name).chain(aliases).enumerate() {
    print!("{}{name}", config.list_prefix.repeat(level + 1));
    for parameter in &recipe.parameters {
      print!(" {}", parameter.color_display(config.color.stdout()));
    }

    let padding =
      max_line_width.saturating_sub(line_widths.get(name).copied().unwrap_or(max_line_width));
    match (i, recipe.doc) {
      (0, Some(doc)) => print_doc_comment(doc, padding, doc_color),
      (0, None) => (),
      _ => {
        let alias_doc = format!("alias for `{}`", recipe.name);
        print_doc_comment(&alias_doc, padding, doc_color);
      }
    }
    println!();
  }
}

fn print_doc_comment(doc: &str, padding: usize, doc_color: Color) {
  print!(
    " {:padding$}{} {}",
    "",
    doc_color.paint("#"),
    doc_color.paint(doc),
    padding = padding
  );
}

fn recipes_by_group<'a>(
  justfile: &'a Justfile,
  sort_order: bool,
) -> BTreeMap<Option<String>, Vec<&'a Recipe<'a>>> {
  let mut by_groups: BTreeMap<Option<String>, Vec<&Recipe<'_>>> = BTreeMap::new();

  for recipe in justfile.public_recipes(sort_order) {
    let groups = recipe.groups();
    if groups.is_empty() {
      by_groups
        .entry(None)
        .and_modify(|e| e.push(recipe))
        .or_insert(vec![recipe]);
    } else {
      for group in groups {
        by_groups
          .entry(Some(group))
          .and_modify(|e| e.push(recipe))
          .or_insert(vec![recipe]);
      }
    }
  }
  by_groups
}

pub(crate) fn list_recipes(config: &Config, level: usize, justfile: &Justfile) {
  let recipe_aliases = get_recipe_aliases(config, justfile);
  let line_widths = get_line_widths(config, justfile, &recipe_aliases);
  let max_line_width = cmp::min(
    line_widths.values().copied().max().unwrap_or(0),
    MAX_LINE_WIDTH,
  );

  if level == 0 {
    print!("{}", config.list_heading);
  }

  let by_groups = recipes_by_group(justfile, config.unsorted);
  let no_recipes_in_group = by_groups.contains_key(&None) && by_groups.len() == 1;

  for (group, recipes) in &by_groups {
    if !no_recipes_in_group {
      println!();
      if let Some(group_name) = group {
        println!("[{group_name}]");
      } else {
        println!("(no group)");
      }
    }
    for recipe in recipes {
      let aliases: &[&str] = recipe_aliases
        .get(recipe.name())
        .map_or(&[], |v| v.as_slice());
      print_recipe(recipe, aliases, level, config, &line_widths, max_line_width);
    }
  }

  for (name, module) in &justfile.modules {
    println!("    {name}:");
    list_recipes(config, level + 1, module);
  }
}
