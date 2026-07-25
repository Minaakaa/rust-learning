//! 問題 03 の解答例。

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
    legs.iter()
        .try_fold(0_u32, |total, leg| total.checked_add(leg.value()))
        .map(Meters::new)
}

fn travel_time(distance: Meters, speed: MetersPerSecond) -> Option<Seconds> {
    let speed = speed.value();
    if speed == 0 {
        return None;
    }

    Some(Seconds::new(distance.value().div_ceil(speed)))
}

fn longest_leg(legs: &[Meters]) -> Option<Meters> {
    legs.iter().copied().max()
}

fn within_limit(distance: Meters, limit: Meters) -> bool {
    distance <= limit
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
