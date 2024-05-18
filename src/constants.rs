use super::*;

pub(crate) fn constants() -> &'static HashMap<&'static str, &'static str> {
  static CONSTANTS: OnceLock<HashMap<&str, &str>> = OnceLock::new();

  CONSTANTS.get_or_init(|| {
    vec![
      ("HEX", "0123456789abcdef"),
      ("HEXLOWER", "0123456789abcdef"),
      ("HEXUPPER", "0123456789ABCDEF"),
    ]
    .into_iter()
    .collect()
  })
}
