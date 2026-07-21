# Maintainer: Your Name <your.email@example.com>
pkgname=noti
pkgver=0.1.0
pkgrel=1
pkgdesc="Minimal, keyboard-first Markdown note editor"
arch=('x86_64')
url="https://github.com/L3-N0X/noti"
license=('MIT')
depends=('gtk4' 'libadwaita' 'gtksourceview5')
makedepends=('cargo')
source=("${pkgname}-${pkgver}.tar.gz::https://github.com/L3-N0X/noti/archive/v${pkgver}.tar.gz")
sha256sums=('SKIP')

build() {
  cd "${pkgname}-${pkgver}"
  cargo build --release --locked
}

package() {
  cd "${pkgname}-${pkgver}"
  install -Dm755 "target/release/${pkgname}" "${pkgdir}/usr/bin/${pkgname}"
  install -Dm644 "resources/io.github.L3-N0X.noti.desktop" "${pkgdir}/usr/share/applications/io.github.L3-N0X.noti.desktop"
  install -Dm644 "resources/io.github.L3-N0X.noti.svg" "${pkgdir}/usr/share/icons/hicolor/scalable/apps/io.github.L3-N0X.noti.svg"
  install -Dm644 "resources/io.github.L3-N0X.noti.metainfo.xml" "${pkgdir}/usr/share/metainfo/io.github.L3-N0X.noti.metainfo.xml"
}

