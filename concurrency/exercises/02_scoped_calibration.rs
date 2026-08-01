#![cfg_attr(not(test), allow(dead_code))]

//! # 問題 02: scoped thread で借用データを並列校正する
//!
//! 配送ロボットから集めたセンサー値を複数の worker に分け、対応する補正値を
//! 加算します
//! `thread::scope` を使い、呼び出し元が所有する slice を複製せず安全に借用する
//! `calibrate_parallel` を完成させてください
//!
//! 仕様:
//! - `worker_count == 0` は他の検証より先に `ZeroWorkers` を返す
//! - slice の長さが異なる場合は `LengthMismatch` で両方の長さを返す
//! - エラーの場合は `samples` を一切変更しない
//! - 長さが同じ空 slice と1以上の worker は `Ok(0)` を返す
//! - 実際に起動する worker 数は `min(worker_count, samples.len())` とする
//! - 要素数を差が最大1の連続した範囲へ分ける
//! - `samples` は `chunks_mut`、`corrections` は対応する `chunks` で分割する
//! - 各 worker は `thread::scope` の中で借用した範囲だけを更新する
//! - 加算には `i32::saturating_add` を使う
//! - 成功時は実際に起動した worker 数を返す
//!
//! ヒント:
//! - 検証をすべて終えてから `samples` の可変範囲を作る
//! - `len / workers` と `len % workers` で短い範囲と1要素長い範囲の数が分かる
//! - 長い範囲の領域と通常の領域を `split_at_mut` で先に分離できる
//! - `scope.spawn(move || ...)` なら、互いに重ならない slice を各 thread へ渡せる
//! - scoped thread は scope を抜ける前にすべて join される

#[allow(
    unused_imports,
    reason = "calibrate_parallel 完成前のスターターでは thread::scope を使用しないため"
)]
use std::thread;

#[derive(Debug, PartialEq, Eq)]
enum CalibrationError {
    ZeroWorkers,
    LengthMismatch { samples: usize, corrections: usize },
}

fn calibrate_parallel(
    samples: &mut [i32],
    corrections: &[i32],
    worker_count: usize,
) -> Result<usize, CalibrationError> {
    todo!(
        "{} samples と {} corrections を {worker_count} workers で校正してください",
        samples.len(),
        corrections.len()
    )
}

fn main() {
    let mut samples = [1_000, 1_010, 990, 1_020, 980];
    let corrections = [5, -10, 10, -20, 20];
    let workers =
        calibrate_parallel(&mut samples, &corrections, 3).expect("校正値と worker 数が有効である");

    println!("{workers} workers で校正: {samples:?}");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn worker数0を長さ不一致より先に報告しsamplesを変更しない() {
        let mut samples = [10, 20, 30];
        let before = samples;

        let result = calibrate_parallel(&mut samples, &[1], 0);

        assert_eq!(result, Err(CalibrationError::ZeroWorkers));
        assert_eq!(samples, before);
    }

    #[test]
    fn 長さ不一致では両方の長さを報告しsamplesを変更しない() {
        let mut short_samples = [100, 200];
        let short_before = short_samples;
        let short_result = calibrate_parallel(&mut short_samples, &[1, 2, 3], 2);

        assert_eq!(
            short_result,
            Err(CalibrationError::LengthMismatch {
                samples: 2,
                corrections: 3,
            })
        );
        assert_eq!(short_samples, short_before);

        let mut long_samples = [100, 200, 300];
        let long_before = long_samples;
        let long_result = calibrate_parallel(&mut long_samples, &[1], 2);

        assert_eq!(
            long_result,
            Err(CalibrationError::LengthMismatch {
                samples: 3,
                corrections: 1,
            })
        );
        assert_eq!(long_samples, long_before);
    }

    #[test]
    fn 空入力はworkerを起動せず0を返す() {
        let mut samples = [];

        let workers = calibrate_parallel(&mut samples, &[], 4).expect("空入力は有効");

        assert_eq!(workers, 0);
        assert!(samples.is_empty());
    }

    #[test]
    fn 単一workerですべてのsampleを校正する() {
        let mut samples = [100, 200, 300, 400];

        let workers =
            calibrate_parallel(&mut samples, &[1, 2, 3, 4], 1).expect("長さと worker 数が有効");

        assert_eq!(workers, 1);
        assert_eq!(samples, [101, 202, 303, 404]);
    }

    #[test]
    fn 割り切れない範囲を指定worker数で漏れなく処理する() {
        let mut samples = [10, 20, 30, 40, 50, 60, 70];
        let corrections = [1, 2, 3, 4, 5, 6, 7];

        let workers =
            calibrate_parallel(&mut samples, &corrections, 4).expect("長さと worker 数が有効");

        assert_eq!(workers, 4);
        assert_eq!(samples, [11, 22, 33, 44, 55, 66, 77]);
    }

    #[test]
    fn sample数より多いworkerを指定しても空workerを作らない() {
        let mut samples = [1_000, 2_000, 3_000];

        let workers = calibrate_parallel(&mut samples, &[10, 20, 30], 20)
            .expect("過剰な worker 数も処理できる");

        assert_eq!(workers, 3);
        assert_eq!(samples, [1_010, 2_020, 3_030]);
    }

    #[test]
    fn 正負と0のcorrectionを要素ごとに適用する() {
        let mut samples = [-100, 0, 100, -50, 50];

        let workers = calibrate_parallel(&mut samples, &[25, -25, 0, -75, 75], 2)
            .expect("符号付き校正値を扱える");

        assert_eq!(workers, 2);
        assert_eq!(samples, [-75, -25, 100, -125, 125]);
    }

    #[test]
    fn 上下限を越える加算はsaturatingする() {
        let mut samples = [i32::MAX, i32::MAX - 2, i32::MIN, i32::MIN + 2];

        let workers = calibrate_parallel(&mut samples, &[1, 10, -1, -10], 4)
            .expect("境界を越える値も処理できる");

        assert_eq!(workers, 4);
        assert_eq!(samples, [i32::MAX, i32::MAX, i32::MIN, i32::MIN]);
    }

    #[test]
    fn 上下限ちょうどになる加算は値を保つ() {
        let mut samples = [i32::MAX - 5, i32::MIN + 5, i32::MAX, i32::MIN];

        let workers = calibrate_parallel(&mut samples, &[5, -5, 0, 0], 3)
            .expect("境界値ちょうどを処理できる");

        assert_eq!(workers, 3);
        assert_eq!(samples, [i32::MAX, i32::MIN, i32::MAX, i32::MIN]);
    }

    #[test]
    fn 大きく不均等な入力もすべて一度ずつ処理する() {
        let mut samples = (0..10_003).collect::<Vec<i32>>();
        let corrections = (0..10_003)
            .map(|index| if index % 2 == 0 { 7 } else { -3 })
            .collect::<Vec<_>>();
        let expected = samples
            .iter()
            .zip(&corrections)
            .map(|(sample, correction)| sample.saturating_add(*correction))
            .collect::<Vec<_>>();

        let workers =
            calibrate_parallel(&mut samples, &corrections, 16).expect("大きな入力を処理できる");

        assert_eq!(workers, 16);
        assert_eq!(samples, expected);
    }
}
