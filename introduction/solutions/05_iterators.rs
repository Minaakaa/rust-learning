//! 問題 05 の解答例。

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

fn delivered_destinations(records: &[DeliveryRecord]) -> Vec<&str> {
    records
        .iter()
        .filter(|record| record.delivered)
        .map(|record| record.destination.as_str())
        .collect()
}

fn delivered_distance(records: &[DeliveryRecord]) -> u32 {
    records
        .iter()
        .filter(|record| record.delivered)
        .map(|record| record.distance_m)
        .sum()
}

fn into_long_route_names(records: Vec<DeliveryRecord>, minimum_distance_m: u32) -> Vec<String> {
    records
        .into_iter()
        .filter(|record| record.distance_m >= minimum_distance_m)
        .map(|record| record.destination)
        .collect()
}

fn parse_distances(inputs: &[&str]) -> Result<Vec<u32>, ParseIntError> {
    inputs.iter().map(|input| input.parse::<u32>()).collect()
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
