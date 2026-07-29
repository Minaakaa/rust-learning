# Chapter 7 — クロージャとイテレータで配送処理を組み立てる

このコースでは、処理そのものを値として渡すクロージャと、必要な分だけ計算するイテレータを学びます
環境のキャプチャから始め、`Fn`・`FnMut`・`FnOnce`、遅延パイプライン、状態付き・失敗可能な反復へ進み、最後に `Iterator` と `IntoIterator` を実装した公平な配送順を完成させます

## 学習目標

| 問題 | 主題 | できるようになること |
| --- | --- | --- |
| 01 | キャプチャと `move` | 環境の値を所有するクロージャを返し、`Fn` 境界の API へ渡す |
| 02 | 呼び出しトレイト | 状態を更新する `FnMut` と値を一度だけ消費する `FnOnce` を使い分ける |
| 03 | 遅延パイプライン | `flat_map`、`filter_map`、`enumerate` を合成し、`impl Iterator` を返す |
| 04 | 状態と短絡 | `scan`、`fuse`、`try_fold` で状態更新と最初の失敗を表す |
| 05 | 独自イテレータ | 関連型 `Item` と `next` を実装し、`IntoIterator` で `for` に対応する |

## この章の考え方

- クロージャは関数のように呼べる匿名の値で、定義した環境から必要な値を自動的にキャプチャする
- `move` はキャプチャした値をクロージャへ移す指定であり、それだけで `FnOnce` になるわけではない
- クロージャ本体が捕捉値を読むだけなら `Fn`、変更するなら `FnMut`、外へ移動するなら `FnOnce` になる
- `Fn` は `FnMut` と `FnOnce` の要件も満たし、`FnMut` は `FnOnce` の要件も満たす
- 呼び出し側が一度しか実行しないなら `FnOnce`、状態変更を許して繰り返すなら `FnMut`、共有参照から繰り返す必要があるなら `Fn` を選ぶ
- クロージャの具体型には名前を書けないため、戻り値では `impl Fn(...)` を使える
- イテレータアダプターは遅延評価され、`next`、`collect`、`count` などで消費されるまで要素を処理しない
- `scan` は状態を更新しながら要素を返し、`try_fold` は `Result` の最初の失敗で短絡する
- `Iterator` の実装に必要なのは関連型 `Item` と `next` で、既存の多くのアダプターは自動的に利用できる
- `IntoIterator` を実装すると `for`、`collect`、各種アダプターへ自然につなげられる

この章では基礎を明確にするため、`Box<dyn Fn>`、async closure、高階ライフタイム境界、GAT、並列イテレータは扱いません

## 進め方

プロジェクトのルートで、最初の問題を実行します

```console
cargo test --example closures_iterators_01_closure_captures
```

未完成の問題は `not yet implemented` で失敗します
問題ファイルの仕様、ヒント、テストを読み、`todo!()` と必要な型境界や戻り型を変更してください

```console
cargo test --example closures_iterators_02_fn_traits
cargo test --example closures_iterators_03_lazy_pipelines
cargo test --example closures_iterators_04_stateful_iteration
cargo test --example closures_iterators_05_fair_dispatch
```

問題は Cargo の example target として分離されています
`cargo test` だけでは問題を実行しないため、必ず `--example closures_iterators_...` を指定してください

## 解答例

同名の解答例が `closures-iterators/solutions` にあります
たとえば問題 04 の解答だけを検証するには、次を実行します

```console
cargo test --test solution_closures_iterators_04_stateful_iteration
```

全コースの解答例は、次のコマンドでまとめて検証できます

```console
cargo test --tests
```

`cargo test --all-targets` は未完成の問題も実行するため、問題を解き終えるまでは失敗します

## 公式リファレンス

- [クロージャ — The Rust Programming Language](https://doc.rust-lang.org/book/ch13-01-closures.html)
- [イテレータで要素列を処理する — The Rust Programming Language](https://doc.rust-lang.org/book/ch13-02-iterators.html)
- [`Fn` — Rust 標準ライブラリ](https://doc.rust-lang.org/std/ops/trait.Fn.html)
- [`FnMut` — Rust 標準ライブラリ](https://doc.rust-lang.org/std/ops/trait.FnMut.html)
- [`FnOnce` — Rust 標準ライブラリ](https://doc.rust-lang.org/std/ops/trait.FnOnce.html)
- [`Iterator` — Rust 標準ライブラリ](https://doc.rust-lang.org/std/iter/trait.Iterator.html)
- [`IntoIterator` — Rust 標準ライブラリ](https://doc.rust-lang.org/std/iter/trait.IntoIterator.html)
- [イテレータの遅延評価 — Rust 標準ライブラリ](https://doc.rust-lang.org/std/iter/#laziness)
