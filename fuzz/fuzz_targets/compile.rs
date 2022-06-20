#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|src: &str| {
  let _ = just::fuzzing::compile(src);
});
