#!/usr/bin/env bash

sed -i '/libpts/s/^/#/' Cargo.toml
libpts/script/vendor.sh
sed -i '/libpts/s/^#//' Cargo.toml
