//! 問題 02 の解答例

use std::thread;

#[derive(Debug, PartialEq, Eq)]
enum CalibrationError {
    ZeroWorkers,
    LengthMismatch { samples: usize, corrections: usize },
}

fn calibrate_chunk(samples: &mut [i32], corrections: &[i32]) {
    for (sample, correction) in samples.iter_mut().zip(corrections) {
        *sample = sample.saturating_add(*correction);
    }
}

fn calibrate_parallel(
    samples: &mut [i32],
    corrections: &[i32],
    worker_count: usize,
) -> Result<usize, CalibrationError> {
    if worker_count == 0 {
        return Err(CalibrationError::ZeroWorkers);
    }
    if samples.len() != corrections.len() {
        return Err(CalibrationError::LengthMismatch {
            samples: samples.len(),
            corrections: corrections.len(),
        });
    }
    if samples.is_empty() {
        return Ok(0);
    }

    let active_workers = worker_count.min(samples.len());
    let base_len = samples.len() / active_workers;
    let longer_workers = samples.len() % active_workers;
    let longer_len = base_len + 1;
    let longer_total = longer_workers * longer_len;

    let (longer_samples, regular_samples) = samples.split_at_mut(longer_total);
    let (longer_corrections, regular_corrections) = corrections.split_at(longer_total);

    thread::scope(|scope| {
        for (sample_chunk, correction_chunk) in longer_samples
            .chunks_mut(longer_len)
            .zip(longer_corrections.chunks(longer_len))
        {
            scope.spawn(move || calibrate_chunk(sample_chunk, correction_chunk));
        }

        for (sample_chunk, correction_chunk) in regular_samples
            .chunks_mut(base_len)
            .zip(regular_corrections.chunks(base_len))
        {
            scope.spawn(move || calibrate_chunk(sample_chunk, correction_chunk));
        }
    });

    Ok(active_workers)
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
