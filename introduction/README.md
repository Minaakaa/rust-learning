# Introduction — 配送ロボットで学ぶ Rust の基礎

このコースでは、大学キャンパス内を走る配送ロボットの小さなプログラムを作ります。5 問を順番に解くと、Rust で状態を表し、分岐し、失敗を安全に扱い、データ列を処理する基本的な流れを体験できます。

## 学習目標

| 問題 | 主題 | できるようになること |
| --- | --- | --- |
| 01 | enum | 種類ごとに異なるデータを 1 つの型で表す |
| 02 | `match` | enum を分解し、ガードと範囲を使って網羅的に分岐する |
| 03 | `Result` | 回復可能な失敗を値として返し、`?` で伝播する |
| 04 | 独自エラー | エラー用 enum と `Display`、`Error`、`From` を実装する |
| 05 | イテレータ | `filter`、`map`、`sum`、`collect` を組み合わせる |

前提は、変数、関数、`struct`、所有権と借用の初歩を学んでいることです。外部クレートは使いません。

## 進め方

プロジェクトのルートで、最初の問題のテストを実行します。

```console
cargo test --example intro_01_enums
```

最初は `not yet implemented` と表示されて失敗します。これは正常です。`introduction/exercises/01_enums.rs` を開き、冒頭の仕様とテストを読み、`todo!()` を自分のコードに置き換えてください。テストがすべて成功したら、番号を 02、03 と進めます。

問題は Cargo の example target として分離されています。`cargo test` だけでは問題のテストを実行しないため、必ず `--example intro_...` を指定してください。

```console
cargo test --example intro_02_match
cargo test --example intro_03_results
cargo test --example intro_04_errors
cargo test --example intro_05_iterators
```

コードを整形するには次を実行します。

```console
cargo fmt
```

## 解答例の確認

行き詰まったときは、`introduction/solutions` に同名の解答例があります。たとえば問題 03 の解答だけを検証するコマンドは次のとおりです。

```console
cargo test --test solution_intro_03_results
```

すべての解答例は、次のコマンドでまとめて検証できます。

```console
cargo test --tests
```

`cargo test --all-targets` は未完成の問題もすべて実行するため、5 問を解き終えるまでは失敗します。

解答例は唯一の正解ではありません。テストを満たした後、自分のコードと比べて、読みやすさ、所有権の扱い、エラー情報の残し方にどのような違いがあるか考えてください。

## 公式リファレンス

問題は Rust 2024 Edition と現在の標準ライブラリで動くように作られています。必要に応じて、次の公式資料を参照してください。

- [列挙型とパターンマッチング — The Rust Programming Language](https://doc.rust-lang.org/book/ch06-00-enums.html)
- [`match` キーワード — Rust 標準ライブラリ](https://doc.rust-lang.org/std/keyword.match.html)
- [回復可能なエラーと `Result`](https://doc.rust-lang.org/book/ch09-02-recoverable-errors-with-result.html)
- [`std::error::Error` トレイト](https://doc.rust-lang.org/std/error/trait.Error.html)
- [イテレータによる一連の要素の処理](https://doc.rust-lang.org/book/ch13-02-iterators.html)
- [`Iterator` トレイト](https://doc.rust-lang.org/std/iter/trait.Iterator.html)
