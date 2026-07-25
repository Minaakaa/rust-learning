# Rust Learning

Rust のコードを実際に直し、テストを通しながら学ぶための教材です。短い構文クイズではなく、大学のキャンパスで働く配送ロボットを題材に、現実的なデータ設計とエラー処理を練習します。

次のコースがあります。

- Chapter 1 [`introduction`](introduction/README.md): enum、`match`、`Result`、エラー、イテレータ
- Chapter 2 [`ownership`](ownership/README.md): 所有権、ムーブ、`Copy` と `Clone`、借用、スライス
- Chapter 3 [`data-modeling`](data-modeling/README.md): 構造体、メソッド、newtype、不変条件、型安全な API

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
- `src/main.rs`: コース案内

まず解答を見ずにテストを実行し、失敗メッセージと問題ファイル内の `TODO` を手掛かりに進めてください。
