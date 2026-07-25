#![cfg_attr(not(test), allow(dead_code))]

//! # 問題 03: newtype で単位の取り違えを防ぐ
//!
//! 距離、時間、速度をすべて `u32` で受け取る API は、引数の順番を間違えても
//! コンパイルできてしまいます。1 フィールドのタプル構造体で newtype を作り、
//! 同じ内部表現でも異なる意味の値を型として区別してください。
//!
//! `Meters`、`Seconds`、`MetersPerSecond` はそれぞれ別の型です。たとえば
//! `travel_time(Seconds::new(10), MetersPerSecond::new(2))` はコンパイルできません。
//!
//! 仕様:
//! - 経路の合計は `checked_add` を使い、`u32` を超える場合は `None` を返す。
//! - 所要時間は端数を切り上げる。100 m を 30 m/s で進むなら 4 秒。
//! - 速度 0 の所要時間は計算できないため `None` を返す。
//! - 最長区間は `Ord` と `Copy` の derive を利用できる。

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Meters(u32);

impl Meters {
    const fn new(value: u32) -> Self {
        Self(value)
    }

    const fn value(self) -> u32 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Seconds(u32);

impl Seconds {
    const fn new(value: u32) -> Self {
        Self(value)
    }

    const fn value(self) -> u32 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct MetersPerSecond(u32);

impl MetersPerSecond {
    const fn new(value: u32) -> Self {
        Self(value)
    }

    const fn value(self) -> u32 {
        self.0
    }
}

fn route_length(legs: &[Meters]) -> Option<Meters> {
    todo!("{} 区間をオーバーフローなしで合計してください", legs.len())
}

fn travel_time(distance: Meters, speed: MetersPerSecond) -> Option<Seconds> {
    todo!(
        "{} m を {} m/s で進む秒数を切り上げてください",
        distance.value(),
        speed.value()
    )
}

fn longest_leg(legs: &[Meters]) -> Option<Meters> {
    todo!("{} 区間から最長距離を返してください", legs.len())
}

fn within_limit(distance: Meters, limit: Meters) -> bool {
    todo!(
        "{} m が上限 {} m 以内か比較してください",
        distance.value(),
        limit.value()
    )
}

fn main() {
    let route = [Meters::new(120), Meters::new(350), Meters::new(80)];
    let distance = route_length(&route).expect("小さな経路は加算できる");
    let duration = travel_time(distance, MetersPerSecond::new(5));

    println!("総距離: {} m、予想時間: {duration:?}", distance.value());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 同じ整数でも別の単位として保持する() {
        let distance = Meters::new(42);
        let duration = Seconds::new(42);
        let speed = MetersPerSecond::new(42);

        assert_eq!(distance.value(), 42);
        assert_eq!(duration.value(), 42);
        assert_eq!(speed.value(), 42);
    }

    #[test]
    fn 距離だけを安全に合計する() {
        assert_eq!(
            route_length(&[Meters::new(120), Meters::new(350), Meters::new(80)]),
            Some(Meters::new(550))
        );
        assert_eq!(route_length(&[]), Some(Meters::new(0)));
    }

    #[test]
    fn 距離の合計があふれたらnoneを返す() {
        assert_eq!(route_length(&[Meters::new(u32::MAX), Meters::new(1)]), None);
    }

    #[test]
    fn 所要時間の端数を切り上げる() {
        assert_eq!(
            travel_time(Meters::new(100), MetersPerSecond::new(30)),
            Some(Seconds::new(4))
        );
        assert_eq!(
            travel_time(Meters::new(120), MetersPerSecond::new(30)),
            Some(Seconds::new(4))
        );
        assert_eq!(
            travel_time(Meters::new(0), MetersPerSecond::new(30)),
            Some(Seconds::new(0))
        );
    }

    #[test]
    fn 速度ゼロでは時間を計算しない() {
        assert_eq!(travel_time(Meters::new(100), MetersPerSecond::new(0)), None);
    }

    #[test]
    fn deriveした順序で距離を比較する() {
        let legs = [Meters::new(80), Meters::new(350), Meters::new(120)];

        assert_eq!(longest_leg(&legs), Some(Meters::new(350)));
        assert_eq!(longest_leg(&[]), None);
        assert!(within_limit(Meters::new(350), Meters::new(350)));
        assert!(!within_limit(Meters::new(351), Meters::new(350)));
    }
}
