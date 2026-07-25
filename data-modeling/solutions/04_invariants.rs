//! 問題 04 の解答例。

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
        if percent <= 100 {
            Ok(Self { percent })
        } else {
            Err(BatteryError::OutOfRange(percent))
        }
    }

    fn percent(&self) -> u8 {
        self.percent
    }

    fn is_low(&self) -> bool {
        self.percent <= 20
    }

    fn consume(&mut self, requested: u8) -> Result<(), BatteryError> {
        if requested > self.percent {
            return Err(BatteryError::Insufficient {
                available: self.percent,
                requested,
            });
        }

        self.percent -= requested;
        Ok(())
    }

    fn recharge(&mut self, amount: u8) {
        self.percent = self.percent.saturating_add(amount).min(100);
    }
}

impl Default for Battery {
    fn default() -> Self {
        Self { percent: 100 }
    }
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
