// The #[sorted] macro is only defined to work on enum types, so this is a test
// to ensure that when it's attached to a struct (or anything else) it produces
// some reasonable error. Your macro will need to look into the syn::Item that
// it parsed to ensure that it represents an enum, returning an error for any
// other type of Item such as a struct.
// [sorted]宏仅被定义为处理enum类型，因此这是一个测试，
// 以确保当它附加到结构（或其他任何东西）时会产生一些合理的错误。
// 您的宏需要查看它解析的syn：：Project以确保它代表enum，
// 并返回任何其他类型的Project（例如struct）的错误。
//
// This is an exercise in exploring how to return errors from procedural macros.
// The goal is to produce an understandable error message which is tailored to
// this specific macro (saying that #[sorted] cannot be applied to things other
// than enum). For this you'll want to look at the syn::Error type, how to
// construct it, and how to return it.
//
// 这是一个探索如何从过程宏返回错误的练习。
// 目标是生成一个可理解的错误消息，该错误消息是针对此特定宏量身定制的（说[排序]不能应用于枚举以外的事物）。为此，您需要查看syn::Error类型，如何构建它，以及如何返回它。
//
// Notice that the return value of an attribute macro is simply a TokenStream,
// not a Result with an error. The syn::Error type provides a method to render
// your error as a TokenStream containing an invocation of the compile_error
// macro.
// 请注意，属性宏的返回值只是一个TokenStream，而不是有错误的结果。
// syn::Error类型提供了一种将您的错误渲染为包含调用compile_error宏的TokenStream的方法。
//
// A final tweak you may want to make is to have the `sorted` function delegate
// to a private helper function which works with Result, so most of the macro
// can be written with Result-returning functions while the top-level function
// handles the conversion down to TokenStream.
//
// 您可能想要做的最后一个调整是将“sorted”函数委托给与Report一起工作的私人助手函数，因此大部分宏都可以用结果返回函数编写，而顶级函数则处理向下转换到TokenStream。
//
//
// Resources
//
//   - The syn::Error type:
//     https://docs.rs/syn/2.0/syn/struct.Error.html

use sorted::sorted;

pub struct Error {
    kind: ErrorKind,
    message: String,
}
#[sorted]
enum ErrorKind {
    Io,
    Syntax,
    Eof,
}

fn main() {
    let a = String::new();
}
