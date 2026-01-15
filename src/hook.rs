use std::mem;

use closure_ffi::traits::{FnPtr, FnThunk};
use winhook::{CConv, Error, HookInstaller};

pub unsafe fn install<F, C, H>(f: F, c: C) -> Result<(), Error>
where
    F: FnPtr + CConv + 'static,
    C: FnOnce(F) -> H + 'static,
    H: Send + Sync + 'static,
    (F::CC, H): FnThunk<F> + Send + Sync + 'static,
{
    unsafe {
        HookInstaller::for_function(f)
            .enable(true)
            .install(c)
            .map(mem::forget)
    }
}
