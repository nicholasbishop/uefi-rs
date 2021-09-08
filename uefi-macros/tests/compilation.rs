use std::env;

#[test]
fn ui() {
    let t = trybuild::TestCases::new();

    // This wrapper script adds the necessary `-Zbuild-std` argument
    // when trybuild invokes cargo.
    let cargo_wrapper = env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .join("../../../../uefi-macros/tests/cargo_wrapper");
    env::set_var("CARGO", cargo_wrapper);

    t.compile_fail("tests/ui/*.rs");
}
