use crate::common::*;

#[derive(Debug, PartialEq)]
pub(crate) enum CompilationErrorKind<'a> {
  AliasShadowsRecipe {
    alias: &'a str,
    recipe_line: usize,
  },
  CircularRecipeDependency {
    recipe: &'a str,
    circle: Vec<&'a str>,
  },
  CircularVariableDependency {
    variable: &'a str,
    circle: Vec<&'a str>,
  },
  DependencyHasParameters {
    recipe: &'a str,
    dependency: &'a str,
  },
  DuplicateAlias {
    alias: &'a str,
    first: usize,
  },
  DuplicateDependency {
    recipe: &'a str,
    dependency: &'a str,
  },
  DuplicateParameter {
    recipe: &'a str,
    parameter: &'a str,
  },
  DuplicateRecipe {
    recipe: &'a str,
    first: usize,
  },
  DuplicateVariable {
    variable: &'a str,
  },
  DuplicateSet {
    setting: &'a str,
    first: usize,
  },
  ExtraLeadingWhitespace,
  FunctionArgumentCountMismatch {
    function: &'a str,
    found: usize,
    expected: usize,
  },
  InconsistentLeadingWhitespace {
    expected: &'a str,
    found: &'a str,
  },
  Internal {
    message: String,
  },
  InvalidEscapeSequence {
    character: char,
  },
  MixedLeadingWhitespace {
    whitespace: &'a str,
  },
  ParameterFollowsVariadicParameter {
    parameter: &'a str,
  },
  ParameterShadowsVariable {
    parameter: &'a str,
  },
  RequiredParameterFollowsDefaultParameter {
    parameter: &'a str,
  },
  UndefinedVariable {
    variable: &'a str,
  },
  UnexpectedToken {
    expected: Vec<TokenKind>,
    found: TokenKind,
  },
  UnknownAliasTarget {
    alias: &'a str,
    target: &'a str,
  },
  UnknownDependency {
    recipe: &'a str,
    unknown: &'a str,
  },
  UnknownFunction {
    function: &'a str,
  },
  UnknownStartOfToken,
  UnknownSetting {
    setting: &'a str,
  },
  UnpairedCarriageReturn,
  UnterminatedInterpolation,
  UnterminatedString,
  UnterminatedBacktick,
}
