use super::*;

pub(crate) fn datetime_format<T>(
  datetime: DateTime<T>,
  format: &str,
) -> Result<String, DatetimeFormatError>
where
  T: TimeZone,
  T::Offset: Display,
{
  let items = StrftimeItems::new(format)
    .parse()
    .context(datetime_format_error::Parse { format })?;

  let mut result = String::new();

  datetime
    .format_with_items(items.iter())
    .write_to(&mut result)
    .ok()
    .context(datetime_format_error::Format { format })?;

  Ok(result)
}
