load("@rules_rust//rust:rust.bzl", "rust_binary")

alias(
    name = "root-canal",
    actual = "@aosp_bt//vendor_libs/test_vendor_lib:root-canal",
)

rust_binary(
    name = "pts-bot",
    edition = "2018",
    srcs = [
      "src/main.rs",
      "src/mmi2grpc.rs",
    ],
    deps = [
      "@libpts",
      "//third_party/cargo:serde",
      "//third_party/cargo:serde_json",
      "//third_party/cargo:anyhow",
      "//third_party/cargo:structopt",
      "//third_party/cargo:termion",
      "//third_party/cargo:dirs",
      "//third_party/cargo:pyo3",
    ]
)

load("@rules_rust//rust:toolchain.bzl", "rust_toolchain", "rust_stdlib_filegroup")

genrule(
    name = "host_rustc",
    executable = True,
    outs = ["rustc"],
    cmd_bash = "ln -s $$(which rustc) $@",
)

genrule(
    name = "host_rustc_lib",
    outs = [ "rustc_lib" ],
    cmd_bash = """
        mkdir -p $@
        for file in \
            $$(rustc --print sysroot)/lib/*.so \
            $$(rustc --print sysroot)/lib/rustlib/x86_64-unknown-linux-gnu/bin/rust-lld
        do
            ln -s $$file $@/$$(basename $$file)
        done
    """,
)

genrule(
    name = "host_rust_lib_files",
    outs = [ "rust_lib" ],
    cmd_bash = """
        mkdir -p $@
        for file in $$(rustc --print sysroot)/lib/rustlib/x86_64-unknown-linux-gnu/lib/*.{rlib,so,a}; do
            ln -s $$file $@/$$(basename $$file)
        done
    """,
)

rust_stdlib_filegroup(
    name = "host_rust_lib",
    srcs = [":host_rust_lib_files"],
)

rust_toolchain(
    name = "host_rust_linux_x86_64_impl",
    rustc = ":host_rustc",
    rustc_lib = ":host_rustc_lib",
    rust_lib = ":host_rust_lib",
    binary_ext = "",
    staticlib_ext = ".a",
    dylib_ext = ".so",
    stdlib_linkflags = ["-lpthread", "-ldl"],
    os = "linux",
    target_triple = "x86_64-unknown-linux-gnu",
    exec_triple = "x86_64-unknown-linux-gnu",
)

toolchain(
    name = "host_rust_linux_x86_64",
    exec_compatible_with = [
        "@platforms//cpu:x86_64",
        "@platforms//os:linux",
    ],
    target_compatible_with = [
        "@platforms//cpu:x86_64",
        "@platforms//os:linux",
    ],
    toolchain = ":host_rust_linux_x86_64_impl",
    toolchain_type = "@rules_rust//rust:toolchain",
)
