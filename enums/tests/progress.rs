#[test]
fn tests() {
    let t = trybuild::TestCases::new();
    t.pass("tests/00-parse.rs");
    t.pass("tests/01-collect_enum_ident.rs");
    t.pass("tests/02-custom-fn-name.rs");

    // TODO: add tests
    //
    // t.pass("tests/01-something-that-works.rs");
    // t.compile_fail("tests/02-some-compiler-error.rs");
}
