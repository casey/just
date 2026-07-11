use super::*;

#[derive(Debug, PartialEq)]
pub(crate) enum CompileErrorKind<'src> {
  ArgAttributeMinExceedsMax {
    min: u64,
    max: u64,
  },
  ArgAttributeRequiresMultipleOrVariadic {
    key: Name<'src>,
  },
  ArgAttributeRequiresOption {
    key: Name<'src>,
  },
  ArgumentCountParse {
    key: Name<'src>,
    value: String,
    source: ParseIntError,
  },
  ArgumentCountValue {
    key: Name<'src>,
    value: String,
  },
  ArgumentPatternRegex {
    source: regex::Error,
  },
  AttributeArgumentCountMismatch {
    attribute: Name<'src>,
    found: usize,
    min: usize,
    max: usize,
  },
  AttributeArgumentExpression {
    attribute: Name<'src>,
  },
  AttributeKeyMissingValue {
    key: Name<'src>,
  },
  AttributeKeyTakesNoValue {
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
  ConstEval(ConstEvalError<'src>),
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
    attribute: Name<'src>,
    first: usize,
  },
  DuplicateAttributeKey {
    attribute: &'src str,
    key: &'src str,
  },
  DuplicateDefault {
    recipe: &'src str,
  },
  DuplicateFunctionParameter {
    function: &'src str,
    parameter: &'src str,
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
  EscapeEndOfFile,
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
  FlagAndPatternArgAttribute {
    parameter: String,
  },
  FlagAndValueArgAttribute {
    parameter: String,
  },
  FlagAttributeTakesNoValue {
    parameter: String,
  },
  FlagWithDefault {
    parameter: String,
  },
  FunctionArgumentCountMismatch {
    arguments: usize,
    expected: RangeInclusive<usize>,
    function: &'src str,
  },
  GuardAndInfallibleSigil,
  IncompatibleSettings {
    first: Keyword,
    first_line: usize,
    second: Keyword,
  },
  InconsistentLeadingWhitespace {
    expected: &'src str,
    found: &'src str,
  },
  Internal {
    message: String,
  },
  InvalidAttribute {
    item_kind: ItemKind,
    item_name: &'src str,
    attribute: Box<Attribute<'src>>,
  },
  InvalidEscapeSequence {
    character: char,
  },
  InvalidIndentation {
    message: &'static str,
  },
  InvalidMinimumVersion {
    source: &'static str,
    version: String,
  },
  InvalidShellRecipeAttribute {
    attribute: Box<Attribute<'src>>,
    recipe: &'src str,
  },
  InvalidSignal {
    signal: String,
  },
  ListFeature(ListFeature),
  MappedDependencyMultipleStarredArguments,
  MappedDependencyWithoutListsSetting,
  MappedDependencyWithoutStarredArgument,
  MinimumVersion {
    current: Version,
    minimum: Version,
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
  OptionNameStartsWithDash {
    parameter: String,
  },
  ParameterFollowsVariadicParameter {
    parameter: &'src str,
  },
  ParsingRecursionDepthExceeded,
  Redefinition {
    first: usize,
    first_type: ItemKind,
    name: &'src str,
    second_type: ItemKind,
  },
  RequiredParameterFollowsDefaultParameter {
    parameter: &'src str,
  },
  ScriptAndShellAttribute {
    recipe: &'src str,
  },
  SettingExpression {
    setting: Keyword,
  },
  ShellExpansion {
    err: shellexpand::LookupError<env::VarError>,
  },
  ShortOptionWithMultipleCharacters {
    parameter: String,
  },
  StarredArgumentOutsideMappedDependency,
  UndefinedArgAttribute {
    argument: String,
  },
  UndefinedFunction {
    function: &'src str,
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
  UnknownAttributeKey {
    attribute: &'src str,
    key: &'src str,
  },
  UnknownDependency {
    recipe: &'src str,
    unknown: Namepath<'src>,
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
}
