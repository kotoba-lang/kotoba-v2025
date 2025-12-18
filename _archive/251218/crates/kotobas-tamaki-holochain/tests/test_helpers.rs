//! 統合テスト用のヘルパー関数
//!
//! Holochain環境がセットアップされていない場合でも、
//! テストの準備やモックデータの作成に使用できます。

use serde_json::json;

/// テスト用のStory JSONを作成
pub fn create_test_story() -> serde_json::Value {
    json!({
        "@context": "https://github.com/com-junkawasaki/kotoba/blob/22712d997449ec6229800adf42698936aa24b386/schemas/kotoba-context.jsonld",
        "@graph": [
            {
                "@id": "kotoba:process/test1",
                "@type": "kotoba:Process",
                "kotoba:label": "Test Process 1",
                "kotoba:performedBy": "kotoba:performer/actor-1"
            },
            {
                "@id": "kotoba:process/test2",
                "@type": "kotoba:Process",
                "kotoba:label": "Test Process 2",
                "kotoba:performedBy": "kotoba:performer/actor-2",
                "kotoba:next": "kotoba:process/test1"
            }
        ]
    })
}

/// テスト用のProcess JSONを作成
pub fn create_test_process() -> serde_json::Value {
    json!({
        "@type": "kotoba:Process",
        "@id": "kotoba:process/test",
        "kotoba:label": "Test Process",
        "kotoba:performedBy": "kotoba:performer/actor-1"
    })
}

/// Holochain環境が利用可能かチェック
pub fn is_holochain_available() -> bool {
    std::process::Command::new("hc")
        .arg("--version")
        .output()
        .is_ok()
        || std::process::Command::new("holochain")
            .arg("--version")
            .output()
            .is_ok()
}

/// テストスキップメッセージを表示
pub fn skip_test_message(test_name: &str, reason: &str) {
    println!("⏭️  Skipping test '{}': {}", test_name, reason);
}

