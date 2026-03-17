fn main() {
  let os = std::env::var("CARGO_CFG_TARGET_OS").unwrap();
  let env = std::env::var("CARGO_CFG_TARGET_ENV").unwrap();
  if os == "windows" {
    if env == "msvc" {
      println!("cargo::rustc-link-arg=/STACK:2097152");
    } else if env == "gnu" {
      println!("cargo::rustc-link-arg=-Wl,--stack,2097152");
    }
  }
}
