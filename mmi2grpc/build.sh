#!/usr/bin/env bash

PYTHON_OUT="lib"
PROTO_FOLDER="proto"

mkdir ${PYTHON_OUT}

python3 -m grpc_tools.protoc \
    -I . \
    --python_out=${PYTHON_OUT} \
    --grpc_python_out=${PYTHON_OUT} \
    "${PROTO_FOLDER}/l2cap.proto" \
    "${PROTO_FOLDER}/neighbor.proto" \
    "${PROTO_FOLDER}/common.proto" \
    "${PROTO_FOLDER}/rootservice.proto"

touch "${PYTHON_OUT}/__init__.py"
touch "${PYTHON_OUT}/proto/__init__.py"