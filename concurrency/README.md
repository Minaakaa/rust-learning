# Chapter 9 — スレッド間で仕事と状態を安全に共有する

このコースでは、配送ロボットの点検や集計を題材に、複数の OS スレッドで処理を進める方法を学びます
所有権をスレッドへ移す基本から始め、スコープ付きスレッドの借用、channel によるメッセージ転送、`Arc<Mutex<T>>` の共有状態へ進み、最後に固定数のワーカーで入力順を保つ並行処理を完成させます

## 学習目標

| 問題 | 主題 | できるようになること |
| --- | --- | --- |
| 01 | `thread::spawn` と `JoinHandle` | 所有値を `move` でスレッドへ渡し、ハンドルから監査結果を回収する |
| 02 | `thread::scope` | `'static` ではないデータを安全に借用し、分割した slice を並行更新する |
| 03 | `mpsc` channel | 複数 producer から値の所有権を送り、sender の破棄で受信終了を表す |
| 04 | `Arc<Mutex<T>>` | スレッド間で状態を共有し、lock の範囲と poison を明示的に扱う |
| 05 | `Send` と `Sync` | 固定数のワーカーへ job を分配し、非決定的な完了結果を入力順へ戻す |

## この章の考え方

- `thread::spawn` の closure は実行元より長く生存できるため、借用ではなく `move` で所有値を渡すことが多い
- `JoinHandle::join` はスレッドの終了を待ち、戻り値または panic の payload を受け取る
- 起動した全 handle を join すると、エラー経路でも処理を途中で切り離さずに済む
- `thread::scope` 内のスレッドは scope 終了前に join されるため、scope より長く生存するローカル値を借用できる
- `mpsc` は複数の `Sender` と1つの `Receiver` を持ち、送信時に値の所有権を移す
- 全 sender が破棄されると channel は切断され、receiver は残りの message を読んだ後に終了を検出できる
- `Rc<T>` と `RefCell<T>` は単一スレッド向けであり、スレッド間の共有には `Arc<T>` と同期機構を使う
- `Arc<T>` は共有所有を提供するが、内部の値を自動的に可変にはしない
- `Mutex<T>` は一度に1スレッドだけへ `MutexGuard` を渡し、guard の drop で lock を解放する
- lock を保持したスレッドが panic すると mutex は poison されるため、回復可能な API では `PoisonError` を扱う
- `Send` は値の所有権を別スレッドへ移せること、`Sync` は共有参照を複数スレッドから安全に使えることを表す
- `Send` と `Sync` は通常コンパイラが自動実装し、この章では独自の `unsafe impl` を書かない
- スレッドの実行順と channel の到着順は前提にせず、入力位置の付与、sort、可換な集計でテスト結果を決定的にする
- `sleep` の長さや特定のスケジュールに依存するテストは、速い環境や高負荷環境で不安定になるため避ける

この章では同期的なスレッド処理に集中し、async/await、atomic 型による同期設計、`RwLock`、`Condvar`、`Barrier`、lock-free 構造、`unsafe`、外部 crate は扱いません

## 進め方

プロジェクトのルートで、最初の問題を実行します

```console
cargo test --example concurrency_01_spawn_audits
```

未完成の問題は `not yet implemented` または未実装部分に対応するテスト失敗になります
問題ファイルの仕様、ヒント、テストを読み、`todo!()` と必要な型境界や処理を変更してください

```console
cargo test --example concurrency_02_scoped_calibration
cargo test --example concurrency_03_telemetry_channels
cargo test --example concurrency_04_shared_ledger
cargo test --example concurrency_05_fixed_workers
```

問題は Cargo の example target として分離されています
`cargo test` だけでは問題を実行しないため、必ず `--example concurrency_...` を指定してください

## 解答例

同名の解答例が `concurrency/solutions` にあります
たとえば問題 05 の解答だけを検証するには、次を実行します

```console
cargo test --test solution_concurrency_05_fixed_workers
```

全コースの解答例は、次のコマンドでまとめて検証できます

```console
cargo test --tests
```

`cargo test --all-targets` は未完成の問題も実行するため、問題を解き終えるまでは失敗します

## 公式リファレンス

- [恐れるな 並行性 — The Rust Programming Language](https://doc.rust-lang.org/book/ch16-00-concurrency.html)
- [スレッドでコードを同時実行する](https://doc.rust-lang.org/book/ch16-01-threads.html)
- [メッセージ受け渡しでデータを転送する](https://doc.rust-lang.org/book/ch16-02-message-passing.html)
- [共有状態の並行処理](https://doc.rust-lang.org/book/ch16-03-shared-state.html)
- [`Sync` と `Send` による拡張可能な並行処理](https://doc.rust-lang.org/book/ch16-04-extensible-concurrency-sync-and-send.html)
- [`thread::scope` — Rust 標準ライブラリ](https://doc.rust-lang.org/std/thread/fn.scope.html)
- [`mpsc` — Rust 標準ライブラリ](https://doc.rust-lang.org/std/sync/mpsc/)
- [`Arc` — Rust 標準ライブラリ](https://doc.rust-lang.org/std/sync/struct.Arc.html)
- [`Mutex` — Rust 標準ライブラリ](https://doc.rust-lang.org/std/sync/struct.Mutex.html)
