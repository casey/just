#![no_main]
#[macro_use] extern crate libfuzzer_sys;
extern crate just;

use std::io;

enum Error<'a> {
  Io(io::Error),
  Compile(just::CompilationError<'a>),
}

impl<'a> From<io::Error> for Error<'a> {
  fn from(error: io::Error) -> Error<'a> {
    Error::Io(error)
  }
}

impl<'a> From<just::CompilationError<'a>> for Error<'a> {
  fn from(error: just::CompilationError<'a>) -> Error<'a> {
    Error::Compile(error)
  }
}

fn lex_and_parse<'a>(input: &'a str) -> Result<(), Error<'a>> {
  let tokens = just::Lexer::lex(input)?;
  let parser = just::Parser::new(input, tokens);
  let _justfile = parser.justfile()?;
  Ok(())
}
fuzz_target!(|data: &[u8]| {
  let input = String::from_utf8_lossy(&data);
  lex_and_parse(&input).ok();
});
