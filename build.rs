
use std::process::Command;
use std::env;
use std::path::Path;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();

    Command::new("as")
        .args(&["src/x86_64.S", "-fPIC", "-o"])
        .arg(&format!("{}/trampoline.o", out_dir))
        .status()
        .unwrap();
    Command::new("ar")
        .args(&["crus", "libtrampoline.a", "trampoline.o"])
        .current_dir(&Path::new(&out_dir))
        .status()
        .unwrap();

    println!("cargo:rustc-link-search=native={}", out_dir);
    println!("cargo:rustc-link-lib=static=trampoline");
}
