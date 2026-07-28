# Chapter 6 — ライフタイムで借用 API を設計する

このコースでは、所有権の基礎を実践的な API 設計へ発展させます
複数の入力から参照を返す関数から始め、入力を複製しないログ解析、借用データを保持する型、独立した2つの参照を段階的に学び、最後に `Cow<'a, str>` で借用と所有を使い分ける通知解析を完成させます

## 学習目標

| 問題 | 主題 | できるようになること |
| --- | --- | --- |
| 01 | 入出力の関係 | 複数の入力のどこから戻り値を借用するか、明示的なライフタイムで表す |
| 02 | ゼロコピー解析 | `LogEntry<'a>` と `LogError<'a>` に入力文字列のスライスを保持する |
| 03 | 借用データを持つ型 | `MissionCatalog<'data>` から、検索キーや `&self` より長く使える参照を返す |
| 04 | 独立した借用 | `Assignment<'mission, 'robot>` で有効期間の異なる2つの参照を扱う |
| 05 | 借用または所有 | `Cow<'a, str>` で必要な場合だけ文字列を割り当てる |

## この章の考え方

- ライフタイム注釈は値を長生きさせる指定ではなく、複数の参照がどのような関係にあるかをコンパイラへ伝える契約
- 同じ `'a` を付けた引数がまったく同じスコープを持つという意味ではなく、呼び出し時に両方へ適用できる範囲として扱われる
- 戻り値は、実際に借用元になり得る入力だけへ結び付ける
- 入力参照が1つなら戻り値へそのライフタイムを引き継ぎ、メソッドなら通常 `&self` のライフタイムを引き継ぐ省略規則がある
- 型が保持している参照を `&self` より長く返したい場合や、複数入力のどれを返すか示す場合は明示的な注釈が必要
- 参照を持つ構造体は参照元より長く存在できない
- `'static` はプログラム全体で有効なデータに使うもので、借用エラーを回避するための指定ではない
- `Cow::Borrowed` は元データを参照し、`Cow::Owned` は必要になったデータを所有する

問題ファイルは未完成の状態でもコンパイルできるよう、最初は一部の値を `String` や所有する構造体として返します
実装本体だけでなく、問題文に従って型定義と関数シグネチャも借用する形へ変更してください
テストは値の一致に加え、ポインタが入力範囲内にあることも確認します

## 進め方

プロジェクトのルートで、最初の問題を実行します

```console
cargo test --example lifetimes_01_lifetime_relations
```

未完成の問題は `not yet implemented` で失敗します
問題ファイルの仕様、ヒント、テストを読み、`todo!()` と所有する型を借用する実装へ変更してください

```console
cargo test --example lifetimes_02_zero_copy_logs
cargo test --example lifetimes_03_borrowed_catalog
cargo test --example lifetimes_04_independent_borrows
cargo test --example lifetimes_05_cow_notices
```

問題は Cargo の example target として分離されています
`cargo test` だけでは問題を実行しないため、必ず `--example lifetimes_...` を指定してください

## 解答例

同名の解答例が `lifetimes/solutions` にあります
たとえば問題 03 の解答だけを検証するには、次を実行します

```console
cargo test --test solution_lifetimes_03_borrowed_catalog
```

全コースの解答例は、次のコマンドでまとめて検証できます

```console
cargo test --tests
```

`cargo test --all-targets` は未完成の問題も実行するため、問題を解き終えるまでは失敗します

## 公式リファレンス

- [ライフタイムで参照を検証する — The Rust Programming Language](https://doc.rust-lang.org/book/ch10-03-lifetime-syntax.html)
- [ライフタイムの省略 — Rust Reference](https://doc.rust-lang.org/reference/lifetime-elision.html)
- [明示的な注釈 — Rust By Example](https://doc.rust-lang.org/rust-by-example/scope/lifetime/explicit.html)
- [`Cow` — Rust 標準ライブラリ](https://doc.rust-lang.org/std/borrow/enum.Cow.html)
- [`str::split_whitespace` — Rust 標準ライブラリ](https://doc.rust-lang.org/std/primitive.str.html#method.split_whitespace)
