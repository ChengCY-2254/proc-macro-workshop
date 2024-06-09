// Does your macro still work if some of the standard library prelude item names
// mean something different in the caller's code?
//
// It may seem unreasonable to consider this case, but it does arise in
// practice. Most commonly for Result, where crates sometimes use a Result type
// alias with a single type parameter which assumes their crate's error type.
// Such a type alias would break macro-generated code that expects Result to
// have two type parameters. As another example, Hyper 0.10 used to define
// hyper::Ok as a re-export of hyper::status::StatusCode::Ok which is totally
// different from Result::Ok. This caused problems in code doing `use hyper::*`
// together with macro-generated code referring to Ok.
//
// Generally all macros (procedural as well as macro_rules) designed to be used
// by other people should refer to every single thing in their expanded code
// through an absolute path, such as std::result::Result.
//
// 如果一些标准库前奏项目名称在调用者的代码中意味着不同的东西，您的宏仍然有效吗？
// 考虑这个例子似乎不合理，但它确实在实践中出现过。
// 最常见的是结果，第三方包有时使用带有单个类型参数的结果类型别名，
// 该参数假设其依赖的错误类型。
//
// 这种类型别名将破坏宏生成的代码，该代码期望结果具有两个类型参数。
// 作为另一个例子，Hyper 0.10用于将hyper::Ok定义为hyper::status::StatusCode::Ok的重新导出，
// 这与Result::Ok完全不同。
//
// 这导致代码在执行`use hyper:::*`以及引用Ok的宏生成代码时出现问题。
// 一般来说，所有设计用于其他人使用的宏（procedural以及macro_rules）
// 都应通过绝对路径引用其扩展代码中的每一件事，例如std::result::Result。
//
//
//
//
use derive_builder::Builder;

type Option = ();
type Some = ();
type None = ();
type Result = ();
type Box = ();

#[derive(Builder)]
pub struct Command {
    executable: String,
}

fn main() {}
