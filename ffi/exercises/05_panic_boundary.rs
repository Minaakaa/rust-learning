#![cfg_attr(not(test), allow(dead_code))]

//! # 問題 05: panic を越境させない status code
//!
//! C caller は Rust の panic payload や unwind を扱えません
//! `catch_unwind` で Rust 側の処理を包み、成功・入力エラー・予期しない panic を ABI 安全な code へ変換してください
//!
//! 仕様:
//! - `StatusCode` は `#[repr(u32)]` とし、C から読める値を持つ
//! - `robot_score` は 0から100の percent を10倍して out pointer へ書く
//! - null の out pointer は `InvalidInput` を返し、書き込みをしない
//! - 計算中の panic は `Panic` として返し、FFI 境界の外へ出さない
//! - 成功時だけ out pointerへ結果を書き込む

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
    let _ = (percent, out, score_percent as fn(u32) -> u32);
    todo!("null検査とcatch_unwindを使い、結果をstatus codeへ変換してください")
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
