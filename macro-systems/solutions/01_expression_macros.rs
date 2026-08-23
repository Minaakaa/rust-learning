//! # 解答 01: 式を返す `macro_rules!`

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
        TelemetryBatch::new($robot_id, vec![$($reading),*])
    }};
}

macro_rules! sum_values {
    ($first:expr $(, $rest:expr)* $(,)?) => {{
        #[allow(
            unused_mut,
            reason = "複数の入力を同じ展開形で加算するため、1件の場合も同じlocalを使う"
        )]
        let mut total = $first;
        $(total += $rest;)*
        total
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
