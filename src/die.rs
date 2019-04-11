macro_rules! die {
  ($($arg:tt)*) => {{
    eprintln!($($arg)*);
    std::process::exit(EXIT_FAILURE)
  }};
}
