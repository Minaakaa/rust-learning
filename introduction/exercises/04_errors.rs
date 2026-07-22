#![cfg_attr(not(test), allow(dead_code))]

//! # 問題 04: 独自エラー型を設計する
//!
//! `"ロボットID,バッテリー残量,最大積載量kg"` 形式の設定を読み込みます。
//! 問題 03 の `String` エラーを、呼び出し側が種類を判定できる `ConfigError` に発展させます。
//!
//! TODO:
//! 1. `Display` で利用者向けのメッセージを作る。
//! 2. `Error::source` で、整数解析エラーの原因を返す。
//! 3. `From<ParseIntError>` を実装し、`parse()?` を使えるようにする。
//! 4. `parse_config` で項目数と値を検証する。
//!
//! 通常の入力エラーに `panic!`、`unwrap`、`expect` は使わないでください。
//! 各項目の空白を除き、空の ID、100 を超えるバッテリー残量、0 kg の最大積載量、
//! 項目の過不足を拒否します。各バリアントと表示文の対応はテストで確認できます。

use std::error::Error;
use std::fmt;
use std::num::ParseIntError;

#[derive(Debug)]
enum ConfigError {
    MissingField(&'static str),
    TooManyFields,
    EmptyRobotId,
    InvalidNumber(ParseIntError),
    BatteryOutOfRange(u8),
    ZeroMaxLoad,
}

impl fmt::Display for ConfigError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let _ = formatter;
        todo!("このエラーを短い日本語で表示してください: {self:?}")
    }
}

impl Error for ConfigError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        todo!("InvalidNumber の原因だけを返してください: {self:?}")
    }
}

impl From<ParseIntError> for ConfigError {
    fn from(source: ParseIntError) -> Self {
        todo!("ParseIntError を失わず包んでください: {source}")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RobotConfig {
    id: String,
    battery_percent: u8,
    max_load_kg: u16,
}

fn parse_config(input: &str) -> Result<RobotConfig, ConfigError> {
    todo!("3 項目を読み、値を検証してください: {input}")
}

fn main() {
    println!("設定: {:?}", parse_config("RB-01,85,12"));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 正しい設定を読み込む() {
        assert_eq!(
            parse_config(" RB-01 , 85 , 12 ").unwrap(),
            RobotConfig {
                id: "RB-01".to_string(),
                battery_percent: 85,
                max_load_kg: 12,
            }
        );
    }

    #[test]
    fn 足りない項目の名前を返す() {
        assert!(matches!(
            parse_config("RB-01"),
            Err(ConfigError::MissingField("battery_percent"))
        ));
        assert!(matches!(
            parse_config("RB-01,85"),
            Err(ConfigError::MissingField("max_load_kg"))
        ));
    }

    #[test]
    fn 余分な項目を拒否する() {
        assert!(matches!(
            parse_config("RB-01,85,12,unused"),
            Err(ConfigError::TooManyFields)
        ));
    }

    #[test]
    fn 空のロボットidを拒否する() {
        assert!(matches!(
            parse_config("  ,85,12"),
            Err(ConfigError::EmptyRobotId)
        ));
    }

    #[test]
    fn 数値エラーの原因を保持する() {
        let error = parse_config("RB-01,eighty,12").unwrap_err();
        assert!(matches!(error, ConfigError::InvalidNumber(_)));
        assert!(error.source().is_some());
        assert_eq!(error.to_string(), "整数を解析できません");
    }

    #[test]
    fn fromで整数解析エラーを変換できる() {
        let source = "not-a-number".parse::<u8>().unwrap_err();
        let error = ConfigError::from(source);
        assert!(matches!(error, ConfigError::InvalidNumber(_)));
        assert!(error.source().is_some());
    }

    #[test]
    fn バッテリーの範囲を検証する() {
        let error = parse_config("RB-01,120,12").unwrap_err();
        assert!(matches!(error, ConfigError::BatteryOutOfRange(120)));
        assert_eq!(
            error.to_string(),
            "バッテリー残量 120 は 0..=100 の範囲外です"
        );
    }

    #[test]
    fn 最大積載量ゼロを拒否する() {
        assert!(matches!(
            parse_config("RB-01,85,0"),
            Err(ConfigError::ZeroMaxLoad)
        ));
    }

    #[test]
    fn 各エラーを利用者向けに表示する() {
        let invalid_number = ConfigError::InvalidNumber("not-a-number".parse::<u8>().unwrap_err());
        assert!(matches!(
            &invalid_number,
            ConfigError::InvalidNumber(source) if !source.to_string().is_empty()
        ));

        assert_eq!(
            ConfigError::MissingField("battery_percent").to_string(),
            "必須項目 battery_percent がありません"
        );
        assert_eq!(
            ConfigError::TooManyFields.to_string(),
            "設定項目が多すぎます"
        );
        assert_eq!(ConfigError::EmptyRobotId.to_string(), "ロボットIDが空です");
        assert_eq!(invalid_number.to_string(), "整数を解析できません");
        assert_eq!(
            ConfigError::BatteryOutOfRange(120).to_string(),
            "バッテリー残量 120 は 0..=100 の範囲外です"
        );
        assert_eq!(
            ConfigError::ZeroMaxLoad.to_string(),
            "最大積載量は 1 kg 以上にしてください"
        );
        assert!(ConfigError::ZeroMaxLoad.source().is_none());
    }

    #[test]
    fn errorトレイトとして扱える() {
        fn assert_error<T: Error>() {}
        assert_error::<ConfigError>();
    }
}
