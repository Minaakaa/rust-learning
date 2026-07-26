//! 問題 03 の解答例。

use std::collections::{HashMap, hash_map::Entry};

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

    fn receive(&mut self, item: &str, quantity: u32) -> Result<u32, InventoryError> {
        let item = item.trim();
        if item.is_empty() {
            return Err(InventoryError::EmptyItemName);
        }
        if quantity == 0 {
            return Err(InventoryError::ZeroQuantity);
        }

        match self.stock.entry(item.to_string()) {
            Entry::Occupied(mut entry) => {
                let current = *entry.get();
                let updated = current.checked_add(quantity).ok_or_else(|| {
                    InventoryError::QuantityOverflow {
                        item: entry.key().clone(),
                        current,
                        added: quantity,
                    }
                })?;
                entry.insert(updated);
                Ok(updated)
            }
            Entry::Vacant(entry) => {
                entry.insert(quantity);
                Ok(quantity)
            }
        }
    }

    fn ship(&mut self, item: &str, quantity: u32) -> Result<u32, InventoryError> {
        let item = item.trim();
        if item.is_empty() {
            return Err(InventoryError::EmptyItemName);
        }
        if quantity == 0 {
            return Err(InventoryError::ZeroQuantity);
        }

        match self.stock.entry(item.to_string()) {
            Entry::Vacant(entry) => Err(InventoryError::UnknownItem(entry.into_key())),
            Entry::Occupied(mut entry) => {
                let available = *entry.get();
                if quantity > available {
                    return Err(InventoryError::InsufficientStock {
                        item: entry.key().clone(),
                        available,
                        requested: quantity,
                    });
                }

                let remaining = available - quantity;
                if remaining == 0 {
                    entry.remove();
                } else {
                    entry.insert(remaining);
                }
                Ok(remaining)
            }
        }
    }

    fn quantity(&self, item: &str) -> u32 {
        self.stock.get(item.trim()).copied().unwrap_or(0)
    }

    fn distinct_items(&self) -> usize {
        self.stock.len()
    }
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
