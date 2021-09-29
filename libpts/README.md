# libPTS

libPTS library runs test suites from the [Bluetooth SIG Profile Tuning Suite (PTS)](https://www.bluetooth.com/develop-with-bluetooth/qualification-listing/qualification-test-tools/profile-tuning-suite/) with extra features:
- Run on non-Windows platforms via Wine
- Work in a headless and automated mode
- Expose the HCI stream

## Getting Started

### Prerequisites

libPTS uses [Bazel](https://bazel.build/) as it's build system and need wine to run
```
sudo apt install bazel wine
```

### Running RootCanal

[RootCanal](https://android.googlesource.com/platform/system/bt/+/refs/heads/master/vendor_libs/test_vendor_lib/) is a virtual bluetooth controller. It can be built as follow

```bash
sudo apt install flex bison
bazel build :root-canal
```

### Running libPTS "demo" binary

A running RootCanal instance is required to run the binary.

```bash
bazel run :root-canal
```

You can start the binary as follow

```bash
bazel run :eiffel -- --eiffel true --profile A2DP
```

`true` in the previous command is the Device Under Test (DUT) binary to be used

### Use Eiffel as DUT

```bash
sudo apt install bash-builtins
git clone sso://eiffel/host
cd host
git fetch sso://eiffel/host refs/changes/07/4807/1 && git checkout -b change-4807 FETCH_HEAD
./Taskfile tool -w --device=rootcanal pts
cd $LIBPTS_DIR
bazel run :eiffel -- --eiffel $EIFFELHOST_DIR/target/posix-gcc-debug/pts --profile A2DP
```

## Snippets

### Run tests

```bash
bazel test :libpts_test --test_output=all
```

### Format

```bash
env RULES_RUST_CRATE_UNIVERSE_BOOTSTRAP=true bazel run @rules_rust//:rustfmt
```
