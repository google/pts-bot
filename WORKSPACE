load("@bazel_tools//tools/build_defs/repo:git.bzl", "git_repository")

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

git_repository(
    name = "rules_proto",
    commit = "cfdc2fa31879c0aebe31ce7702b1a9c8a4be02d2",
    remote = "https://github.com/bazelbuild/rules_proto.git",
    shallow_since = "1610710171 +0100",
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