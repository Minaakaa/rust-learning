#![cfg_attr(not(test), allow(dead_code))]

//! # 問題 01: async task でセンサー値を校正する
//!
//! 配送ロボットから届いたセンサー値を、Tokio task へ所有権ごと移して校正します
//! `async fn`、`.await`、`tokio::spawn`、`JoinHandle` を使い、task の完了順ではなく
//! 入力順で結果を返してください
//!
//! 仕様:
//! - `calibrate_sample` は最初に `tokio::task::yield_now().await` で実行権を譲る
//! - `millivolts + offset` を `i32` で計算し、`u16` の範囲なら校正済み値を返す
//! - 範囲外なら元の `SensorSample` と `offset` を `OutOfRange` で返す
//! - `spawn_calibrations` は job ごとに `tokio::spawn` で独立した task を起動する
//! - 各 task へ sample を複製せず `move` し、全 `JoinHandle` を入力順に await する
//! - `String` を含む所有値を `.await` の前後で保持し、spawn する future を `Send + 'static` にする
//!
//! ヒント:
//! - `async fn` の呼び出しは、すぐに結果を返すのではなく `Future` を作る
//! - `tokio::spawn` へ渡す future は runtime thread 間を移動できる必要がある
//! - `calibrate_sample(sample, offset)` は所有値だけを持つため、そのまま spawn できる
//! - handle を作った順に `await` すれば、task の実行順に依存せず結果を並べられる

use tokio::task::JoinHandle;

#[derive(Debug, PartialEq, Eq)]
struct SensorSample {
    robot_id: String,
    millivolts: u16,
}

impl SensorSample {
    fn new(robot_id: String, millivolts: u16) -> Self {
        Self {
            robot_id,
            millivolts,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
struct CalibratedSample {
    robot_id: String,
    millivolts: u16,
}

impl CalibratedSample {
    fn robot_id(&self) -> &str {
        &self.robot_id
    }

    const fn millivolts(&self) -> u16 {
        self.millivolts
    }
}

#[derive(Debug, PartialEq, Eq)]
#[allow(
    dead_code,
    reason = "calibrate_sample 完成前のスターターではエラーを構築しないため"
)]
enum CalibrationError {
    OutOfRange { sample: SensorSample, offset: i16 },
}

async fn calibrate_sample(
    sample: SensorSample,
    offset: i16,
) -> Result<CalibratedSample, CalibrationError> {
    todo!("{sample:?}へoffset {offset}を適用してください")
}

fn spawn_calibrations(
    jobs: Vec<(SensorSample, i16)>,
) -> Vec<JoinHandle<Result<CalibratedSample, CalibrationError>>> {
    todo!("{}件のjobを独立したtaskとして起動してください", jobs.len())
}

async fn calibrate_fleet(
    jobs: Vec<(SensorSample, i16)>,
) -> Vec<Result<CalibratedSample, CalibrationError>> {
    let handles = spawn_calibrations(jobs);
    todo!("{}個のJoinHandleを入力順にawaitしてください", handles.len())
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let reports = calibrate_fleet(vec![
        (
            SensorSample::new(String::from("配送ロボット-1001"), 3_280),
            20,
        ),
        (
            SensorSample::new(String::from("配送ロボット-1002"), 2_950),
            -10,
        ),
    ])
    .await;

    for report in reports {
        match report {
            Ok(sample) => println!("{}: {} mV", sample.robot_id(), sample.millivolts()),
            Err(error) => println!("校正失敗: {error:?}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{future::Future, task::Context};

    fn sample(robot_id: &str, millivolts: u16) -> SensorSample {
        SensorSample::new(robot_id.to_owned(), millivolts)
    }

    fn assert_send<T: Send>(_: T) {}

    #[test]
    fn sampleを保持するfutureはsendになる() {
        assert_send(calibrate_sample(sample("R-send", 1_200), 5));
    }

    #[tokio::test(flavor = "current_thread")]
    async fn 最初のpollでは実行権を譲る() {
        let mut future = Box::pin(calibrate_sample(sample("R-yield", 500), 5));
        let mut context = Context::from_waker(std::task::Waker::noop());

        assert!(Future::poll(future.as_mut(), &mut context).is_pending());
        assert!(Future::poll(future.as_mut(), &mut context).is_ready());
    }

    #[tokio::test(flavor = "current_thread")]
    async fn jobごとに別のtaskを起動する() {
        let handles = spawn_calibrations(vec![
            (sample("R-task-1", 100), 1),
            (sample("R-task-2", 200), 2),
            (sample("R-task-3", 300), 3),
        ]);

        assert_eq!(handles.len(), 3);
        assert_ne!(handles[0].id(), handles[1].id());
        assert_ne!(handles[1].id(), handles[2].id());

        for handle in handles {
            handle
                .await
                .expect("校正taskはpanicしない")
                .expect("校正値は範囲内に収まる");
        }
    }

    #[tokio::test]
    async fn 正のoffsetを加えて校正できる() {
        let calibrated = calibrate_sample(sample("R-plus", 1_200), 35)
            .await
            .expect("範囲内に収まる");

        assert_eq!(calibrated.robot_id(), "R-plus");
        assert_eq!(calibrated.millivolts(), 1_235);
    }

    #[tokio::test]
    async fn 負のoffsetを加えて校正できる() {
        let calibrated = calibrate_sample(sample("R-minus", 800), -125)
            .await
            .expect("範囲内に収まる");

        assert_eq!(calibrated.millivolts(), 675);
    }

    #[tokio::test]
    async fn 範囲外なら元のsampleを複製せず返す() {
        let input = sample("R-owned", 4);
        let robot_id_pointer = input.robot_id.as_ptr();

        let error = calibrate_sample(input, -5)
            .await
            .expect_err("0未満はu16にできない");

        let CalibrationError::OutOfRange { sample, offset } = error;
        assert_eq!(sample, SensorSample::new(String::from("R-owned"), 4));
        assert_eq!(sample.robot_id.as_ptr(), robot_id_pointer);
        assert_eq!(offset, -5);
    }

    #[tokio::test]
    async fn taskの結果を入力順で返す() {
        let reports = calibrate_fleet(vec![
            (sample("R-03", 300), 3),
            (sample("R-01", 100), 1),
            (sample("R-02", 200), 2),
        ])
        .await;

        let reports = reports
            .into_iter()
            .collect::<Result<Vec<_>, _>>()
            .expect("全jobを校正できる");
        assert_eq!(
            reports
                .iter()
                .map(CalibratedSample::robot_id)
                .collect::<Vec<_>>(),
            ["R-03", "R-01", "R-02"]
        );
        assert_eq!(
            reports
                .iter()
                .map(CalibratedSample::millivolts)
                .collect::<Vec<_>>(),
            [303, 101, 202]
        );
    }

    #[tokio::test]
    async fn 成功と失敗を同じ位置へ返す() {
        let reports = calibrate_fleet(vec![
            (sample("本郷🤖", 0), 0),
            (sample("駒場🚚", u16::MAX), 1),
            (sample("柏📦", 20), -20),
        ])
        .await;

        assert!(matches!(&reports[0], Ok(value) if value.robot_id() == "本郷🤖"));
        assert!(matches!(
            &reports[1],
            Err(CalibrationError::OutOfRange { sample, offset: 1 })
                if sample.robot_id == "駒場🚚"
        ));
        assert!(matches!(&reports[2], Ok(value) if value.millivolts() == 0));
    }
}
