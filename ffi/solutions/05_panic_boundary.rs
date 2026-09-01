//! # 解答 05: panic を越境させない status code

use std::panic::{AssertUnwindSafe, catch_unwind};

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusCode {
    Ok = 0,
    InvalidInput = 1,
    Panic = 2,
}

fn score_percent(percent: u32) -> u32 {
    assert!(percent <= 100, "percentは100以下でなければなりません");
    percent * 10
}

/// percent を計算し、結果を呼び出し側の領域へ書き込む
///
/// # Safety
///
/// `out` は null でない場合、`u32` 1個分の書き込み可能な領域を指さなければならない
#[unsafe(no_mangle)]
pub unsafe extern "C" fn robot_score(percent: u32, out: *mut u32) -> StatusCode {
    if out.is_null() {
        return StatusCode::InvalidInput;
    }

    let result = catch_unwind(AssertUnwindSafe(|| {
        let score = score_percent(percent);
        // SAFETY: null検査済みで、呼び出し側がu32 1個分の書き込み可能領域を渡す契約
        unsafe { *out = score };
    }));

    match result {
        Ok(()) => StatusCode::Ok,
        Err(_) => StatusCode::Panic,
    }
}

fn main() {
    let mut score = 0;
    let status = unsafe { robot_score(82, &mut score) };
    println!("status={status:?} score={score}");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 成功時だけscoreを書き込む() {
        let mut score = 0;

        assert_eq!(unsafe { robot_score(82, &mut score) }, StatusCode::Ok);
        assert_eq!(score, 820);
    }

    #[test]
    fn nullのoutは入力エラーになる() {
        assert_eq!(
            unsafe { robot_score(82, std::ptr::null_mut()) },
            StatusCode::InvalidInput
        );
    }

    #[test]
    fn panicはstatusへ変換される() {
        let mut score = 999;

        assert_eq!(unsafe { robot_score(101, &mut score) }, StatusCode::Panic);
        assert_eq!(score, 999);
    }

    #[test]
    fn statusの数値表現を固定する() {
        assert_eq!(StatusCode::Ok as u32, 0);
        assert_eq!(StatusCode::InvalidInput as u32, 1);
        assert_eq!(StatusCode::Panic as u32, 2);
    }
}
