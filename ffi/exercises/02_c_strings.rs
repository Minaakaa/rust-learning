#![cfg_attr(not(test), allow(dead_code))]

//! # 問題 02: `CString` と `CStr`
//!
//! C の文字列は nul 終端の byte 列であり、Rust の UTF-8 `str` と同じではありません
//! 所有する `CString` と借用する `CStr` を安全な変換関数の中で使ってください
//!
//! 仕様:
//! - `to_c_label` は Rust の `&str` を `CString` へ変換する
//! - 内部 nul は `LabelError::InteriorNul` として拒否する
//! - `from_c_label` は null pointer を拒否する
//! - 不正な UTF-8 は `LabelError::InvalidUtf8` として拒否する
//! - raw pointer を使う unsafe block は C文字列の契約をコメントで説明する

use std::ffi::CString;
use std::os::raw::c_char;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LabelError {
    NullPointer,
    InteriorNul,
    InvalidUtf8,
}

fn to_c_label(label: &str) -> Result<CString, LabelError> {
    let _ = label;
    todo!("RustのstrをCStringへ変換し、内部nulをエラーにしてください")
}

fn from_c_label(ptr: *const c_char) -> Result<String, LabelError> {
    let _ = ptr;
    todo!("nullとUTF-8を検証してCStrからStringへ変換してください")
}

fn main() {
    let label = to_c_label("配送ロボット🤖").expect("nulを含まないlabelです");
    let restored = from_c_label(label.as_ptr()).expect("有効なC文字列です");
    println!("{restored}");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rustからcへnul終端で渡せる() {
        let label = to_c_label("本郷ロボット🤖").expect("変換できます");

        assert_eq!(label.to_bytes_with_nul().last(), Some(&0));
        assert_eq!(label.to_bytes(), "本郷ロボット🤖".as_bytes());
    }

    #[test]
    fn 内部nulを拒否する() {
        assert_eq!(to_c_label("robot\0id"), Err(LabelError::InteriorNul));
    }

    #[test]
    fn cからutf8を復元する() {
        let label = CString::new("工学部・倉庫").expect("変換できます");

        assert_eq!(from_c_label(label.as_ptr()), Ok("工学部・倉庫".to_owned()));
    }

    #[test]
    fn null_pointerを拒否する() {
        assert_eq!(from_c_label(std::ptr::null()), Err(LabelError::NullPointer));
    }

    #[test]
    fn 不正なutf8を拒否する() {
        let bytes = [0xff_u8, 0];
        let ptr = bytes.as_ptr().cast::<c_char>();

        assert_eq!(from_c_label(ptr), Err(LabelError::InvalidUtf8));
    }
}
