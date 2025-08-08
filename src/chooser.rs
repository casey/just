use crate::config::Config;
use crate::error::Error;
use crate::justfile::Justfile;
use crate::recipe::Recipe;
use crate::search::Search;
use skim::options::SkimOptionsBuilder;
use skim::prelude::SkimItemReader;
use skim::Skim;
use std::ffi::OsString;
use std::io::{Cursor, Write};
use std::process::Stdio;

pub enum Chooser<'a, 's: 'a> {
  Command {
    chooser: OsString,
    justfile: &'a Justfile<'s>,
    search: &'a Search,
    config: &'a Config,
  },
  Skim,
}

impl<'a> Chooser<'_, 'a> {
  fn run_with_command(self, recipes: Vec<&Recipe>) -> Result<Vec<String>, Error<'a>> {
    let Self::Command {
      chooser,
      justfile,
      search,
      config,
    } = self
    else {
      panic!("must be command chooser")
    };

    let result = justfile
      .settings
      .shell_command(config)
      .arg(&chooser)
      .current_dir(&search.working_directory)
      .stdin(Stdio::piped())
      .stdout(Stdio::piped())
      .spawn();

    let mut child = match result {
      Ok(child) => child,
      Err(io_error) => {
        let (shell_binary, shell_arguments) = justfile.settings.shell(config);
        return Err(Error::ChooserInvoke {
          shell_binary: shell_binary.to_owned(),
          shell_arguments: shell_arguments.join(" "),
          chooser,
          io_error,
        });
      }
    };

    let stdin = child.stdin.as_mut().unwrap();
    for recipe in recipes {
      if let Err(io_error) = writeln!(stdin, "{}", recipe.spaced_namepath()) {
        if io_error.kind() != std::io::ErrorKind::BrokenPipe {
          return Err(Error::ChooserWrite { io_error, chooser });
        }
      }
    }

    let output = match child.wait_with_output() {
      Ok(output) => output,
      Err(io_error) => {
        return Err(Error::ChooserRead { io_error, chooser });
      }
    };

    if !output.status.success() {
      return Err(Error::ChooserStatus {
        status: output.status,
        chooser,
      });
    }

    let stdout = String::from_utf8_lossy(&output.stdout);

    let recipes = stdout
      .split_whitespace()
      .map(str::to_owned)
      .collect::<Vec<String>>();

    Ok(recipes)
  }

  fn run_with_skim(recipes: Vec<&Recipe>) -> Result<Vec<String>, Error<'a>> {
    let options = SkimOptionsBuilder::default()
      .height(String::from("100%"))
      .multi(true)
      .preview(Some("just --unstable --color always --show {}".to_string()))
      .build()
      .unwrap();

    // `SkimItemReader` is a helper to turn any `BufRead` into a stream of `SkimItem`
    // `SkimItem` was implemented for `AsRef<str>` by default
    let item_reader = SkimItemReader::default();
    let items = item_reader.of_bufread(Cursor::new(
      recipes
        .iter()
        .map(|r| r.spaced_namepath())
        .collect::<Vec<_>>()
        .join("\n"),
    ));

    // `run_with` would read and show items from the stream
    let selected_items =
      Skim::run_with(&options, Some(items)).map_or_else(Vec::new, |out| out.selected_items);

    Ok(
      selected_items
        .iter()
        .map(|selected| selected.text().to_string())
        .collect(),
    )
  }

  pub fn run(self, recipe: Vec<&Recipe>) -> Result<Vec<String>, Error<'a>> {
    match self {
      Self::Skim => Self::run_with_skim(recipe),
      Self::Command { .. } => self.run_with_command(recipe),
    }
  }
}
