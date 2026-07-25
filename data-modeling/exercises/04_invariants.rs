#![cfg_attr(not(test), allow(dead_code))]

//! # 問題 04: 不変条件を守るバッテリー型
//!
//! 裸の `u8` では 150% のバッテリー残量も表せてしまいます。フィールドを外から直接
//! 作る代わりに検証付きコンストラクタを通し、その後のすべてのメソッドも
//! 「残量は常に 0..=100」という不変条件を保つようにしてください。
//!
//! 仕様:
//! - `Battery::new` は 0 と 100 を含む範囲だけを受理する。
//! - 20% 以下を低残量と判定する。
//! - 消費量が残量を超える場合はエラーを返し、残量を変更しない。
//! - 充電は 100% で止め、`u8` の加算オーバーフローも起こさない。
//! - `Default` は満充電を返す。
//! - `percent` 以外から直接 `Battery` を構築しない。

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BatteryError {
    OutOfRange(u8),
    Insufficient { available: u8, requested: u8 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Battery {
    percent: u8,
}

impl Battery {
    fn new(percent: u8) -> Result<Self, BatteryError> {
        todo!("残量 {percent}% が 0..=100 なら Battery を作ってください")
    }

    fn percent(&self) -> u8 {
        todo!("検証済み残量 {} を返してください", self.percent)
    }

    fn is_low(&self) -> bool {
        todo!("残量 {}% が 20% 以下か判定してください", self.percent)
    }

    fn consume(&mut self, requested: u8) -> Result<(), BatteryError> {
        todo!(
            "残量 {}% から {requested}% を安全に消費してください",
            self.percent
        )
    }

    fn recharge(&mut self, amount: u8) {
        todo!(
            "残量 {}% に {amount}% を加え、100% で止めてください",
            self.percent
        )
    }
}

impl Default for Battery {
    fn default() -> Self {
        todo!("満充電の Battery を返してください")
    }
}

fn main() {
    let mut battery = Battery::default();
    battery.consume(35).expect("満充電なら 35% 消費できる");
    battery.recharge(10);

    println!("現在のバッテリー残量: {}%", battery.percent());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 有効な境界値だけを構築する() {
        assert_eq!(Battery::new(0), Ok(Battery { percent: 0 }));
        assert_eq!(Battery::new(100), Ok(Battery { percent: 100 }));
        assert_eq!(Battery::new(101), Err(BatteryError::OutOfRange(101)));
        assert_eq!(
            Battery::new(u8::MAX),
            Err(BatteryError::OutOfRange(u8::MAX))
        );
    }

    #[test]
    fn getterと低残量判定が不変条件を利用する() {
        let empty = Battery::new(0).unwrap();
        let boundary = Battery::new(20).unwrap();
        let enough = Battery::new(21).unwrap();

        assert_eq!(boundary.percent(), 20);
        assert!(empty.is_low());
        assert!(boundary.is_low());
        assert!(!enough.is_low());
    }

    #[test]
    fn 残量の範囲で消費する() {
        let mut battery = Battery::new(70).unwrap();

        assert_eq!(battery.consume(20), Ok(()));
        assert_eq!(battery.percent(), 50);
        assert_eq!(battery.consume(50), Ok(()));
        assert_eq!(battery.percent(), 0);
        assert_eq!(battery.consume(0), Ok(()));
    }

    #[test]
    fn 消費できない場合は状態を変えない() {
        let mut battery = Battery::new(30).unwrap();

        assert_eq!(
            battery.consume(31),
            Err(BatteryError::Insufficient {
                available: 30,
                requested: 31,
            })
        );
        assert_eq!(battery.percent(), 30);
    }

    #[test]
    fn 充電しても100パーセントを超えない() {
        let mut battery = Battery::new(90).unwrap();
        battery.recharge(20);
        assert_eq!(battery.percent(), 100);

        battery.recharge(u8::MAX);
        assert_eq!(battery.percent(), 100);
    }

    #[test]
    fn defaultとderiveした比較を利用できる() {
        assert_eq!(Battery::default(), Battery::new(100).unwrap());
        assert!(Battery::new(40).unwrap() < Battery::new(80).unwrap());
    }
}
