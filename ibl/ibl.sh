#!/usr/bin/env bash

ROOT=$(cd -- "$(dirname -- "${BASH_SOURCE[0]:-$0}")" && pwd)
docker build -t ibl "$ROOT"
docker run --init -it --rm -v "$ROOT/..:/terp3d" ibl "$@"
