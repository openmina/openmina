#!/usr/bin/env bash

FILES=(`find . -name Cargo.toml`)

if [ -z "$1" ]; then
  echo "No version specified! e.g.: ./versions.sh 0.0.2"
  exit 1
fi
VERSION=$1
version_prefix="version = "

echo "Setting version to: '$VERSION'"
for file in "${FILES[@]}"
do
  # This version is handled manually
  if [ "$file" = "./mina-p2p-messages/Cargo.toml" ]; then
    continue
  fi
  old_version=$(grep -m 1 ^"$version_prefix" "$file")
  if [ -z "$old_version" ]; then
    continue
  fi

  new_version="version = \"$VERSION\""

  sed -i '' "s/^$old_version/$new_version/g" "$file"

  version_after=$(grep ^version "$file" | tr -d "$version_prefix")

  version_before="${old_version/#$version_prefix}"
  version_before="${version_before//\"}"

  echo "$file $version_before -> $version_after"
done
