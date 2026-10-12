use std::{env, path::PathBuf};

fn main() {
    if env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }

    let arch = match env::var("CARGO_CFG_TARGET_ARCH").as_deref() {
        Ok("x86_64") => "x64",
        Ok("x86") => "x86",
        _ => panic!("Неподдерживаемая архитектура Npcap"),
    };

    let lib_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("vendor")
        .join("npcap-sdk")
        .join("Lib")
        .join(arch);

    assert!(
        lib_dir.join("Packet.lib").is_file(),
        "Не найден Packet.lib: {}",
        lib_dir.display()
    );

    println!("cargo:rustc-link-search=native={}", lib_dir.display());
}
