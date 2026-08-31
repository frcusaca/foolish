use std::{env, fs, path::Path};

fn main() {
    let manifest = env::var("CARGO_MANIFEST_DIR").unwrap();
    let src = Path::new(&manifest).join("../system/system.foo");
    let out = Path::new(&env::var("OUT_DIR").unwrap()).join("system.foo");
    fs::copy(&src, &out).expect("copy system/system.foo into OUT_DIR");
    println!("cargo:rerun-if-changed=../system/system.foo");
}
