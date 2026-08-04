//! # 解答 08: RPITITで実装固有のiteratorを隠す

#[derive(Debug, PartialEq, Eq)]
struct RouteStep {
    name: String,
    enabled: bool,
}

impl RouteStep {
    fn enabled(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            enabled: true,
        }
    }

    fn disabled(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            enabled: false,
        }
    }
}

trait RoutePlan {
    /// 通常のmethodはvtable経由で呼び出せる
    fn name(&self) -> &str;

    /// 通常のmethodはtrait objectからも呼び出せる
    fn enabled_count(&self) -> usize;

    /// RPITITでは、各implが異なる隠れた具体型を選べる
    ///
    /// 呼び出し側に公開される能力は`DoubleEndedIterator`と`Iterator`だけになる
    /// また、1つのimplの同一method内では、すべてのreturn pathが同じ具体型へ
    /// 解決されなければならない
    ///
    /// `Self: Sized`によりこのmethodをvtableから分離し、trait全体のdyn互換性を保つ
    fn enabled_steps(&self) -> impl DoubleEndedIterator<Item = &str> + '_
    where
        Self: Sized;
}

#[derive(Debug, PartialEq, Eq)]
struct LinearPlan {
    name: String,
    steps: Vec<String>,
}

impl LinearPlan {
    fn new(name: impl Into<String>, steps: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self {
            name: name.into(),
            steps: steps.into_iter().map(Into::into).collect(),
        }
    }
}

impl RoutePlan for LinearPlan {
    fn name(&self) -> &str {
        &self.name
    }

    fn enabled_count(&self) -> usize {
        self.steps.len()
    }

    fn enabled_steps(&self) -> impl DoubleEndedIterator<Item = &str> + '_ {
        self.steps.iter().map(String::as_str)
    }
}

#[derive(Debug, PartialEq, Eq)]
struct LayeredPlan {
    name: String,
    priority: Vec<RouteStep>,
    routine: Vec<RouteStep>,
}

impl LayeredPlan {
    fn new(
        name: impl Into<String>,
        priority: impl IntoIterator<Item = RouteStep>,
        routine: impl IntoIterator<Item = RouteStep>,
    ) -> Self {
        Self {
            name: name.into(),
            priority: priority.into_iter().collect(),
            routine: routine.into_iter().collect(),
        }
    }
}

impl RoutePlan for LayeredPlan {
    fn name(&self) -> &str {
        &self.name
    }

    fn enabled_count(&self) -> usize {
        self.priority
            .iter()
            .chain(&self.routine)
            .filter(|step| step.enabled)
            .count()
    }

    fn enabled_steps(&self) -> impl DoubleEndedIterator<Item = &str> + '_ {
        self.priority
            .iter()
            .chain(&self.routine)
            .filter(|step| step.enabled)
            .map(|step| step.name.as_str())
    }
}

/// RPITITの隠れた型を名指しせず、traitが公開する`Iterator`の能力だけを使う
fn collect_enabled<P>(plan: &P) -> Vec<&str>
where
    P: RoutePlan,
{
    plan.enabled_steps().collect()
}

/// 公開境界に`DoubleEndedIterator`があるため、genericな呼び出し側でも`rev`を使える
fn collect_enabled_in_reverse<P>(plan: &P) -> Vec<&str>
where
    P: RoutePlan,
{
    plan.enabled_steps().rev().collect()
}

/// iterator自体を`Box`化せず、借用した要素をその場で集計する
fn enabled_name_bytes<P>(plan: &P) -> usize
where
    P: RoutePlan,
{
    plan.enabled_steps().map(str::len).sum()
}

/// RPITIT methodは呼ばず、dyn互換な通常methodだけを動的dispatchする
fn describe_dyn(plan: &dyn RoutePlan) -> String {
    format!("{}: 有効な手順 {}件", plan.name(), plan.enabled_count())
}

fn main() {
    let linear = LinearPlan::new("直線経路", ["起動", "走行", "停止"]);
    println!("{}", describe_dyn(&linear));
    println!("有効な手順: {:?}", collect_enabled(&linear));
    println!("逆順: {:?}", collect_enabled_in_reverse(&linear));
    println!("手順名の合計byte数: {}", enabled_name_bytes(&linear));

    let plan = LayeredPlan::new(
        "月面探査",
        [RouteStep::enabled("緊急通信")],
        [RouteStep::disabled("休眠"), RouteStep::enabled("試料採取")],
    );
    println!("{}", describe_dyn(&plan));
    println!("有効な手順: {:?}", collect_enabled(&plan));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slice_iteratorの順序を保つ() {
        let plan = LinearPlan::new("直線経路", ["起動", "走行", "停止"]);

        assert_eq!(collect_enabled(&plan), ["起動", "走行", "停止"]);
        assert_eq!(plan.enabled_count(), 3);
    }

    #[test]
    fn chainした層を順番にfilterする() {
        let plan = LayeredPlan::new(
            "優先経路",
            [
                RouteStep::disabled("無効な先頭"),
                RouteStep::enabled("緊急停止確認"),
            ],
            [
                RouteStep::enabled("通常走行"),
                RouteStep::disabled("無効な末尾"),
            ],
        );

        assert_eq!(collect_enabled(&plan), ["緊急停止確認", "通常走行"]);
        assert_eq!(plan.enabled_count(), 2);
    }

    #[test]
    fn 空のplanは空のiteratorを返す() {
        let linear = LinearPlan::new("空", std::iter::empty::<String>());
        let layered = LayeredPlan::new("空の層", [], []);

        assert!(collect_enabled(&linear).is_empty());
        assert!(collect_enabled(&layered).is_empty());
        assert_eq!(linear.enabled_count(), 0);
        assert_eq!(layered.enabled_count(), 0);
    }

    #[test]
    fn utf8の名前を壊さず借用する() {
        let plan = LayeredPlan::new(
            "月面ロボット🚀",
            [RouteStep::enabled("東京大学🤖")],
            [RouteStep::enabled("試料採取🪨")],
        );

        assert_eq!(plan.name(), "月面ロボット🚀");
        assert_eq!(collect_enabled(&plan), ["東京大学🤖", "試料採取🪨"]);
    }

    #[test]
    fn 要素をcloneせず元の文字列を借用する() {
        let plan = LinearPlan::new("借用", ["同じ領域"]);
        let names = collect_enabled(&plan);

        assert!(std::ptr::eq(names[0], plan.steps[0].as_str()));
    }

    #[test]
    fn 呼び出すたびに独立したiteratorを作る() {
        let plan = LinearPlan::new("再利用", ["A", "B", "C"]);
        let mut first = plan.enabled_steps();

        assert_eq!(first.next(), Some("A"));
        assert_eq!(collect_enabled(&plan), ["A", "B", "C"]);
        assert_eq!(first.next(), Some("B"));
    }

    #[test]
    fn 公開したdouble_ended境界をgeneric側で使う() {
        let linear = LinearPlan::new("逆順", ["A", "B", "C"]);
        let layered = LayeredPlan::new(
            "逆順の層",
            [RouteStep::enabled("P")],
            [RouteStep::disabled("skip"), RouteStep::enabled("R")],
        );

        assert_eq!(collect_enabled_in_reverse(&linear), ["C", "B", "A"]);
        assert_eq!(collect_enabled_in_reverse(&layered), ["R", "P"]);
    }

    #[test]
    fn 異なる隠れた型を同じgeneric_consumerで扱う() {
        let linear = LinearPlan::new("線形", ["ab", "cde"]);
        let layered = LayeredPlan::new(
            "層状",
            [RouteStep::enabled("四")],
            [RouteStep::enabled("five")],
        );

        assert_eq!(enabled_name_bytes(&linear), 5);
        assert_eq!(enabled_name_bytes(&layered), 7);
    }

    #[test]
    fn box化したtrait_objectから通常methodだけを呼ぶ() {
        let plans: Vec<Box<dyn RoutePlan>> = vec![
            Box::new(LinearPlan::new("線形", ["A", "B"])),
            Box::new(LayeredPlan::new(
                "層状",
                [RouteStep::enabled("P")],
                [RouteStep::disabled("skip")],
            )),
        ];

        let descriptions: Vec<_> = plans
            .iter()
            .map(|plan| describe_dyn(plan.as_ref()))
            .collect();

        assert_eq!(
            descriptions,
            ["線形: 有効な手順 2件", "層状: 有効な手順 1件"]
        );
    }
}
