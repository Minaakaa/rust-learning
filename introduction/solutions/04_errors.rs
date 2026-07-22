//! 問題 04 の解答例。

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
        match self {
            Self::MissingField(name) => write!(formatter, "必須項目 {name} がありません"),
            Self::TooManyFields => write!(formatter, "設定項目が多すぎます"),
            Self::EmptyRobotId => write!(formatter, "ロボットIDが空です"),
            Self::InvalidNumber(_) => write!(formatter, "整数を解析できません"),
            Self::BatteryOutOfRange(value) => {
                write!(formatter, "バッテリー残量 {value} は 0..=100 の範囲外です")
            }
            Self::ZeroMaxLoad => write!(formatter, "最大積載量は 1 kg 以上にしてください"),
        }
    }
}

impl Error for ConfigError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidNumber(source) => Some(source),
            _ => None,
        }
    }
}

impl From<ParseIntError> for ConfigError {
    fn from(source: ParseIntError) -> Self {
        Self::InvalidNumber(source)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RobotConfig {
    id: String,
    battery_percent: u8,
    max_load_kg: u16,
}

fn parse_config(input: &str) -> Result<RobotConfig, ConfigError> {
    let mut fields = input.split(',').map(str::trim);
    let id = fields.next().unwrap_or_default();
    let battery = fields
        .next()
        .ok_or(ConfigError::MissingField("battery_percent"))?;
    let max_load = fields
        .next()
        .ok_or(ConfigError::MissingField("max_load_kg"))?;

    if fields.next().is_some() {
        return Err(ConfigError::TooManyFields);
    }
    if id.is_empty() {
        return Err(ConfigError::EmptyRobotId);
    }

    let battery_percent = battery.parse::<u8>()?;
    if battery_percent > 100 {
        return Err(ConfigError::BatteryOutOfRange(battery_percent));
    }

    let max_load_kg = max_load.parse::<u16>()?;
    if max_load_kg == 0 {
        return Err(ConfigError::ZeroMaxLoad);
    }

    Ok(RobotConfig {
        id: id.to_string(),
        battery_percent,
        max_load_kg,
    })
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
