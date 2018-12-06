#!/bin/bash

set -ex

cargo doc

echo "<meta http-equiv=refresh content=0;url=`echo $TRAVIS_REPO_SLUG | cut -d '/' -f 2`/index.html>" > target/doc/index.html
