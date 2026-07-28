//! 問題 02 の解答例

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LogLevel {
    Info,
    Warn,
    Error,
}

#[derive(Debug, PartialEq, Eq)]
struct LogEntry<'a> {
    robot_id: &'a str,
    level: LogLevel,
    message: &'a str,
}

impl<'a> LogEntry<'a> {
    fn robot_id(&self) -> &str {
        self.robot_id
    }

    const fn level(&self) -> LogLevel {
        self.level
    }

    fn message(&self) -> &str {
        self.message
    }

    fn into_message(self) -> &'a str {
        self.message
    }
}

#[derive(Debug, PartialEq, Eq)]
enum LogError<'a> {
    MalformedLine,
    EmptyRobotId,
    UnknownLevel(&'a str),
    EmptyMessage,
}

/// `robot_id|LEVEL|message` 形式の1行を解析する
fn parse_log<'a>(line: &'a str) -> Result<LogEntry<'a>, LogError<'a>> {
    let mut fields = line.splitn(3, '|');
    let robot_id = fields.next().ok_or(LogError::MalformedLine)?.trim();
    let level = fields.next().ok_or(LogError::MalformedLine)?.trim();
    let message = fields.next().ok_or(LogError::MalformedLine)?.trim();

    if robot_id.is_empty() {
        return Err(LogError::EmptyRobotId);
    }

    let level = match level {
        "INFO" => LogLevel::Info,
        "WARN" => LogLevel::Warn,
        "ERROR" => LogLevel::Error,
        unknown => return Err(LogError::UnknownLevel(unknown)),
    };

    if message.is_empty() {
        return Err(LogError::EmptyMessage);
    }

    Ok(LogEntry {
        robot_id,
        level,
        message,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn points_inside(whole: &str, part: &str) -> bool {
        let whole_start = whole.as_ptr() as usize;
        let whole_end = whole_start + whole.len();
        let part_start = part.as_ptr() as usize;
        let part_end = part_start + part.len();

        whole_start <= part_start && part_end <= whole_end
    }

    fn as_str<T>(value: &T) -> &str
    where
        T: AsRef<str> + ?Sized,
    {
        value.as_ref()
    }

    #[test]
    fn すべてのログレベルを解析する() {
        for (level, expected) in [
            ("INFO", LogLevel::Info),
            ("WARN", LogLevel::Warn),
            ("ERROR", LogLevel::Error),
        ] {
            let line = format!("RB-610|{level}|動作確認");
            let entry = parse_log(&line).unwrap();

            assert_eq!(entry.level(), expected);
            assert_eq!(entry.robot_id(), "RB-610");
            assert_eq!(entry.message(), "動作確認");
        }
    }

    #[test]
    fn 日本語と絵文字をそのまま解析する() {
        let line = String::from("配送ロボット-七|INFO|図書館🚚へ到着");
        let entry = parse_log(&line).unwrap();

        assert_eq!(entry.robot_id(), "配送ロボット-七");
        assert_eq!(entry.message(), "図書館🚚へ到着");
    }

    #[test]
    fn message内の区切り文字を保持する() {
        let entry = parse_log("RB-620|WARN|荷物|温度|確認").unwrap();

        assert_eq!(entry.message(), "荷物|温度|確認");
    }

    #[test]
    fn 空白を除いたフィールドは入力の内部を指す() {
        let line = String::from("  RB-630  |  ERROR  |  充電が必要  ");
        let entry = parse_log(&line).unwrap();

        assert_eq!(entry.robot_id(), "RB-630");
        assert_eq!(entry.message(), "充電が必要");
        assert!(points_inside(&line, entry.robot_id()));
        assert!(points_inside(&line, entry.message()));
    }

    #[test]
    fn フィールドが足りない行を拒否する() {
        assert_eq!(parse_log("壊れたログ"), Err(LogError::MalformedLine));
        assert_eq!(parse_log("RB-640|INFO"), Err(LogError::MalformedLine));
    }

    #[test]
    fn 空のrobot_idとmessageを区別する() {
        assert_eq!(parse_log("  |INFO|到着"), Err(LogError::EmptyRobotId));
        assert_eq!(parse_log("RB-650|WARN|  "), Err(LogError::EmptyMessage));
    }

    #[test]
    fn 未知のレベルも入力の内部を借用する() {
        let line = String::from("RB-660|  DEBUG  |診断中");

        match parse_log(&line) {
            Err(LogError::UnknownLevel(level)) => {
                let level = as_str(&level);
                assert_eq!(level, "DEBUG");
                assert!(points_inside(&line, level));
            }
            result => panic!("UnknownLevel を期待しました: {result:?}"),
        }

        match parse_log("RB-661|info|出発") {
            Err(LogError::UnknownLevel(level)) => assert_eq!(as_str(&level), "info"),
            result => panic!("小文字のレベルを拒否する必要があります: {result:?}"),
        }
    }

    #[test]
    fn entryを消費してもmessageの借用を返せる() {
        let line = String::from("RB-670|INFO|研究棟へ移動");
        let message = parse_log(&line).unwrap().into_message();
        let message = as_str(&message);

        assert_eq!(message, "研究棟へ移動");
        assert!(points_inside(&line, message));
    }
}
