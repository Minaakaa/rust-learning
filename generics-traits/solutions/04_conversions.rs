//! 問題 04 の解答例

#[derive(Debug, Clone, PartialEq, Eq)]
struct Label(String);

impl Label {
    fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for Label {
    fn from(value: &str) -> Self {
        Self(value.to_owned())
    }
}

impl From<String> for Label {
    fn from(value: String) -> Self {
        Self(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BatteryError {
    OutOfRange(u16),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct BatteryPercent(u8);

impl BatteryPercent {
    const fn value(self) -> u8 {
        self.0
    }
}

impl TryFrom<u16> for BatteryPercent {
    type Error = BatteryError;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match u8::try_from(value) {
            Ok(percent @ 0..=100) => Ok(Self(percent)),
            _ => Err(BatteryError::OutOfRange(value)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RobotStatus {
    label: Label,
    battery: BatteryPercent,
}

impl RobotStatus {
    fn new(label: impl Into<Label>, battery: BatteryPercent) -> Self {
        Self {
            label: label.into(),
            battery,
        }
    }

    fn label(&self) -> &str {
        self.label.as_str()
    }

    const fn battery_percent(&self) -> u8 {
        self.battery.value()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fromで文字列スライスを所有値へ変換する() {
        let label = Label::from("図書館配送ロボット");

        assert_eq!(label.as_str(), "図書館配送ロボット");
    }

    #[test]
    fn fromで所有文字列を受け取る() {
        let owned = String::from("実験棟ロボット🚚");
        let original_ptr = owned.as_ptr();
        let original_capacity = owned.capacity();
        let label = Label::from(owned);

        assert_eq!(label.as_str(), "実験棟ロボット🚚");
        assert_eq!(label.0.as_ptr(), original_ptr);
        assert_eq!(label.0.capacity(), original_capacity);
    }

    #[test]
    fn fromの包括実装によってintoを利用できる() {
        let borrowed: Label = "受付ロボット".into();
        let owned: Label = String::from("倉庫ロボット").into();

        assert_eq!(borrowed, Label("受付ロボット".to_string()));
        assert_eq!(owned, Label("倉庫ロボット".to_string()));
    }

    #[test]
    fn コンストラクタが借用文字列と所有文字列を受け取る() {
        let battery = BatteryPercent::try_from(80).unwrap();
        let borrowed = RobotStatus::new("RB-80", battery);
        let owned = RobotStatus::new(String::from("RB-81"), battery);

        assert_eq!(borrowed.label(), "RB-80");
        assert_eq!(owned.label(), "RB-81");
        assert_eq!(borrowed.battery_percent(), 80);
        assert_eq!(owned.battery_percent(), 80);
    }

    #[test]
    fn コンストラクタが既存のlabelも受け取る() {
        let label = Label::from("RB-82");
        let status = RobotStatus::new(label, BatteryPercent::try_from(45).unwrap());

        assert_eq!(status.label(), "RB-82");
        assert_eq!(status.battery_percent(), 45);
    }

    #[test]
    fn try_fromが有効な境界値を受理する() {
        assert_eq!(BatteryPercent::try_from(0), Ok(BatteryPercent(0)));
        assert_eq!(BatteryPercent::try_from(100), Ok(BatteryPercent(100)));
    }

    #[test]
    fn try_fromが範囲外の元の値をエラーに残す() {
        assert_eq!(
            BatteryPercent::try_from(101),
            Err(BatteryError::OutOfRange(101))
        );
        assert_eq!(
            BatteryPercent::try_from(u16::MAX),
            Err(BatteryError::OutOfRange(u16::MAX))
        );
    }

    #[test]
    fn try_fromの包括実装によってtry_intoを利用できる() {
        let valid: Result<BatteryPercent, BatteryError> = 63_u16.try_into();
        let invalid: Result<BatteryPercent, BatteryError> = 500_u16.try_into();

        assert_eq!(valid, Ok(BatteryPercent(63)));
        assert_eq!(invalid, Err(BatteryError::OutOfRange(500)));
    }
}
