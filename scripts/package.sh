version=$(grep ^version Cargo.toml | head -n1 | cut -d'"' -f2)
arch=$(uname -m)
kernel=$(uname -s)
mkdir -p dist
cp target/release/dir-nuke dist/dir-nuke

cd dist
tar -czf dir-nuke-v${version}-${arch}-${kernel}.tar.gz dir-nuke
cd ..

