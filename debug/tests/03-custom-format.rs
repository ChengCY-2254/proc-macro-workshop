// Look for a field attribute #[debug = "..."] on each field. If present, find a
// way to format the field according to the format string given by the caller in
// the attribute.
//
// In order for the compiler to recognize this inert attribute as associated
// with your derive macro, it will need to be declared at the entry point of the
// derive macro.
//
//     #[proc_macro_derive(CustomDebug, attributes(debug))]
//
// These are called inert attributes. The word "inert" indicates that these
// attributes do not correspond to a macro invocation on their own; they are
// simply looked at by other macro invocations.
//
//
//在每个字段上查找字段属性[debug = "..."]。
//如果存在，请找到一种方法，根据调用者在属性中给出的格式字符串格式化字段。
//为了使编译器将此惰性属性识别为与您的派生宏相关联的属性，
//需要在派生宏的入口点声明它。
//[proc_macro_derive（CustomDebug，attributes(debug)）]这些被称为惰性属性。
//“惰性”一词表示这些属性本身与宏调用不对应；它们只是通过其他宏调用来查看。
//
// Resources:
//
//   - Relevant syntax tree type:
//     https://docs.rs/syn/2.0/syn/struct.Attribute.html
//
//   - Macro for applying a format string to some runtime value:
//     https://doc.rust-lang.org/std/macro.format_args.html

use derive_debug::CustomDebug;

#[derive(CustomDebug)]
pub struct Field {
    name: &'static str,
    #[debug = "0b{:08b}"]
    bitmask: u8,
}

fn main() {
    let f = Field {
        name: "F",
        bitmask: 0b00011100,
    };

    let debug = format!("{:?}", f);
    let expected = r#"Field { name: "F", bitmask: 0b00011100 }"#;

    assert_eq!(debug, expected);
}
