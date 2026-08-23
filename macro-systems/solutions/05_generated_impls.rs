//! # 解答 05: macro-generated implでtrait実装をまとめる

trait Score {
    fn score(&self) -> u64;
}

macro_rules! impl_score {
    ($type:ty, |$value:ident| $body:expr) => {
        impl Score for $type {
            fn score(&self) -> u64 {
                let $value = self;
                $body
            }
        }
    };
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
struct BatteryReading {
    percent: u8,
}

#[derive(Debug, PartialEq, Eq)]
struct CargoReading {
    mass_grams: u32,
    fragile: bool,
}

impl_score!(BatteryReading, |reading| u64::from(reading.percent));
impl_score!(CargoReading, |reading| {
    u64::from(reading.mass_grams / 100) + u64::from(reading.fragile)
});

fn score_all<T: Score>(items: &[T]) -> u64 {
    items.iter().map(Score::score).sum()
}

fn main() {
    let battery = BatteryReading { percent: 82 };
    let cargo = CargoReading {
        mass_grams: 2_400,
        fragile: true,
    };
    println!("battery={}, cargo={}", battery.score(), cargo.score());
    println!(
        "battery total={}",
        score_all(std::slice::from_ref(&battery))
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn batteryのscoreを生成implから計算する() {
        let reading = BatteryReading { percent: 82 };

        assert_eq!(reading.score(), 82);
    }

    #[test]
    fn cargoは重量とfragileを生成implへ渡す() {
        let normal = CargoReading {
            mass_grams: 2_400,
            fragile: false,
        };
        let fragile = CargoReading {
            mass_grams: 2_400,
            fragile: true,
        };

        assert_eq!(normal.score(), 24);
        assert_eq!(fragile.score(), 25);
    }

    #[test]
    fn genericなscore_allは異なる型へそれぞれ使える() {
        let batteries = [
            BatteryReading { percent: 10 },
            BatteryReading { percent: 20 },
            BatteryReading { percent: 30 },
        ];
        let cargo = [CargoReading {
            mass_grams: 500,
            fragile: false,
        }];

        assert_eq!(score_all(&batteries), 60);
        assert_eq!(score_all(&cargo), 5);
    }

    #[test]
    fn 境界値をoverflowさせずに扱う() {
        let battery = BatteryReading { percent: u8::MAX };
        let cargo = CargoReading {
            mass_grams: u32::MAX,
            fragile: true,
        };

        assert_eq!(battery.score(), 255);
        assert_eq!(cargo.score(), u64::from(u32::MAX / 100) + 1);
    }

    #[test]
    fn scoreは元の値を変更しない() {
        let reading = BatteryReading { percent: 77 };
        let before = reading;
        let _ = reading.score();

        assert_eq!(reading, before);
    }
}
