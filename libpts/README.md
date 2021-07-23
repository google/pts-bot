# libPTS

libPTS library runs test suites from the [Bluetooth SIG Profile Tuning Suite (PTS)](https://www.bluetooth.com/develop-with-bluetooth/qualification-listing/qualification-test-tools/profile-tuning-suite/) with extra features:
- Run on non-Windows platforms via Wine
- Work in a headless and automated mode
- Expose the HCI stream

## Getting Started

### Prerequisites

libPTS uses [Bazel](https://bazel.build/) as it's build system
```
sudo apt install bazel
```

### Building Wine from Source

On debian/gLinux the wine version shipped in the distribution is causing some issues.
If you are able to get a another wine version you may skip this step

```bash
sudo apt install flex bison libx11-dev:i386
git clone https://github.com/wine-mirror/wine
cd wine
./configure --without-freetype
make -j$(nproc)
sudo make install
```

### Adding a binary of the PTS Installer

To compile libPTS you need to add a `pts_setup_8_0_3.exe` file in the root of the project.
It can be downloaded from the [Bluetooth SIG website](https://apps.bluetooth.com/mysettings#/ptsdownload)

### Running RootCanal

[RootCanal](https://android.googlesource.com/platform/system/bt/+/refs/heads/master/vendor_libs/test_vendor_lib/) is a virtual bluetooth controller. It can be built as follow

```bash
sudo apt install flex bison
bazel build :root-canal
```

### Running libPTS "demo" binary

A running RootCanal instance is required to run the binary.
Warning:  You will need to restart it after each invocation of the binary

```bash
bazel run :root-canal
```

You can start the binary as follow

```bash
bazel run :pts true
```

`true` in the previous command is the Device Under Test (DUT) binary to be used

### Use Eiffel as DUT

```bash
sudo apt install bash-builtins
git clone sso://eiffel/host
cd host
git fetch sso://eiffel/host refs/changes/40/2340/1 && git checkout -b change-2340 FETCH_HEAD
./Taskfile tool -w --device=rootcanal pts
cd $LIBPTS_DIR
bazel run :pts $EIFFELHOST_DIR/target/posix-gcc-debug/pts
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
