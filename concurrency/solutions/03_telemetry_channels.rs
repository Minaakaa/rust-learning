#![cfg_attr(not(test), allow(dead_code))]

//! # 問題 03: チャネルでテレメトリを集約する
//!
//! 複数の配送ロボットが作ったテレメトリのバッチを、producer thread ごとに
//! `mpsc` チャネルへ送信し、管制側で1つの一覧へ集約します
//! スレッドの実行順序には依存せず、入力バッチ内の位置を使って決定的な順序へ
//! 正規化してください
//!
//! 仕様:
//! - 入力の各バッチにつき1つの producer thread を起動する
//! - 各スレッドは `Sender` を複製し、`Telemetry` を複製せず `Envelope` へ移す
//! - `Envelope` は元の `batch_index` とバッチ内の `position` を保持する
//! - すべての producer thread を join してから、元の `Sender` を明示的に drop する
//! - 受信には `try_recv` を使い、ブロックせず切断まで読み取る
//! - 受信結果は `(batch_index, position)` の昇順へ並べ替える
//! - 送信側がまだ残っている場合は、受信済みの値を失わず
//!   `GatherError::ChannelStillOpen` を返す
//!
//! 制約:
//! - `Telemetry` や内部の `String` を複製しない
//! - `thread::sleep`、外部 crate、`unsafe` を使わない
//! - スレッドの完了順やチャネルへの到着順を前提にしない
//!
//! ヒント:
//! - バッチごとに `sender.clone()` し、`move` closure へバッチと一緒に渡す
//! - `try_recv` は値、空だが接続中、切断済みの3状態を区別する
//! - 全 handle を join した後に root sender を drop すると、受信側で切断を確認できる
//! - `sort_by_key` で受信順を `(batch_index, position)` 順へ正規化できる

use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::thread;

#[derive(Debug, PartialEq, Eq)]
struct Telemetry {
    robot_id: String,
    payload: String,
    value: i64,
}

impl Telemetry {
    fn new(robot_id: String, payload: String, value: i64) -> Self {
        Self {
            robot_id,
            payload,
            value,
        }
    }

    fn robot_id(&self) -> &str {
        &self.robot_id
    }

    fn payload(&self) -> &str {
        &self.payload
    }

    const fn value(&self) -> i64 {
        self.value
    }
}

#[derive(Debug, PartialEq, Eq)]
struct Envelope {
    batch_index: usize,
    position: usize,
    telemetry: Telemetry,
}

impl Envelope {
    const fn batch_index(&self) -> usize {
        self.batch_index
    }

    const fn position(&self) -> usize {
        self.position
    }

    const fn telemetry(&self) -> &Telemetry {
        &self.telemetry
    }
}

#[derive(Debug, PartialEq, Eq)]
enum GatherError {
    ChannelStillOpen { received: Vec<Envelope> },
}

fn normalize(envelopes: &mut [Envelope]) {
    envelopes.sort_by_key(|envelope| (envelope.batch_index, envelope.position));
}

fn collect_disconnected(receiver: Receiver<Envelope>) -> Result<Vec<Envelope>, GatherError> {
    let mut received = Vec::new();

    loop {
        match receiver.try_recv() {
            Ok(envelope) => received.push(envelope),
            Err(TryRecvError::Disconnected) => {
                normalize(&mut received);
                return Ok(received);
            }
            Err(TryRecvError::Empty) => {
                normalize(&mut received);
                return Err(GatherError::ChannelStillOpen { received });
            }
        }
    }
}

fn gather_telemetry(batches: Vec<Vec<Telemetry>>) -> Vec<Envelope> {
    let (sender, receiver) = mpsc::channel();
    let mut handles = Vec::with_capacity(batches.len());

    for (batch_index, batch) in batches.into_iter().enumerate() {
        let producer = sender.clone();
        handles.push(thread::spawn(move || {
            for (position, telemetry) in batch.into_iter().enumerate() {
                producer
                    .send(Envelope {
                        batch_index,
                        position,
                        telemetry,
                    })
                    .expect("管制側の Receiver は producer の完了まで生存する");
            }
        }));
    }

    for handle in handles {
        handle.join().expect("producer thread が正常に完了する");
    }
    drop(sender);

    collect_disconnected(receiver).expect("すべての Sender を破棄した後に受信する")
}

fn main() {
    let envelopes = gather_telemetry(vec![
        vec![
            Telemetry::new(
                String::from("配送ロボット-903"),
                String::from("battery"),
                82,
            ),
            Telemetry::new(
                String::from("配送ロボット-903"),
                String::from("cargo-temperature"),
                6,
            ),
        ],
        vec![Telemetry::new(
            String::from("配送ロボット-904"),
            String::from("battery"),
            74,
        )],
    ]);

    for envelope in envelopes {
        println!(
            "batch={} position={} {} {}={}",
            envelope.batch_index(),
            envelope.position(),
            envelope.telemetry().robot_id(),
            envelope.telemetry().payload(),
            envelope.telemetry().value()
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn telemetry(robot_id: &str, payload: &str, value: i64) -> Telemetry {
        Telemetry::new(robot_id.to_owned(), payload.to_owned(), value)
    }

    #[test]
    fn 切断済みchannelは全packetを読み切って位置順に返す() {
        let (sender, receiver) = mpsc::channel();
        for (batch_index, position, payload) in [(2, 1, "最後"), (0, 0, "先頭"), (2, 0, "途中")]
        {
            sender
                .send(Envelope {
                    batch_index,
                    position,
                    telemetry: telemetry("R-closed", payload, position as i64),
                })
                .expect("Receiver が生存している");
        }
        drop(sender);

        let received = collect_disconnected(receiver).expect("切断まで全件を受信できる");
        let actual = received
            .iter()
            .map(|envelope| {
                (
                    envelope.batch_index(),
                    envelope.position(),
                    envelope.telemetry().payload(),
                )
            })
            .collect::<Vec<_>>();

        assert_eq!(actual, [(0, 0, "先頭"), (2, 0, "途中"), (2, 1, "最後")]);
    }

    #[test]
    fn バッチがなければ空の一覧を返す() {
        assert!(gather_telemetry(Vec::new()).is_empty());
    }

    #[test]
    fn 単一バッチの位置と値を保持する() {
        let gathered = gather_telemetry(vec![vec![
            telemetry("R-01", "battery", 90),
            telemetry("R-01", "temperature", -4),
            telemetry("R-01", "distance", 1_250),
        ]]);

        let keys = gathered
            .iter()
            .map(|envelope| (envelope.batch_index(), envelope.position()))
            .collect::<Vec<_>>();
        let values = gathered
            .iter()
            .map(|envelope| envelope.telemetry().value())
            .collect::<Vec<_>>();

        assert_eq!(keys, [(0, 0), (0, 1), (0, 2)]);
        assert_eq!(values, [90, -4, 1_250]);
    }

    #[test]
    fn 複数producerの結果を入力位置順へ正規化する() {
        let gathered = gather_telemetry(vec![
            vec![telemetry("R-A", "a0", 0), telemetry("R-A", "a1", 1)],
            vec![telemetry("R-B", "b0", 10)],
            vec![
                telemetry("R-C", "c0", 20),
                telemetry("R-C", "c1", 21),
                telemetry("R-C", "c2", 22),
            ],
        ]);

        let actual = gathered
            .iter()
            .map(|envelope| {
                (
                    envelope.batch_index(),
                    envelope.position(),
                    envelope.telemetry().payload(),
                )
            })
            .collect::<Vec<_>>();

        assert_eq!(
            actual,
            [
                (0, 0, "a0"),
                (0, 1, "a1"),
                (1, 0, "b0"),
                (2, 0, "c0"),
                (2, 1, "c1"),
                (2, 2, "c2"),
            ]
        );
    }

    #[test]
    fn 空バッチがあってもbatch_indexを詰めない() {
        let gathered = gather_telemetry(vec![
            Vec::new(),
            vec![telemetry("R-1", "ready", 1)],
            Vec::new(),
            vec![telemetry("R-3", "ready", 3)],
        ]);

        let keys = gathered
            .iter()
            .map(|envelope| (envelope.batch_index(), envelope.position()))
            .collect::<Vec<_>>();

        assert_eq!(keys, [(1, 0), (3, 0)]);
    }

    #[test]
    fn 同じ内容のpacketもそれぞれ保持する() {
        let gathered = gather_telemetry(vec![
            Vec::new(),
            vec![
                telemetry("R-dup", "signal", 7),
                telemetry("R-dup", "signal", 7),
            ],
        ]);

        assert_eq!(gathered.len(), 2);
        assert_eq!(gathered[0].position(), 0);
        assert_eq!(gathered[1].position(), 1);
        assert_eq!(gathered[0].telemetry(), gathered[1].telemetry());
    }

    #[test]
    fn stringのallocationを複製せずenvelopeへ移す() {
        let packet = telemetry("配送ロボット-移動", "荷室温度センサー", -8);
        let robot_id_pointer = packet.robot_id.as_ptr();
        let payload_pointer = packet.payload.as_ptr();

        let gathered = gather_telemetry(vec![vec![packet]]);
        let moved = gathered[0].telemetry();

        assert_eq!(moved.robot_id().as_ptr(), robot_id_pointer);
        assert_eq!(moved.payload().as_ptr(), payload_pointer);
    }

    #[test]
    fn 日本語と絵文字を変更せず集約する() {
        let gathered = gather_telemetry(vec![vec![telemetry("柏ロボット🤖", "荷室温度🌡️", -12)]]);

        assert_eq!(gathered[0].telemetry().robot_id(), "柏ロボット🤖");
        assert_eq!(gathered[0].telemetry().payload(), "荷室温度🌡️");
        assert_eq!(gathered[0].telemetry().value(), -12);
    }

    #[test]
    fn 多数のproducerとpacketを漏れなく集約する() {
        let batches = (0..12)
            .map(|batch_index| {
                (0..40)
                    .map(|position| {
                        telemetry(
                            &format!("R-{batch_index:02}"),
                            &format!("sensor-{position:02}"),
                            (batch_index * 100 + position) as i64,
                        )
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();

        let gathered = gather_telemetry(batches);

        assert_eq!(gathered.len(), 480);
        assert_eq!((gathered[0].batch_index(), gathered[0].position()), (0, 0));
        assert_eq!(
            (
                gathered[479].batch_index(),
                gathered[479].position(),
                gathered[479].telemetry().value(),
            ),
            (11, 39, 1_139)
        );
    }

    #[test]
    fn senderが残っていれば受信済みpacketを含むエラーを返す() {
        let (sender, receiver) = mpsc::channel();
        sender
            .send(Envelope {
                batch_index: 4,
                position: 2,
                telemetry: telemetry("R-open", "connected", 1),
            })
            .expect("Receiver が生存している");

        let error = collect_disconnected(receiver).expect_err("Sender がまだ生存している");

        let GatherError::ChannelStillOpen { received } = error;
        assert_eq!(received.len(), 1);
        assert_eq!(received[0].batch_index(), 4);
        assert_eq!(received[0].position(), 2);
        assert_eq!(received[0].telemetry().payload(), "connected");
        drop(sender);
    }
}
