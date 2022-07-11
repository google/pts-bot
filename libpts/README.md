# libPTS

libPTS library runs test suites from the [Bluetooth SIG Profile Tuning Suite (PTS)](https://www.bluetooth.com/develop-with-bluetooth/qualification-listing/qualification-test-tools/profile-tuning-suite/) with extra features:
- Run on non-Windows platforms via Wine
- Work in a headless and automated mode
- Expose the HCI stream

## Getting Started

### Prerequisites

libPTS uses [Bazel](https://bazel.build/) as it's build system and need wine, cabextract and xvfb to run
```
sudo apt install bazel wine wine32-tools cabextract xvfb
```

## Snippets

### Run tests

```bash
bazel test :libpts_test --test_output=all
```

### Format

```bash
bazel run :format
```
