//! # 解答 02: `CString` と `CStr`

use std::ffi::{CStr, CString};
use std::os::raw::c_char;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LabelError {
    NullPointer,
    InteriorNul,
    InvalidUtf8,
}

fn to_c_label(label: &str) -> Result<CString, LabelError> {
    CString::new(label).map_err(|_| LabelError::InteriorNul)
}

fn from_c_label(ptr: *const c_char) -> Result<String, LabelError> {
    if ptr.is_null() {
        return Err(LabelError::NullPointer);
    }

    // SAFETY: 呼び出し側は有効な nul 終端 byte 列を所有し、変換中に書き換えない
    let label = unsafe { CStr::from_ptr(ptr) };
    label
        .to_str()
        .map(str::to_owned)
        .map_err(|_| LabelError::InvalidUtf8)
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
