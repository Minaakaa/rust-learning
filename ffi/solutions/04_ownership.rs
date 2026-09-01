//! # 解答 04: FFI 境界の所有権

use std::ffi::{CStr, CString};
use std::os::raw::c_char;

/// C文字列を複製し、Rust allocator が所有する pointer を返す
///
/// # Safety
///
/// `label` は有効な nul 終端 byte 列を指し、関数の実行中は読み取り可能でなければならない
#[unsafe(no_mangle)]
pub unsafe extern "C" fn robot_message_new(label: *const c_char) -> *mut c_char {
    if label.is_null() {
        return std::ptr::null_mut();
    }

    // SAFETY: 呼び出し側は有効な nul 終端 byte 列を渡す契約になっている
    let label = unsafe { CStr::from_ptr(label) };
    CString::new(label.to_bytes())
        .map(CString::into_raw)
        .unwrap_or_else(|_| std::ptr::null_mut())
}

/// `robot_message_new` が返した pointer の所有権を回収する
///
/// # Safety
///
/// `message` はこの module の `CString::into_raw` が返した pointer か null でなければならず、一度だけ渡す
#[unsafe(no_mangle)]
pub unsafe extern "C" fn robot_message_free(message: *mut c_char) {
    if message.is_null() {
        return;
    }

    // SAFETY: messageはrobot_message_newのinto_rawから返り、まだ回収されていない
    unsafe { drop(CString::from_raw(message)) };
}

fn main() {
    let input = CString::new("配送完了").expect("内部nulを含まない文字列です");
    let message = unsafe { robot_message_new(input.as_ptr()) };
    if !message.is_null() {
        // SAFETY: messageは直前のrobot_message_newが返した所有pointer
        let text = unsafe { CStr::from_ptr(message) }.to_string_lossy();
        println!("{text}");
        // SAFETY: 所有権を一度だけ専用解放関数へ返す
        unsafe { robot_message_free(message) };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn c文字列を複製してrust側で読める() {
        let input = CString::new("本郷ロボット🤖").expect("変換できます");
        let raw = unsafe { robot_message_new(input.as_ptr()) };

        assert!(!raw.is_null());
        // SAFETY: rawはrobot_message_newから返った有効なnul終端pointer
        assert_eq!(unsafe { CStr::from_ptr(raw) }.to_bytes(), input.as_bytes());
        // SAFETY: rawの所有権を専用解放関数へ一度だけ返す
        unsafe { robot_message_free(raw) };
    }

    #[test]
    fn null入力はnullを返す() {
        assert!(unsafe { robot_message_new(std::ptr::null()) }.is_null());
    }

    #[test]
    fn 空のc文字列も複製できる() {
        let input = CString::new("").expect("空文字列を作れます");
        let raw = unsafe { robot_message_new(input.as_ptr()) };

        assert!(!raw.is_null());
        // SAFETY: rawはrobot_message_newから返った有効なnul終端pointer
        assert_eq!(unsafe { CStr::from_ptr(raw) }.to_bytes(), input.as_bytes());
        // SAFETY: rawの所有権を専用解放関数へ一度だけ返す
        unsafe { robot_message_free(raw) };
    }

    #[test]
    fn null解放は安全なno_op() {
        // SAFETY: nullはfree関数が受け付ける特別な値
        unsafe { robot_message_free(std::ptr::null_mut()) };
    }
}
