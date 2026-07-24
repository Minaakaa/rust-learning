#![cfg_attr(not(test), allow(dead_code))]

//! # 問題 02: Copy と Clone を使い分ける
//!
//! `Position` は小さな整数だけを持つため `Copy` です。代入や関数呼び出しで暗黙に
//! コピーされ、元の値も引き続き使えます。一方、`Mission` は `String` と `Vec` を
//! 所有するため `Copy` にはできませんが、明示的に `clone()` できます。
//!
//! 制約:
//! - `Position` では `clone()` を呼ばない。
//! - `Mission` を複製する箇所では、フィールドを一つずつ作り直さず `clone()` を使う。
//! - `make_retry` は元のミッションと再試行ミッションの両方を返す。
//! - 試行回数は `u8::MAX` を超えないよう `saturating_add` を使う。

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Position {
    x: i32,
    y: i32,
}

impl Position {
    fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Mission {
    id: String,
    destination: String,
    checkpoint: Position,
    attempts: u8,
    notes: Vec<String>,
}

impl Mission {
    fn new(id: &str, destination: &str, checkpoint: Position) -> Self {
        Self {
            id: id.to_string(),
            destination: destination.to_string(),
            checkpoint,
            attempts: 0,
            notes: Vec::new(),
        }
    }
}

/// 元の座標と、差分を加えた座標を返す。
///
/// `Position` が `Copy` なので、同じ `start` の値を両方に利用できる。
fn translate(start: Position, dx: i32, dy: i32) -> (Position, Position) {
    todo!(
        "座標 ({}, {}) を ({dx}, {dy}) だけ移動してください",
        start.x,
        start.y
    )
}

/// 元のミッションを保持したまま、独立した再試行用ミッションを作る。
fn make_retry(original: Mission, retry_id: &str, reason: &str) -> (Mission, Mission) {
    todo!(
        "ミッション {} を clone し、再試行 {} を作って理由「{}」を追加してください",
        original.id,
        retry_id,
        reason
    )
}

fn main() {
    let start = Position::new(10, 20);
    let (original_position, next_position) = translate(start, 3, -2);
    println!("座標: {original_position:?} -> {next_position:?}");

    let mission = Mission::new("M-01", "図書館", start);
    let (original, retry) = make_retry(mission, "M-01-R1", "通路が混雑");
    println!("元のミッション: {}", original.id);
    println!("再試行: {}", retry.id);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 型の複製方法が意図どおりである() {
        fn assert_copy<T: Copy>() {}
        fn assert_clone<T: Clone>() {}

        assert_copy::<Position>();
        assert_clone::<Mission>();
    }

    #[test]
    fn copyされた元の座標を引き続き使える() {
        let start = Position::new(10, 20);

        let (original, translated) = translate(start, 3, -2);

        assert_eq!(original, Position::new(10, 20));
        assert_eq!(translated, Position::new(13, 18));
        assert_eq!(start, Position::new(10, 20));
    }

    #[test]
    fn cloneで独立した再試行ミッションを作る() {
        let mut mission = Mission::new("M-02", "研究棟", Position::new(4, 8));
        mission.attempts = 2;
        mission.notes.push("受付で待機".to_string());

        let (original, retry) = make_retry(mission, "M-02-R3", "エレベーター点検中");

        assert_eq!(original.id, "M-02");
        assert_eq!(original.attempts, 2);
        assert_eq!(original.notes, vec!["受付で待機"]);
        assert_eq!(retry.id, "M-02-R3");
        assert_eq!(retry.destination, "研究棟");
        assert_eq!(retry.checkpoint, Position::new(4, 8));
        assert_eq!(retry.attempts, 3);
        assert_eq!(retry.notes, vec!["受付で待機", "エレベーター点検中"]);
    }

    #[test]
    fn cloneしたヒープデータは独立している() {
        let mission = Mission::new("M-03", "学生寮", Position::new(1, 1));
        let (original, mut retry) = make_retry(mission, "M-03-R1", "再配達");

        retry.destination.push_str("・東棟");
        retry.notes.push("受取人へ連絡済み".to_string());

        assert_eq!(original.destination, "学生寮");
        assert!(original.notes.is_empty());
        assert_eq!(retry.destination, "学生寮・東棟");
        assert_eq!(retry.notes, vec!["再配達", "受取人へ連絡済み"]);
    }

    #[test]
    fn 試行回数をオーバーフローさせない() {
        let mut mission = Mission::new("M-04", "食堂", Position::new(0, 0));
        mission.attempts = u8::MAX;

        let (_, retry) = make_retry(mission, "M-04-R", "最終確認");

        assert_eq!(retry.attempts, u8::MAX);
    }
}
