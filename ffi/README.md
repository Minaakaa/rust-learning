# Chapter 14 — FFI と C ABI

この章では、Rust の型安全なコードを C ABI の境界へ公開・接続する設計を学びます
配送ロボットのフレーム、ラベル、callback、ハンドル、エラーコードを題材に、unsafe を小さな安全ラッパへ閉じ込めます

## 学習目標

| 問題 | 主題 | できるようになること |
| --- | --- | --- |
| 01 | `repr(C)` | C と共有できるレイアウト、固定幅整数、明示的な enum 表現を設計する |
| 02 | C 文字列 | `CString` と `CStr` を使い、nul と UTF-8 の失敗を `Result` へ変換する |
| 03 | callback | `extern "C" fn` と nullable callback、opaque context の契約を扱う |
| 04 | 所有権 | `into_raw` / `from_raw` と `#[unsafe(no_mangle)]` の解放 API を設計する |
| 05 | panic 境界 | C ABI 関数から panic を越境させず、明示した整数表現の status code で結果を返す |

## FFI の基本契約

- FFI の関数シグネチャには、C が理解できる固定幅整数、`#[repr(C)]` struct、raw pointer、関数ポインタなどだけを置く
- `String`、`Vec<T>`、`&str`、`&CStr` は Rust 側の安全ラッパ内で扱い、C の公開シグネチャへ直接置かない
- `#[repr(C)]` は field の順序と C 互換の layout を保証するが、field の値域や pointer の有効性までは保証しない
- Edition 2024 では外部宣言に `unsafe extern "C"` が必要で、宣言した signature の正しさは宣言者の責任になる
- C から受け取った pointer の null、長さ、終端、所有権、aliasing、解放者を必ず契約として文書化する
- Rust の panic は通常の `extern "C"` 境界を越えさせず、`catch_unwind` と status code などで処理する
- `extern "C-unwind"` は foreign exception や Rust panic を越境させるための別 ABI であり、通常の C API の代用品ではない

## 進め方

```console
cargo test --example ffi_01_c_layout
cargo test --example ffi_02_c_strings
cargo test --example ffi_03_callbacks
cargo test --example ffi_04_ownership
cargo test --example ffi_05_panic_boundary
```

問題は Cargo の example target として分離されています
同名の解答例が `ffi/solutions` にあります

```console
cargo test --test solution_ffi_01_c_layout
cargo test --test solution_ffi_05_panic_boundary
cargo test --tests
```

## 発展課題

この章では C コンパイラや外部ライブラリへの依存を増やさず、Rust 側で C ABI の契約を検証します
実プロジェクトでは `build.rs`、`cc`、`bindgen`、ヘッダの version 管理、クロスコンパイル、C++ の例外境界も追加で検討してください

## 公式リファレンス

- [Unsafe extern blocks — The Rust Edition Guide](https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-extern.html)
- [Unsafe keyword — The Rust Reference](https://doc.rust-lang.org/reference/unsafe-keyword.html)
- [External blocks — The Rust Reference](https://doc.rust-lang.org/reference/items/external-blocks.html)
- [`std::ffi` — Rust standard library](https://doc.rust-lang.org/std/ffi/)
- [`CStr` — Rust standard library](https://doc.rust-lang.org/std/ffi/c_str/struct.CStr.html)
- [`CString` — Rust standard library](https://doc.rust-lang.org/std/ffi/struct.CString.html)
- [FFI — The Rustonomicon](https://doc.rust-lang.org/nomicon/ffi.html)
