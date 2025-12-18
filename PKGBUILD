# Maintainer: Your Name <your.email@example.com>
pkgname=biu-git
_pkgname=biu-tauri
pkgver=1.6.0.beta.11.r476.ga7075c5 # Updated to match your current checkout
pkgrel=1
pkgdesc="A cross-platform music desktop application based on Bilibili API"
arch=('x86_64')
url="https://github.com/WMsans/biu-tauri"
license=('custom:PolyForm-Noncommercial-1.0.0')
depends=('gtk3' 'nss' 'alsa-lib' 'libxtst' 'libxss' 'xdg-utils')
makedepends=('git' 'pnpm' 'node-gyp' 'npm')
provides=("biu")
conflicts=("biu" "biu-bin")
source=("$_pkgname::git+https://github.com/WMsans/biu-tauri.git#branch=arch")
sha256sums=('SKIP')

pkgver() {
  cd "$srcdir/$_pkgname"
  
  local _rawver
  if git describe --long --tags >/dev/null 2>&1; then
    _rawver=$(git describe --long --tags | sed 's/^v//;s/\([^-]*-g\)/r\1/;s/-/./g')
  else
    local _ver=$(grep -m1 '"version":' package.json | cut -d '"' -f4)
    _rawver=$(printf "%s.r%s.g%s" "$_ver" "$(git rev-list --count HEAD)" "$(git rev-parse --short HEAD)")
  fi

  # Replace all hyphens with dots to satisfy Arch Linux versioning requirements
  echo "$_rawver" | tr '-' '.'
}

prepare() {
  cd "$srcdir/$_pkgname"

  if [ -f "plugins/electron-build.ts" ]; then
    sed -i 's/{ target: "deb",/{ target: "dir",/g' plugins/electron-build.ts
    sed -i '/target: "AppImage"/d' plugins/electron-build.ts
    sed -i '/target: "rpm"/d' plugins/electron-build.ts
  fi

  pnpm install
}

build() {
  cd "$srcdir/$_pkgname"
  pnpm build
}

package() {
  cd "$srcdir/$_pkgname"

  local _install_dir="/opt/biu"
  local _bin_dir="/usr/bin"

  install -d "$pkgdir$_install_dir"
  
  # Note: The arch branch of biu-tauri likely outputs to a different folder
  # than the electron version. Adjust this path if 'pnpm build' doesn't 
  # put files in dist/artifacts/linux-unpacked
  cp -r dist/artifacts/linux-unpacked/* "$pkgdir$_install_dir/" || true

  install -d "$pkgdir$_bin_dir"
  ln -s "$_install_dir/Biu" "$pkgdir$_bin_dir/biu"

  # Fallback for icon if the electron folder doesn't exist in the tauri branch
  if [ -f "electron/icons/logo.png" ]; then
    install -Dm644 "electron/icons/logo.png" "$pkgdir/usr/share/icons/hicolor/512x512/apps/biu.png"
  fi

  install -Dm644 /dev/stdin "$pkgdir/usr/share/applications/biu.desktop" <<EOF
[Desktop Entry]
Name=Biu
Comment=Bilibili music desktop application
Exec=biu %U
Terminal=false
Type=Application
Icon=biu
StartupWMClass=Biu
Categories=AudioVideo;Audio;Music;Player;
Keywords=music;bilibili;
EOF

  install -Dm644 LICENSE "$pkgdir/usr/share/licenses/$pkgname/LICENSE"
}
