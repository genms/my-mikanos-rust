use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    println!("cargo:rustc-link-search=native={}", out_dir);

    build_hankaku();
    build_asm();
    build_driver();
}

fn build_hankaku() {
    let out_dir = env::var("OUT_DIR").unwrap();

    println!("cargo:rustc-link-lib=hankaku");
    println!("cargo:rerun-if-changed=hankaku");

    Command::new("hankaku/makefont.py")
        .arg("-o")
        .arg(&format!("{}/hankaku.bin", out_dir))
        .arg("hankaku/hankaku.txt")
        .status()
        .unwrap();

    Command::new("objcopy")
        .args(&["-I", "binary", "-O", "elf64-x86-64", "-B", "i386:x86-64"])
        .arg("hankaku.bin")
        .arg("hankaku.o")
        .current_dir(&Path::new(&out_dir))
        .status()
        .unwrap();

    Command::new("ar")
        .arg("rcs")
        .arg("libhankaku.a")
        .arg("hankaku.o")
        .current_dir(&Path::new(&out_dir))
        .status()
        .unwrap();
}

fn build_asm() {
    let out_dir = env::var("OUT_DIR").unwrap();

    println!("cargo:rustc-link-lib=asm");
    println!("cargo:rerun-if-changed=asm");

    Command::new("nasm")
        .args(&["-f", "elf64", "-o"])
        .arg(&format!("{}/asmfunc.o", out_dir))
        .arg("asm/asmfunc.asm")
        .status()
        .unwrap();

    Command::new("ar")
        .arg("rcs")
        .arg("libasm.a")
        .arg("asmfunc.o")
        .current_dir(&Path::new(&out_dir))
        .status()
        .unwrap();
}

fn build_driver() {
    let out_dir = env::var("OUT_DIR").unwrap();

    println!("cargo:rustc-link-lib=driver");
    println!("cargo:rerun-if-changed=driver");

    Command::new("make")
        .current_dir(&Path::new("driver"))
        .status()
        .unwrap();

    fs::copy("driver/libdriver.a", out_dir + "/libdriver.a").unwrap();
}
