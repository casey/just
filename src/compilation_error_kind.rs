use crate::common::*;

#[derive(Debug, PartialEq)]
pub(crate) enum CompilationErrorKind<'src> {
  AliasShadowsRecipe {
    alias:       &'src str,
    recipe_line: usize,
  },
  CircularRecipeDependency {
    recipe: &'src str,
    circle: Vec<&'src str>,
  },
  CircularVariableDependency {
    variable: &'src str,
    circle:   Vec<&'src str>,
  },
  DependencyArgumentCountMismatch {
    dependency: &'src str,
    found:      usize,
    min:        usize,
    max:        usize,
  },
  DuplicateAlias {
    alias: &'src str,
    first: usize,
  },
  DuplicateParameter {
    recipe:    &'src str,
    parameter: &'src str,
  },
  DuplicateRecipe {
    recipe: &'src str,
    first:  usize,
  },
  DuplicateVariable {
    variable: &'src str,
  },
  DuplicateSet {
    setting: &'src str,
    first:   usize,
  },
  ExpectedKeyword {
    expected: Keyword,
    found:    &'src str,
  },
  ExtraLeadingWhitespace,
  FunctionArgumentCountMismatch {
    function: &'src str,
    found:    usize,
    expected: usize,
  },
  InconsistentLeadingWhitespace {
    expected: &'src str,
    found:    &'src str,
  },
  Internal {
    message: String,
  },
  InvalidEscapeSequence {
    character: char,
  },
  MixedLeadingWhitespace {
    whitespace: &'src str,
  },
  ParameterFollowsVariadicParameter {
    parameter: &'src str,
  },
  ParameterShadowsVariable {
    parameter: &'src str,
  },
  RequiredParameterFollowsDefaultParameter {
    parameter: &'src str,
  },
  UndefinedVariable {
    variable: &'src str,
  },
  UnexpectedToken {
    expected: Vec<TokenKind>,
    found:    TokenKind,
  },
  UnknownAliasTarget {
    alias:  &'src str,
    target: &'src str,
  },
  UnknownDependency {
    recipe:  &'src str,
    unknown: &'src str,
  },
  UnknownFunction {
    function: &'src str,
  },
  UnknownStartOfToken,
  UnexpectedCharacter {
    expected: char,
  },
  UnknownSetting {
    setting: &'src str,
  },
  UnpairedCarriageReturn,
  UnexpectedClosingDelimiter {
    close: Delimiter,
  },
  MismatchedClosingDelimiter {
    close:     Delimiter,
    open:      Delimiter,
    open_line: usize,
  },
  UnterminatedInterpolation,
  UnterminatedString,
  UnterminatedBacktick,
}
