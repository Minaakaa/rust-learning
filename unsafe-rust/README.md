# Chapter 11 — Unsafe Rust で sound な低レベル抽象化を作る

このコースでは、コンパイラが自動では証明できない不変条件を人間が明示し、小さな `unsafe` core を safe API で包む方法を学びます
raw pointer から始め、aliasing と `UnsafeCell`、strict provenance、`MaybeUninit`、drop・panic safety へ進み、最後に部分初期化した配列を安全に完成させます

## 学習目標

| 問題 | 主題 | できるようになること |
| --- | --- | --- |
| 01 | raw pointer と安全な slice view | pointer、長さ、lifetime の契約を `unsafe fn` へ書き、safe な読み取り API を公開する |
| 02 | aliasing と `UnsafeCell` | 共有参照からの変更を局所化し、参照を外へ逃がさない single-thread 用 cell を作る |
| 03 | strict provenance | 生アドレスではなく offset を保存し、元 allocation の provenance を保って pointer を解決する |
| 04 | `MaybeUninit` 固定長 buffer | 初期化済み範囲だけを読み書きし、各値を正確に1回 drop する |
| 05 | 部分初期化と panic safety | guard で初期化数を追跡し、エラーや panic でも構築済み要素を回収する |

## この章の考え方

- `unsafe` は validity、aliasing、lifetime、data race などの規則を無効にせず、違反は未定義動作（undefined behavior、UB）になり得る
- `unsafe` が追加で許可する操作には raw pointer の dereference、`unsafe fn` の呼び出し、`static mut` へのアクセス、`unsafe trait` の実装、`union` field の読み出しがある
- `unsafe fn` は呼び出し側が満たす契約を表し、`unsafe {}` は実装側が根拠を確認した操作だけを囲む
- Rust 2024 では `unsafe fn` の本体でも unsafe 操作に明示的な `unsafe {}` が必要になる
- safe な呼び出しだけで UB へ到達できない unsafe 実装を sound と呼ぶ
- safety contract には pointer の non-null、alignment、validity、初期化、有効範囲、単一 allocation、aliasing、lifetime を具体的に書く
- raw pointer の作成や保持自体は可能だが、dereference して値や参照を作る時に全前提が必要になる
- 長さ0の slice でも pointer は non-null かつ aligned である必要があり、`NonNull::dangling()` を dereference してはいけない
- `UnsafeCell<T>` が緩和するのは共有参照の不変性だけで、aliasing した `&mut T` や data race は引き続き UB になる
- `UnsafeCell<T>` は `Sync` ではないため、同期なしで複数 thread から共有する safe API にはならない
- pointer の provenance はどの allocation へアクセスできるかという情報で、数値アドレスだけでは置き換えられない
- `addr` と `with_addr` などの Strict Provenance API は元 pointer の provenance を保ってアドレス部分を扱う
- `MaybeUninit<T>` は未初期化 memory を `T` として読み出さずに保持する型で、初期化済みという事実は別の状態で追跡する
- 全 bit が 0 でも有効な `T` になるとは限らないため、`MaybeUninit::zeroed` の結果も型ごとの validity を確認せず `assume_init` してはいけない
- `write` は古い値を drop せず、`assume_init_ref`、`assume_init_read`、`assume_init_drop` は初期化済みという証明を要求する
- 初期化数や buffer 長は unsafe 操作の正しさを支える不変条件なので、panic が起きても嘘にならない順番で更新する
- user code が panic し得る処理では、先に不変条件を安全な状態へ移すか、`Drop` guard で unwind 時に復旧する
- 状態更新で二重 drop は防げても、すでに unwind 中の `T::drop` が再び panic すると process は abort し、未破棄の値が leak し得る
- 各 `unsafe` block はできるだけ小さくし、直前の `SAFETY` コメントで成立する前提を説明する
- Miri は実行された経路の UB を検出する補助ツールであり、未実行経路を含む soundness の証明そのものではない

この章では FFI、`union`、`unsafe trait` / `unsafe impl`、`static mut`、独自 allocator、SIMD、lock-free 構造は扱いません

## 進め方

プロジェクトのルートで、最初の問題を実行します

```console
cargo test --example unsafe_rust_01_raw_slice
```

未完成の問題は `not yet implemented` または未実装部分に対応するテスト失敗になります
問題ファイルの safety contract、`SAFETY` コメント、テストを読み、`todo!()` を変更してください

```console
cargo test --example unsafe_rust_02_unsafe_cell
cargo test --example unsafe_rust_03_strict_provenance
cargo test --example unsafe_rust_04_fixed_buffer
cargo test --example unsafe_rust_05_panic_safe_array
```

問題は Cargo の example target として分離されています
`cargo test` だけでは問題を実行しないため、必ず `--example unsafe_rust_...` を指定してください

テストが通ったら、追加した unsafe block の根拠も Clippy で確認します

```console
cargo clippy --example unsafe_rust_01_raw_slice -- -D warnings
```

## 解答例

同名の解答例が `unsafe-rust/solutions` にあります
たとえば問題05の解答だけを検証するには、次を実行します

```console
cargo test --test solution_unsafe_rust_05_panic_safe_array
```

全コースの解答例は、次のコマンドでまとめて検証できます

```console
cargo test --tests
```

`cargo test --all-targets` は未完成の問題も実行するため、問題を解き終えるまでは失敗します

## Miri で unsafe の前提を検査する

Miri は nightly toolchain の component として提供されています
導入後、まず自分が完成させた問題を検査します

```console
rustup +nightly component add miri
cargo +nightly miri test --example unsafe_rust_01_raw_slice
```

同じ方法で、Chapter 11 の解答例も1つずつ検査できます

```console
cargo +nightly miri test --test solution_unsafe_rust_01_raw_slice
cargo +nightly miri test --test solution_unsafe_rust_02_unsafe_cell
cargo +nightly miri test --test solution_unsafe_rust_03_strict_provenance
cargo +nightly miri test --test solution_unsafe_rust_04_fixed_buffer
cargo +nightly miri test --test solution_unsafe_rust_05_panic_safe_array
```

Miri が成功しても、safety contract の全条件を人間が確認する作業は残ります

## 公式リファレンス

- [Unsafe Rust — The Rust Programming Language](https://doc.rust-lang.org/book/ch20-01-unsafe-rust.html)
- [`unsafe` keyword — Rust 標準ライブラリ](https://doc.rust-lang.org/std/keyword.unsafe.html)
- [未定義動作 — The Rust Reference](https://doc.rust-lang.org/reference/behavior-considered-undefined.html)
- [`unsafe_op_in_unsafe_fn` — Rust 2024 Edition Guide](https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-op-in-unsafe-fn.html)
- [`NonNull` — Rust 標準ライブラリ](https://doc.rust-lang.org/std/ptr/struct.NonNull.html)
- [`slice::from_raw_parts` — Rust 標準ライブラリ](https://doc.rust-lang.org/std/slice/fn.from_raw_parts.html)
- [`UnsafeCell` — Rust 標準ライブラリ](https://doc.rust-lang.org/std/cell/struct.UnsafeCell.html)
- [Strict Provenance — raw pointer API](https://doc.rust-lang.org/std/primitive.pointer.html)
- [`MaybeUninit` — Rust 標準ライブラリ](https://doc.rust-lang.org/std/mem/union.MaybeUninit.html)
- [Exception Safety — Rustonomicon](https://doc.rust-lang.org/nomicon/exception-safety.html)
- [Miri — rust-lang/miri](https://github.com/rust-lang/miri)
