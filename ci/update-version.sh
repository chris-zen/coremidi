#!/bin/bash

set -ex

VERSION=${TRAVIS_TAG:-$(git describe --tags)}

sed -i '' "s/version = \"0.0.0\"/version = \"$VERSION\"/g" Cargo.toml
