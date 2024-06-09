// Emit an implementation of std::fmt::Debug for a basic struct with named
// fields and no generic type parameters.
//
// Note that there is no enforced relationship between the name of a derive
// macro and the trait that it implements. Here the macro is named CustomDebug
// but the trait impls it generates are for Debug. As a convention, typically
// derive macros implement a trait with the same name as a macro.
//
// 为具有命名字段且没有泛型类型参数的基本结构实现std::fmt::Debug。
// 请注意，派生宏的名称与其实现的特征之间没有强制关系。
// 在这里，宏被命名为CustomDebug，但它生成的特质暗示是用于Debug的。
// 作为惯例，通常派生宏实现与宏同名的特征。
//
//
// Resources:
//
//   - The Debug trait:
//     https://doc.rust-lang.org/std/fmt/trait.Debug.html
//
//   - The DebugStruct helper for formatting structs correctly:
//     https://doc.rust-lang.org/std/fmt/struct.DebugStruct.html

use derive_debug::CustomDebug;

#[derive(CustomDebug)]
pub struct Field {
    name: &'static str,
    bitmask: u8,
}

fn main() {
    let f = Field {
        name: "F",
        bitmask: 0b00011100,
    };

    let debug = format!("{:?}", f);

    assert!(debug.starts_with(r#"Field { name: "F","#));
}
