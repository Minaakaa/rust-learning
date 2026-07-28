#![cfg_attr(not(test), allow(dead_code))]

//! # 問題 01: 参照のライフタイム関係を表す
//!
//! 配送先の候補と経路ラベルを、新しい `String` に複製せず借用して返します
//! ライフタイム注釈は参照を長生きさせるものではなく、入力と出力の関係を表す契約です
//!
//! TODO:
//! - `choose_destination` の戻り値を `&'a str` に変更し、両方の候補と同じ `'a` を付ける
//! - `district_before` の戻り値を `Option<&'route str>` に変更し、`route` だけへ結び付ける
//! - `standby_message` はプログラム全体で有効な文字列リテラルを返す
//!
//! 仕様:
//! - `prefer_first` が `true` なら `first`、`false` なら `second` をそのまま借用する
//! - `district_before` は最初の区切り文字より前を `trim` して借用する
//! - 区切り文字が空、または経路内に見つからない場合は `None` を返す
//! - `String` の生成、`clone`、`Box::leak` は使わない

/// 選ばれた配送先を借用して返す
fn choose_destination(first: &str, second: &str, prefer_first: bool) -> String {
    todo!("候補 {first:?} と {second:?} から prefer_first={prefer_first} に従って選んでください")
}

/// 経路内で最初の区切り文字より前にある地区名を借用して返す
fn district_before(route: &str, separator: &str) -> Option<String> {
    todo!("経路 {route:?} を区切り文字 {separator:?} で分けてください")
}

/// 管制室の待機メッセージをプログラム全体で有効な参照として返す
fn standby_message() -> &'static str {
    todo!("固定の待機メッセージを返してください")
}

fn main() {
    let primary = String::from("工学部2号館");
    let secondary = String::from("総合図書館");
    let selected = choose_destination(&primary, &secondary, true);

    println!("選択した配送先: {selected}");
    println!(
        "経路の地区: {:?}",
        district_before("本郷地区::工学部2号館", "::")
    );
    println!("状態: {}", standby_message());
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_points_into(candidate: impl AsRef<str>, source: &str) {
        let candidate = candidate.as_ref();
        let source_start = source.as_ptr() as usize;
        let source_end = source_start + source.len();
        let candidate_start = candidate.as_ptr() as usize;
        let candidate_end = candidate_start + candidate.len();

        assert!(candidate_start >= source_start);
        assert!(candidate_end <= source_end);
    }

    #[test]
    fn firstを選び同じ領域を借用する() {
        let first = String::from("工学部2号館");
        let second = String::from("総合図書館");

        let selected = choose_destination(&first, &second, true);

        assert_eq!(selected, "工学部2号館");
        assert_eq!(selected.as_ptr(), first.as_ptr());
        assert_points_into(selected, &first);
    }

    #[test]
    fn secondのutf8文字列をそのまま借用する() {
        let first = String::from("実験棟");
        let second = String::from("ロボット研究室🤖");

        let selected = choose_destination(&first, &second, false);

        assert_eq!(selected, "ロボット研究室🤖");
        assert_eq!(selected.as_ptr(), second.as_ptr());
        assert_points_into(selected, &second);
    }

    #[test]
    fn 地区名をtrimしてroute内から借用する() {
        let route = String::from("  本郷地区  :: 工学部2号館");

        let district = district_before(&route, "::").expect("地区名がある");

        assert_eq!(district, "本郷地区");
        let expected_offset = route.find("本郷地区").expect("地区名が含まれる");
        assert_eq!(district.as_ptr(), route[expected_offset..].as_ptr());
        assert_points_into(district, &route);
    }

    #[test]
    fn 短命なseparatorに結果を結び付けない() {
        let route = String::from("柏地区→実験棟🔬");

        let district = {
            let separator = String::from("→");
            district_before(&route, &separator).expect("地区名がある")
        };

        assert_eq!(district, "柏地区");
        assert_points_into(district, &route);
    }

    #[test]
    fn 空のseparatorを拒否する() {
        assert_eq!(district_before("本郷地区::図書館", ""), None);
    }

    #[test]
    fn separatorがなければnoneを返す() {
        assert_eq!(district_before("駒場地区・食堂", "::"), None);
    }

    #[test]
    fn 区切り前が空でも借用した空文字列を返す() {
        let route = String::from("::工学部2号館");

        let district = district_before(&route, "::").expect("区切り文字は存在する");

        assert!(district.is_empty());
        assert_points_into(district, &route);
    }

    #[test]
    fn 待機メッセージはstatic参照である() {
        fn require_static(_: &'static str) {}

        let message: &'static str = standby_message();

        require_static(message);
        assert_eq!(message, "管制室で待機");
    }
}
