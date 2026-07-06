use super::*;

pub(crate) fn datetime_format<Tz>(
  datetime: chrono::DateTime<Tz>,
  format: &str,
) -> RunResult<'static, String>
where
  Tz: TimeZone,
  Tz::Offset: Display,
{
  let items = StrftimeItems::new(format)
    .parse()
    .map_err(|source| Error::DatetimeFormatParse {
      format: format.into(),
      source,
    })?;

  let mut result = String::new();

  datetime
    .format_with_items(items.iter())
    .write_to(&mut result)
    .map_err(|fmt::Error| Error::DatetimeFormat {
      format: format.into(),
    })?;

  Ok(result)
}
