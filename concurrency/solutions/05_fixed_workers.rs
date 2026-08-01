//! 問題 05 の解答例

#[derive(Debug, PartialEq, Eq)]
enum WorkerError<T> {
    ZeroWorkers(Vec<T>),
    WorkersPanicked(Vec<usize>),
}

fn run_workers<T, U, F>(
    jobs: Vec<T>,
    worker_count: usize,
    inspect: F,
) -> Result<Vec<U>, WorkerError<T>>
where
    T: Send + 'static,
    U: Send + 'static,
    F: Fn(T) -> U + Send + Sync + 'static,
{
    if worker_count == 0 {
        return Err(WorkerError::ZeroWorkers(jobs));
    }
    if jobs.is_empty() {
        return Ok(Vec::new());
    }

    let active_workers = worker_count.min(jobs.len());
    let mut batches: Vec<Vec<(usize, T)>> = (0..active_workers).map(|_| Vec::new()).collect();
    for (index, job) in jobs.into_iter().enumerate() {
        batches[index % active_workers].push((index, job));
    }

    let inspect = std::sync::Arc::new(inspect);
    let (sender, receiver) = std::sync::mpsc::channel();
    let handles = batches
        .into_iter()
        .enumerate()
        .map(|(worker_index, batch)| {
            let worker_inspect = std::sync::Arc::clone(&inspect);
            let worker_sender = sender.clone();
            let handle = std::thread::spawn(move || {
                for (input_index, job) in batch {
                    let report = worker_inspect(job);
                    worker_sender
                        .send((input_index, report))
                        .expect("receiver は全ワーカーの終了まで保持される");
                }
            });
            (worker_index, handle)
        })
        .collect::<Vec<_>>();
    drop(sender);

    let mut panicked_workers = Vec::new();
    for (worker_index, handle) in handles {
        if handle.join().is_err() {
            panicked_workers.push(worker_index);
        }
    }

    let mut reports = receiver.into_iter().collect::<Vec<_>>();
    if !panicked_workers.is_empty() {
        return Err(WorkerError::WorkersPanicked(panicked_workers));
    }

    reports.sort_unstable_by_key(|(input_index, _)| *input_index);
    Ok(reports.into_iter().map(|(_, report)| report).collect())
}

fn main() {
    let jobs = vec![
        (String::from("配送ロボット-901"), 82_u8),
        (String::from("配送ロボット-902"), 67_u8),
        (String::from("配送ロボット-903"), 94_u8),
    ];
    let reports = run_workers(jobs, 2, |(id, battery)| {
        format!("{id}: バッテリー {battery}%")
    })
    .expect("点検を完了できる");

    println!("点検結果: {reports:?}");
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };

    #[test]
    fn worker数ゼロはjobを所有したまま返す() {
        let calls = Arc::new(AtomicUsize::new(0));
        let worker_calls = Arc::clone(&calls);
        let jobs = vec![String::from("R-01"), String::from("R-02")];
        let pointers = jobs.iter().map(|job| job.as_ptr()).collect::<Vec<_>>();

        let error = run_workers(jobs, 0, move |job: String| {
            worker_calls.fetch_add(1, Ordering::SeqCst);
            job.len()
        })
        .expect_err("ワーカー数ゼロは失敗する");

        let WorkerError::ZeroWorkers(returned) = error else {
            panic!("想定外のエラー: {error:?}");
        };
        assert_eq!(returned, ["R-01", "R-02"]);
        assert_eq!(returned[0].as_ptr(), pointers[0]);
        assert_eq!(returned[1].as_ptr(), pointers[1]);
        assert_eq!(calls.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn worker数ゼロは空入力より優先される() {
        let error = run_workers(Vec::<u8>::new(), 0, u16::from)
            .expect_err("空入力でもワーカー数ゼロは失敗する");

        assert_eq!(error, WorkerError::ZeroWorkers(Vec::new()));
    }

    #[test]
    fn 非ゼロworkerと空入力ではinspectを呼ばない() {
        let calls = Arc::new(AtomicUsize::new(0));
        let worker_calls = Arc::clone(&calls);

        let reports: Vec<usize> = run_workers(Vec::<String>::new(), 4, move |job| {
            worker_calls.fetch_add(1, Ordering::SeqCst);
            job.len()
        })
        .expect("空入力は成功する");

        assert!(reports.is_empty());
        assert_eq!(calls.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn worker一台でも入力順で全jobを処理する() {
        let reports =
            run_workers(vec![5_u32, 1, 9, 3], 1, |job| job * 10).expect("1台で処理できる");

        assert_eq!(reports, [50, 10, 90, 30]);
    }

    #[test]
    fn job数より多いworkerを指定しても各jobを一度だけ処理する() {
        let calls: Arc<[AtomicUsize; 4]> = Arc::new(std::array::from_fn(|_| AtomicUsize::new(0)));
        let worker_calls = Arc::clone(&calls);

        let reports = run_workers((0_usize..4).collect(), 20, move |job| {
            worker_calls[job].fetch_add(1, Ordering::SeqCst);
            job + 100
        })
        .expect("過剰なワーカー指定でも処理できる");

        assert_eq!(reports, [100, 101, 102, 103]);
        assert!(calls.iter().all(|count| count.load(Ordering::SeqCst) == 1));
    }

    #[test]
    fn 複数workerの結果をchannel到着順ではなく入力順へ戻す() {
        let reports = run_workers(vec![12_i32, -4, 7, 0, 21, -9, 3], 3, |job| {
            format!("点検-{job}")
        })
        .expect("複数ワーカーで処理できる");

        assert_eq!(
            reports,
            [
                "点検-12",
                "点検--4",
                "点検-7",
                "点検-0",
                "点検-21",
                "点検--9",
                "点検-3"
            ]
        );
    }

    #[test]
    fn 非cloneのstringを複製せず結果へ移す() {
        let jobs = vec![
            String::from("R-owned-1"),
            String::from("R-owned-2"),
            String::from("R-owned-3"),
        ];
        let pointers = jobs.iter().map(|job| job.as_ptr()).collect::<Vec<_>>();

        let reports = run_workers(jobs, 2, |job| job).expect("所有権を結果へ移せる");

        assert_eq!(reports, ["R-owned-1", "R-owned-2", "R-owned-3"]);
        assert_eq!(reports[0].as_ptr(), pointers[0]);
        assert_eq!(reports[1].as_ptr(), pointers[1]);
        assert_eq!(reports[2].as_ptr(), pointers[2]);
    }

    #[test]
    fn 複数workerのpanicを昇順で報告し他のworkerもjoinする() {
        let completed: Arc<[AtomicUsize; 8]> =
            Arc::new(std::array::from_fn(|_| AtomicUsize::new(0)));
        let worker_completed = Arc::clone(&completed);

        let error = run_workers((0_usize..8).collect(), 4, move |job| {
            if job == 1 || job == 6 {
                panic!("点検不能: {job}");
            }
            worker_completed[job].fetch_add(1, Ordering::SeqCst);
            job
        })
        .expect_err("worker 1 と worker 2 が panic する");

        assert_eq!(error, WorkerError::WorkersPanicked(vec![1, 2]));
        assert_eq!(completed[0].load(Ordering::SeqCst), 1);
        assert_eq!(completed[2].load(Ordering::SeqCst), 1);
        assert_eq!(completed[3].load(Ordering::SeqCst), 1);
        assert_eq!(completed[4].load(Ordering::SeqCst), 1);
        assert_eq!(completed[7].load(Ordering::SeqCst), 1);
        assert_eq!(completed[1].load(Ordering::SeqCst), 0);
        assert_eq!(completed[5].load(Ordering::SeqCst), 0);
        assert_eq!(completed[6].load(Ordering::SeqCst), 0);
    }

    #[test]
    fn utf8のjobを内容と順序を保って処理する() {
        let reports = run_workers(
            vec![
                String::from("本郷🤖"),
                String::from("駒場🚚"),
                String::from("柏📦"),
            ],
            2,
            |campus| format!("{campus}: 点検完了"),
        )
        .expect("UTF-8 の job を処理できる");

        assert_eq!(
            reports,
            ["本郷🤖: 点検完了", "駒場🚚: 点検完了", "柏📦: 点検完了"]
        );
    }

    #[test]
    fn 大量のjobも欠落や重複なく処理する() {
        const JOBS: usize = 512;
        let calls = Arc::new((0..JOBS).map(|_| AtomicUsize::new(0)).collect::<Vec<_>>());
        let worker_calls = Arc::clone(&calls);

        let reports = run_workers((0..JOBS).collect(), 7, move |job| {
            worker_calls[job].fetch_add(1, Ordering::SeqCst);
            job * job
        })
        .expect("大量の job を処理できる");

        assert_eq!(reports, (0..JOBS).map(|job| job * job).collect::<Vec<_>>());
        assert!(calls.iter().all(|count| count.load(Ordering::SeqCst) == 1));
    }
}
