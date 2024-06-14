// Figure out what impl needs to be generated for the Debug impl of Field<T>.
// This will involve adding a trait bound to the T type parameter of the
// generated impl.
//
// Callers should be free to instantiate Field<T> with a type parameter T which
// does not implement Debug, but such a Field<T> will not fulfill the trait
// bounds of the generated Debug impl and so will not be printable via Debug.
//
// 弄清楚需要为Field<T>的Debug impl生成什么impl。
// 这将涉及添加绑定到生成的impl的T类型参数的特征。
// 调用者应该可以自由地使用不实现调试的类型参数T实例化字段<T>，
// 但这样的字段<T>不会满足生成的调试输入的特征边界，因此无法通过调试打印。
//
// Resources:
//
//   - Representation of generics in the Syn syntax tree:
//     https://docs.rs/syn/2.0/syn/struct.Generics.html
//
//   - A helper for placing generics into an impl signature:
//     https://docs.rs/syn/2.0/syn/struct.Generics.html#method.split_for_impl
//
//   - Example code from Syn which deals with type parameters:
//     https://github.com/dtolnay/syn/tree/master/examples/heapsize

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
