#!/bin/sh

set -e

: ${TRAVIS:?'This should only be run on Travis CI'}
GITHUB_TOKEN=${1:?'Must provide github token'}
REPO_SLUG=${2:?'Must provide repo slug'}

echo "machine github.com login $GITHUB_TOKEN password x-oauth-basic" >> ~/.netrc
chmod 0600 ~/.netrc
git remote set-url --push origin "https://github.com/$REPO_SLUG"
