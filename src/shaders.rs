use std::{
    arch::naked_asm,
    ffi::c_void,
    mem,
    sync::atomic::{AtomicBool, AtomicU32, Ordering},
};

use windows::{
    Win32::System::Memory::{PAGE_EXECUTE_READWRITE, PAGE_PROTECTION_FLAGS, VirtualProtect},
    core::PCWSTR,
};
use winhook::HookInstaller;

use crate::{
    program::Program,
    rva::{ADD_PIXEL_SHADER_RVA, CB_FISHEYE_HOOK_RVA, USES_DITHERING_RVA},
};

static TONE_MAP_HOOK: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/ToneMap_PostHook.ppo"));

pub fn hook_shaders(program: Program) -> eyre::Result<()> {
    unsafe {
        let add_pixel_shader = program.derva_ptr::<unsafe extern "C" fn(
            *mut c_void,
            PCWSTR,
            *const u8,
            usize,
        ) -> *mut c_void>(ADD_PIXEL_SHADER_RVA);

        HookInstaller::for_function(add_pixel_shader)
            .enable(true)
            .install(|original| {
                move |repository, name, mut blob, mut len| {
                    if name
                        .to_string()
                        .is_ok_and(|name| name == "ToneMap_PostOETFPS")
                    {
                        blob = TONE_MAP_HOOK.as_ptr();
                        len = TONE_MAP_HOOK.len();
                    }

                    original(repository, name, blob, len)
                }
            })
            .map(mem::forget)
            .unwrap();

        let uses_dithering = program
            .derva_ptr::<unsafe extern "C" fn(*const c_void, *mut c_void, u32) -> bool>(
                USES_DITHERING_RVA,
            );

        HookInstaller::for_function(uses_dithering)
            .enable(true)
            .install(|original| {
                move |param_1, param_2, param_3| {
                    ENABLE_DITHERING.load(Ordering::Relaxed) && original(param_1, param_2, param_3)
                }
            })
            .map(mem::forget)
            .unwrap();

        hook_shader_cb(program)
    }
}

unsafe fn hook_shader_cb(program: Program) -> eyre::Result<()> {
    #[unsafe(naked)]
    extern "C" fn fisheye_distortion_cb_hook() {
        naked_asm! {
            // Original code start...
            "divss xmm1,xmm9",
            "shufps xmm1,xmm1,0",
            "movaps [rbp+0x150],xmm1",
            // ...original code end.
            "push rax",
            // Forward the flags to the constant buffer (see "shaders/ToneMap_PostHook.hlsl").
            "mov eax,[rip+{}]",
            "pinsrd xmm1,eax,3",
            // Force the shader on.
            "and al,1",
            "mov [r15+0xcb0],al",
            "pop rax",
            "ret",
            sym FLAGS,
        }
    }

    // 00 CALL [0x08]
    // 06 JMP 0x10
    // 08 DQ `fisheye_distortion_cb_hook``
    // 10 ...
    let cb_hook_buf = {
        let [b0, b1, b2, b3, b4, b5, b6, b7] = u64::to_le_bytes(fisheye_distortion_cb_hook as u64);
        [
            0xff, 0x15, 0x02, 0x00, 0x00, 0x00, 0xeb, 0x08, b0, b1, b2, b3, b4, b5, b6, b7,
        ]
    };

    let cb_hook_mem = program.derva::<[u8; 16]>(CB_FISHEYE_HOOK_RVA);

    unsafe {
        VirtualProtect(
            cb_hook_mem as *const c_void,
            cb_hook_buf.len(),
            PAGE_EXECUTE_READWRITE,
            &mut PAGE_PROTECTION_FLAGS::default(),
        )?;

        cb_hook_mem.write(cb_hook_buf);
    }

    Ok(())
}

static ENABLE_DITHERING: AtomicBool = AtomicBool::new(true);

pub fn enable_dithering(state: bool) {
    ENABLE_DITHERING.store(state, Ordering::Relaxed);
}

pub fn enable_fisheye_distortion(state: bool) {
    set_flag_inner(state, 0);
}

pub fn enable_crosshair(state: bool) {
    set_flag_inner(state, 1);
}

static FLAGS: AtomicU32 = AtomicU32::new(0);

fn set_flag_inner(state: bool, pos: u32) -> u32 {
    let flag = 1 << pos;
    match state {
        true => FLAGS.fetch_or(flag, Ordering::Relaxed),
        false => FLAGS.fetch_and(!flag, Ordering::Relaxed),
    }
}
