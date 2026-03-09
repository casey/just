use super::*;

#[derive(Debug, PartialEq)]
pub(crate) enum CompileErrorKind<'src> {
  ArgAttributeValueRequiresOption,
  ArgumentPatternRegex {
    source: regex::Error,
  },
  AttributeArgumentCountMismatch {
    attribute: Name<'src>,
    found: usize,
    min: usize,
    max: usize,
  },
  AttributeKeyMissingValue {
    key: Name<'src>,
  },
  AttributePositionalFollowsKeyword,
  BacktickShebang,
  CircularRecipeDependency {
    recipe: &'src str,
    circle: Vec<&'src str>,
  },
  CircularVariableDependency {
    variable: &'src str,
    circle: Vec<&'src str>,
  },
  DependencyArgumentCountMismatch {
    dependency: Namepath<'src>,
    found: usize,
    min: usize,
    max: usize,
  },
  DuplicateArgAttribute {
    arg: String,
    first: usize,
  },
  DuplicateAttribute {
    attribute: &'src str,
    first: usize,
  },
  DuplicateDefault {
    recipe: &'src str,
  },
  DuplicateEnvAttribute {
    variable: String,
    first: usize,
  },
  DuplicateOption {
    recipe: &'src str,
    option: Switch,
  },
  DuplicateParameter {
    recipe: &'src str,
    parameter: &'src str,
  },
  DuplicateSet {
    setting: &'src str,
    first: usize,
  },
  DuplicateUnexport {
    variable: &'src str,
  },
  DuplicateVariable {
    variable: &'src str,
  },
  ExitMessageAndNoExitMessageAttribute {
    recipe: &'src str,
  },
  ExpectedKeyword {
    expected: Vec<Keyword>,
    found: Token<'src>,
  },
  ExportUnexported {
    variable: &'src str,
  },
  ExtraLeadingWhitespace,
  ExtraneousAttributes {
    count: usize,
  },
  FunctionArgumentCountMismatch {
    function: &'src str,
    found: usize,
    expected: RangeInclusive<usize>,
  },
  Include,
  InconsistentLeadingWhitespace {
    expected: &'src str,
    found: &'src str,
  },
  Internal {
    message: String,
  },
  InvalidAttribute {
    item_kind: &'static str,
    item_name: &'src str,
    attribute: Box<Attribute<'src>>,
  },
  InvalidEscapeSequence {
    character: char,
  },
  MismatchedClosingDelimiter {
    close: Delimiter,
    open: Delimiter,
    open_line: usize,
  },
  MixedLeadingWhitespace {
    whitespace: &'src str,
  },
  NoCdAndWorkingDirectoryAttribute {
    recipe: &'src str,
  },
  OptionNameContainsEqualSign {
    parameter: String,
  },
  OptionNameEmpty {
    parameter: String,
  },
  ParameterFollowsVariadicParameter {
    parameter: &'src str,
  },
  ParsingRecursionDepthExceeded,
  Redefinition {
    first: usize,
    first_type: &'static str,
    name: &'src str,
    second_type: &'static str,
  },
  RequiredParameterFollowsDefaultParameter {
    parameter: &'src str,
  },
  ShellExpansion {
    err: shellexpand::LookupError<env::VarError>,
  },
  ShortOptionWithMultipleCharacters {
    parameter: String,
  },
  UndefinedArgAttribute {
    argument: String,
  },
  UndefinedVariable {
    variable: &'src str,
  },
  UnexpectedCharacter {
    expected: Vec<char>,
  },
  UnexpectedClosingDelimiter {
    close: Delimiter,
  },
  UnexpectedEndOfToken {
    expected: Vec<char>,
  },
  UnexpectedToken {
    expected: Vec<TokenKind>,
    found: TokenKind,
  },
  UnicodeEscapeCharacter {
    character: char,
  },
  UnicodeEscapeDelimiter {
    character: char,
  },
  UnicodeEscapeEmpty,
  UnicodeEscapeLength {
    hex: String,
  },
  UnicodeEscapeRange {
    hex: String,
  },
  UnicodeEscapeUnterminated,
  UnknownAliasTarget {
    alias: &'src str,
    target: Namepath<'src>,
  },
  UnknownAttribute {
    attribute: &'src str,
  },
  UnknownAttributeKeyword {
    attribute: &'src str,
    keyword: &'src str,
  },
  UnknownDependency {
    recipe: &'src str,
    unknown: Namepath<'src>,
  },
  UnknownFunction {
    function: &'src str,
  },
  UnknownSetting {
    setting: &'src str,
  },
  UnknownStartOfToken {
    start: char,
  },
  UnpairedCarriageReturn,
  UnterminatedBacktick,
  UnterminatedInterpolation,
  UnterminatedString,
  VariadicParameterWithOption,
}
