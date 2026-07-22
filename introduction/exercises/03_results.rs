#![cfg_attr(not(test), allow(dead_code))]

//! # 問題 03: Result で経路データを読み込む
//!
//! 管理画面から届く `"建物名:距離"` 形式の文字列を、安全に経路へ変換します。
//! 入力の誤りは `panic!` させず、理由を持つ `Err(String)` として呼び出し元へ返してください。
//!
//! 使用する要素:
//! - `split_once` が返す `Option` と `ok_or_else`
//! - `parse` が返す `Result` と `map_err`
//! - `?` による早期リターン
//! - `checked_add` で得る `Option` のエラー変換
//!
//! `unwrap` と `expect` は使わないでください。
//! 空白を `trim` してから、建物名の空文字、整数でない距離、距離 0、合計値の
//! オーバーフローを拒否します。返すエラーメッセージは各テストが仕様です。

#[derive(Debug, Clone, PartialEq, Eq)]
struct RouteStop {
    building: String,
    distance_m: u32,
}

fn parse_stop(input: &str) -> Result<RouteStop, String> {
    todo!("入力を分割・検証してください: {input}")
}

fn total_distance(inputs: &[&str]) -> Result<u32, String> {
    todo!(
        "各入力を parse_stop で読み、オーバーフローを確認してください: {} 件",
        inputs.len()
    )
}

fn main() {
    let route = ["図書館:120", "研究棟: 350"];
    println!("経路の総距離: {:?}", total_distance(&route));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 前後の空白を除いて停車地点を作る() {
        assert_eq!(
            parse_stop("  図書館 : 120 "),
            Ok(RouteStop {
                building: "図書館".to_string(),
                distance_m: 120,
            })
        );
    }

    #[test]
    fn 区切り文字がなければ入力も含めて報告する() {
        assert_eq!(
            parse_stop("図書館120"),
            Err("区切り文字 ':' がありません: 図書館120".to_string())
        );
    }

    #[test]
    fn 空の建物名を拒否する() {
        assert_eq!(parse_stop("  :120"), Err("建物名が空です".to_string()));
    }

    #[test]
    fn 整数でない距離を元の文字列とともに報告する() {
        assert_eq!(
            parse_stop("研究棟:遠い"),
            Err("距離が整数ではありません: 遠い".to_string())
        );
    }

    #[test]
    fn 距離ゼロを拒否する() {
        assert_eq!(
            parse_stop("研究棟:0"),
            Err("距離は 1 以上でなければなりません".to_string())
        );
    }

    #[test]
    fn すべて成功したときだけ距離を合計する() {
        assert_eq!(
            total_distance(&["図書館:120", "研究棟:350", "食堂:80"]),
            Ok(550)
        );
    }

    #[test]
    fn 途中の解析エラーをそのまま伝播する() {
        assert_eq!(
            total_distance(&["図書館:120", "研究棟:不明", "食堂:80"]),
            Err("距離が整数ではありません: 不明".to_string())
        );
    }

    #[test]
    fn 合計のオーバーフローをエラーにする() {
        assert_eq!(
            total_distance(&["遠方施設:4294967295", "図書館:1"]),
            Err("総距離が u32 の上限を超えました".to_string())
        );
    }
}
