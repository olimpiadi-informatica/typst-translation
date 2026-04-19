#!/bin/bash

DIR=$1

if [ -z "$DIR" ]; then
  echo "Usage: $0 <output_directory>" >&2
  exit 1
fi

set -xe

mkdir -p "$DIR"

pushd frontend
trunk build --release
popd

pushd backend
cargo build --release
popd

pushd frontend/dist
find . -type f -exec brotli -v {} \;
popd

rsync -avL backend/target/release/backend frontend/dist "$DIR"

if ! [ -f "$DIR/db.sqlite3" ]; then
  touch "$DIR/db.sqlite3"
fi

if ! [ -f "$DIR/.env" ]; then
  echo 'DATABASE_URL="sqlite:./db.sqlite"' >"$DIR/.env"
fi

set +e

echo "Done. After ensuring a configuration file is present, you can run the translation system from $DIR."
