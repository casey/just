#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
  std::str::from_utf8(data).map(just::fuzzing::compile).ok();
});
