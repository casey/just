#[derive(Debug, PartialEq, Clone, Copy, Ord, PartialOrd, Eq)]
pub(crate) enum StringDelimiter {
  Backtick,
  QuoteDouble,
  QuoteSingle,
}
