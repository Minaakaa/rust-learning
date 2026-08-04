#![cfg_attr(not(test), allow(dead_code))]

//! # 問題 01: 関連型でsensorごとの測定契約を表す
//!
//! `BatterySensor`と`CargoScanner`は、どちらも1件の測定値を返します
//! ただし、成功時の型も失敗時の型も互いに異なります
//! `Sensor`の関連型`Reading`と`Error`を使い、1つのgeneric関数から両方を扱ってください
//!
//! 仕様:
//! - `Sensor`は`Identified`をsupertraitとし、すべてのsampleへsensor IDを付ける
//! - `BatterySensor`は入力を先頭から1件消費し、0から100までを`BatteryReading`にする
//! - battery値が101以上なら、元の値を`BatteryError::OutOfRange`へ残す
//! - battery入力がなければ`BatteryError::NoReading`を返す
//! - `CargoScanner`は荷物を先頭から所有値として取り出す
//! - 荷物がなければ`CargoScanError::NoCargo`を返す
//! - `sample_once`は`Sample<S::Reading>`を返し、具体的なreading型を列挙しない
//! - `read_battery_percent`は関連型の等値制約で`BatteryReading`を要求する
//! - `Clone`、`Copy`、`Debug`など、処理に不要な境界を追加しない
//!
//! TODO:
//! - 2つの`Sensor::read`をそれぞれの関連型に従って実装する
//! - `sample_once`で測定に成功したsensor IDとreadingを1つの`Sample`へまとめる
//! - `read_battery_percent`でreadingからbattery残量を取り出す
//!
//! ヒント:
//! - genericな文脈では関連型を`S::Reading`と書ける
//! - 完全修飾構文では`<BatterySensor as Sensor>::Reading`と書ける
//! - `VecDeque::pop_front`は値の所有権を取り出すため、荷物の複製は不要

use std::collections::VecDeque;

trait Identified {
    fn id(&self) -> &str;
}

trait Sensor: Identified {
    type Reading;
    type Error;

    fn read(&mut self) -> Result<Self::Reading, Self::Error>;
}

#[derive(Debug, PartialEq, Eq)]
struct Sample<R> {
    sensor_id: String,
    reading: R,
}

impl<R> Sample<R> {
    fn sensor_id(&self) -> &str {
        &self.sensor_id
    }

    fn reading(&self) -> &R {
        &self.reading
    }

    fn into_parts(self) -> (String, R) {
        (self.sensor_id, self.reading)
    }
}

fn sample_once<S>(sensor: &mut S) -> Result<Sample<S::Reading>, S::Error>
where
    S: Sensor,
{
    // 未実行のclosureで必要なmethodだけを型検査する
    let _required_methods = |sensor: &mut S| {
        let _ = sensor.id();
        let _: Result<S::Reading, S::Error> = sensor.read();
    };
    let _ = sensor;
    todo!("sensor固有の関連型を保ったまま1件測定してください")
}

#[derive(Debug, PartialEq, Eq)]
struct BatteryReading {
    percent: u8,
}

impl BatteryReading {
    const fn percent(&self) -> u8 {
        self.percent
    }
}

#[derive(Debug, PartialEq, Eq)]
enum BatteryError {
    NoReading,
    OutOfRange(u16),
}

struct BatterySensor {
    id: String,
    pending: VecDeque<u16>,
}

impl BatterySensor {
    fn new(id: &str, pending: impl IntoIterator<Item = u16>) -> Self {
        Self {
            id: id.to_owned(),
            pending: pending.into_iter().collect(),
        }
    }

    fn remaining(&self) -> usize {
        self.pending.len()
    }
}

impl Identified for BatterySensor {
    fn id(&self) -> &str {
        &self.id
    }
}

impl Sensor for BatterySensor {
    type Reading = BatteryReading;
    type Error = BatteryError;

    fn read(&mut self) -> Result<Self::Reading, Self::Error> {
        todo!(
            "残り{}件からbattery値を1件検証してください",
            self.pending.len()
        )
    }
}

fn read_battery_percent<S>(sensor: &mut S) -> Result<u8, S::Error>
where
    S: Sensor<Reading = BatteryReading>,
{
    let _ = sensor;
    todo!("関連型がBatteryReadingであるsensorからpercentを読み取ってください")
}

#[derive(Debug, PartialEq, Eq)]
struct CargoReading {
    cargo_id: String,
    mass_grams: u32,
}

impl CargoReading {
    fn new(cargo_id: impl Into<String>, mass_grams: u32) -> Self {
        Self {
            cargo_id: cargo_id.into(),
            mass_grams,
        }
    }

    fn cargo_id(&self) -> &str {
        &self.cargo_id
    }

    const fn mass_grams(&self) -> u32 {
        self.mass_grams
    }
}

#[derive(Debug, PartialEq, Eq)]
enum CargoScanError {
    NoCargo,
}

struct CargoScanner {
    id: String,
    pending: VecDeque<CargoReading>,
}

impl CargoScanner {
    fn new(id: &str, pending: impl IntoIterator<Item = CargoReading>) -> Self {
        Self {
            id: id.to_owned(),
            pending: pending.into_iter().collect(),
        }
    }

    fn remaining(&self) -> usize {
        self.pending.len()
    }
}

impl Identified for CargoScanner {
    fn id(&self) -> &str {
        &self.id
    }
}

impl Sensor for CargoScanner {
    type Reading = CargoReading;
    type Error = CargoScanError;

    fn read(&mut self) -> Result<Self::Reading, Self::Error> {
        todo!(
            "残り{}件から荷物を所有値として取り出してください",
            self.pending.len()
        )
    }
}

fn main() {
    let mut sensor = BatterySensor::new("配送ロボット-1201", [82]);
    let sample = sample_once(&mut sensor).expect("バッテリーを測定できる");

    println!(
        "{}: バッテリー {}%",
        sample.sensor_id(),
        sample.reading().percent()
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 完全修飾した関連型でbattery_sampleを受け取る() {
        let mut sensor = BatterySensor::new("BAT-01", [64]);

        let sample: Result<
            Sample<<BatterySensor as Sensor>::Reading>,
            <BatterySensor as Sensor>::Error,
        > = sample_once(&mut sensor);
        let sample = sample.expect("測定値がある");

        assert_eq!(sample.sensor_id(), "BAT-01");
        assert_eq!(sample.reading().percent(), 64);
        assert_eq!(sensor.remaining(), 0);
    }

    #[test]
    fn 関連型の等値制約でbattery_readingを要求する() {
        let mut sensor = BatterySensor::new("BAT-equality", [73]);

        let percent: Result<u8, <BatterySensor as Sensor>::Error> =
            read_battery_percent(&mut sensor);

        assert_eq!(percent, Ok(73));
        assert_eq!(sensor.remaining(), 0);
    }

    #[test]
    fn batteryの境界値を入力順に読み取る() {
        let mut sensor = BatterySensor::new("BAT-boundary", [0, 100]);

        assert_eq!(sample_once(&mut sensor).unwrap().reading().percent(), 0);
        assert_eq!(sample_once(&mut sensor).unwrap().reading().percent(), 100);
        assert_eq!(sensor.remaining(), 0);
    }

    #[test]
    fn batteryの範囲外値を元の値付きで拒否する() {
        let mut sensor = BatterySensor::new("BAT-invalid", [101, u16::MAX, 55]);

        assert_eq!(sample_once(&mut sensor), Err(BatteryError::OutOfRange(101)));
        assert_eq!(
            sample_once(&mut sensor),
            Err(BatteryError::OutOfRange(u16::MAX))
        );
        assert_eq!(sample_once(&mut sensor).unwrap().reading().percent(), 55);
    }

    #[test]
    fn batteryの入力が空なら専用errorを返す() {
        let mut sensor = BatterySensor::new("BAT-empty", []);

        assert_eq!(sample_once(&mut sensor), Err(BatteryError::NoReading));
        assert_eq!(sensor.remaining(), 0);
    }

    #[test]
    fn scannerはbatteryと異なるreadingとerrorを使う() {
        let mut scanner = CargoScanner::new("SCAN-01", [CargoReading::new("CG-1201", 2_400)]);

        let sample: Result<
            Sample<<CargoScanner as Sensor>::Reading>,
            <CargoScanner as Sensor>::Error,
        > = sample_once(&mut scanner);
        let sample = sample.expect("荷物がある");

        assert_eq!(sample.sensor_id(), "SCAN-01");
        assert_eq!(sample.reading().cargo_id(), "CG-1201");
        assert_eq!(sample.reading().mass_grams(), 2_400);
        assert_eq!(scanner.remaining(), 0);
    }

    #[test]
    fn scannerの入力が空ならbatteryとは異なるerrorを返す() {
        let mut scanner = CargoScanner::new("SCAN-empty", []);

        assert_eq!(sample_once(&mut scanner), Err(CargoScanError::NoCargo));
    }

    #[test]
    fn cargo_idを複製せずsampleへ移す() {
        let cargo_id = String::from("所有荷物📦-1202");
        let cargo_id_pointer = cargo_id.as_ptr();
        let mut scanner = CargoScanner::new("SCAN-owned", [CargoReading::new(cargo_id, 9_800)]);

        let sample = sample_once(&mut scanner).expect("荷物がある");
        let (sensor_id, reading) = sample.into_parts();

        assert_eq!(sensor_id, "SCAN-owned");
        assert_eq!(reading.cargo_id(), "所有荷物📦-1202");
        assert_eq!(reading.cargo_id.as_ptr(), cargo_id_pointer);
    }

    #[test]
    fn cloneできない独自readingにもgeneric関数を使える() {
        #[derive(Debug, PartialEq, Eq)]
        struct NonCloneReading(String);

        struct OneShotSensor {
            id: String,
            reading: Option<NonCloneReading>,
        }

        impl Identified for OneShotSensor {
            fn id(&self) -> &str {
                &self.id
            }
        }

        impl Sensor for OneShotSensor {
            type Reading = NonCloneReading;
            type Error = &'static str;

            fn read(&mut self) -> Result<Self::Reading, Self::Error> {
                self.reading.take().ok_or("測定済み")
            }
        }

        let reading = String::from("複製できない校正値");
        let reading_pointer = reading.as_ptr();
        let mut sensor = OneShotSensor {
            id: "ONE-SHOT".to_owned(),
            reading: Some(NonCloneReading(reading)),
        };

        let sample = sample_once(&mut sensor).expect("最初の1回は測定できる");
        let (_, reading) = sample.into_parts();

        assert_eq!(reading.0, "複製できない校正値");
        assert_eq!(reading.0.as_ptr(), reading_pointer);
        assert_eq!(sample_once(&mut sensor), Err("測定済み"));
    }

    #[test]
    fn utf8のsensor_idとcargo_idをそのまま保持する() {
        let mut scanner = CargoScanner::new(
            "本郷・荷物scanner🔎-七",
            [CargoReading::new("精密機器📦-あ", 1)],
        );

        let sample = sample_once(&mut scanner).expect("荷物がある");

        assert_eq!(sample.sensor_id(), "本郷・荷物scanner🔎-七");
        assert_eq!(sample.reading().cargo_id(), "精密機器📦-あ");
    }
}
