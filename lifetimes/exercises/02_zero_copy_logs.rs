#![cfg_attr(not(test), allow(dead_code))]

//! # 問題 02: ログをゼロコピーで解析する
//!
//! Chapter 4 では、解析したフィールドを新しい `String` として所有する `LogEntry` を
//! 作りました
//! この問題では入力行の一部分を `&str` として借用し、文字列を複製しない解析 API へ
//! 作り替えます
//!
//! 現在の `LogEntry` と `LogError` は文字列を所有しています
//! 次の変更を行ってから `parse_log` を完成させてください
//!
//! - `LogEntry<'a>` を定義し、`robot_id` と `message` を `&'a str` にする
//! - `LogError<'a>::UnknownLevel` が `&'a str` を保持するようにする
//! - `parse_log` の入力と戻り値に同じライフタイムの関係を表す
//! - accessor と `into_message` の戻り型を借用データに合わせる
//!
//! ログ形式は `robot_id|LEVEL|message` です
//!
//! - `LEVEL` は `INFO`、`WARN`、`ERROR` のいずれか
//! - 各フィールドの外側にある空白は `trim` で除く
//! - message 内の `|` は内容として保持する
//! - robot ID と message は空にできない
//! - 入力の文字列から `String` を作らない

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LogLevel {
    Info,
    Warn,
    Error,
}

#[derive(Debug, PartialEq, Eq)]
struct LogEntry {
    robot_id: String,
    level: LogLevel,
    message: String,
}

impl LogEntry {
    fn robot_id(&self) -> &str {
        &self.robot_id
    }

    const fn level(&self) -> LogLevel {
        self.level
    }

    fn message(&self) -> &str {
        &self.message
    }

    fn into_message(self) -> String {
        self.message
    }
}

#[derive(Debug, PartialEq, Eq)]
enum LogError {
    MalformedLine,
    EmptyRobotId,
    #[cfg_attr(test, allow(dead_code))]
    UnknownLevel(String),
    EmptyMessage,
}

/// `robot_id|LEVEL|message` 形式の1行を解析する
fn parse_log(line: &str) -> Result<LogEntry, LogError> {
    todo!("ログ {line:?} を入力から借用して解析してください")
}

fn main() {
    let line = String::from("RB-602|INFO|図書館🚚へ出発");

    match parse_log(&line) {
        Ok(entry) => println!(
            "ロボット: {}、レベル: {:?}、内容: {}",
            entry.robot_id(),
            entry.level(),
            entry.message()
        ),
        Err(error) => println!("解析エラー: {error:?}"),
    }
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
