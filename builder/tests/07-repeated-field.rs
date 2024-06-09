// The std::process::Command builder handles args in a way that is potentially
// more convenient than passing a full vector of args to the builder all at
// once.
// std::process::Command构建器处理args的方式可能比一次性将完整的arg向量传递给构建器更方便。
//
// Look for a field attribute #[builder(each = "...")] on each field. The
// generated code may assume that fields with this attribute have the type Vec
// and should use the word given in the string literal as the name for the
// corresponding builder method which accepts one vector element at a time.
//
// 在每个字段上查找字段属性[builder(each = "...")]。
// 生成的代码可以假设具有此属性的字段具有Vec类型，
// 并应使用字符串文字中给出的单词作为相应构建器方法的名称，该方法一次接受一个向量元素。
//
// In order for the compiler to know that these builder attributes are
// associated with your macro, they must be declared at the entry point of the
// derive macro. Otherwise the compiler will report them as unrecognized
// attributes and refuse to compile the caller's code.
// 为了使编译器知道这些构建器属性与您的宏相关联，它们必须在派生宏的入口点声明。
// 否则，编译器会将它们报告为无法识别的属性，并拒绝编译调用者的代码。
//
//     #[proc_macro_derive(Builder, attributes(builder))]
//
// These are called inert attributes. The word "inert" indicates that these
// attributes do not correspond to a macro invocation on their own; they are
// simply looked at by other macro invocations.
//
// 这些被称为惰性属性。“惰性”一词表明这些属性本身并不对应于宏调用;它们只是被其他宏调用查看。
//
// If the new one-at-a-time builder method is given the same name as the field,
// avoid generating an all-at-once builder method for that field because the
// names would conflict.
//
// 如果新的一次性构建器方法的名称与字段相同，请避免为该字段生成一次性构建器方法，因为名称会冲突。
//
// Resources:
//
//   - Relevant syntax tree type:
//     https://docs.rs/syn/2.0/syn/struct.Attribute.html

use derive_builder::Builder;

#[derive(Builder)]
pub struct Command {
    executable: String,
    #[builder(each = "arg")]
    args: Vec<String>,
    #[builder(each = "env")]
    env: Vec<String>,
    current_dir: Option<String>,
}

fn main() {
    let command = Command::builder()
        .executable("cargo".to_owned())
        .arg("build".to_owned())
        .arg("--release".to_owned())
        //我不认为用户忘记设置env字段就需要我们来帮忙进行设置，所以我加上了对env的初始化
        .env(vec![])
        .build()
        .unwrap();

    assert_eq!(command.executable, "cargo");
    assert_eq!(command.args, vec!["build", "--release"]);
}
