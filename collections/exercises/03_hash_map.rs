#![cfg_attr(not(test), allow(dead_code))]

//! # 問題 03: `HashMap` で部品在庫を集計する
//!
//! 配送ロボットの整備倉庫では、同じ部品が何度も入荷し、修理のたびに在庫から
//! 取り出されます。部品名をキー、個数を値とする `HashMap` で在庫を管理してください。
//!
//! 仕様:
//! - 部品名は `trim` してから使い、空ならエラーにする。
//! - 部品名の検証を個数の検証より先に行い、個数 0 もエラーにする。
//! - 入荷と出庫では `HashMap::entry` と `Entry` を使い、検索を重ねない。
//! - 入荷数の加算には `checked_add` を使う。オーバーフロー時は在庫を変更しない。
//! - 在庫不足や未登録の部品を出庫するときも、在庫を変更しない。
//! - 在庫がちょうど 0 になった部品はマップから削除する。

use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
enum InventoryError {
    EmptyItemName,
    ZeroQuantity,
    QuantityOverflow {
        item: String,
        current: u32,
        added: u32,
    },
    UnknownItem(String),
    InsufficientStock {
        item: String,
        available: u32,
        requested: u32,
    },
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct Inventory {
    stock: HashMap<String, u32>,
}

impl Inventory {
    fn new() -> Self {
        Self::default()
    }

    /// 部品を入荷し、入荷後の在庫数を返す。
    fn receive(&mut self, item: &str, quantity: u32) -> Result<u32, InventoryError> {
        todo!("部品「{item}」を {quantity} 個、entry と checked_add で入荷してください")
    }

    /// 部品を出庫し、出庫後の在庫数を返す。
    fn ship(&mut self, item: &str, quantity: u32) -> Result<u32, InventoryError> {
        todo!("部品「{item}」を {quantity} 個、Entry を使って安全に出庫してください")
    }

    /// 未登録の部品を含め、現在の在庫数を返す。
    fn quantity(&self, item: &str) -> u32 {
        todo!("trim した部品名「{item}」の在庫数を検索してください")
    }

    fn distinct_items(&self) -> usize {
        todo!("在庫がある部品の種類数を返してください")
    }
}

fn main() {
    let mut inventory = Inventory::new();
    inventory
        .receive("  駆動モーター  ", 4)
        .expect("有効な入荷");
    inventory.receive("駆動モーター", 3).expect("有効な入荷");

    println!(
        "駆動モーター: {} 個（全 {} 種類）",
        inventory.quantity("駆動モーター"),
        inventory.distinct_items()
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 同じ部品への入荷をentryでまとめる() {
        let mut inventory = Inventory::new();

        assert_eq!(inventory.receive("  駆動モーター  ", 4), Ok(4));
        assert_eq!(inventory.receive("駆動モーター", 3), Ok(7));
        assert_eq!(inventory.quantity(" 駆動モーター "), 7);
        assert_eq!(inventory.distinct_items(), 1);
    }

    #[test]
    fn 異なる部品は別々に保持する() {
        let mut inventory = Inventory::new();

        assert_eq!(inventory.receive("距離センサー", 8), Ok(8));
        assert_eq!(inventory.receive("交換タイヤ", 12), Ok(12));
        assert_eq!(inventory.quantity("距離センサー"), 8);
        assert_eq!(inventory.quantity("交換タイヤ"), 12);
        assert_eq!(inventory.quantity("未登録"), 0);
        assert_eq!(inventory.distinct_items(), 2);
    }

    #[test]
    fn 部品名を個数より先に検証して失敗時は変更しない() {
        let mut inventory = Inventory::new();
        inventory.receive("制御基板", 2).unwrap();
        let before = inventory.clone();

        assert_eq!(
            inventory.receive("   ", 0),
            Err(InventoryError::EmptyItemName)
        );
        assert_eq!(
            inventory.receive("制御基板", 0),
            Err(InventoryError::ZeroQuantity)
        );
        assert_eq!(inventory.ship("\t", 0), Err(InventoryError::EmptyItemName));
        assert_eq!(
            inventory.ship("制御基板", 0),
            Err(InventoryError::ZeroQuantity)
        );
        assert_eq!(inventory, before);
    }

    #[test]
    fn 未登録と在庫不足では状態を変更しない() {
        let mut inventory = Inventory::new();
        inventory.receive("交換タイヤ", 5).unwrap();

        assert_eq!(
            inventory.ship("予備バッテリー", 1),
            Err(InventoryError::UnknownItem("予備バッテリー".to_string()))
        );
        assert_eq!(
            inventory.ship(" 交換タイヤ ", 6),
            Err(InventoryError::InsufficientStock {
                item: "交換タイヤ".to_string(),
                available: 5,
                requested: 6,
            })
        );
        assert_eq!(inventory.quantity("交換タイヤ"), 5);
        assert_eq!(inventory.distinct_items(), 1);
    }

    #[test]
    fn 出庫してゼロになった部品を削除する() {
        let mut inventory = Inventory::new();
        inventory.receive("固定ボルト", 10).unwrap();

        assert_eq!(inventory.ship("固定ボルト", 4), Ok(6));
        assert_eq!(inventory.quantity("固定ボルト"), 6);
        assert_eq!(inventory.ship("固定ボルト", 6), Ok(0));
        assert_eq!(inventory.quantity("固定ボルト"), 0);
        assert_eq!(inventory.distinct_items(), 0);
        assert_eq!(
            inventory.ship("固定ボルト", 1),
            Err(InventoryError::UnknownItem("固定ボルト".to_string()))
        );
    }

    #[test]
    fn 入荷数がオーバーフローしたら元の在庫を保つ() {
        let mut inventory = Inventory::new();
        inventory.receive("超小型ヒューズ", u32::MAX).unwrap();

        assert_eq!(
            inventory.receive(" 超小型ヒューズ ", 1),
            Err(InventoryError::QuantityOverflow {
                item: "超小型ヒューズ".to_string(),
                current: u32::MAX,
                added: 1,
            })
        );
        assert_eq!(inventory.quantity("超小型ヒューズ"), u32::MAX);
        assert_eq!(inventory.distinct_items(), 1);
    }
}
