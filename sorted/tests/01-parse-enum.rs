// This test checks that an attribute macro #[sorted] exists and is imported
// correctly in the module system. If you make the macro return an empty token
// stream or exactly the original token stream, this test will already pass!
//
// 此测试检查属性宏[排序]是否存在并在模块系统中正确导入。
// 如果您让宏返回一个空的TokenStream或恰好是原始的TokenStream，此测试将直接通过！
//
// Be aware that the meaning of the return value of an attribute macro is
// slightly different from that of a derive macro. Derive macros are only
// allowed to *add* code to the caller's crate. Thus, what they return is
// compiled *in addition to* the struct/enum that is the macro input. On the
// other hand attribute macros are allowed to add code to the caller's crate but
// also modify or remove whatever input the attribute is on. The TokenStream
// returned by the attribute macro completely replaces the input item.
//
// 请注意，属性宏的返回值的含义与派生宏的含义略有不同。
// 派生宏只允许将代码添加到调用者的板条箱中。
// 因此，除了作为宏输入的结构/列举外，还编译了它们返回的内容。
// 另一方面，属性宏允许向调用者的板条箱添加代码，也可以修改或删除属性上的任何输入。
// 属性宏返回的TokenStream完全取代了输入项。
//
// Before moving on to the next test, I recommend also parsing the input token
// stream as a syn::Item. In order for Item to be available you will need to
// enable `features = ["full"]` of your dependency on Syn, since by default Syn
// only builds the minimum set of parsers needed for derive macros.
//
// After parsing, the macro can return back exactly the original token stream so
// that the input enum remains in the callers code and continues to be usable by
// code in the rest of the crate.
//
// 在进行下一个测试之前，我建议将输入令牌流解析为syn::Item。
// 为了使项目可用，您需要启用对Syn依赖的`features = ["full"]`，
// 因为默认情况下，Syn只构建派生宏所需的最小解析器集。
// 解析后，宏可以准确地返回原始令牌流，以便输入枚举保留在调用者代码中，
// 并继续通过板条箱其余部分的代码使用。
//
// Resources:
//
//   - The Syn crate for parsing procedural macro input:
//     https://github.com/dtolnay/syn
//
//   - The syn::Item type which represents a parsed enum as a syntax tree:
//     https://docs.rs/syn/2.0/syn/enum.Item.html

use sorted::sorted;

#[sorted]
pub enum Conference {
    RustBeltRust,
    RustConf,
    RustFest,
    RustLatam,
    RustRush,
}

fn main() {}
