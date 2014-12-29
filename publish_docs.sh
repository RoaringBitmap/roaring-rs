#!/bin/sh

set -e

: ${TRAVIS:?'This should only be run on Travis CI'}
TAG=${1:?'Must provide tag'}

# Setup this repo to publish the docs
git fetch origin gh-pages
git checkout -b gh-pages FETCH_HEAD

# Move the built docs into versioned folder
mv target/doc docs/$TAG

# Update the index to point to the versioned docs
sed -i '' -e '/<!-- DOCS INDEX -->/a\
<li><a href="docs/'"$TAG"'/roaring/">'"$TAG"'</a></li>' index.html

# Add the changes
git add docs/$TAG
git add index.html

# Commit and push
git commit -m "Add API docs for $TAG"
git push origin gh-pages:gh-pages
