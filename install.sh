set -e

cargo build --release
cp target/release/jired ~/.local/bin/jired
