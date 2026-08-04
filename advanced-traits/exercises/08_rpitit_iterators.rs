//! # 問題 08: RPITITで実装固有のiteratorを隠す
//!
//! return-position `impl Trait` in trait（RPITIT）を使うと、trait methodの
//! 戻り値を公開境界だけで表し、複雑な具体型を実装の内側へ隠せます
//! この問題では、経路計画ごとに異なるiterator pipelineを選びながら、同じgenericな
//! 呼び出し側から有効な手順を処理します
//!
//! 仕様:
//! - `RoutePlan::enabled_steps`は`impl DoubleEndedIterator<Item = &str>`を返す
//! - `LinearPlan`は`Vec<String>`のslice iteratorを変換し、格納順にすべて返す
//! - `LayeredPlan`はpriorityとroutineをchainし、無効な手順をfilterして返す
//! - priorityはroutineより先に現れ、各層の順序も維持する
//! - `enabled_count`は有効な手順数を返す通常methodにする
//! - `collect_enabled`などのgeneric consumerは隠れた具体型を名指ししない
//! - iteratorの要素は元の`String`から借用し、cloneしない
//! - `enabled_steps`を呼ぶたびに独立した新しいiteratorを返す
//! - `Self: Sized`でRPITIT methodをvtableから除外し、`RoutePlan`をdyn互換に保つ
//! - `dyn RoutePlan`からは`name`と`enabled_count`だけを呼び出す
//!
//! RPITITの重要な境界:
//! - 各implは互いに異なる隠れた具体的iterator型を選べる
//! - ただし、1つのimplの同一method内では全return pathを1つの具体型へ統一する
//! - callerが使える能力はreturn typeに公開されたtrait boundだけになる
//! - `impl Iterator`は具体型を保つため、iteratorを`Box<dyn Iterator>`へ格納する
//!   heap allocationやvtable dispatchを必要としない
//!
//! TODO:
//! - 2つの`enabled_steps`を異なるiterator pipelineで実装する
//! - `LayeredPlan::enabled_count`で有効な手順だけを数える
//! - 3つのgeneric consumerを公開済みのiterator境界だけで実装する
//! - `describe_dyn`でdyn互換な通常methodだけを呼ぶ
//!
//! ヒント:
//! - `String::as_str`は`&String`を`&str`へ変換する
//! - `Iterator::chain`、`filter`、`map`はiterator adapterを返す
//! - `DoubleEndedIterator`を公開しているため、callerは`rev`を利用できる
//! - 異なる型を条件分岐から直接返したくなっても、安易に`Box`で型消去せず、
//!   同じadapter pipeline内で条件を表せないか検討する

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

    /// RPITIT methodをvtableから分離し、trait全体のdyn互換性を保つ
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
        let _ = self;
        std::iter::once_with(|| todo!("slice iteratorを&strへ変換してください"))
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
        todo!("2つの層にある有効な手順だけを数えてください")
    }

    fn enabled_steps(&self) -> impl DoubleEndedIterator<Item = &str> + '_ {
        let _ = self;
        std::iter::once_with(|| {
            todo!("priorityとroutineをchainし、有効な手順だけを&strで返してください")
        })
    }
}

/// RPITITの隠れた型を名指しせず、traitが公開する`Iterator`の能力だけを使う
fn collect_enabled<P>(plan: &P) -> Vec<&str>
where
    P: RoutePlan,
{
    // 未実行のclosureでRPITIT methodだけを型検査する
    let _required_method = |plan: &P| {
        let _ = plan.enabled_steps();
    };
    let _ = plan;
    todo!("有効な手順を順番に集めてください")
}

/// 公開境界に`DoubleEndedIterator`があるため、genericな呼び出し側でも`rev`を使える
fn collect_enabled_in_reverse<P>(plan: &P) -> Vec<&str>
where
    P: RoutePlan,
{
    let _ = plan;
    todo!("公開された境界だけを使って逆順に集めてください")
}

/// iterator自体を`Box`化せず、借用した要素をその場で集計する
fn enabled_name_bytes<P>(plan: &P) -> usize
where
    P: RoutePlan,
{
    let _ = plan;
    todo!("各手順名のbyte数をiterator上で合計してください")
}

/// RPITIT methodは呼ばず、dyn互換な通常methodだけを動的dispatchする
fn describe_dyn(plan: &dyn RoutePlan) -> String {
    // 未実行のclosureでdyn互換なmethodだけを型検査する
    let _required_methods = |plan: &dyn RoutePlan| {
        let _ = plan.name();
        let _ = plan.enabled_count();
    };
    let _ = plan;
    todo!("plan名と有効な手順数を日本語で説明してください")
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
