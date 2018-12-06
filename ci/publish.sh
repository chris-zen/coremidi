#!/bin/bash

set -ex

cargo login $CRATES_TOKEN
cargo publish --dry-run
