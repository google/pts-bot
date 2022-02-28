#!/usr/bin/env bash

export KEEP=fastrand

sed -i '/libpts/s/^/#/' Cargo.toml
libpts/script/vendor.sh
sed -i '/libpts/s/^#//' Cargo.toml
