//! Guard: the server and the WASM bundle must both be built with `--cfg erase_components`
//! (see .cargo/config.toml). If a build environment overrides the rustflags, stop here and
//! show what it passed, instead of shipping pages whose hydration markers don't match.
fn main() {
    println!("cargo::rustc-check-cfg=cfg(erase_components)");
    println!("cargo::rerun-if-env-changed=RUSTFLAGS");
    println!("cargo::rerun-if-env-changed=CARGO_ENCODED_RUSTFLAGS");
    if std::env::var_os("CARGO_CFG_ERASE_COMPONENTS").is_none() {
        let show = |k: &str| format!("{k}={:?}", std::env::var(k).ok());
        panic!(
            "missing --cfg erase_components; {} {} {} {}",
            show("RUSTFLAGS"),
            show("CARGO_ENCODED_RUSTFLAGS"),
            show("TARGET"),
            show("CARGO_BUILD_TARGET")
        );
    }
}
