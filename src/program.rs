use std::{ffi::c_void, mem, sync::LazyLock};

use windows::{
    Win32::{Foundation::HMODULE, System::LibraryLoader::GetModuleHandleW},
    core::Error,
    core::PCWSTR,
};

#[derive(Clone, Copy, Debug, Hash)]
pub struct Program(*mut c_void);

impl Program {
    pub fn current() -> Self {
        Self::try_current().expect("GetModuleHandleW failed")
    }

    pub fn try_current() -> Result<Self, Error> {
        static CURRENT: LazyLock<Result<Program, Error>> = LazyLock::new(|| unsafe {
            GetModuleHandleW(PCWSTR::null()).map(Program::from_hmodule)
        });
        CURRENT.clone()
    }

    /// # Safety
    ///
    /// Safe, but using the resulting pointer is incredibly unsafe.
    pub fn derva<T>(self, rva: u32) -> *mut T {
        self.0.wrapping_byte_add(rva as usize).cast()
    }

    /// # Safety
    ///
    /// Incredibly unsafe and may cause spontaneous burst pipes, gas leaks, explosions, etc.
    pub unsafe fn derva_ptr<P>(self, rva: u32) -> P {
        assert!(size_of::<P>() == size_of::<*mut ()>());
        unsafe { mem::transmute_copy::<*mut (), P>(&self.derva::<()>(rva)) }
    }

    fn from_hmodule(HMODULE(base): HMODULE) -> Self {
        Self(base)
    }
}

unsafe impl Send for Program {}

unsafe impl Sync for Program {}
