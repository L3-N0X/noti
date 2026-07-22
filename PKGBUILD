# Maintainer: L3-N0X <leon.goett@web.de>
pkgname=noti-notes
_binname=noti
pkgver=0.1.0
pkgrel=1
pkgdesc="Minimal, keyboard-first Markdown note editor for Linux/Hyprland"
arch=('x86_64' 'aarch64')
url="https://github.com/L3-N0X/noti"
license=('MIT')
depends=('gtk4' 'libadwaita' 'gtksourceview5')
makedepends=('cargo')
# Both packages install /usr/bin/noti.
conflicts=('noti')
source=("${pkgname}-${pkgver}.tar.gz::${url}/archive/refs/tags/v${pkgver}.tar.gz")
# Run `updpkgsums` after tagging a new release to refresh this.
sha256sums=('SKIP')

build() {
  cd "${_binname}-${pkgver}"
  cargo build --release --locked
}

package() {
  cd "${_binname}-${pkgver}"
  install -Dm755 "target/release/${_binname}" "${pkgdir}/usr/bin/${_binname}"
  install -Dm644 "resources/io.github.L3-N0X.noti.desktop" \
    "${pkgdir}/usr/share/applications/io.github.L3-N0X.noti.desktop"
  install -Dm644 "resources/io.github.L3-N0X.noti.svg" \
    "${pkgdir}/usr/share/icons/hicolor/scalable/apps/io.github.L3-N0X.noti.svg"
  install -Dm644 "resources/io.github.L3-N0X.noti.metainfo.xml" \
    "${pkgdir}/usr/share/metainfo/io.github.L3-N0X.noti.metainfo.xml"
  install -Dm644 "LICENSE" "${pkgdir}/usr/share/licenses/${pkgname}/LICENSE"
}

