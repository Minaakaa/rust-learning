//! # 問題 01: 式を返す `macro_rules!`
//!
//! 関数のように値を返す宣言的マクロを作ります
//! `expr` fragment と反復を使い、配送ロボットの測定値をまとめて構築してください
//!
//! 仕様:
//! - `telemetry!` は `telemetry!(robot_id; value, value, ...)` の形を受け取る
//! - `telemetry!` は `TelemetryBatch` を構築し、入力順と所有権を保つ
//! - `sum_values!` は1個以上の式を受け取り、合計値を返す
//! - 末尾のcommaはあってもなくてもよい
//! - 入力式は一度だけ評価する
//!
//! ヒント:
//! - `expr` はリテラルだけでなく変数や計算式も受け取れる
//! - `$(...),*` は0個以上、`$(...),+` は1個以上の反復を表す
//! - 式マクロの展開全体を `{{ ... }}` で囲むと、local variableのscopeを閉じ込められる

#[derive(Debug, PartialEq, Eq)]
struct TelemetryBatch {
    robot_id: String,
    readings: Vec<u16>,
}

impl TelemetryBatch {
    fn new(robot_id: impl Into<String>, readings: Vec<u16>) -> Self {
        Self {
            robot_id: robot_id.into(),
            readings,
        }
    }

    fn robot_id(&self) -> &str {
        &self.robot_id
    }

    fn readings(&self) -> &[u16] {
        &self.readings
    }

    fn total(&self) -> u16 {
        self.readings.iter().copied().sum()
    }
}

macro_rules! telemetry {
    ($robot_id:expr; $($reading:expr),* $(,)?) => {{
        if std::hint::black_box(false) {
            todo!("robot_idと複数の測定値からTelemetryBatchを構築してください")
        }
        let _ = ($($reading),*);
        TelemetryBatch::new($robot_id, Vec::new())
    }};
}

macro_rules! sum_values {
    ($first:expr $(, $rest:expr)* $(,)?) => {{
        if std::hint::black_box(false) {
            todo!("先頭と残りの式を一度ずつ評価して合計してください")
        }
        let _ = stringify!($($rest),*);
        $first
    }};
}

fn main() {
    let batch = telemetry!("配送ロボット-1301"; 82, 81, 80);
    println!(
        "{}: {:?} / 合計={}",
        batch.robot_id(),
        batch.readings(),
        batch.total()
    );
    println!("式の合計={}", sum_values!(10_u16, 20, 12));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn telemetry_macroは入力順とutf8を保つ() {
        let batch = telemetry!("本郷ロボット🤖"; 82, 79, 91);

        assert_eq!(batch.robot_id(), "本郷ロボット🤖");
        assert_eq!(batch.readings(), [82, 79, 91]);
        assert_eq!(batch.total(), 252);
    }

    #[test]
    fn telemetry_macroは空の測定値も構築する() {
        let batch = telemetry!("empty";);

        assert!(batch.readings().is_empty());
        assert_eq!(batch.total(), 0);
    }

    #[test]
    fn telemetry_macroは式を受け取る() {
        let base = 40_u16;
        let batch = telemetry!("calculated"; base + 2, 100 / 2, 3 * 4);

        assert_eq!(batch.readings(), [42, 50, 12]);
    }

    #[test]
    fn sum_values_macroは一つの式を受け取る() {
        assert_eq!(sum_values!(7_u16), 7);
    }

    #[test]
    fn sum_values_macroは末尾commaと境界値を扱う() {
        assert_eq!(sum_values!(u16::MAX, 0,), u16::MAX);
        assert_eq!(sum_values!(1_u16, 2, 3, 4), 10);
    }

    #[test]
    fn 入力式を一度だけ評価する() {
        let mut calls = 0_u16;
        let batch = telemetry!("once"; {
            calls += 1;
            calls
        });

        assert_eq!(calls, 1);
        assert_eq!(batch.readings(), [1]);
    }
}
