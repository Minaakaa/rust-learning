#![cfg_attr(not(test), allow(dead_code))]

//! # 解答 01: async task でセンサー値を校正する

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
enum CalibrationError {
    OutOfRange { sample: SensorSample, offset: i16 },
}

async fn calibrate_sample(
    sample: SensorSample,
    offset: i16,
) -> Result<CalibratedSample, CalibrationError> {
    tokio::task::yield_now().await;

    let corrected = i32::from(sample.millivolts) + i32::from(offset);
    let Ok(millivolts) = u16::try_from(corrected) else {
        return Err(CalibrationError::OutOfRange { sample, offset });
    };

    Ok(CalibratedSample {
        robot_id: sample.robot_id,
        millivolts,
    })
}

fn spawn_calibrations(
    jobs: Vec<(SensorSample, i16)>,
) -> Vec<JoinHandle<Result<CalibratedSample, CalibrationError>>> {
    jobs.into_iter()
        .map(|(sample, offset)| tokio::spawn(calibrate_sample(sample, offset)))
        .collect()
}

async fn calibrate_fleet(
    jobs: Vec<(SensorSample, i16)>,
) -> Vec<Result<CalibratedSample, CalibrationError>> {
    let handles = spawn_calibrations(jobs);

    let mut reports = Vec::with_capacity(handles.len());
    for handle in handles {
        reports.push(handle.await.expect("校正taskはpanicしない"));
    }
    reports
}

fn main() {}

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
