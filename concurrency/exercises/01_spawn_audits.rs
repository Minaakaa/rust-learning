#![cfg_attr(not(test), allow(dead_code))]

//! # 問題 01: 所有する監査データをスレッドへ移す
//!
//! 配送ロボットのセンサー値を監査する処理を、呼び出し元とは別のスレッドで
//! 実行します
//! `InspectionJob` の所有権を worker thread へ移し、`JoinHandle` を通して
//! 監査結果を受け取る `launch_audit` を完成させてください
//!
//! 仕様:
//! - `launch_audit` は `thread::spawn` で新しいスレッドを起動する
//! - closure は `move` を使い、`InspectionJob` 全体の所有権を受け取る
//! - `millivolts` は `checked_add` で合計する
//! - 合計できた場合は `robot_id` と全 `label` を入力順のまま `AuditReport` へ移す
//! - 空の `readings` の合計は `0` とする
//! - overflow の場合は `AuditError::SignalOverflow` で元の job 全体を返す
//! - 成功時も失敗時も `String` や `Vec` の中身を複製しない
//!
//! ヒント:
//! - 所有値を変更せず返す可能性がある処理は、最初に共有借用で検証する
//! - `Iterator::try_fold` と `u32::checked_add` を組み合わせられる
//! - 合計成功後に job を分解すると、`robot_id` と各 `label` をムーブできる
//! - `JoinHandle::join` はスレッドの戻り値を `Result` で返す

use std::thread::{self, JoinHandle};

#[derive(Debug, PartialEq, Eq)]
struct SensorReading {
    label: String,
    millivolts: u32,
}

impl SensorReading {
    fn new(label: String, millivolts: u32) -> Self {
        Self { label, millivolts }
    }
}

#[derive(Debug, PartialEq, Eq)]
struct InspectionJob {
    robot_id: String,
    readings: Vec<SensorReading>,
}

impl InspectionJob {
    fn new(robot_id: String, readings: Vec<SensorReading>) -> Self {
        Self { robot_id, readings }
    }

    fn robot_id(&self) -> &str {
        &self.robot_id
    }

    fn readings(&self) -> &[SensorReading] {
        &self.readings
    }
}

#[derive(Debug, PartialEq, Eq)]
struct AuditReport {
    robot_id: String,
    labels: Vec<String>,
    total_millivolts: u32,
}

impl AuditReport {
    fn robot_id(&self) -> &str {
        &self.robot_id
    }

    fn labels(&self) -> &[String] {
        &self.labels
    }

    fn total_millivolts(&self) -> u32 {
        self.total_millivolts
    }
}

#[derive(Debug, PartialEq, Eq)]
#[allow(
    dead_code,
    reason = "launch_audit 完成前のスターターでは error を構築しないため"
)]
enum AuditError {
    SignalOverflow(InspectionJob),
}

fn launch_audit(job: InspectionJob) -> JoinHandle<Result<AuditReport, AuditError>> {
    thread::spawn(move || todo!("{job:?} を監査し、所有データを結果またはエラーへ移してください"))
}

fn main() {
    let job = InspectionJob::new(
        String::from("配送ロボット-901"),
        vec![
            SensorReading::new(String::from("左モーター"), 1_200),
            SensorReading::new(String::from("右モーター"), 1_180),
        ],
    );

    let report = launch_audit(job)
        .join()
        .expect("監査スレッドが完了する")
        .expect("信号値を合計できる");

    println!(
        "{}: {} 系統、合計 {} mV",
        report.robot_id(),
        report.labels().len(),
        report.total_millivolts()
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    fn reading(label: &str, millivolts: u32) -> SensorReading {
        SensorReading::new(String::from(label), millivolts)
    }

    fn completed(
        handle: JoinHandle<Result<AuditReport, AuditError>>,
    ) -> Result<AuditReport, AuditError> {
        handle.join().expect("監査スレッドは panic しない")
    }

    #[test]
    fn 単一のreadingを別スレッドで監査できる() {
        let job = InspectionJob::new(String::from("R-01"), vec![reading("主電源", 3_300)]);

        let report = completed(launch_audit(job)).expect("合計できる");

        assert_eq!(report.robot_id(), "R-01");
        assert_eq!(report.labels(), ["主電源"]);
        assert_eq!(report.total_millivolts(), 3_300);
    }

    #[test]
    fn readingの入力順を保ってchecked合計を返す() {
        let job = InspectionJob::new(
            String::from("R-order"),
            vec![
                reading("前方センサー", 900),
                reading("側面センサー", 1_100),
                reading("後方センサー", 700),
            ],
        );

        let report = completed(launch_audit(job)).expect("合計できる");

        assert_eq!(
            report.labels(),
            ["前方センサー", "側面センサー", "後方センサー"]
        );
        assert_eq!(report.total_millivolts(), 2_700);
    }

    #[test]
    fn readingが空なら合計は0になる() {
        let job = InspectionJob::new(String::from("R-empty"), Vec::new());

        let report = completed(launch_audit(job)).expect("空の監査も成功する");

        assert_eq!(report.robot_id(), "R-empty");
        assert!(report.labels().is_empty());
        assert_eq!(report.total_millivolts(), 0);
    }

    #[test]
    fn u32最大値ちょうどの合計は成功する() {
        let job = InspectionJob::new(
            String::from("R-boundary"),
            vec![reading("高電圧系", u32::MAX - 5), reading("補助系", 5)],
        );

        let report = completed(launch_audit(job)).expect("境界値は合計できる");

        assert_eq!(report.total_millivolts(), u32::MAX);
        assert_eq!(report.labels(), ["高電圧系", "補助系"]);
    }

    #[test]
    fn overflowではjob全体をerrorで返す() {
        let job = InspectionJob::new(
            String::from("R-overflow"),
            vec![
                reading("系統A", u32::MAX),
                reading("系統B", 1),
                reading("系統C", 42),
            ],
        );

        let error = completed(launch_audit(job)).expect_err("合計が overflow する");
        let AuditError::SignalOverflow(returned) = error;

        assert_eq!(returned.robot_id(), "R-overflow");
        assert_eq!(
            returned.readings(),
            [
                reading("系統A", u32::MAX),
                reading("系統B", 1),
                reading("系統C", 42),
            ]
        );
    }

    #[test]
    fn overflowでもjob内のallocationを複製しない() {
        let robot_id = String::from("R-returned");
        let robot_pointer = robot_id.as_ptr();
        let first_label = String::from("一次信号");
        let first_pointer = first_label.as_ptr();
        let second_label = String::from("二次信号");
        let second_pointer = second_label.as_ptr();
        let readings = vec![
            SensorReading::new(first_label, u32::MAX),
            SensorReading::new(second_label, 1),
        ];
        let readings_pointer = readings.as_ptr();
        let job = InspectionJob::new(robot_id, readings);

        let error = completed(launch_audit(job)).expect_err("合計が overflow する");
        let AuditError::SignalOverflow(returned) = error;

        assert_eq!(returned.robot_id.as_ptr(), robot_pointer);
        assert_eq!(returned.readings.as_ptr(), readings_pointer);
        assert_eq!(returned.readings[0].label.as_ptr(), first_pointer);
        assert_eq!(returned.readings[1].label.as_ptr(), second_pointer);
    }

    #[test]
    fn 成功時はrobot_idとlabelのallocationをreportへ移す() {
        let robot_id = String::from("R-moved");
        let robot_pointer = robot_id.as_ptr();
        let left_label = String::from("左駆動系");
        let left_pointer = left_label.as_ptr();
        let right_label = String::from("右駆動系");
        let right_pointer = right_label.as_ptr();
        let job = InspectionJob::new(
            robot_id,
            vec![
                SensorReading::new(left_label, 1_500),
                SensorReading::new(right_label, 1_500),
            ],
        );

        let report = completed(launch_audit(job)).expect("合計できる");

        assert_eq!(report.robot_id.as_ptr(), robot_pointer);
        assert_eq!(report.labels[0].as_ptr(), left_pointer);
        assert_eq!(report.labels[1].as_ptr(), right_pointer);
    }

    #[test]
    fn 複数のhandleをそれぞれjoinできる() {
        let first = launch_audit(InspectionJob::new(
            String::from("R-A"),
            vec![reading("A1", 10), reading("A2", 20)],
        ));
        let second = launch_audit(InspectionJob::new(
            String::from("R-B"),
            vec![reading("B1", 30), reading("B2", 40)],
        ));

        let second_report = completed(second).expect("2件目を完了できる");
        let first_report = completed(first).expect("1件目を完了できる");

        assert_eq!(first_report.total_millivolts(), 30);
        assert_eq!(second_report.total_millivolts(), 70);
    }

    #[test]
    fn 途中に0があっても合計と順序を保つ() {
        let job = InspectionJob::new(
            String::from("R-zero"),
            vec![
                reading("起動前", 0),
                reading("稼働中", 2_400),
                reading("停止後", 0),
            ],
        );

        let report = completed(launch_audit(job)).expect("合計できる");

        assert_eq!(report.total_millivolts(), 2_400);
        assert_eq!(report.labels(), ["起動前", "稼働中", "停止後"]);
    }

    #[test]
    fn utf8のrobot_idとlabelを変更しない() {
        let job = InspectionJob::new(
            String::from("配送ロボット🤖-九号"),
            vec![reading("左腕センサー🦾", 808), reading("カメラ📷", 1_212)],
        );

        let report = completed(launch_audit(job)).expect("UTF-8 を扱える");

        assert_eq!(report.robot_id(), "配送ロボット🤖-九号");
        assert_eq!(report.labels(), ["左腕センサー🦾", "カメラ📷"]);
        assert_eq!(report.total_millivolts(), 2_020);
    }
}
