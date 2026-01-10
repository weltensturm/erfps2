use std::{env, io, process::Command};

fn main() -> io::Result<()> {
    println!("cargo::rerun-if-changed=shaders/ToneMap_PostHook.hlsl");

    let out = format!("{}/ToneMap_PostHook.ppo", env::var("OUT_DIR").unwrap());
    let dxc = env::var("DXC_PATH").unwrap_or_else(|_| "dxc".to_owned());

    Command::new(dxc)
        .args([
            "-E",
            "PSMain",
            "-T",
            "ps_6_0",
            "-Fo",
            &out,
            "shaders/ToneMap_PostHook.hlsl",
        ])
        .spawn()?
        .wait()?;

    Ok(())
}
