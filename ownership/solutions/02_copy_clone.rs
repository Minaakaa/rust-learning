//! 問題 02 の解答例。

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

fn translate(start: Position, dx: i32, dy: i32) -> (Position, Position) {
    let mut translated = start;
    translated.x += dx;
    translated.y += dy;
    (start, translated)
}

fn make_retry(original: Mission, retry_id: &str, reason: &str) -> (Mission, Mission) {
    let mut retry = original.clone();
    retry.id = retry_id.to_string();
    retry.attempts = retry.attempts.saturating_add(1);
    retry.notes.push(reason.to_string());
    (original, retry)
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
