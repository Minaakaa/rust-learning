//! 問題 05 の解答例。

use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum LogLevel {
    Info,
    Warn,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq)]
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
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum LogError {
    MalformedLine,
    EmptyRobotId,
    UnknownLevel(String),
    EmptyMessage,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct LogBook {
    entries: Vec<LogEntry>,
    counts_by_level: HashMap<LogLevel, usize>,
    robot_ids: HashSet<String>,
}

fn scalar_count(text: &str) -> usize {
    text.chars().count()
}

fn utf8_prefix(text: &str, max_chars: usize) -> &str {
    match text.char_indices().nth(max_chars) {
        Some((byte_index, _)) => &text[..byte_index],
        None => text,
    }
}

fn preview(text: &str, max_chars: usize) -> String {
    let prefix = utf8_prefix(text, max_chars);
    if prefix.len() < text.len() {
        format!("{prefix}…")
    } else {
        prefix.to_string()
    }
}

fn parse_log(line: &str) -> Result<LogEntry, LogError> {
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
        unknown => return Err(LogError::UnknownLevel(unknown.to_string())),
    };

    if message.is_empty() {
        return Err(LogError::EmptyMessage);
    }

    Ok(LogEntry {
        robot_id: robot_id.to_string(),
        level,
        message: message.to_string(),
    })
}

impl LogBook {
    fn new() -> Self {
        Self::default()
    }

    fn record(&mut self, line: &str) -> Result<(), LogError> {
        let entry = parse_log(line)?;

        *self.counts_by_level.entry(entry.level).or_insert(0) += 1;
        self.robot_ids.insert(entry.robot_id.clone());
        self.entries.push(entry);

        Ok(())
    }

    fn len(&self) -> usize {
        self.entries.len()
    }

    fn count(&self, level: LogLevel) -> usize {
        self.counts_by_level.get(&level).copied().unwrap_or(0)
    }

    fn unique_robot_count(&self) -> usize {
        self.robot_ids.len()
    }

    fn has_robot(&self, robot_id: &str) -> bool {
        self.robot_ids.contains(robot_id.trim())
    }

    fn latest(&self) -> Option<&LogEntry> {
        self.entries.last()
    }

    fn latest_preview(&self, max_chars: usize) -> Option<String> {
        self.latest()
            .map(|entry| preview(entry.message(), max_chars))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn バイト数とunicodeスカラー値の個数を区別する() {
        let text = "図書館🚚";

        assert_eq!(text.len(), 13);
        assert_eq!(scalar_count(text), 4);
        assert_ne!(text.len(), scalar_count(text));

        // 結合文字付きの e は見た目が 1 文字でも、ここでは 2 スカラー値として扱う。
        let combining = "e\u{301}";
        assert_eq!(combining.len(), 3);
        assert_eq!(scalar_count(combining), 2);
    }

    #[test]
    fn unicodeスカラー値の境界で接頭辞を借用する() {
        let text = String::from("図書館🚚");

        assert_eq!(utf8_prefix(&text, 0), "");
        assert_eq!(utf8_prefix(&text, 1), "図");
        assert_eq!(utf8_prefix(&text, 3), "図書館");
        assert_eq!(utf8_prefix(&text, 4), "図書館🚚");
        assert_eq!(utf8_prefix(&text, 99), "図書館🚚");
        assert_eq!(utf8_prefix("", 3), "");

        // 戻り値は新しい String ではなく、元の String から借用した &str。
        let prefix: &str = utf8_prefix(&text, 2);
        assert_eq!(prefix, "図書");
        assert_eq!(text, "図書館🚚");
    }

    #[test]
    fn 実際に短縮したときだけ省略記号を付ける() {
        assert_eq!(preview("図書館🚚", 2), "図書…");
        assert_eq!(preview("図書館🚚", 4), "図書館🚚");
        assert_eq!(preview("図書館🚚", 10), "図書館🚚");
        assert_eq!(preview("図書館🚚", 0), "…");
        assert_eq!(preview("", 0), "");
        assert_eq!(preview("e\u{301}", 1), "e…");
    }

    #[test]
    fn 日本語と区切り文字を含むログを解析する() {
        let entry = parse_log("  RB-50 | WARN | 図書館の荷物|確認  ").unwrap();

        assert_eq!(entry.robot_id(), "RB-50");
        assert_eq!(entry.level(), LogLevel::Warn);
        assert_eq!(entry.message(), "図書館の荷物|確認");

        let emoji = parse_log("RB-51|INFO|🚚 到着").unwrap();
        assert_eq!(emoji.level(), LogLevel::Info);
        assert_eq!(emoji.message(), "🚚 到着");
    }

    #[test]
    fn 不正なログを具体的なエラーにする() {
        assert_eq!(parse_log("壊れたログ"), Err(LogError::MalformedLine));
        assert_eq!(parse_log("RB-50|INFO"), Err(LogError::MalformedLine));
        assert_eq!(parse_log(" |INFO|到着"), Err(LogError::EmptyRobotId));
        assert_eq!(
            parse_log("RB-50|DEBUG|到着"),
            Err(LogError::UnknownLevel("DEBUG".to_string()))
        );
        assert_eq!(
            parse_log("RB-50|info|到着"),
            Err(LogError::UnknownLevel("info".to_string()))
        );
        assert_eq!(parse_log("RB-50|ERROR|   "), Err(LogError::EmptyMessage));
    }

    #[test]
    fn 記録順とレベル別件数と一意なロボットを管理する() {
        let mut book = LogBook::new();

        book.record("RB-60|INFO|出発").unwrap();
        book.record("RB-60|WARN|混雑").unwrap();
        book.record("RB-61|INFO|到着").unwrap();

        assert_eq!(book.len(), 3);
        assert_eq!(book.count(LogLevel::Info), 2);
        assert_eq!(book.count(LogLevel::Warn), 1);
        assert_eq!(book.count(LogLevel::Error), 0);
        assert_eq!(book.unique_robot_count(), 2);
        assert!(book.has_robot("RB-60"));
        assert!(book.has_robot("  RB-61  "));
        assert!(!book.has_robot("RB-99"));
        assert_eq!(book.latest().unwrap().robot_id(), "RB-61");
        assert_eq!(book.latest().unwrap().message(), "到着");
    }

    #[test]
    fn 不正なログではどのコレクションも変更しない() {
        let mut book = LogBook::new();
        book.record("RB-70|ERROR|充電が必要").unwrap();
        let before = book.clone();

        assert_eq!(
            book.record("RB-71|UNKNOWN|無視する"),
            Err(LogError::UnknownLevel("UNKNOWN".to_string()))
        );
        assert_eq!(book, before);
    }

    #[test]
    fn 最新メッセージをutf8境界で安全に短縮する() {
        let mut book = LogBook::new();
        assert_eq!(book.latest(), None);
        assert_eq!(book.latest_preview(4), None);

        book.record("RB-80|ERROR|図書館🚚へ戻る").unwrap();

        assert_eq!(book.latest_preview(4), Some("図書館🚚…".to_string()));
        assert_eq!(book.latest_preview(20), Some("図書館🚚へ戻る".to_string()));
    }
}
