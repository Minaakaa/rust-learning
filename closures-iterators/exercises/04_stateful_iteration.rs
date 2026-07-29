#![cfg_attr(not(test), allow(dead_code))]

//! # 問題 04: 状態付き・失敗可能な反復
//!
//! 配送経路を、電池残量を更新しながら遅延評価します
//! 同じ区間列から、オーバーフローを検査する合計値も求めます
//!
//! 仕様:
//! - `reachable_checkpoints` の戻り値を `impl Iterator<Item = BatteryCheckpoint<'a>> + 'a` に変更する
//! - `scan` の状態として残り電力量を保持する
//! - 各区間を走行できれば残量を減らし、採用した区間だけへ1から始まる連番を付ける
//! - 最初に走行できない区間で終了し、その後の安い区間で再開しない
//! - `scan` の後に `fuse` を使い、最初の `None` 以降を常に `None` にする
//! - `checked_totals` は `try_fold` と `checked_add` で距離と電力量を合計する
//! - 同じ区間で両方があふれる場合は距離のエラーを先に返す
//! - どちらの処理でも区間を複製しない
//!
//! ヒント:
//! - `scan` のクロージャは状態への `&mut` と次の要素を受け取る
//! - `scan` 自体は `None` の後に再び `Some` を返す可能性がある
//! - `try_fold` のクロージャで `?` を使うと最初のエラーで短絡できる

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
) -> std::iter::Empty<BatteryCheckpoint<'a>> {
    todo!(
        "{} 区間を残量 {battery_wh} Wh から遅延評価してください",
        legs.len()
    )
}

fn checked_totals(legs: &[RouteLeg]) -> Result<RouteTotals, TotalsError> {
    todo!(
        "{} 区間の距離と電力量をオーバーフロー検査付きで合計してください",
        legs.len()
    )
}

fn main() {
    let legs = vec![
        RouteLeg::new("管制室→図書館", 300, 4),
        RouteLeg::new("図書館→工学部2号館", 450, 6),
    ];
    let checkpoints: Vec<_> = reachable_checkpoints(&legs, 12).collect();

    println!("到達可能な区間: {checkpoints:?}");
    println!("経路合計: {:?}", checked_totals(&legs));
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
