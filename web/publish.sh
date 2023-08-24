#!/bin/bash

set -e
cd $(dirname -- $(dirname -- $(readlink --canonicalize-existing -- $BASH_SOURCE)))
./web/build.sh
git subtree push --prefix web/src/ origin gh-pages
