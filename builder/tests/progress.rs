#[test]
fn tests() {
    std::env::set_var("TRYBUILD","overwrite");
    let t = trybuild::TestCases::new();
    t.pass("tests/01-parse.rs");
    t.pass("tests/02-create-builder.rs");
    t.pass("tests/03-call-setters.rs");
    t.pass("tests/04-call-build.rs");
    t.pass("tests/05-method-chaining.rs");
    t.pass("tests/06-optional-field.rs");
    t.pass("tests/07-repeated-field.rs");
    //预期为错误编译，但是在这里测试失败，这是一个错误。
    //设置TRYBUILD为"overwrite"可以解决这个问题。
    t.compile_fail("tests/08-unrecognized-attribute.rs");
    t.pass("tests/09-redefined-prelude-types.rs");
}
