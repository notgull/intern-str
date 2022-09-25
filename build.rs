//! The build flags emitted by this script are not public API.

fn main() {
    // Probe for the latest Rust version.
    let rustc = match autocfg::AutoCfg::new() {
        Ok(rustc) => rustc,
        Err(e) => {
            println!("cargo:warning={}: unable to determine version: {}", env!("CARGO_PKG_NAME"), e);
            return;
        }
    };

    // Note that this is `no_`*, not `has_*`. This allows treating as the latest
    // stable rustc is used when the build script doesn't run. This is useful
    // for non-cargo build systems that don't run the build script.

    // alloc stabilized in Rust 1.36 (nightly-2019-04-15)
    if !rustc.probe_rustc_version(1, 36) {
        println!("cargo:rustc-cfg=intern_str_no_alloc");
    } 
}