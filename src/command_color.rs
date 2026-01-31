use super::*;

#[derive(Copy, Clone, ValueEnum)]
pub(crate) enum CommandColor {
  Black,
  Blue,
  Cyan,
  Green,
  Purple,
  Red,
  Yellow,
}

impl From<CommandColor> for ansi_term::Color {
  fn from(command_color: CommandColor) -> Self {
    match command_color {
      CommandColor::Black => Self::Black,
      CommandColor::Blue => Self::Blue,
      CommandColor::Cyan => Self::Cyan,
      CommandColor::Green => Self::Green,
      CommandColor::Purple => Self::Purple,
      CommandColor::Red => Self::Red,
      CommandColor::Yellow => Self::Yellow,
    }
  }
}

/// Parse a color string into ansi_term::Color
/// Supports: basic color names (blue, green, etc.), hex colors (#f9e2af), RGB (249,226,175)
pub(crate) fn parse_color(s: &str) -> Result<ansi_term::Color, String> {
  let s = s.trim();

  // Try basic color names first
  match s.to_lowercase().as_str() {
    "black" => return Ok(ansi_term::Color::Black),
    "red" => return Ok(ansi_term::Color::Red),
    "green" => return Ok(ansi_term::Color::Green),
    "yellow" => return Ok(ansi_term::Color::Yellow),
    "blue" => return Ok(ansi_term::Color::Blue),
    "purple" | "magenta" => return Ok(ansi_term::Color::Purple),
    "cyan" => return Ok(ansi_term::Color::Cyan),
    "white" => return Ok(ansi_term::Color::White),
    _ => {}
  }

  // Try hex format: #RRGGBB or RRGGBB
  if let Some(hex) = s.strip_prefix('#') {
    return parse_hex_color(hex);
  } else if s.len() == 6 && s.chars().all(|c| c.is_ascii_hexdigit()) {
    return parse_hex_color(s);
  }

  // Try RGB format: 249,226,175 or rgb(249,226,175)
  let rgb_str = s.strip_prefix("rgb(")
    .and_then(|s| s.strip_suffix(')'))
    .unwrap_or(s);

  let parts: Vec<&str> = rgb_str.split(',').collect();
  if parts.len() == 3 {
    let r = parts[0].trim().parse::<u8>()
      .map_err(|_| format!("Invalid red value: {}", parts[0]))?;
    let g = parts[1].trim().parse::<u8>()
      .map_err(|_| format!("Invalid green value: {}", parts[1]))?;
    let b = parts[2].trim().parse::<u8>()
      .map_err(|_| format!("Invalid blue value: {}", parts[2]))?;
    return Ok(ansi_term::Color::RGB(r, g, b));
  }

  Err(format!("Invalid color format: '{}'. Expected: color name (blue, green), hex (#f9e2af), or RGB (249,226,175)", s))
}

fn parse_hex_color(hex: &str) -> Result<ansi_term::Color, String> {
  if hex.len() != 6 {
    return Err(format!("Hex color must be 6 characters: #{}", hex));
  }

  let r = u8::from_str_radix(&hex[0..2], 16)
    .map_err(|_| format!("Invalid hex color: #{}", hex))?;
  let g = u8::from_str_radix(&hex[2..4], 16)
    .map_err(|_| format!("Invalid hex color: #{}", hex))?;
  let b = u8::from_str_radix(&hex[4..6], 16)
    .map_err(|_| format!("Invalid hex color: #{}", hex))?;

  Ok(ansi_term::Color::RGB(r, g, b))
}
