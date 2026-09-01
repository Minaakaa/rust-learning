//! # 解答 03: C ABI callback と opaque context

use std::ffi::c_void;

type ReadingCallback = extern "C" fn(u16, *mut c_void);

fn emit_readings(
    readings: &[u16],
    callback: Option<ReadingCallback>,
    context: *mut c_void,
) -> usize {
    let Some(callback) = callback else {
        return 0;
    };

    for &reading in readings {
        callback(reading, context);
    }
    readings.len()
}

extern "C" fn print_reading(reading: u16, _context: *mut c_void) {
    println!("reading={reading}");
}

fn main() {
    let readings = [82, 79, 91];
    let count = emit_readings(&readings, Some(print_reading), std::ptr::null_mut());
    println!("通知件数={count}");
}

#[cfg(test)]
mod tests {
    use super::*;

    extern "C" fn collect(reading: u16, context: *mut c_void) {
        // SAFETY: テストは有効なVecへのpointerを同期的なcallbackへ渡している
        unsafe { (&mut *context.cast::<Vec<u16>>()).push(reading) }
    }

    #[test]
    fn callbackを入力順に呼び出す() {
        let mut seen = Vec::new();
        let count = emit_readings(
            &[10, 20, 30],
            Some(collect),
            (&mut seen as *mut Vec<u16>).cast::<c_void>(),
        );

        assert_eq!(count, 3);
        assert_eq!(seen, [10_u16, 20, 30]);
    }

    #[test]
    fn noneのcallbackは安全に無視する() {
        assert_eq!(emit_readings(&[1, 2], None, std::ptr::null_mut()), 0);
    }

    #[test]
    fn 空sliceではcallbackを呼ばない() {
        let mut seen = Vec::new();
        let count = emit_readings(
            &[],
            Some(collect),
            (&mut seen as *mut Vec<u16>).cast::<c_void>(),
        );

        assert_eq!(count, 0);
        assert!(seen.is_empty());
    }

    #[test]
    fn contextの所有権を移動しない() {
        let mut seen = Vec::new();
        let context = (&mut seen as *mut Vec<u16>).cast::<c_void>();

        emit_readings(&[7], Some(collect), context);

        assert_eq!(seen, [7_u16]);
    }
}
