#![cfg_attr(not(test), allow(dead_code))]

//! # 問題 04: clone可能なtrait objectで配送計画を複製する
//!
//! `Clone::clone`は`Self`を返すため、`Clone`をそのまま`DispatchPolicy`のsupertraitには
//! できません。object-safeな`PolicyClone::clone_box`へ具体型のcloneを委譲し、
//! 異なるpolicyを保持する`DispatchPlanner`全体をclone可能にしてください
//!
//! 仕様:
//! - `PolicyClone`を`DispatchPolicy`のsupertraitにする
//! - `Clone + DispatchPolicy + 'static`な具体型へ`PolicyClone`をblanket実装する
//! - `Box<dyn DispatchPolicy>`へ`Clone`を実装する
//! - `PriorityPolicy`は最大priorityの先頭要素を選ぶ
//! - `RoundRobinPolicy`は選択位置、呼び出し回数、選択履歴を保持する
//! - `DispatchPlanner`はpolicyの登録順で結果を返し、`#[derive(Clone)]`できる
//! - missionが空なら`None`を返し、選択履歴とcursorを変更しない
//!
//! `clone_box`が保証するのは、各具体型の`Clone`実装に従った複製です
//! `String`や`Vec`は独立した内容を持つ一方、`Arc`などは共有を維持できます

#[derive(Debug, Clone, PartialEq, Eq)]
struct Mission {
    id: String,
    priority: u8,
}

impl Mission {
    fn new(id: &str, priority: u8) -> Self {
        Self {
            id: id.to_string(),
            priority,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PlanDecision {
    policy_label: String,
    mission_id: Option<String>,
}

trait PolicyClone {
    fn clone_box(&self) -> Box<dyn DispatchPolicy>;
}

trait DispatchPolicy: PolicyClone {
    fn label(&self) -> &str;
    fn rename(&mut self, label: String);
    fn select(&mut self, missions: &[Mission]) -> Option<usize>;
    fn calls(&self) -> usize;
    fn selected_ids(&self) -> &[String];

    /// 具体型だけで利用する関連関数はdyn dispatchの対象から外す
    fn kind() -> &'static str
    where
        Self: Sized;
}

impl<T> PolicyClone for T
where
    T: DispatchPolicy + Clone + 'static,
{
    fn clone_box(&self) -> Box<dyn DispatchPolicy> {
        todo!(
            "{}をcloneしてBox<dyn DispatchPolicy>へ型消去してください",
            std::any::type_name::<T>()
        )
    }
}

impl Clone for Box<dyn DispatchPolicy> {
    fn clone(&self) -> Self {
        let _required_method = || self.as_ref().clone_box();
        todo!("{}をPolicyClone経由でcloneしてください", self.label())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PriorityPolicy {
    label: String,
    calls: usize,
    selected_ids: Vec<String>,
}

impl PriorityPolicy {
    fn new(label: &str) -> Self {
        Self {
            label: label.to_string(),
            calls: 0,
            selected_ids: Vec::new(),
        }
    }
}

impl DispatchPolicy for PriorityPolicy {
    fn label(&self) -> &str {
        &self.label
    }

    fn rename(&mut self, label: String) {
        self.label = label;
    }

    fn select(&mut self, missions: &[Mission]) -> Option<usize> {
        todo!(
            "呼び出し回数を更新し、{}件から最大priorityの先頭要素を選んで履歴へ記録してください",
            missions.len()
        )
    }

    fn calls(&self) -> usize {
        self.calls
    }

    fn selected_ids(&self) -> &[String] {
        &self.selected_ids
    }

    fn kind() -> &'static str {
        "緊急度優先"
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RoundRobinPolicy {
    label: String,
    cursor: usize,
    calls: usize,
    selected_ids: Vec<String>,
}

impl RoundRobinPolicy {
    fn new(label: &str) -> Self {
        Self {
            label: label.to_string(),
            cursor: 0,
            calls: 0,
            selected_ids: Vec::new(),
        }
    }
}

impl DispatchPolicy for RoundRobinPolicy {
    fn label(&self) -> &str {
        &self.label
    }

    fn rename(&mut self, label: String) {
        self.label = label;
    }

    fn select(&mut self, missions: &[Mission]) -> Option<usize> {
        todo!(
            "呼び出し回数を更新し、cursor {}から{}件を巡回して履歴へ記録してください",
            self.cursor,
            missions.len()
        )
    }

    fn calls(&self) -> usize {
        self.calls
    }

    fn selected_ids(&self) -> &[String] {
        &self.selected_ids
    }

    fn kind() -> &'static str {
        "巡回選択"
    }
}

#[derive(Clone, Default)]
struct DispatchPlanner {
    policies: Vec<Box<dyn DispatchPolicy>>,
}

impl DispatchPlanner {
    fn add<P>(&mut self, policy: P)
    where
        P: DispatchPolicy + 'static,
    {
        self.policies.push(Box::new(policy));
    }

    fn len(&self) -> usize {
        self.policies.len()
    }

    fn is_empty(&self) -> bool {
        self.policies.is_empty()
    }

    fn policy(&self, index: usize) -> Option<&dyn DispatchPolicy> {
        self.policies.get(index).map(Box::as_ref)
    }

    fn rename_policy(&mut self, index: usize, label: String) -> bool {
        let Some(policy) = self.policies.get_mut(index) else {
            return false;
        };
        policy.rename(label);
        true
    }

    fn plan_all(&mut self, missions: &[Mission]) -> Vec<PlanDecision> {
        let _required_method = |policy: &mut dyn DispatchPolicy| policy.select(missions);
        todo!(
            "{}個のpolicyを登録順に呼び、{}件のmissionに対する結果を集めてください",
            self.policies.len(),
            missions.len()
        )
    }
}

fn main() {
    let missions = vec![Mission::new("M-1201", 3), Mission::new("M-1202", 9)];
    let mut planner = DispatchPlanner::default();
    planner.add(PriorityPolicy::new("緊急便🚨"));
    planner.add(RoundRobinPolicy::new("巡回便🤖"));

    for decision in planner.plan_all(&missions) {
        println!("{}: {:?}", decision.policy_label, decision.mission_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };

    fn missions() -> Vec<Mission> {
        vec![
            Mission::new("M-401", 3),
            Mission::new("M-402", 9),
            Mission::new("M-403", 6),
        ]
    }

    fn selected_ids(policy: &dyn DispatchPolicy) -> Vec<&str> {
        policy.selected_ids().iter().map(String::as_str).collect()
    }

    #[test]
    fn 異なるpolicyを登録順に動的dispatchできる() {
        let mut planner = DispatchPlanner::default();
        planner.add(PriorityPolicy::new("priority"));
        planner.add(RoundRobinPolicy::new("round-robin"));

        let decisions = planner.plan_all(&missions());

        assert_eq!(planner.len(), 2);
        assert!(!planner.is_empty());
        assert_eq!(decisions[0].mission_id.as_deref(), Some("M-402"));
        assert_eq!(decisions[1].mission_id.as_deref(), Some("M-401"));
        assert_eq!(PriorityPolicy::kind(), "緊急度優先");
        assert_eq!(RoundRobinPolicy::kind(), "巡回選択");
    }

    #[test]
    fn round_robinは呼び出し回数と選択履歴を保持する() {
        let mut planner = DispatchPlanner::default();
        planner.add(RoundRobinPolicy::new("巡回"));
        let missions = missions();

        let selected = (0..4)
            .map(|_| planner.plan_all(&missions)[0].mission_id.clone().unwrap())
            .collect::<Vec<_>>();

        assert_eq!(selected, ["M-401", "M-402", "M-403", "M-401"]);
        let policy = planner.policy(0).unwrap();
        assert_eq!(policy.calls(), 4);
        assert_eq!(selected_ids(policy), ["M-401", "M-402", "M-403", "M-401"]);
    }

    #[test]
    fn cloneしたplannerのstringとvecとcounterは独立して変化する() {
        let mut original = DispatchPlanner::default();
        original.add(RoundRobinPolicy::new("複製前"));
        original.plan_all(&missions());
        let mut cloned = original.clone();

        assert!(original.rename_policy(0, String::from("原本だけ変更")));
        original.plan_all(&missions());

        let original_policy = original.policy(0).unwrap();
        let cloned_policy = cloned.policy(0).unwrap();
        assert_eq!(original_policy.label(), "原本だけ変更");
        assert_eq!(cloned_policy.label(), "複製前");
        assert_eq!(original_policy.calls(), 2);
        assert_eq!(cloned_policy.calls(), 1);
        assert_eq!(selected_ids(original_policy), ["M-401", "M-402"]);
        assert_eq!(selected_ids(cloned_policy), ["M-401"]);

        cloned.plan_all(&missions());
        assert_eq!(selected_ids(cloned.policy(0).unwrap()), ["M-401", "M-402"]);
    }

    #[test]
    fn policyがないplannerは空の計画を返す() {
        let mut planner = DispatchPlanner::default();

        assert!(planner.is_empty());
        assert!(planner.plan_all(&missions()).is_empty());
        assert!(!planner.rename_policy(0, String::from("存在しない")));
        assert!(planner.policy(usize::MAX).is_none());
    }

    #[test]
    fn missionが空でも各policyを一度呼び履歴は追加しない() {
        let mut planner = DispatchPlanner::default();
        planner.add(PriorityPolicy::new("priority"));
        planner.add(RoundRobinPolicy::new("round-robin"));

        let decisions = planner.plan_all(&[]);

        assert_eq!(
            decisions,
            [
                PlanDecision {
                    policy_label: String::from("priority"),
                    mission_id: None,
                },
                PlanDecision {
                    policy_label: String::from("round-robin"),
                    mission_id: None,
                },
            ]
        );
        for index in 0..2 {
            let policy = planner.policy(index).unwrap();
            assert_eq!(policy.calls(), 1);
            assert!(policy.selected_ids().is_empty());
        }

        assert_eq!(
            planner.plan_all(&missions())[1].mission_id.as_deref(),
            Some("M-401")
        );
    }

    #[test]
    fn priorityの同点では先頭を選び入力順を変更しない() {
        let missions = vec![Mission::new("先着", 8), Mission::new("後着", 8)];
        let original = missions.clone();
        let mut planner = DispatchPlanner::default();
        planner.add(PriorityPolicy::new("同点確認"));

        let decision = planner.plan_all(&missions).remove(0);

        assert_eq!(decision.mission_id.as_deref(), Some("先着"));
        assert_eq!(missions, original);
    }

    #[test]
    fn utf8のlabelとmission_idを失わない() {
        let missions = vec![Mission::new("配送🤖-東京大学", u8::MAX)];
        let mut planner = DispatchPlanner::default();
        planner.add(PriorityPolicy::new("最優先便🚨"));

        let decision = planner.plan_all(&missions).remove(0);

        assert_eq!(decision.policy_label, "最優先便🚨");
        assert_eq!(decision.mission_id.as_deref(), Some("配送🤖-東京大学"));
        assert_eq!(
            selected_ids(planner.policy(0).unwrap()),
            ["配送🤖-東京大学"]
        );
    }

    #[derive(Clone)]
    struct DropTrackingPolicy {
        label: String,
        drops: Arc<AtomicUsize>,
    }

    impl Drop for DropTrackingPolicy {
        fn drop(&mut self) {
            self.drops.fetch_add(1, Ordering::SeqCst);
        }
    }

    impl DispatchPolicy for DropTrackingPolicy {
        fn label(&self) -> &str {
            &self.label
        }

        fn rename(&mut self, label: String) {
            self.label = label;
        }

        fn select(&mut self, _missions: &[Mission]) -> Option<usize> {
            None
        }

        fn calls(&self) -> usize {
            0
        }

        fn selected_ids(&self) -> &[String] {
            &[]
        }

        fn kind() -> &'static str {
            "drop確認"
        }
    }

    #[test]
    fn trait_objectの原本とcloneを各一度だけdropする() {
        let drops = Arc::new(AtomicUsize::new(0));
        {
            let mut original = DispatchPlanner::default();
            original.add(DropTrackingPolicy {
                label: String::from("drop対象"),
                drops: Arc::clone(&drops),
            });
            let cloned = original.clone();

            drop(cloned);
            assert_eq!(drops.load(Ordering::SeqCst), 1);
        }

        assert_eq!(drops.load(Ordering::SeqCst), 2);
    }
}
