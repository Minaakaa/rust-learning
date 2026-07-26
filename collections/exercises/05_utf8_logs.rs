#![cfg_attr(not(test), allow(dead_code))]

//! # 問題 05: UTF-8 のログブックを完成させる
//!
//! 配送ロボットのログを安全に短縮して解析し、これまで学んだコレクションへ記録します。
//! `String` と `str` は UTF-8 です。`len()` が返すのはバイト数であり、`chars()` が扱う
//! のは Unicode スカラー値です。Unicode スカラー値は、画面上で 1 文字に見える
//! 書記素クラスタ（grapheme cluster）とは限りません。たとえば `e\u{301}` は、見た目は
//! 1 文字でも 2 個の Unicode スカラー値です。この問題では書記素クラスタではなく、
//! Unicode スカラー値を数えます。
//!
//! ログの形式は `robot_id|LEVEL|message` です。
//!
//! - `LEVEL` は `INFO`、`WARN`、`ERROR` のいずれか。
//! - 3 つのフィールドの前後にある空白は取り除く。
//! - メッセージには `|` を含められるため、分割は最大 3 フィールドにする。
//! - ロボット ID とメッセージは空にできない。
//!
//! `LogBook` は記録順を保つ `Vec`、レベル別件数を持つ `HashMap`、一意なロボット ID を
//! 持つ `HashSet` をまとめます。不正なログを受け取った場合は、どのコレクションも変更
//! しないでください。`HashMap` と `HashSet` の反復順序には依存しません。
//!
//! ヒント:
//! - `char_indices()` は `(バイト位置, Unicode スカラー値)` を返す。
//! - `splitn(3, '|')` なら、3 番目のフィールド内にある `|` が残る。
//! - 解析に成功してから 3 つのコレクションを更新すると、失敗時の変更を防げる。

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

/// UTF-8 文字列に含まれる Unicode スカラー値の個数を返す。
fn scalar_count(text: &str) -> usize {
    todo!("{text:?} の Unicode スカラー値を数えてください")
}

/// 先頭から最大 `max_chars` 個の Unicode スカラー値を、借用した `str` として返す。
///
/// バイト境界ではなく `char_indices()` が示す境界を使い、途中で切らないこと。
fn utf8_prefix(text: &str, max_chars: usize) -> &str {
    todo!("{text:?} の先頭 {max_chars} 個を安全に借用してください")
}

/// `max_chars` 個を超えるときだけ、借用した接頭辞の後ろに `…` を付ける。
fn preview(text: &str, max_chars: usize) -> String {
    todo!("{text:?} のプレビューを最大 {max_chars} 個で作ってください")
}

/// `robot_id|LEVEL|message` 形式の 1 行を解析する。
fn parse_log(line: &str) -> Result<LogEntry, LogError> {
    todo!("ログ {line:?} を最大 3 フィールドに分割して検証してください")
}

impl LogBook {
    fn new() -> Self {
        Self::default()
    }

    /// 解析に成功したログだけを、3 つのコレクションへまとめて記録する。
    fn record(&mut self, line: &str) -> Result<(), LogError> {
        todo!("ログ {line:?} を解析してからアトミックに記録してください")
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

    /// 前後の空白を除いた ID が記録済みか、借用した `str` で検索する。
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

fn main() {
    let destination = "図書館🚚";
    println!(
        "{destination}: {} バイト、{} スカラー値",
        destination.len(),
        scalar_count(destination)
    );
    println!("プレビュー: {}", preview(destination, 3));

    let mut book = LogBook::new();
    book.record("RB-50|INFO|図書館へ出発")
        .expect("固定値は有効なログ");
    book.record("RB-50|WARN|荷物|確認")
        .expect("メッセージ内の区切り文字は有効");

    println!("記録件数: {}", book.len());
    println!("最新: {:?}", book.latest());
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
