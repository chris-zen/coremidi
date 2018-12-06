#!/bin/bash

set -ex

VERSION=$(git describe --always --dirty=-dirty)

sed -i '' "s/version = \"0.0.0\"/version = \"$VERSION\"/g" Cargo.toml
