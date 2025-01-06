load("@bazel_tools//tools/build_defs/repo:git.bzl", "git_repository")

local_repository(
    name = "bazel_skylib",
    path = "libpts/third_party/bazel-skylib",
)

local_repository(
    name = "rules_rust",
    path = "libpts/third_party/rules_rust",
)

local_repository(
    name = "rules_java",
    path = "libpts/third_party/rules_java",
)

local_repository(
    name = "rules_cc",
    path = "libpts/third_party/rules_cc",
)

load("@rules_rust//rust:repositories.bzl", "rust_repository_set")

rust_repository_set(
    name = "rust_linux_x86_64",
    version = "1.54.0",
    rustfmt_version = "1.54.0",
    exec_triple = "x86_64-unknown-linux-gnu"
)

# libpts

local_repository(
    name = "libpts",
    path = "libpts",
)

# libpts deps

local_repository(
    name = "rules_foreign_cc",
    path = "libpts/third_party/rules_foreign_cc",
)

load("@rules_foreign_cc//foreign_cc:repositories.bzl", "rules_foreign_cc_dependencies")

rules_foreign_cc_dependencies(register_built_tools = False)

new_local_repository(
    name = "wine",
    path = "libpts/third_party/wine",
    build_file = "@libpts//:third_party/wine.BUILD",
)

# Root canal

git_repository(
    name = "aosp_bt",
    remote = "https://android.googlesource.com/platform/system/bt",
    # https://android-review.googlesource.com/c/platform/system/bt/+/1754311
    commit = "a4b8b58b27cf465fc8db6b2eca9d41d128632888",
    shallow_since = "1626957129 +0000",
    patch_cmds = [
        # https://android-review.googlesource.com/c/platform/system/bt/+/1772606
        "git fetch origin refs/changes/06/1772606/1 && git cherry-pick FETCH_HEAD",
    ],
)

# Root canal deps

local_repository(
    name = "rules_proto",
    path = "libpts/third_party/rules_proto",
)

load("@rules_proto//proto:repositories.bzl", "rules_proto_dependencies", "rules_proto_toolchains")
rules_proto_dependencies()
rules_proto_toolchains()

load("@com_google_protobuf//:protobuf_deps.bzl", "protobuf_deps")
protobuf_deps()

git_repository(
    name = "jsoncpp",
    commit = "375a1119f8bbbf42e5275f31b281b5d87f2e17f2",
    remote = "https://github.com/open-source-parsers/jsoncpp.git",
    shallow_since = "1620266582 -0500",
)

git_repository(
    name = "gflags",
    commit = "e171aa2d15ed9eb17054558e0b3a6a413bb01067",
    remote = "https://github.com/gflags/gflags.git",
    shallow_since = "1541971260 +0000",
)
