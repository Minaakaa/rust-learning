//! 問題 05 の解答例

#[derive(Debug, PartialEq, Eq)]
struct Mission {
    id: String,
    destination: String,
    priority: u8,
    distance_m: u32,
}

impl Mission {
    fn new(id: &str, destination: &str, priority: u8, distance_m: u32) -> Self {
        Self {
            id: id.to_string(),
            destination: destination.to_string(),
            priority,
            distance_m,
        }
    }

    fn id(&self) -> &str {
        &self.id
    }

    fn destination(&self) -> &str {
        &self.destination
    }
}

trait DispatchPolicy {
    fn prefers(&self, candidate: &Mission, current: &Mission) -> bool;
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct UrgentFirst;

impl DispatchPolicy for UrgentFirst {
    fn prefers(&self, candidate: &Mission, current: &Mission) -> bool {
        candidate.priority > current.priority
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct ShortestFirst;

impl DispatchPolicy for ShortestFirst {
    fn prefers(&self, candidate: &Mission, current: &Mission) -> bool {
        candidate.distance_m < current.distance_m
    }
}

#[derive(Debug, PartialEq, Eq)]
struct Dispatcher<P> {
    policy: P,
}

impl<P> Dispatcher<P> {
    fn new(policy: P) -> Self {
        Self { policy }
    }

    fn policy(&self) -> &P {
        &self.policy
    }
}

impl<P> Dispatcher<P>
where
    P: DispatchPolicy,
{
    fn select_index(&self, waiting: &[Mission]) -> Option<usize> {
        let mut missions = waiting.iter().enumerate();
        let (mut selected, mut current) = missions.next()?;

        for (index, candidate) in missions {
            if self.policy.prefers(candidate, current) {
                selected = index;
                current = candidate;
            }
        }

        Some(selected)
    }

    fn dispatch(&self, waiting: &mut Vec<Mission>) -> Option<Mission> {
        let selected = self.select_index(waiting)?;
        Some(waiting.remove(selected))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mission(id: &str, priority: u8, distance_m: u32) -> Mission {
        Mission::new(id, &format!("{id} の配送先"), priority, distance_m)
    }

    #[test]
    fn 緊急度優先は最も大きいpriorityを選ぶ() {
        let dispatcher = Dispatcher::new(UrgentFirst);
        let mut waiting = vec![
            mission("M-510", 2, 100),
            mission("M-511", 10, 2_000),
            mission("M-512", 6, 50),
        ];

        assert!(dispatcher.policy().prefers(&waiting[1], &waiting[0]));
        assert!(!dispatcher.policy().prefers(&waiting[0], &waiting[1]));
        assert_eq!(dispatcher.select_index(&waiting), Some(1));
        let selected = dispatcher.dispatch(&mut waiting).unwrap();

        assert_eq!(selected.id(), "M-511");
        assert_eq!(selected.destination(), "M-511 の配送先");
    }

    #[test]
    fn 距離優先は最も近い配送先を選ぶ() {
        let dispatcher = Dispatcher::new(ShortestFirst);
        let mut waiting = vec![
            mission("M-520", 10, 900),
            mission("M-521", 1, 120),
            mission("M-522", 5, 400),
        ];

        assert!(dispatcher.policy().prefers(&waiting[1], &waiting[0]));
        assert!(!dispatcher.policy().prefers(&waiting[0], &waiting[1]));
        assert_eq!(dispatcher.select_index(&waiting), Some(1));
        let selected = dispatcher.dispatch(&mut waiting).unwrap();

        assert_eq!(selected.id(), "M-521");
    }

    #[test]
    fn 同点では待機列の先頭側を選ぶ() {
        let urgent = Dispatcher::new(UrgentFirst);
        let shortest = Dispatcher::new(ShortestFirst);

        let mut urgent_waiting = vec![mission("M-530", 255, 800), mission("M-531", 255, 100)];
        let mut short_waiting = vec![mission("M-532", 0, 0), mission("M-533", 255, 0)];

        assert_eq!(urgent.dispatch(&mut urgent_waiting).unwrap().id(), "M-530");
        assert_eq!(shortest.dispatch(&mut short_waiting).unwrap().id(), "M-532");
    }

    #[test]
    fn 選んだ要素だけを削除して残りの順序を保つ() {
        let dispatcher = Dispatcher::new(UrgentFirst);
        let mut waiting = vec![
            mission("M-540", 2, 100),
            mission("M-541", 9, 200),
            mission("M-542", 4, 300),
            mission("M-543", 1, 400),
        ];

        let selected = dispatcher.dispatch(&mut waiting).unwrap();

        assert_eq!(selected.id(), "M-541");
        let remaining_ids: Vec<_> = waiting.iter().map(Mission::id).collect();
        assert_eq!(remaining_ids, ["M-540", "M-542", "M-543"]);
    }

    #[test]
    fn 空の待機列は変更せずnoneを返す() {
        let dispatcher = Dispatcher::new(UrgentFirst);
        let mut waiting = Vec::new();

        assert_eq!(dispatcher.select_index(&waiting), None);
        assert_eq!(dispatcher.dispatch(&mut waiting), None);
        assert!(waiting.is_empty());
    }

    #[test]
    fn 末尾の最適候補と一件だけの待機列を処理する() {
        let dispatcher = Dispatcher::new(UrgentFirst);
        let mut waiting = vec![
            mission("M-560", 1, 100),
            mission("M-561", 2, 200),
            mission("M-562", 3, 300),
        ];

        assert_eq!(dispatcher.select_index(&waiting), Some(2));
        assert_eq!(dispatcher.dispatch(&mut waiting).unwrap().id(), "M-562");

        let mut one = vec![mission("M-563", 0, u32::MAX)];
        assert_eq!(dispatcher.select_index(&one), Some(0));
        assert_eq!(dispatcher.dispatch(&mut one).unwrap().id(), "M-563");
        assert!(one.is_empty());
    }

    #[test]
    fn テスト内の独自ポリシーもdispatcherへ差し込める() {
        struct LastId;

        impl DispatchPolicy for LastId {
            fn prefers(&self, candidate: &Mission, current: &Mission) -> bool {
                candidate.id() > current.id()
            }
        }

        let dispatcher = Dispatcher::new(LastId);
        let mut waiting = vec![
            mission("M-550", 255, 0),
            mission("M-599", 0, 9_999),
            mission("M-570", 100, 10),
        ];

        assert_eq!(dispatcher.dispatch(&mut waiting).unwrap().id(), "M-599");
    }

    #[test]
    fn policyを使わない基本apiにはトレイト境界が不要() {
        let dispatcher = Dispatcher::new(String::from("手動運用"));

        assert_eq!(dispatcher.policy(), "手動運用");
    }
}
