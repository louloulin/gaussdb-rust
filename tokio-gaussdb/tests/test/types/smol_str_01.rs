use smol_str_01::SmolStr;

use crate::types::test_type;

#[tokio::test]
async fn test_smol_str() {
    test_type(
        "VARCHAR",
        &[
            (Some(SmolStr::new("hello world")), "'hello world'"),
            (
                Some(SmolStr::new("イロハニホヘ�?チリヌル�?)),
                "'イロハニホヘ�?チリヌル�?",
            ),
            (None, "NULL"),
        ],
    )
    .await;
}
