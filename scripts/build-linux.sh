version=$(grep ^version Cargo.toml | head -n1 | cut -d'"' -f2)
mkdir -p dist
cp target/release/dir-nuke dist/dir-nuke

cd dist
tar -czf dir-nuke-v${version}-x86_64-unknown-linux-gnu.tar.gz dir-nuke
cd ..

