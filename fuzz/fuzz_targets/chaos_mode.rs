#![no_main]
#[macro_use]
extern crate libfuzzer_sys;

use cranelift_codegen::machinst::buffer::test_fuzzing;

// one fuzz target per crate / module / test?
fuzz_target!(|data: &[u8]| {
    cranelift_codegen::chaos_mode::init_unstructured(data);
    test_fuzzing();
    cranelift_codegen::chaos_mode::drop_unstructured(data);
});
