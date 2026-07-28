#![cfg_attr(not(test), allow(dead_code))]

//! # 問題 05: `Cow` で必要なときだけ所有する
//!
//! 管制室へ届く通知からロボット ID と配送先を取り出します
//! 多くの配送先は入力の一部をそのまま借用できますが、空白表記が乱れた場合だけ新しい文字列が必要です
//! `Cow<'a, str>` を使い、借用と所有のどちらも表せる通知へ変更してください
//!
//! 仕様:
//! - 入力形式は `robot_id|destination`
//! - 最初の `|` より後はすべて配送先として扱う
//! - ID と配送先の前後の空白は `trim` で除く
//! - 前後を除くだけでよい配送先は入力から借用する
//! - 配送先内部の連続空白、タブ、改行、全角空白は単一の半角空白へ直して所有する
//! - 区切りがない入力、空の ID、空の配送先はそれぞれ対応するエラーにする
//! - `into_destination` は入力から独立した `String` を返す
//! - `Box::leak` や文字列の無条件な複製は使わない
//!
//! ヒント:
//! - starter の所有フィールドを `DispatchNotice<'a>` の借用と `Cow<'a, str>` へ変更する
//! - `Cow::Borrowed` と `Cow::Owned` は共通して `&str` として読める
//! - `split_whitespace` は Unicode の空白も扱う
//! - 変換が必要か先に判定すると、借用できる経路では割り当てを避けられる

#[derive(Debug, PartialEq, Eq)]
enum NoticeError {
    MissingSeparator,
    EmptyRobotId,
    EmptyDestination,
}

#[derive(Debug, PartialEq, Eq)]
struct DispatchNotice {
    robot_id: String,
    destination: String,
    normalized: bool,
}

impl DispatchNotice {
    fn robot_id(&self) -> &str {
        &self.robot_id
    }

    fn destination(&self) -> &str {
        &self.destination
    }

    fn destination_was_normalized(&self) -> bool {
        self.normalized
    }

    fn into_destination(self) -> String {
        self.destination
    }
}

fn parse_notice(line: &str) -> Result<DispatchNotice, NoticeError> {
    todo!("通知 {line:?} を借用または所有して解析してください")
}

fn main() {
    let notice = parse_notice("RB-605 | 工学部   2号館").expect("正しい通知です");
    println!(
        "{} の配送先: {}（正規化: {}）",
        notice.robot_id(),
        notice.destination(),
        notice.destination_was_normalized()
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    fn points_inside(fragment: &str, source: &str) -> bool {
        let source_start = source.as_ptr() as usize;
        let source_end = source_start + source.len();
        let fragment_start = fragment.as_ptr() as usize;
        let fragment_end = fragment_start + fragment.len();

        source_start <= fragment_start && fragment_end <= source_end
    }

    #[test]
    fn 前後の空白だけなら入力を借用する() {
        let line = String::from("  RB-605  |  工学部2号館  ");

        let notice = parse_notice(&line).unwrap();

        assert_eq!(notice.robot_id(), "RB-605");
        assert_eq!(notice.destination(), "工学部2号館");
        assert!(!notice.destination_was_normalized());
        assert!(points_inside(notice.robot_id(), &line));
        assert!(points_inside(notice.destination(), &line));
    }

    #[test]
    fn 連続する半角空白があれば所有して正規化する() {
        let line = String::from("RB-606|工学部   2号館");

        let notice = parse_notice(&line).unwrap();

        assert_eq!(notice.destination(), "工学部 2号館");
        assert!(notice.destination_was_normalized());
        assert!(points_inside(notice.robot_id(), &line));
        assert!(!points_inside(notice.destination(), &line));
    }

    #[test]
    fn unicode空白とタブを半角空白へそろえる() {
        let line = String::from("RB-607|総合\u{3000}\t研究棟");

        let notice = parse_notice(&line).unwrap();

        assert_eq!(notice.destination(), "総合 研究棟");
        assert!(notice.destination_was_normalized());
        assert!(!points_inside(notice.destination(), &line));
    }

    #[test]
    fn 日本語と絵文字を変更せず借用する() {
        let line = String::from("配送🤖-01|工学部Ａ棟🔧");

        let notice = parse_notice(&line).unwrap();

        assert_eq!(notice.robot_id(), "配送🤖-01");
        assert_eq!(notice.destination(), "工学部Ａ棟🔧");
        assert!(!notice.destination_was_normalized());
        assert!(points_inside(notice.destination(), &line));
    }

    #[test]
    fn 最初の区切りより後をすべて配送先にする() {
        let line = String::from("RB-608|工学部|地下倉庫");

        let notice = parse_notice(&line).unwrap();

        assert_eq!(notice.destination(), "工学部|地下倉庫");
        assert!(!notice.destination_was_normalized());
        assert!(points_inside(notice.destination(), &line));
    }

    #[test]
    fn 区切りがない通知を拒否する() {
        assert_eq!(
            parse_notice("RB-609 工学部2号館"),
            Err(NoticeError::MissingSeparator)
        );
    }

    #[test]
    fn 空のrobot_idを拒否する() {
        assert_eq!(
            parse_notice("  | 工学部2号館"),
            Err(NoticeError::EmptyRobotId)
        );
    }

    #[test]
    fn 空白だけの配送先を拒否する() {
        assert_eq!(
            parse_notice("RB-610| \t\u{3000}"),
            Err(NoticeError::EmptyDestination)
        );
    }

    #[test]
    fn into_destinationは入力より長く使えるstringを返す() {
        let destination = {
            let line = String::from("RB-611|情報基盤センター");
            parse_notice(&line).unwrap().into_destination()
        };

        assert_eq!(destination, "情報基盤センター");
    }
}
