# Chapter 10 — 非同期タスクでロボット遠隔測定を制御する

このコースでは、待ち時間のある処理を OS スレッドでブロックせず進める非同期 Rust を学びます
`async` / `.await` と task から始め、`Future::poll`、`Poll`、`Waker`、`Pin`、小さな executor へ進み、最後に bounded channel、timeout、キャンセル、`select!`、graceful shutdown を組み合わせた遠隔測定システムを完成させます

## 学習目標

| 問題 | 主題 | できるようになること |
| --- | --- | --- |
| 01 | `async` / `.await` と task | 所有値を `Send` な future へ移し、複数 task の結果を入力順で回収する |
| 02 | `Future` の手動実装 | `Pin<&mut Self>` から状態を調べ、`Poll::Pending` 時に `Waker` を登録する |
| 03 | executor と task | `Wake` で task を実行キューへ戻し、future を完了まで必要な時だけ poll する |
| 04 | backpressure とキャンセル | bounded channel の空きを timeout 付きで待ち、future の破棄で資源を解放する |
| 05 | `select!` と graceful shutdown | 同時実行数を制限し、受付済み task をすべて待って終了する |

## この章の考え方

- `async fn` を呼ぶと処理結果ではなく `Future` が作られ、poll または `.await` されるまで本体は進まない
- `.await` は future が `Pending` なら現在の task を中断し、executor が同じスレッドで別の task を進められるようにする
- `Future::poll` は結果があれば `Ready`、まだなら `Pending` を返し、進行可能になった時に最新の `Waker` を wake する責任を持つ
- `Waker::wake` は future をその場で実行する操作ではなく、executor へ再 poll 可能になったことを通知する操作
- `Pin<&mut Self>` は主に `!Unpin` な future を pin した場所から drop まで動かさないという保証を表し、compiler が作る async state machine にも使われる
- executor は ready queue から task を取り出して poll し、`Pending` な task を busy loop で繰り返し poll しない
- 同じ task への重複 wake は ready queue 上の1件へ集約し、通知の集中による queue の増幅を防ぐ
- Tokio の task は OS スレッドより軽量だが、`tokio::spawn` した future は runtime thread 間を移動できるよう `Send + 'static` を満たす必要がある
- `.await` をまたいで保持する値が `Send` でなければ future 全体も `Send` にならず、`tokio::spawn` へ渡せない
- bounded `mpsc` channel が満杯の時に送信側を待たせることで、流入量を処理能力へ合わせる backpressure を作れる
- channel から無制限に task へ移すと backpressure が途切れるため、処理中 task が上限に達した間は受信を止める
- timeout の期限到達などで future が drop されると処理はキャンセルされるため、途中状態と RAII cleanup を意識する
- `select!` の未選択 branch が所有する future も破棄され得る一方、pin した future を `&mut` で渡して次の反復へ残す設計もできる
- `JoinSet` では処理の `Result` と task 自体の `JoinError` を区別し、失敗後も残りを `join_next` してから終了する
- graceful shutdown は終了条件の検出、追加受付の停止、buffer の drain、受付済み task の完了待ちという段階で設計する
- 非同期テストでは実時間の `sleep` や task 実行順を前提にせず、channel、`Waker`、paused time で状態変化を同期する

この章では `unsafe` な `RawWaker`、実ネットワーク I/O、`Stream`、async trait、複数 runtime 間の連携は扱いません

## 進め方

プロジェクトのルートで、最初の問題を実行します

```console
cargo test --example async_rust_01_async_tasks
```

未完成の問題は `not yet implemented` または未実装部分に対応するテスト失敗になります
問題ファイルの仕様、ヒント、テストを読み、`todo!()` と必要な型境界や処理を変更してください

```console
cargo test --example async_rust_02_manual_future
cargo test --example async_rust_03_mini_executor
cargo test --example async_rust_04_backpressure_cancellation
cargo test --example async_rust_05_graceful_telemetry
```

問題は Cargo の example target として分離されています
`cargo test` だけでは問題を実行しないため、必ず `--example async_rust_...` を指定してください

## 解答例

同名の解答例が `async-rust/solutions` にあります
たとえば問題 05 の解答だけを検証するには、次を実行します

```console
cargo test --test solution_async_rust_05_graceful_telemetry
```

全コースの解答例は、次のコマンドでまとめて検証できます

```console
cargo test --tests
```

`cargo test --all-targets` は未完成の問題も実行するため、問題を解き終えるまでは失敗します

## 公式リファレンス

- [非同期プログラミングの基礎 — The Rust Programming Language](https://doc.rust-lang.org/book/ch17-00-async-await.html)
- [`async` keyword — Rust標準ライブラリ](https://doc.rust-lang.org/std/keyword.async.html)
- [`Future` — Rust標準ライブラリ](https://doc.rust-lang.org/std/future/trait.Future.html)
- [`Poll` — Rust標準ライブラリ](https://doc.rust-lang.org/std/task/enum.Poll.html)
- [`Waker` — Rust標準ライブラリ](https://doc.rust-lang.org/std/task/struct.Waker.html)
- [`Pin` — Rust標準ライブラリ](https://doc.rust-lang.org/std/pin/)
- [Spawning — Tokio Tutorial](https://tokio.rs/tokio/tutorial/spawning)
- [Channels and backpressure — Tokio Tutorial](https://tokio.rs/tokio/tutorial/channels)
- [`select!` and cancellation — Tokio Tutorial](https://tokio.rs/tokio/tutorial/select)
- [Graceful Shutdown — Tokio](https://tokio.rs/tokio/topics/shutdown)
- [`JoinSet` — Tokio](https://docs.rs/tokio/latest/tokio/task/struct.JoinSet.html)
- [Paused time による非同期テスト — Tokio](https://tokio.rs/tokio/topics/testing)
