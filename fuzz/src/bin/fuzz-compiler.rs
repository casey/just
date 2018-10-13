#![no_main]

#[macro_use]
extern crate libfuzzer_sys;
extern crate just;

fuzz_target!(|data: &[u8]| {
  std::str::from_utf8(data).map(just::fuzzing::compile).ok();
});
