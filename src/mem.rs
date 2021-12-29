pub fn replace<T, R>(v: &mut T, change: impl FnOnce(T) -> (T, R)) -> R {
    use std::{panic, ptr};

    unsafe {
        let old_t = ptr::read(v);
        let (new_t, r) = panic::catch_unwind(panic::AssertUnwindSafe(|| change(old_t)))
            .unwrap_or_else(|_| ::std::process::abort());
        ptr::write(v, new_t);
        r
    }
}
