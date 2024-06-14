// Some generic types implement Debug even when their type parameters do not.
// One example is PhantomData which has this impl:
//
// 一些泛型类型实现Debug，即使它们的类型参数没有实现。一个例子是PhantomData，它具有以下内容：
//
//     impl<T: ?Sized> Debug for PhantomData<T> {...}
//
// To accommodate this sort of situation, one way would be to generate a trait
// bound `#field_ty: Debug` for each field type in the input, rather than
// `#param: Debug` for each generic parameter. For example in the case of the
// struct Field<T> in the test case below, it would be:
//
// 为了适应这种情况，一种方法是为输入中的每个字段类型生成绑定的“#field_ty: Debug”的trait，
// 而不是为每个通用参数生成“#param: Debug”。
// 例如，在以下测试用例中的结构字段<T>的情况下，它将是：
//
//     impl<T> Debug for Field<T>
//     where
//         PhantomData<T>: Debug,
//     {...}
//
// This approach has fatal downsides that will be covered in subsequent test
// cases.
//
// Instead we'll recognize PhantomData as a special case since it is so common,
// and later provide an escape hatch for the caller to override inferred bounds
// in other application-specific special cases.
// 这种方法有致命的缺点，将在随后的测试案例中涵盖。
// 相反，我们会将PhantomData识别为特殊情况，因为它非常常见，然后为调用者提供一个转义，
// 以在其他特定于应用程序的特殊情况下覆盖推断的边界。
// Concretely, for each type parameter #param in the input, you will need to
// determine whether it is only ever mentioned inside of a PhantomData and if so
// then avoid emitting a `#param: Debug` bound on that parameter. For the
// purpose of the test suite it is sufficient to look for exactly the field type
// PhantomData<#param>. In reality we may also care about recognizing other
// possible arrangements like PhantomData<&'a #param> if the semantics of the
// trait we are deriving would make it likely that callers would end up with
// that sort of thing in their code.
//
// 具体来说，对于输入中的每个类型参数参数，您需要确定它是否只在PhantomData中提及，如果是这样，
// 则避免在该参数上发出“#param：Debug”绑定。
// 为了测试套件的目的，准确地寻找字段类型PhantomData<#param>就足够了。
// 在现实中，我们也可能关心识别其他可能的安排，如PhantomData<&'a param>，
// 如果我们推导出的特征的语义使调用者可能会在他们的代码中最终出现这种东西。
//
// Notice that we are into the realm of heuristics at this point. In Rust's
// macro system it is not possible for a derive macro to infer the "correct"
// bounds in general. Doing so would require name-resolution, i.e. the ability
// for the macro to look up what trait impl corresponds to some field's type by
// name. The Rust compiler has chosen to perform all macro expansion fully
// before name resolution (not counting name resolution of macros themselves,
// which operates in a more restricted way than Rust name resolution in general
// to make this possible).
//
// 请注意，在这一点上，我们进入了启发式领域。
// 在Rust的宏系统中，派生宏不可能推断出一般“正确”的界限。
// 这样做需要名称分辨率，即宏能够按名称查找与某个字段类型对应的特征。
// Rust编译器选择在名称解析之前完全执行所有宏扩展（不包括宏本身的名称解析，宏本身的操作方式通常比Rust名称解析更受限，以使这成为可能）。
//
// The clean separation between macro expansion and name resolution has huge
// advantages that outweigh the limitation of not being able to expose type
// information to procedural macros, so there are no plans to change it. Instead
// macros rely on domain-specific heuristics and escape hatches to substitute
// for type information where unavoidable or, more commonly, rely on the Rust
// trait system to defer the need for name resolution. In particular pay
// attention to how the derive macro invocation below is able to expand to code
// that correctly calls String's Debug impl despite having no way to know that
// the word "S" in its input refers to the type String.
//
// 宏扩展和名称解析之间的干净分离具有巨大的优势，超过了无法将类型信息暴露给过程宏的局限性，因此没有计划改变它。
// 相反，宏依赖于特定于域的启发法和转义舱口来替代不可避免的类型信息，或者更常见的是，依靠Rust特征系统来推迟对名称解析的需求。
// 特别注意下面的派生宏调用如何能够扩展到正确调用String的Debug impl的代码，尽管无法知道其输入中的单词“S”指的是类型String。

use derive_debug::CustomDebug;
use std::fmt::Debug;
use std::marker::PhantomData;

type S = String;

#[derive(CustomDebug)]
pub struct Field<T> {
    marker: PhantomData<T>,
    string: S,
    #[debug = "0b{:08b}"]
    bitmask: u8,
}

fn assert_debug<F: Debug>() {}

fn main() {
    // Does not implement Debug.
    struct NotDebug;

    assert_debug::<PhantomData<NotDebug>>();
    assert_debug::<Field<NotDebug>>();
}
