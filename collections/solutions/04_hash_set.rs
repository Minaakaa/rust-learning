//! 問題 04 の解答例。

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
        Self {
            items: items.iter().copied().collect(),
        }
    }

    fn len(&self) -> usize {
        self.items.len()
    }

    fn grant(&mut self, item: Capability) -> bool {
        self.items.insert(item)
    }

    fn revoke(&mut self, item: Capability) -> bool {
        self.items.remove(&item)
    }

    fn has(&self, item: Capability) -> bool {
        self.items.contains(&item)
    }

    fn shared_with(&self, other: &Self) -> Self {
        Self {
            items: self.items.intersection(&other.items).copied().collect(),
        }
    }

    fn combined_with(&self, other: &Self) -> Self {
        Self {
            items: self.items.union(&other.items).copied().collect(),
        }
    }

    fn missing_for(&self, required: &Self) -> Self {
        Self {
            items: required.items.difference(&self.items).copied().collect(),
        }
    }

    fn satisfies(&self, required: &Self) -> bool {
        self.items.is_superset(&required.items)
    }
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
