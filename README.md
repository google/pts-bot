# PTS-bot

See [overview.md](doc/overview.md) for more details

## Build

```
git submodules update --init
cargo build
```

## Build for Ubuntu 18.04

```bash
DOCKER_BUILDKIT=1 docker build -f script/build-ubuntu-18.04 -o out .
```
