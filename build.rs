fn main() {
  let target_os = std::env::var("CARGO_CFG_TARGET_OS");
  let target_env = std::env::var("CARGO_CFG_TARGET_ENV");

  if target_os.as_deref() == Ok("windows") {
    if target_env.as_deref() == Ok("msvc") {
      println!("cargo::rustc-link-arg=/STACK:8388608");
    } else if target_env.as_deref() == Ok("gnu") {
      println!("cargo::rustc-link-arg=-Wl,--stack,8388608");
    }
  }
}
