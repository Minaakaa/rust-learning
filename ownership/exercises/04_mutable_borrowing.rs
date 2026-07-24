#![cfg_attr(not(test), allow(dead_code))]

//! # 問題 04: 可変借用で安全に更新する
//!
//! `&mut T` は値を所有せず、一定期間だけ排他的な更新権を借ります。関数が終わると
//! 借用も終わるため、呼び出し元は同じ値を再び使えます。また、`&mut Robot` を別の
//! 関数へ渡すと、その呼び出しの間だけ自動的に再借用されます。
//!
//! 制約:
//! - `Robot` に `Clone` を追加しない。
//! - 関数の引数を所有する `Robot` に変更しない。
//! - 充電量は 100% で飽和させ、実際に増えた量を返す。
//! - 荷物の引き渡しでは `String` を複製せず、元の `Vec` から移動する。
//! - 空のロボットから引き渡す場合、どちらのロボットも変更しない。
//!
//! 同じ `Robot` を `transfer_front(&mut robot, &mut robot)` のように 2 回渡すことは
//! できません。1 つの値に同時に 2 つの可変参照ができるためです。

#[derive(Debug, PartialEq, Eq)]
struct Robot {
    id: String,
    battery_percent: u8,
    cargo: Vec<String>,
    events: Vec<String>,
}

impl Robot {
    fn new(id: &str, battery_percent: u8, cargo: Vec<String>) -> Self {
        Self {
            id: id.to_string(),
            battery_percent,
            cargo,
            events: Vec::new(),
        }
    }
}

/// バッテリーを最大 100% まで充電し、実際に増えた量を返す。
fn recharge(robot: &mut Robot, amount: u8) -> u8 {
    todo!("{} を {amount}% までの範囲で充電してください", robot.id)
}

/// イベントをログの末尾へ追加する。
fn record_event(robot: &mut Robot, event: &str) {
    todo!("{} のログへ「{event}」を追加してください", robot.id)
}

/// 同じ可変参照を `recharge` と `record_event` へ順番に再借用する。
///
/// バッテリーを 100% まで充電し、`"シフト準備: N% 充電"` を記録して実充電量を返す。
fn prepare_for_shift(robot: &mut Robot) -> u8 {
    todo!(
        "{} を充電し、その同じ可変参照でログを記録してください",
        robot.id
    )
}

/// `source` の先頭の荷物を `destination` の末尾へ移す。
///
/// 成功時は両方へ `"荷物 ID を引き渡し"` を記録して `true`、荷物がなければ
/// 何も変更せず `false` を返す。
fn transfer_front(source: &mut Robot, destination: &mut Robot) -> bool {
    todo!(
        "{} の先頭荷物を {} へムーブしてください",
        source.id,
        destination.id
    )
}

fn main() {
    let mut source = Robot::new("RB-20", 65, vec!["P-20".to_string()]);
    let mut destination = Robot::new("RB-21", 90, Vec::new());

    prepare_for_shift(&mut source);
    transfer_front(&mut source, &mut destination);

    println!("引き渡し元: {source:?}");
    println!("引き渡し先: {destination:?}");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 最大100パーセントまで充電する() {
        let mut robot = Robot::new("RB-20", 92, Vec::new());

        let added = recharge(&mut robot, 15);

        assert_eq!(added, 8);
        assert_eq!(robot.battery_percent, 100);
        assert!(robot.events.is_empty());
    }

    #[test]
    fn 可変借用でイベントを追記する() {
        let mut robot = Robot::new("RB-21", 80, Vec::new());

        record_event(&mut robot, "点検完了");
        record_event(&mut robot, "出発");

        assert_eq!(robot.id, "RB-21");
        assert_eq!(robot.events, vec!["点検完了", "出発"]);
    }

    #[test]
    fn 同じ可変参照を順番に再借用する() {
        let mut robot = Robot::new("RB-22", 65, Vec::new());

        let added = prepare_for_shift(&mut robot);

        assert_eq!(added, 35);
        assert_eq!(robot.battery_percent, 100);
        assert_eq!(robot.events, vec!["シフト準備: 35% 充電"]);
    }

    #[test]
    fn stringを複製せず別のロボットへ移す() {
        let mut source = Robot::new("RB-23", 70, vec!["P-230".to_string(), "P-231".to_string()]);
        let mut destination = Robot::new("RB-24", 80, vec!["P-240".to_string()]);

        let transferred = transfer_front(&mut source, &mut destination);

        assert!(transferred);
        assert_eq!(source.cargo, vec!["P-231"]);
        assert_eq!(destination.cargo, vec!["P-240", "P-230"]);
        assert_eq!(source.events, vec!["荷物 P-230 を引き渡し"]);
        assert_eq!(destination.events, vec!["荷物 P-230 を引き渡し"]);
    }

    #[test]
    fn 荷物がなければ何も変更しない() {
        let mut source = Robot::new("RB-25", 70, Vec::new());
        let mut destination = Robot::new("RB-26", 80, vec!["P-260".to_string()]);

        let transferred = transfer_front(&mut source, &mut destination);

        assert!(!transferred);
        assert!(source.cargo.is_empty());
        assert!(source.events.is_empty());
        assert_eq!(destination.cargo, vec!["P-260"]);
        assert!(destination.events.is_empty());
    }
}
