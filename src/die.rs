macro_rules! die {
  ($($arg:tt)*) => {{
    extern crate std;
    eprintln!($($arg)*);
    process::exit(EXIT_FAILURE)
  }};
}
