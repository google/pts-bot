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
