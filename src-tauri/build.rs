fn main() {
    println!("cargo:rerun-if-changed=src/qa_snapshot.m");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("macos")
        && std::env::var("PROFILE").as_deref() == Ok("debug")
    {
        cc::Build::new()
            .file("src/qa_snapshot.m")
            .flag("-fobjc-arc")
            .flag("-fblocks")
            .compile("kindling_qa_snapshot");
        println!("cargo:rustc-link-lib=framework=WebKit");
        println!("cargo:rustc-link-lib=framework=AppKit");
    }
    tauri_build::build()
}
