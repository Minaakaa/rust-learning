#![cfg_attr(not(test), allow(dead_code))]

//! # 問題 05: イテレータで配送記録を集計する
//!
//! 配送記録の一覧を、明示的な `for` や `while` を使わずに処理します。
//! 4 つの TODO をイテレータメソッドのチェーンで完成させてください。
//!
//! ヒント:
//! - `iter()` は要素を借用し、`into_iter()` はコレクションと要素を消費する。
//! - `filter` と `map` は新しいイテレータを作るだけで、`collect` や `sum` が処理を完了させる。
//! - `Iterator<Item = Result<T, E>>` は `Result<Vec<T>, E>` に `collect` できる。

use std::num::ParseIntError;

#[derive(Debug, Clone, PartialEq, Eq)]
struct DeliveryRecord {
    destination: String,
    distance_m: u32,
    delivered: bool,
}

impl DeliveryRecord {
    fn new(destination: &str, distance_m: u32, delivered: bool) -> Self {
        Self {
            destination: destination.to_string(),
            distance_m,
            delivered,
        }
    }
}

/// 完了した配送先を、元の順序のまま借用して返す。
fn delivered_destinations(records: &[DeliveryRecord]) -> Vec<&str> {
    todo!(
        "iter、filter、map、collect を組み合わせてください: {} 件",
        records.len()
    )
}

/// 完了した配送だけの距離を合計する。
fn delivered_distance(records: &[DeliveryRecord]) -> u32 {
    todo!(
        "完了した記録の距離を sum してください: {} 件",
        records.len()
    )
}

/// 指定距離以上の配送先を所有した String として返し、入力の Vec を消費する。
fn into_long_route_names(records: Vec<DeliveryRecord>, minimum_distance_m: u32) -> Vec<String> {
    todo!(
        "into_iter で要素を所有してください: {} 件、最小距離 {minimum_distance_m} m",
        records.len()
    )
}

/// すべての文字列を u32 に変換する。1 つでも失敗したら最初のエラーを返す。
fn parse_distances(inputs: &[&str]) -> Result<Vec<u32>, ParseIntError> {
    todo!(
        "Result のイテレータを collect してください: {} 件",
        inputs.len()
    )
}

fn main() {
    let records = vec![
        DeliveryRecord::new("図書館", 120, true),
        DeliveryRecord::new("研究棟", 650, false),
    ];
    println!("配送済み: {:?}", delivered_destinations(&records));
    println!("配送済み距離: {} m", delivered_distance(&records));
    println!("長距離便: {:?}", into_long_route_names(records, 500));
    println!("解析結果: {:?}", parse_distances(&["120", "650"]));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn records() -> Vec<DeliveryRecord> {
        vec![
            DeliveryRecord::new("図書館", 120, true),
            DeliveryRecord::new("研究棟", 650, false),
            DeliveryRecord::new("学生寮", 900, true),
            DeliveryRecord::new("食堂", 80, true),
        ]
    }

    #[test]
    fn 完了した配送先を順序どおり借用する() {
        let records = records();
        assert_eq!(
            delivered_destinations(&records),
            vec!["図書館", "学生寮", "食堂"]
        );
        assert_eq!(records.len(), 4);
    }

    #[test]
    fn 完了した距離だけを合計する() {
        assert_eq!(delivered_distance(&records()), 1_100);
        assert_eq!(delivered_distance(&[]), 0);
    }

    #[test]
    fn 長距離便の名前を所有して返す() {
        assert_eq!(
            into_long_route_names(records(), 650),
            vec!["研究棟".to_string(), "学生寮".to_string()]
        );
    }

    #[test]
    fn すべての距離を解析する() {
        assert_eq!(parse_distances(&["120", "650", "0"]), Ok(vec![120, 650, 0]));
        assert_eq!(parse_distances(&[]), Ok(Vec::new()));
    }

    #[test]
    fn 解析できない距離があればエラーにする() {
        assert!(parse_distances(&["120", "遠い", "80"]).is_err());
    }
}
