pkgname=dir-nuke
pkgver=0.1.0
pkgrel=1
arch=('x86_64')
url="https://github.com/brunohaetinger/dir-nuke"
license=('MIT')
depends=()
makedepends=('cargo')
source=("$pkgname-$pkgver.tar.gz::https://github.com/brunohaetinger/dir-nuke/archive/refs/tags/v$pkgver.tar.gz")
sha256sums=('SKIP')

build() {
  cd "$srcdir/$pkgname-$pkgver"
  cargo build --release
}

package() {
  install -Dm755 "$srcdir/$pkgname-$pkgver/target/release/dir-nuke" "$pkgdir/usr/bin/dir-nuke"
}
