// Copyright 2025 Google LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     https://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

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
