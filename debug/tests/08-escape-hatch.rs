// There are some cases where no heuristic would be sufficient to infer the
// right trait bounds based only on the information available during macro
// expansion.
//
// 在某些情况下，没有启发式足以仅根据宏观扩展期间可用的信息来推断正确的特征界限。
//
// When this happens, we'll turn to attributes as a way for the caller to
// handwrite the correct trait bounds themselves.
//
// 当这种情况发生时，我们将转向属性，作为调用者自己手写正确特征边界的一种方式。
//
// The impl for Wrapper<T> in the code below will need to include the bounds
// provided in the `debug(bound = "...")` attribute. When such an attribute is
// present, also disable all inference of bounds so that the macro does not
// attach its own `T: Debug` inferred bound.
//
// 以下代码中Wrapper<T>的impl需要包括`debug(bound = "...")`属性中提供的边界。
// 当存在此类属性时，也请禁用所有边界推断，以便宏不会附加自己的“T：调试”推断绑定。
//
//     impl<T: Trait> Debug for Wrapper<T>
//     where
//         T::Value: Debug,
//     {...}
//
// 可选地，尽管测试套件不涵盖这一点，但也接受单个字段上的`debug(bound = "...")`属性。
// 这应该只替换根据该字段的类型推断的任何边界，而不删除根据其他字段推断的边界：
//
// Optionally, though this is not covered by the test suite, also accept
// `debug(bound = "...")` attributes on individual fields. This should
// substitute only whatever bounds are inferred based on that field's type,
// without removing bounds inferred based on the other fields:
//
//     #[derive(CustomDebug)]
//     pub struct Wrapper<T: Trait, U> {
//         #[debug(bound = "T::Value: Debug")]
//         field: Field<T>,
//         normal: U,
//     }

use derive_debug::CustomDebug;
use std::fmt::Debug;

pub trait Trait {
    type Value;
}

#[derive(CustomDebug)]
#[debug(bound = "T::Value: Debug")]
pub struct Wrapper<T: Trait> {
    field: Field<T>,
}

#[derive(CustomDebug)]
struct Field<T: Trait> {
    values: Vec<T::Value>,
}

fn assert_debug<F: Debug>() {}

fn main() {
    struct Id;

    impl Trait for Id {
        type Value = u8;
    }

    assert_debug::<Wrapper<Id>>();
}
