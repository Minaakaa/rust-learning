//! 問題 04 の解答例

#[derive(Debug, PartialEq, Eq)]
struct RouteLeg {
    name: String,
    distance_m: u32,
    energy_wh: u32,
}

impl RouteLeg {
    fn new(name: &str, distance_m: u32, energy_wh: u32) -> Self {
        Self {
            name: name.to_string(),
            distance_m,
            energy_wh,
        }
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn distance_m(&self) -> u32 {
        self.distance_m
    }

    fn energy_wh(&self) -> u32 {
        self.energy_wh
    }
}

#[derive(Debug, PartialEq, Eq)]
struct BatteryCheckpoint<'a> {
    sequence: usize,
    leg: &'a RouteLeg,
    remaining_wh: u32,
}

impl BatteryCheckpoint<'_> {
    fn sequence(&self) -> usize {
        self.sequence
    }

    fn leg(&self) -> &RouteLeg {
        self.leg
    }

    fn remaining_wh(&self) -> u32 {
        self.remaining_wh
    }
}

#[derive(Debug, Default, PartialEq, Eq)]
struct RouteTotals {
    distance_m: u32,
    energy_wh: u32,
}

#[derive(Debug, PartialEq, Eq)]
enum TotalsError {
    DistanceOverflow { sequence: usize },
    EnergyOverflow { sequence: usize },
}

fn reachable_checkpoints<'a>(
    legs: &'a [RouteLeg],
    battery_wh: u32,
) -> impl Iterator<Item = BatteryCheckpoint<'a>> + 'a {
    legs.iter()
        .scan(battery_wh, |remaining_wh, leg| {
            if leg.energy_wh() > *remaining_wh {
                return None;
            }

            *remaining_wh -= leg.energy_wh();
            Some((leg, *remaining_wh))
        })
        .enumerate()
        .map(|(index, (leg, remaining_wh))| BatteryCheckpoint {
            sequence: index + 1,
            leg,
            remaining_wh,
        })
        .fuse()
}

fn checked_totals(legs: &[RouteLeg]) -> Result<RouteTotals, TotalsError> {
    legs.iter()
        .enumerate()
        .try_fold(RouteTotals::default(), |totals, (index, leg)| {
            let sequence = index + 1;
            let distance_m = totals
                .distance_m
                .checked_add(leg.distance_m())
                .ok_or(TotalsError::DistanceOverflow { sequence })?;
            let energy_wh = totals
                .energy_wh
                .checked_add(leg.energy_wh())
                .ok_or(TotalsError::EnergyOverflow { sequence })?;

            Ok(RouteTotals {
                distance_m,
                energy_wh,
            })
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 残量を更新しながら区間を順番に借用する() {
        let legs = vec![
            RouteLeg::new("管制室→図書館", 300, 3),
            RouteLeg::new("図書館→工学部", 400, 4),
            RouteLeg::new("工学部→食堂", 200, 2),
        ];

        let checkpoints: Vec<_> = reachable_checkpoints(&legs, 10).collect();

        assert_eq!(checkpoints.len(), 3);
        assert_eq!(checkpoints[0].sequence(), 1);
        assert_eq!(checkpoints[1].sequence(), 2);
        assert_eq!(checkpoints[2].sequence(), 3);
        assert_eq!(checkpoints[0].remaining_wh(), 7);
        assert_eq!(checkpoints[1].remaining_wh(), 3);
        assert_eq!(checkpoints[2].remaining_wh(), 1);
        assert_eq!(checkpoints[1].leg().name(), "図書館→工学部");
        assert_eq!(
            checkpoints[1].leg().name().as_ptr(),
            legs[1].name().as_ptr()
        );
    }

    #[test]
    fn 残量ちょうどの区間まで到達する() {
        let legs = vec![
            RouteLeg::new("A", 10, 2),
            RouteLeg::new("B", 20, 3),
            RouteLeg::new("C", 30, 1),
        ];

        let checkpoints: Vec<_> = reachable_checkpoints(&legs, 5).collect();

        assert_eq!(checkpoints.len(), 2);
        assert_eq!(checkpoints[1].leg().name(), "B");
        assert_eq!(checkpoints[1].remaining_wh(), 0);
    }

    #[test]
    fn 走行不能後の安い区間でも再開しない() {
        let legs = vec![
            RouteLeg::new("到達", 10, 3),
            RouteLeg::new("走行不能", 20, 8),
            RouteLeg::new("消費ゼロ", 30, 0),
        ];
        let mut checkpoints = reachable_checkpoints(&legs, 5);

        assert_eq!(checkpoints.next().unwrap().leg().name(), "到達");
        assert_eq!(checkpoints.next(), None);
        assert_eq!(checkpoints.next(), None);
    }

    #[test]
    fn 最初の区間が走行不能なら常にnoneを返す() {
        let legs = vec![RouteLeg::new("急坂", 50, 1), RouteLeg::new("平地", 10, 0)];
        let mut checkpoints = reachable_checkpoints(&legs, 0);

        assert_eq!(checkpoints.next(), None);
        assert_eq!(checkpoints.next(), None);
    }

    #[test]
    fn 残量ゼロでも消費ゼロの区間へ到達する() {
        let legs = vec![
            RouteLeg::new("充電台内", 0, 0),
            RouteLeg::new("下り坂", 120, 0),
        ];

        let checkpoints: Vec<_> = reachable_checkpoints(&legs, 0).collect();

        assert_eq!(checkpoints.len(), 2);
        assert!(checkpoints.iter().all(|step| step.remaining_wh() == 0));
    }

    #[test]
    fn 空の経路では空の反復とゼロの合計を返す() {
        assert_eq!(reachable_checkpoints(&[], 100).next(), None);
        assert_eq!(checked_totals(&[]), Ok(RouteTotals::default()));
    }

    #[test]
    fn 距離と電力量を検査付きで合計する() {
        let legs = vec![
            RouteLeg::new("本郷地区🚚", 300, 4),
            RouteLeg::new("工学部Ａ棟🔧", 450, 6),
        ];

        assert_eq!(
            checked_totals(&legs),
            Ok(RouteTotals {
                distance_m: 750,
                energy_wh: 10,
            })
        );
    }

    #[test]
    fn 距離のオーバーフロー位置を返す() {
        let legs = vec![
            RouteLeg::new("最大距離", u32::MAX, 0),
            RouteLeg::new("追加距離", 1, 0),
            RouteLeg::new("未処理", 1, 0),
        ];

        assert_eq!(
            checked_totals(&legs),
            Err(TotalsError::DistanceOverflow { sequence: 2 })
        );
    }

    #[test]
    fn 電力量のオーバーフロー位置を返す() {
        let legs = vec![
            RouteLeg::new("最大電力", 0, u32::MAX),
            RouteLeg::new("追加電力", 0, 1),
        ];

        assert_eq!(
            checked_totals(&legs),
            Err(TotalsError::EnergyOverflow { sequence: 2 })
        );
    }

    #[test]
    fn 同じ区間で両方があふれるなら距離を先に返す() {
        let legs = vec![
            RouteLeg::new("最大値", u32::MAX, u32::MAX),
            RouteLeg::new("両方を加算", 1, 1),
        ];

        assert_eq!(
            checked_totals(&legs),
            Err(TotalsError::DistanceOverflow { sequence: 2 })
        );
    }
}
