#![no_main]
#[macro_use]
extern crate libfuzzer_sys;

use cranelift_codegen::machinst::buffer::test_fuzzing;

// one fuzz target per crate / module / test?
fuzz_target!(|data: &[u8]| { test_fuzzing() });
