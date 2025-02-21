use std::env;
use std::process::Command;

fn main() {
    if env::var("SERVER_PATH").is_ok() {
        return;
    }

    let out_dir: String = env::var("OUT_DIR").unwrap();

    // Build ETSManager for wine
    let status = Command::new("winebuild")
        .arg("--def")
        .arg("-E")
        .arg("./server/ETSManager.spec")
        .arg("-o")
        .arg(format!("{}/libETSManager.def", &out_dir))
        .status()
        .expect("Failed to winebuild");
    assert!(status.success());

    let status = Command::new("winegcc")
        .arg("-m32")
        .arg("./server/main.c")
        .arg("-L")
        .arg(&out_dir)
        .arg("-lETSManager")
        .arg("-o")
        .arg(format!("{}/server.exe.so", &out_dir))
        .status()
        .expect("Failed to winegcc");
    assert!(status.success());

    println!("cargo:rustc-env=SERVER_PATH={}/server.exe.so", &out_dir);
}
