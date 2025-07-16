VERSION=$(grep ^version Cargo.toml | head -n1 | cut -d'"' -f2)
ARCH=$(uname -m)
OS=$(uname -s)

# Determine the correct target triple
case "$OS" in
  Linux)
    case "$ARCH" in
      x86_64) TARGET="x86_64-unknown-linux-gnu" ;;
      aarch64) TARGET="aarch64-unknown-linux-gnu" ;;
      *) echo "Unsupported architecture: $ARCH"; exit 1 ;;
    esac
    ;;
  Darwin)
    case "$ARCH" in
      x86_64) TARGET="x86_64-apple-darwin" ;;
      arm64) TARGET="aarch64-apple-darwin" ;;
      *) echo "Unsupported architecture: $ARCH"; exit 1 ;;
    esac
    ;;
  *)
    echo "Unsupported OS: $OS"
    exit 1
    ;;
esac

mkdir -p dist
cp target/release/dir-nuke dist/dir-nuke

cd dist
tar -czf dir-nuke-v${VERSION}-${TARGET}.tar.gz dir-nuke
cd ..

