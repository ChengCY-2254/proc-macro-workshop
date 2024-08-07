use derive_enums::Enums;
/// 收集枚举标识符并生成方法
#[derive(Enums)]
pub enum Html {
    Body,
    Div,
    H1,
}

fn main() {
    let enums: Vec<&str> = Html::values().to_vec();
    let expect = vec!["Body", "Div", "H1"];
    assert_eq!(enums, expect);
}
