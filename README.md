# PTS-bot

See [overview.md](doc/overview.md) for more details

## Build

```
git submodules update --init
cargo build
```

## Build for Ubuntu 22.04

```bash
DOCKER_BUILDKIT=1 docker build -f script/build-ubuntu-22.04 -o out .
```

## Release for gLinux

```bash
blaze run //goobuntu/benz/cli:benz -- \
    --server=blade:benz-prod build \
    --branch main \
    --target-prefix blueberry \
    --git rpc://blueberry/PTS-bot
```
