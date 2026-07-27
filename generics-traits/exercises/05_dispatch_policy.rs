#![cfg_attr(not(test), allow(dead_code))]

//! # 問題 05: 配送ポリシーを交換できる Dispatcher を作る
//!
//! 最終課題では、待機中の配送ミッションから次の 1 件を選ぶ規則をトレイトとして
//! 切り出します。同じ `Dispatcher<P>` を、緊急度優先と距離優先の両方で使えるように
//! してください
//!
//! 仕様:
//! - `UrgentFirst` は `priority` が大きいミッションを優先する
//! - `ShortestFirst` は `distance_m` が小さいミッションを優先する
//! - 比較値が同じ場合は、待機列で先に現れたミッションを残す
//! - 空の待機列では `None` を返し、待機列を変更しない
//! - 配送するミッションだけを取り出し、残りの相対順序を保つ
//! - `Mission` や `Dispatcher<P>` に不要な `Clone` 境界を付けない
//!
//! `DispatchPolicy::prefers` は、`candidate` が現在の選択 `current` より優先されるときだけ
//! `true` を返します。同点で `false` を返すと、先に見つけたミッションを維持できます
//!
//! ヒント:
//! - 最初の要素の添字を暫定の選択として、残りを順番に比較する
//! - 候補の添字が決まってから `Vec::remove` を呼ぶ
//! - `Dispatcher<P>` 自体ではなく、ポリシーを使う `impl` ブロックだけに境界を付ける

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
    /// `candidate` を `current` より先に配送するときだけ `true` を返す
    fn prefers(&self, candidate: &Mission, current: &Mission) -> bool;
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct UrgentFirst;

impl DispatchPolicy for UrgentFirst {
    fn prefers(&self, candidate: &Mission, current: &Mission) -> bool {
        todo!(
            "緊急度 {} と {} を比較してください",
            candidate.priority,
            current.priority
        )
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct ShortestFirst;

impl DispatchPolicy for ShortestFirst {
    fn prefers(&self, candidate: &Mission, current: &Mission) -> bool {
        todo!(
            "距離 {} m と {} m を比較してください",
            candidate.distance_m,
            current.distance_m
        )
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
    /// 待機列を変更せず、ポリシーが選ぶ要素の添字を返す
    fn select_index(&self, waiting: &[Mission]) -> Option<usize> {
        todo!("{} 件の待機ミッションを比較してください", waiting.len())
    }

    /// 選ばれた 1 件だけを待機列から所有値として取り出す
    fn dispatch(&self, waiting: &mut Vec<Mission>) -> Option<Mission> {
        let selected = self.select_index(waiting)?;
        Some(waiting.remove(selected))
    }
}

fn main() {
    let mut waiting = vec![
        Mission::new("M-501", "図書館", 3, 900),
        Mission::new("M-502", "実験棟", 9, 1_500),
        Mission::new("M-503", "食堂", 5, 300),
    ];

    let dispatcher = Dispatcher::new(UrgentFirst);
    println!("次の配送: {:?}", dispatcher.dispatch(&mut waiting));
    println!("残り: {waiting:#?}");
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
