#[test]
fn ui_tests() {
    let t = trybuild::TestCases::new();
    t.pass("tests/ui/ok_*.rs");
    t.compile_fail("tests/ui/fail_*.rs");
}
