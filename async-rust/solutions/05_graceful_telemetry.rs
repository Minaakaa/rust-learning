//! # 解答 05: 遠隔測定サービスを graceful shutdown する

use std::fmt;
use std::time::Duration;
use tokio::sync::{mpsc, oneshot};
use tokio::task::JoinSet;
use tokio::time::timeout;

const MAX_IN_FLIGHT: usize = 2;

#[derive(Debug, PartialEq, Eq)]
struct TelemetryJob {
    sequence: u64,
    robot_id: String,
    value: i64,
    processing_time: Duration,
}

impl TelemetryJob {
    fn new(sequence: u64, robot_id: String, value: i64, processing_time: Duration) -> Self {
        Self {
            sequence,
            robot_id,
            value,
            processing_time,
        }
    }

    const fn sequence(&self) -> u64 {
        self.sequence
    }

    fn robot_id(&self) -> &str {
        &self.robot_id
    }
}

#[derive(Debug, PartialEq, Eq)]
struct TelemetryReport {
    sequence: u64,
    robot_id: String,
    adjusted_value: i64,
}

impl TelemetryReport {
    const fn sequence(&self) -> u64 {
        self.sequence
    }

    fn robot_id(&self) -> &str {
        &self.robot_id
    }

    const fn adjusted_value(&self) -> i64 {
        self.adjusted_value
    }
}

#[derive(Debug, PartialEq, Eq)]
enum SubmitError {
    TimedOut(TelemetryJob),
    Closed(TelemetryJob),
}

#[derive(Debug, PartialEq, Eq)]
enum ProcessingError {
    AdjustedValueOverflow { sequence: u64, value: i64 },
}

impl fmt::Display for ProcessingError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AdjustedValueOverflow { sequence, value } => write!(
                formatter,
                "sequence {sequence} の値 {value} を2倍にできません"
            ),
        }
    }
}

impl std::error::Error for ProcessingError {}

#[derive(Debug, PartialEq, Eq)]
struct ServiceError {
    failed_tasks: usize,
}

impl ServiceError {
    const fn failed_tasks(&self) -> usize {
        self.failed_tasks
    }
}

#[derive(Debug, PartialEq, Eq)]
struct ServiceSummary {
    reports: Vec<TelemetryReport>,
    shutdown_requested: bool,
}

impl ServiceSummary {
    fn reports(&self) -> &[TelemetryReport] {
        &self.reports
    }

    const fn shutdown_requested(&self) -> bool {
        self.shutdown_requested
    }
}

async fn submit_with_timeout(
    sender: &mpsc::Sender<TelemetryJob>,
    job: TelemetryJob,
    wait: Duration,
) -> Result<(), SubmitError> {
    match timeout(wait, sender.reserve()).await {
        Ok(Ok(permit)) => {
            permit.send(job);
            Ok(())
        }
        Ok(Err(_)) => Err(SubmitError::Closed(job)),
        Err(_) => Err(SubmitError::TimedOut(job)),
    }
}

async fn process_job(job: TelemetryJob) -> Result<TelemetryReport, ProcessingError> {
    let TelemetryJob {
        sequence,
        robot_id,
        value,
        processing_time,
    } = job;

    tokio::time::sleep(processing_time).await;

    let adjusted_value = value
        .checked_mul(2)
        .ok_or(ProcessingError::AdjustedValueOverflow { sequence, value })?;

    Ok(TelemetryReport {
        sequence,
        robot_id,
        adjusted_value,
    })
}

async fn run_telemetry_service(
    mut receiver: mpsc::Receiver<TelemetryJob>,
    mut shutdown: oneshot::Receiver<()>,
) -> Result<ServiceSummary, ServiceError> {
    let mut tasks = JoinSet::new();
    let mut reports = Vec::new();
    let mut failed_tasks = 0;
    let mut receiving = true;
    let mut shutdown_open = true;
    let mut shutdown_requested = false;

    while receiving || !tasks.is_empty() {
        tokio::select! {
            biased;

            shutdown_result = &mut shutdown, if shutdown_open => {
                shutdown_open = false;
                if shutdown_result.is_ok() {
                    shutdown_requested = true;
                    receiver.close();
                }
            }
            joined = tasks.join_next(), if !tasks.is_empty() => {
                if let Some(joined) = joined {
                    match joined {
                        Ok(Ok(report)) => reports.push(report),
                        Ok(Err(_)) | Err(_) => failed_tasks += 1,
                    }
                }
            }
            received = receiver.recv(), if receiving && tasks.len() < MAX_IN_FLIGHT => {
                match received {
                    Some(job) => {
                        tasks.spawn(process_job(job));
                    }
                    None => receiving = false,
                }
            }
        }
    }

    reports.sort_by(|left, right| {
        left.sequence
            .cmp(&right.sequence)
            .then_with(|| left.robot_id.cmp(&right.robot_id))
            .then_with(|| left.adjusted_value.cmp(&right.adjusted_value))
    });

    if failed_tasks == 0 {
        Ok(ServiceSummary {
            reports,
            shutdown_requested,
        })
    } else {
        Err(ServiceError { failed_tasks })
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let (sender, receiver) = mpsc::channel(2);
    let (_shutdown_sender, shutdown_receiver) = oneshot::channel();

    submit_with_timeout(
        &sender,
        TelemetryJob::new(
            1,
            String::from("配送ロボット-1001"),
            24,
            Duration::from_millis(10),
        ),
        Duration::from_secs(1),
    )
    .await
    .expect("遠隔測定を受け付けられる");
    drop(sender);

    let summary = run_telemetry_service(receiver, shutdown_receiver)
        .await
        .expect("全taskを完了できる");
    println!("処理完了: {}件", summary.reports().len());
}

#[cfg(test)]
mod tests {
    use super::*;

    fn job(sequence: u64, robot_id: &str, value: i64, seconds: u64) -> TelemetryJob {
        TelemetryJob::new(
            sequence,
            robot_id.to_owned(),
            value,
            Duration::from_secs(seconds),
        )
    }

    fn sequences(summary: &ServiceSummary) -> Vec<u64> {
        summary
            .reports()
            .iter()
            .map(TelemetryReport::sequence)
            .collect()
    }

    async fn wait_for_capacity(sender: &mpsc::Sender<TelemetryJob>, expected: usize) {
        for _ in 0..16 {
            if sender.capacity() == expected {
                return;
            }
            tokio::task::yield_now().await;
        }
        assert_eq!(sender.capacity(), expected);
    }

    #[tokio::test(flavor = "current_thread", start_paused = true)]
    async fn 満杯のchannelではtimeout後にjobを返す() {
        let (sender, _receiver) = mpsc::channel(1);
        sender
            .send(job(1, "R-first", 1, 0))
            .await
            .expect("最初の job を送信できる");
        let pending = job(2, "R-returned", 2, 0);
        let id_pointer = pending.robot_id.as_ptr();
        let worker_sender = sender.clone();
        let handle = tokio::spawn(async move {
            submit_with_timeout(&worker_sender, pending, Duration::from_secs(2)).await
        });

        tokio::task::yield_now().await;
        tokio::time::advance(Duration::from_secs(2)).await;

        let error = handle
            .await
            .expect("送信 task は panic しない")
            .expect_err("送信枠を予約できず timeout する");
        let SubmitError::TimedOut(returned) = error else {
            panic!("想定外のエラー: {error:?}");
        };
        assert_eq!(returned.sequence(), 2);
        assert_eq!(returned.robot_id(), "R-returned");
        assert_eq!(returned.robot_id.as_ptr(), id_pointer);
    }

    #[tokio::test(flavor = "current_thread", start_paused = true)]
    async fn 空きができると待機中のjobを受け付ける() {
        let (sender, mut receiver) = mpsc::channel(1);
        sender
            .send(job(1, "R-first", 1, 0))
            .await
            .expect("最初の job を送信できる");
        let worker_sender = sender.clone();
        let handle = tokio::spawn(async move {
            submit_with_timeout(
                &worker_sender,
                job(2, "R-second", 2, 0),
                Duration::from_secs(5),
            )
            .await
        });

        tokio::task::yield_now().await;
        assert!(!handle.is_finished());
        assert_eq!(receiver.recv().await.unwrap().sequence(), 1);
        handle
            .await
            .expect("送信 task は panic しない")
            .expect("空きができれば送信できる");
        assert_eq!(receiver.recv().await.unwrap().sequence(), 2);
    }

    #[tokio::test(flavor = "current_thread", start_paused = true)]
    async fn 通常終了でも全taskを待ちreportをsequence順に返す() {
        let (sender, receiver) = mpsc::channel(3);
        let (_shutdown_sender, shutdown_receiver) = oneshot::channel();
        sender.send(job(30, "R-30", 30, 3)).await.unwrap();
        sender.send(job(10, "R-10", 10, 1)).await.unwrap();
        sender.send(job(20, "R-20", 20, 2)).await.unwrap();
        drop(sender);

        let summary = run_telemetry_service(receiver, shutdown_receiver)
            .await
            .expect("全taskを完了できる");

        assert!(!summary.shutdown_requested());
        assert_eq!(sequences(&summary), [10, 20, 30]);
        assert_eq!(summary.reports()[0].robot_id(), "R-10");
        assert_eq!(summary.reports()[0].adjusted_value(), 20);
        assert_eq!(summary.reports()[1].adjusted_value(), 40);
        assert_eq!(summary.reports()[2].adjusted_value(), 60);
    }

    #[tokio::test(flavor = "current_thread", start_paused = true)]
    async fn shutdownは新規受付を閉じbuffer済みjobをdrainする() {
        let (sender, receiver) = mpsc::channel(2);
        sender.send(job(2, "R-accepted-2", 20, 2)).await.unwrap();
        sender.send(job(1, "R-accepted-1", 10, 1)).await.unwrap();

        let rejected = job(3, "R-rejected", 30, 0);
        let rejected_pointer = rejected.robot_id.as_ptr();
        let waiting_sender = sender.clone();
        let waiting = tokio::spawn(async move {
            submit_with_timeout(&waiting_sender, rejected, Duration::from_secs(60)).await
        });
        tokio::task::yield_now().await;
        assert!(!waiting.is_finished());

        let (shutdown_sender, shutdown_receiver) = oneshot::channel();
        shutdown_sender.send(()).expect("shutdown を通知できる");

        let summary = run_telemetry_service(receiver, shutdown_receiver)
            .await
            .expect("受付済みtaskを完了できる");
        let error = waiting
            .await
            .expect("送信 task は panic しない")
            .expect_err("shutdown 後の予約は失敗する");
        let SubmitError::Closed(returned) = error else {
            panic!("想定外のエラー: {error:?}");
        };

        assert!(sender.is_closed());
        assert!(summary.shutdown_requested());
        assert_eq!(sequences(&summary), [1, 2]);
        assert_eq!(returned.sequence(), 3);
        assert_eq!(returned.robot_id.as_ptr(), rejected_pointer);
    }

    #[tokio::test(flavor = "current_thread", start_paused = true)]
    async fn shutdown後も起動済みtaskの完了まで待つ() {
        let (sender, receiver) = mpsc::channel(1);
        sender.send(job(9, "R-slow", 9, 60)).await.unwrap();
        let (shutdown_sender, shutdown_receiver) = oneshot::channel();
        let service = tokio::spawn(run_telemetry_service(receiver, shutdown_receiver));

        for _ in 0..8 {
            if sender.capacity() == 1 {
                break;
            }
            tokio::task::yield_now().await;
        }
        assert_eq!(sender.capacity(), 1);
        shutdown_sender.send(()).expect("shutdown を通知できる");
        tokio::task::yield_now().await;
        assert!(!service.is_finished());

        tokio::time::advance(Duration::from_secs(59)).await;
        assert!(!service.is_finished());
        tokio::time::advance(Duration::from_secs(1)).await;

        let summary = service
            .await
            .expect("service task は panic しない")
            .expect("起動済みtaskを完了できる");
        assert!(summary.shutdown_requested());
        assert_eq!(sequences(&summary), [9]);
    }

    #[tokio::test(flavor = "current_thread", start_paused = true)]
    async fn 並行数上限は処理完了までproducerへbackpressureを伝える() {
        let (sender, receiver) = mpsc::channel(1);
        let (_shutdown_sender, shutdown_receiver) = oneshot::channel();

        sender.send(job(1, "R-1", 1, 10)).await.unwrap();
        let service = tokio::spawn(run_telemetry_service(receiver, shutdown_receiver));
        wait_for_capacity(&sender, 1).await;

        sender.send(job(2, "R-2", 2, 20)).await.unwrap();
        wait_for_capacity(&sender, 1).await;
        sender.send(job(3, "R-buffered", 3, 0)).await.unwrap();
        assert_eq!(sender.capacity(), 0);

        let waiting_sender = sender.clone();
        let waiting = tokio::spawn(async move {
            submit_with_timeout(
                &waiting_sender,
                job(4, "R-waiting", 4, 0),
                Duration::from_secs(30),
            )
            .await
        });
        tokio::task::yield_now().await;
        assert!(!waiting.is_finished());

        tokio::time::advance(Duration::from_secs(9)).await;
        tokio::task::yield_now().await;
        assert!(!waiting.is_finished());

        tokio::time::advance(Duration::from_secs(1)).await;
        waiting
            .await
            .expect("送信 task は panic しない")
            .expect("処理 task の完了後は送信できる");

        drop(sender);
        tokio::time::advance(Duration::from_secs(10)).await;
        let summary = service
            .await
            .expect("service task は panic しない")
            .expect("すべての job を処理できる");
        assert_eq!(sequences(&summary), [1, 2, 3, 4]);
    }

    #[tokio::test(flavor = "current_thread", start_paused = true)]
    async fn overflowは処理失敗として数える() {
        let processing_error = process_job(job(99, "R-overflow-message", i64::MAX, 0))
            .await
            .expect_err("2倍にすると overflow する");
        assert_eq!(
            processing_error.to_string(),
            format!("sequence 99 の値 {} を2倍にできません", i64::MAX)
        );

        let (sender, receiver) = mpsc::channel(3);
        let (_shutdown_sender, shutdown_receiver) = oneshot::channel();
        sender
            .send(job(1, "R-overflow-max", i64::MAX, 0))
            .await
            .unwrap();
        sender
            .send(job(2, "R-overflow-min", i64::MIN, 0))
            .await
            .unwrap();
        sender.send(job(3, "R-success", 21, 0)).await.unwrap();
        drop(sender);

        let error = run_telemetry_service(receiver, shutdown_receiver)
            .await
            .expect_err("2件の処理失敗を返す");
        assert_eq!(error.failed_tasks(), 2);
    }

    #[tokio::test(flavor = "current_thread", start_paused = true)]
    async fn 処理失敗後も他の起動済みtaskを待つ() {
        let (sender, receiver) = mpsc::channel(2);
        let (_shutdown_sender, shutdown_receiver) = oneshot::channel();
        sender.send(job(1, "R-failure", i64::MAX, 1)).await.unwrap();
        sender.send(job(2, "R-slow", 2, 60)).await.unwrap();

        let service = tokio::spawn(run_telemetry_service(receiver, shutdown_receiver));
        wait_for_capacity(&sender, 2).await;
        drop(sender);

        tokio::time::advance(Duration::from_secs(1)).await;
        tokio::task::yield_now().await;
        assert!(!service.is_finished());
        tokio::time::advance(Duration::from_secs(58)).await;
        assert!(!service.is_finished());
        tokio::time::advance(Duration::from_secs(1)).await;

        let error = service
            .await
            .expect("service task は panic しない")
            .expect_err("処理失敗を返す");
        assert_eq!(error.failed_tasks(), 1);
    }
}
