// This test case should not require any code change in your macro if you have
// everything up to this point already passing, but is here to demonstrate why
// inferring `#field_ty: Trait` bounds as mentioned in the previous test case is
// not viable.
//
// 如果您到目前为止的所有内容都已通过，则此测试用例不应要求在宏中进行任何代码更改，
// 但在这里是为了证明为什么推断上一个测试用例中提到的“#field_ty：特征”边界是不可行的。
//
//     #[derive(CustomDebug)]
//     pub struct One<T> {
//         value: T,
//         two: Option<Box<Two<T>>>,
//     }
//
//     #[derive(CustomDebug)]
//     struct Two<T> {
//         one: Box<One<T>>,
//     }
//
// The problematic expansion would come out as:
// 有问题的扩展将如下：
//
//     impl<T> Debug for One<T>
//     where
//         T: Debug,
//         Option<Box<Two<T>>>: Debug,
//     {...}
//
//     impl<T> Debug for Two<T>
//     where
//         Box<One<T>>: Debug,
//     {...}
//
// There are two things wrong here.
//
// First, taking into account the relevant standard library impls `impl<T> Debug
// for Option<T> where T: Debug` and `impl<T> Debug for Box<T> where T: ?Sized +
// Debug`, we have the following cyclic definition:
//
// 这里有两件事不对。
//
// 首先，考虑到相关的标准库
// impls `impl<T> Debug for Option<T> where T: Debug` and `impl<T> Debug for Box<T> where T: ?Sized+Debug`，
// 我们有以下循环定义：
//
//   - One<T> implements Debug if there is an impl for Option<Box<Two<T>>>;
//   - Option<Box<Two<T>>> implements Debug if there is an impl for Box<Two<T>>;
//   - Box<Two<T>> implements Debug if there is an impl for Two<T>;
//   - Two<T> implements Debug if there is an impl for Box<One<T>>;
//   - Box<One<T>> implements Debug if there is an impl for One<T>; cycle!
//
// The Rust compiler detects and rejects this cycle by refusing to assume that
// an impl for any of these types exists out of nowhere. The error manifests as:
//
//     error[E0275]: overflow evaluating the requirement `One<u8>: std::fmt::Debug`
//      -->
//       |     assert_debug::<One<u8>>();
//       |     ^^^^^^^^^^^^^^^^^^^^^^^
//
// There is a technique known as co-inductive reasoning that may allow a
// revamped trait solver in the compiler to process cycles like this in the
// future, though there is still uncertainty about whether co-inductive
// semantics would lead to unsoundness in some situations when applied to Rust
// trait impls. There is no current activity pursuing this but some discussion
// exists in a GitHub issue called "#[derive] sometimes uses incorrect bounds":
// https://github.com/rust-lang/rust/issues/26925
//
// The second thing wrong is a private-in-public violation:
//
//     error[E0446]: private type `Two<T>` in public interface
//      -->
//       |   struct Two<T> {
//       |   - `Two<T>` declared as private
//     ...
//       | / impl<T> Debug for One<T>
//       | | where
//       | |     T: Debug,
//       | |     Option<Box<Two<T>>>: Debug,
//     ... |
//       | | }
//       | |_^ can't leak private type
//
// Public APIs in Rust are not allowed to be defined in terms of private types.
// That includes the argument types and return types of public function
// signatures, as well as trait bounds on impls of public traits for public
// types.

use derive_debug::CustomDebug;
use std::fmt::Debug;

#[derive(CustomDebug)]
pub struct One<T> {
    value: T,
    two: Option<Box<Two<T>>>,
}

#[derive(CustomDebug)]
struct Two<T> {
    one: Box<One<T>>,
}

fn assert_debug<F: Debug>() {}

fn main() {
    assert_debug::<One<u8>>();
    assert_debug::<Two<u8>>();
}
