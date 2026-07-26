#![cfg_attr(not(test), allow(dead_code))]

//! # 問題 04: `HashSet` でロボットの能力を比較する
//!
//! 配送先ごとに必要な能力は異なります。ロボットが持つ能力を `HashSet` に保存し、
//! 追加・削除・所属確認と、積集合・和集合・差集合を実装してください。
//!
//! `HashSet` の要素には `Eq` と `Hash` が必要です。`Capability` の derive に注目し、
//! 集合の反復順序には依存しないでください。
//!
//! 仕様:
//! - 同じ能力を複数回受け取っても、集合には 1 つだけ保存する。
//! - `shared_with` は両方にある能力、`combined_with` はどちらかにある全能力を返す。
//! - `missing_for` は「必要だが、このロボットにはない能力」を返す。
//! - 集合演算では、どちらの入力も変更しない。

use std::collections::HashSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Capability {
    IndoorNavigation,
    Elevator,
    HeavyCargo,
    NightDelivery,
    RainOperation,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct Capabilities {
    items: HashSet<Capability>,
}

impl Capabilities {
    fn new() -> Self {
        Self::default()
    }

    fn from_slice(items: &[Capability]) -> Self {
        todo!(
            "{} 個の値を重複のない HashSet に集めてください",
            items.len()
        )
    }

    fn len(&self) -> usize {
        todo!("能力の種類数を返してください")
    }

    /// 新しく追加した場合だけ `true` を返す。
    fn grant(&mut self, item: Capability) -> bool {
        todo!("能力 {item:?} を追加してください")
    }

    /// 存在した能力を削除した場合だけ `true` を返す。
    fn revoke(&mut self, item: Capability) -> bool {
        todo!("能力 {item:?} を削除してください")
    }

    fn has(&self, item: Capability) -> bool {
        todo!("能力 {item:?} が含まれるか確認してください")
    }

    fn shared_with(&self, other: &Self) -> Self {
        todo!(
            "{} 個と {} 個の能力の積集合を返してください",
            self.items.len(),
            other.items.len()
        )
    }

    fn combined_with(&self, other: &Self) -> Self {
        todo!(
            "{} 個と {} 個の能力の和集合を返してください",
            self.items.len(),
            other.items.len()
        )
    }

    /// `required - self`、つまりこのロボットに不足している能力を返す。
    fn missing_for(&self, required: &Self) -> Self {
        todo!(
            "必要な {} 個から保有する {} 個を引いてください",
            required.items.len(),
            self.items.len()
        )
    }

    fn satisfies(&self, required: &Self) -> bool {
        todo!(
            "保有する {} 個が必要な {} 個をすべて含むか確認してください",
            self.items.len(),
            required.items.len()
        )
    }
}

fn main() {
    let robot = Capabilities::from_slice(&[
        Capability::IndoorNavigation,
        Capability::Elevator,
        Capability::HeavyCargo,
    ]);
    let library = Capabilities::from_slice(&[Capability::IndoorNavigation, Capability::Elevator]);

    println!("図書館へ配送可能: {}", robot.satisfies(&library));
    println!("不足している能力: {:?}", robot.missing_for(&library));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn capabilities(items: &[Capability]) -> Capabilities {
        Capabilities::from_slice(items)
    }

    #[test]
    fn 重複する能力を一つだけ保持する() {
        let set = capabilities(&[
            Capability::IndoorNavigation,
            Capability::Elevator,
            Capability::IndoorNavigation,
        ]);

        assert_eq!(set.len(), 2);
        assert!(set.has(Capability::IndoorNavigation));
        assert!(set.has(Capability::Elevator));
        assert!(!set.has(Capability::HeavyCargo));
    }

    #[test]
    fn 追加と削除が実際に変更したかを返す() {
        let mut set = Capabilities::new();

        assert!(set.grant(Capability::NightDelivery));
        assert!(!set.grant(Capability::NightDelivery));
        assert!(set.has(Capability::NightDelivery));
        assert!(set.revoke(Capability::NightDelivery));
        assert!(!set.revoke(Capability::NightDelivery));
        assert!(!set.has(Capability::NightDelivery));
        assert_eq!(set.len(), 0);
    }

    #[test]
    fn 積集合と和集合を順序に依存せず求める() {
        let first = capabilities(&[
            Capability::IndoorNavigation,
            Capability::Elevator,
            Capability::HeavyCargo,
        ]);
        let second = capabilities(&[Capability::Elevator, Capability::NightDelivery]);
        let first_before = first.clone();
        let second_before = second.clone();

        assert_eq!(
            first.shared_with(&second),
            capabilities(&[Capability::Elevator])
        );
        assert_eq!(
            first.combined_with(&second),
            capabilities(&[
                Capability::IndoorNavigation,
                Capability::Elevator,
                Capability::HeavyCargo,
                Capability::NightDelivery,
            ])
        );
        assert_eq!(first, first_before);
        assert_eq!(second, second_before);
    }

    #[test]
    fn 必要だが持っていない能力だけを返す() {
        let robot = capabilities(&[Capability::IndoorNavigation]);
        let required = capabilities(&[
            Capability::IndoorNavigation,
            Capability::Elevator,
            Capability::RainOperation,
        ]);

        assert_eq!(
            robot.missing_for(&required),
            capabilities(&[Capability::Elevator, Capability::RainOperation])
        );
        assert!(!robot.satisfies(&required));
    }

    #[test]
    fn 同じ能力または余分な能力があれば要件を満たす() {
        let required = capabilities(&[Capability::IndoorNavigation, Capability::Elevator]);
        let exact = required.clone();
        let extra = capabilities(&[
            Capability::IndoorNavigation,
            Capability::Elevator,
            Capability::HeavyCargo,
        ]);

        assert!(exact.satisfies(&required));
        assert!(extra.satisfies(&required));
        assert_eq!(extra.missing_for(&required), Capabilities::new());
    }

    #[test]
    fn 空集合との演算も集合の法則に従う() {
        let empty = Capabilities::new();
        let robot = capabilities(&[Capability::IndoorNavigation, Capability::RainOperation]);

        assert_eq!(robot.shared_with(&empty), empty);
        assert_eq!(robot.combined_with(&Capabilities::new()), robot);
        assert_eq!(robot.missing_for(&Capabilities::new()), Capabilities::new());
        assert_eq!(Capabilities::new().missing_for(&robot), robot);
        assert!(Capabilities::new().satisfies(&Capabilities::new()));
        assert!(robot.satisfies(&Capabilities::new()));
    }
}
