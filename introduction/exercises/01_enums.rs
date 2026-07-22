#![cfg_attr(not(test), allow(dead_code))]

//! # 問題 01: enum で荷物を表現する
//!
//! 配送ロボットは、書類、精密部品、空荷という性質の違う状態を扱います。
//! `Cargo` の各バリアントが持つデータを読み取り、次の 4 つの TODO を完成させてください。
//!
//! 学習ポイント:
//! - unit-like、tuple-like、struct-like なバリアント
//! - `String` を enum の中に所有させる方法
//! - `&self` で enum を借用し、中のデータを読む方法
//!
//! 仕様:
//! - サンプルは「Rust入門ノート」48 ページ、壊れ物の「距離センサー」3 個、空荷の順にする。
//! - `units` は書類なら 1、部品なら個数、空荷なら 0 を返す。
//! - `label` の形式はテストに示す「書類…」「部品…」「空荷」とする。
//! - `needs_care` は部品の壊れ物フラグだけを反映する。

#[derive(Debug, Clone, PartialEq, Eq)]
enum Cargo {
    Documents { title: String, pages: u16 },
    Components(String, u16, bool),
    Empty,
}

impl Cargo {
    /// 荷物の個数を返す。書類は冊数を 1、空荷は 0 と数える。
    fn units(&self) -> u16 {
        todo!("各バリアントが表す個数を返してください")
    }

    /// 操作画面に表示する短いラベルを作る。
    fn label(&self) -> String {
        todo!("各バリアントのデータからラベルを作ってください")
    }

    /// 壊れ物として注意が必要かを返す。
    fn needs_care(&self) -> bool {
        todo!("Components の 3 番目の値を確認してください")
    }
}

/// テストで指定された 3 種類の荷物を、この順番で作る。
fn sample_cargo() -> Vec<Cargo> {
    todo!("Cargo の 3 種類のバリアントを構築してください")
}

fn main() {
    for cargo in sample_cargo() {
        println!(
            "{}: {} 個、取扱注意={}",
            cargo.label(),
            cargo.units(),
            cargo.needs_care()
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 指定された荷物を構築できる() {
        assert_eq!(
            sample_cargo(),
            vec![
                Cargo::Documents {
                    title: "Rust入門ノート".to_string(),
                    pages: 48,
                },
                Cargo::Components("距離センサー".to_string(), 3, true),
                Cargo::Empty,
            ]
        );
    }

    #[test]
    fn バリアントごとの個数を返す() {
        let cargo = sample_cargo();
        assert_eq!(cargo[0].units(), 1);
        assert_eq!(cargo[1].units(), 3);
        assert_eq!(cargo[2].units(), 0);
    }

    #[test]
    fn 中のデータを使ってラベルを作る() {
        let cargo = sample_cargo();
        assert_eq!(cargo[0].label(), "書類「Rust入門ノート」(48ページ)");
        assert_eq!(cargo[1].label(), "部品「距離センサー」x3");
        assert_eq!(cargo[2].label(), "空荷");
    }

    #[test]
    fn 精密部品のフラグだけを取扱注意に反映する() {
        assert!(Cargo::Components("カメラ".to_string(), 1, true).needs_care());
        assert!(!Cargo::Components("ねじ".to_string(), 20, false).needs_care());
        assert!(
            !Cargo::Documents {
                title: "提出レポート".to_string(),
                pages: 12,
            }
            .needs_care()
        );
        assert!(!Cargo::Empty.needs_care());
    }
}
