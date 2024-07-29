// At this point we have an enum and we need to check whether the variants
// appear in sorted order!
// 此时，我们有一个枚举，我们需要检查变体是否按顺序显示！
//
// When your implementation notices a variant that compares lexicographically
// less than one of the earlier variants, you'll want to construct a syn::Error
// that gets converted to TokenStream by the already existing code that handled
// this conversion during the previous test case.
//
// 当您的实现注意到一个在词典上比较小于早期变体之一的变体时，
// 您需要构建一个syn::Error，
// 该错误被在上一个测试用例中处理此转换的现有代码转换为TokenStream。
//
// The "span" of your error, which determines where the compiler places the
// resulting error message, will be the span of whichever variant name that is
// not in the right place. Ideally the error message should also identify which
// other variant the user needs to move this one to be in front of.
// 您的错误的"span"决定了编译器放置结果错误消息的位置，
// 将是任何不在正确位置的变体名称的"span"。
// 理想情况下，错误消息还应该识别用户需要将该变体移到前面的其他变体。

use sorted::sorted;

#[sorted]
pub enum Error {
    ThatFailed,
    ThisFailed,
    SomethingFailed,
    WhoKnowsWhatFailed,
}

fn main() {}
