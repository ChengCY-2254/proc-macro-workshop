// Ensure that your macro reports a reasonable error message when the caller
// mistypes the inert attribute in various ways. This is a compile_fail test.
//
// The preferred way to report an error from a procedural macro is by including
// an invocation of the standard library's compile_error macro in the code
// emitted by the procedural macro.
//
// 当调用者以各种方式错误键入惰性属性时，确保您的宏报告合理的错误消息。
// 这是一个compile_fail测试。
// 从过程宏报告错误的首选方法是将标准库的compile_error宏的调用包含在过程宏发出的代码中。
//
//
// Resources:
//
//   - The compile_error macro for emitting basic custom errors:
//     https://doc.rust-lang.org/std/macro.compile_error.html
//
//   - Lowering a syn::Error into an invocation of compile_error:
//     https://docs.rs/syn/2.0/syn/struct.Error.html#method.to_compile_error

use derive_builder::Builder;

#[derive(Builder)]
pub struct Command {
    executable: String,
    #[builder(eac = "arg")]
    args: Vec<String>,
    env: Vec<String>,
    current_dir: Option<String>,
}

fn main() {}
