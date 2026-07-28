//! 問題 05 の解答例

use std::borrow::Cow;

#[derive(Debug, PartialEq, Eq)]
enum NoticeError {
    MissingSeparator,
    EmptyRobotId,
    EmptyDestination,
}

#[derive(Debug, PartialEq, Eq)]
struct DispatchNotice<'a> {
    robot_id: &'a str,
    destination: Cow<'a, str>,
}

impl<'a> DispatchNotice<'a> {
    fn robot_id(&self) -> &str {
        self.robot_id
    }

    fn destination(&self) -> &str {
        &self.destination
    }

    fn destination_was_normalized(&self) -> bool {
        matches!(self.destination, Cow::Owned(_))
    }

    fn into_destination(self) -> String {
        self.destination.into_owned()
    }
}

fn parse_notice(line: &str) -> Result<DispatchNotice<'_>, NoticeError> {
    let (robot_id, destination) = line.split_once('|').ok_or(NoticeError::MissingSeparator)?;
    let robot_id = robot_id.trim();
    let destination = destination.trim();

    if robot_id.is_empty() {
        return Err(NoticeError::EmptyRobotId);
    }
    if destination.is_empty() {
        return Err(NoticeError::EmptyDestination);
    }

    let destination = if needs_normalization(destination) {
        Cow::Owned(destination.split_whitespace().collect::<Vec<_>>().join(" "))
    } else {
        Cow::Borrowed(destination)
    };

    Ok(DispatchNotice {
        robot_id,
        destination,
    })
}

fn needs_normalization(destination: &str) -> bool {
    let mut previous_was_space = false;

    for character in destination.chars() {
        if character.is_whitespace() {
            if character != ' ' || previous_was_space {
                return true;
            }
            previous_was_space = true;
        } else {
            previous_was_space = false;
        }
    }

    false
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
