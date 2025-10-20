# Maintainer: hotaq <hootoo2016@gmail.com>
pkgname=sprite-git
pkgver=0.1.0.r0.g69eac90
pkgrel=1
pkgdesc="A robust command-line toolkit for managing multiple AI coding agents in isolated tmux sessions"
arch=('x86_64')
url="https://github.com/hotaq/Sprit-mutil"
license=('MIT')
depends=('tmux' 'git')
makedepends=('cargo' 'git')
provides=('sprite')
conflicts=('sprite')
source=('git+https://github.com/hotaq/Sprit-mutil.git')
sha256sums=('SKIP')

pkgver() {
  cd Sprit-mutil
  git describe --long --tags --abbrev=7 | sed 's/^v//;s/\([^-]*-g\)/r\1/;s/-/./g'
}

prepare() {
  cd Sprit-mutil
  export RUSTUP_TOOLCHAIN=stable
  cargo fetch --locked --target "$(rustc -vV | sed -n 's/host: //p')"
}

build() {
  cd Sprit-mutil
  export RUSTUP_TOOLCHAIN=stable
  export CARGO_TARGET_DIR=target
  cargo build --frozen --release --all-features
}

check() {
  cd Sprit-mutil
  export RUSTUP_TOOLCHAIN=stable
  cargo test --frozen --all-features
}

package() {
  cd Sprit-mutil
  install -Dm0755 -t "$pkgdir/usr/bin/" "target/release/sprite"
  install -Dm0644 -t "$pkgdir/usr/share/licenses/$pkgname/" LICENSE
  install -Dm0644 -t "$pkgdir/usr/share/man/man1/" sprite.1
  install -Dm0644 -t "$pkgdir/usr/share/doc/$pkgname/" README.md
  install -Dm0644 -t "$pkgdir/usr/share/doc/$pkgname/" CHANGELOG.md
}