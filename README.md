# Rust Learning

Rust のコードを実際に直し、テストを通しながら学ぶための教材です。短い構文クイズではなく、大学のキャンパスで働く配送ロボットを題材に、現実的なデータ設計とエラー処理を練習します。

次のコースがあります。

- Chapter 1 [`introduction`](introduction/README.md): enum、`match`、`Result`、エラー、イテレータ
- Chapter 2 [`ownership`](ownership/README.md): 所有権、ムーブ、`Copy` と `Clone`、借用、スライス
- Chapter 3 [`data-modeling`](data-modeling/README.md): 構造体、メソッド、newtype、不変条件、型安全な API
- Chapter 4 [`collections`](collections/README.md): `Vec`、`VecDeque`、`HashMap`、`HashSet`、UTF-8 文字列
- Chapter 5 [`generics-traits`](generics-traits/README.md): ジェネリック型、独自トレイト、トレイト境界、標準変換トレイト、静的多相
- Chapter 6 [`lifetimes`](lifetimes/README.md): 明示的なライフタイム、借用する構造体、ゼロコピー API、独立した借用、`Cow`
- Chapter 7 [`closures-iterators`](closures-iterators/README.md): クロージャのキャプチャ、`Fn` 系トレイト、遅延評価、状態付き反復、独自イテレータ
- Chapter 8 [`smart-pointers`](smart-pointers/README.md): `Box`、`Deref`、`Drop`、`Rc`、`RefCell`、`Weak`
- Chapter 9 [`concurrency`](concurrency/README.md): スレッド、スコープ付きスレッド、チャネル、`Arc`、`Mutex`、`Send`、`Sync`
- Chapter 10 [`async-rust`](async-rust/README.md): `async` / `.await`、`Future`、`Waker`、`Pin`、executor、task、backpressure、キャンセル、graceful shutdown
- Chapter 11 [`unsafe-rust`](unsafe-rust/README.md): raw pointer、`UnsafeCell`、strict provenance、`MaybeUninit`、drop・panic safety、Miri
- Chapter 12 [`advanced-traits`](advanced-traits/README.md): 関連型、blanket impl、coherence、dyn 互換性、trait object、type erasure、`Any`

次のコマンドで教材一覧を表示できます。

```console
cargo run
```

## ディレクトリ構成

- `introduction/exercises`: 学生が編集する問題
- `introduction/solutions`: 問題と同じ名前の解答例
- `ownership/exercises`: 所有権コースの問題
- `ownership/solutions`: 所有権コースの解答例
- `data-modeling/exercises`: データ設計コースの問題
- `data-modeling/solutions`: データ設計コースの解答例
- `collections/exercises`: コレクションコースの問題
- `collections/solutions`: コレクションコースの解答例
- `generics-traits/exercises`: ジェネリクスとトレイトコースの問題
- `generics-traits/solutions`: ジェネリクスとトレイトコースの解答例
- `lifetimes/exercises`: ライフタイムコースの問題
- `lifetimes/solutions`: ライフタイムコースの解答例
- `closures-iterators/exercises`: クロージャとイテレータコースの問題
- `closures-iterators/solutions`: クロージャとイテレータコースの解答例
- `smart-pointers/exercises`: スマートポインタコースの問題
- `smart-pointers/solutions`: スマートポインタコースの解答例
- `concurrency/exercises`: 並行処理コースの問題
- `concurrency/solutions`: 並行処理コースの解答例
- `async-rust/exercises`: 非同期処理コースの問題
- `async-rust/solutions`: 非同期処理コースの解答例
- `unsafe-rust/exercises`: Unsafe Rust コースの問題
- `unsafe-rust/solutions`: Unsafe Rust コースの解答例
- `advanced-traits/exercises`: 高度なトレイト設計コースの問題
- `advanced-traits/solutions`: 高度なトレイト設計コースの解答例
- `src/main.rs`: コース案内

まず解答を見ずにテストを実行し、失敗メッセージと問題ファイル内の `TODO` を手掛かりに進めてください。
