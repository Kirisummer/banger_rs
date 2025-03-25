pkgname=banger_rs
pkgver=0.1.0
pkgrel=1
pkgdesc="Service that imitates DuckDuckGo's bangs"
arch=(any)
url='https://github.com/Kirisummer/banger_rs'
license=(MIT)
depends=(gcc-libs)
makedepends=(cargo)

prepare() {
    export RUSTUP_TOOLCHAIN=stable
    cargo fetch --locked --target "$(rustc -vV | sed -n 's/host: //p')"
}

build() {
    export RUSTUP_TOOLCHAIN=stable
    export CARGO_TARGET_DIR=../target
    cargo build --frozen --release
}

check() {
    export RUSTUP_TOOLCHAIN=stable
    cargo test --frozen
}

package() {
    install -Dm0755 -t "$pkgdir/usr/bin" "../target/release/$pkgname"
    pwd
    install -Dm644 ../LICENSE "${pkgdir}/usr/share/licenses/${pkgname}/LICENSE"
}
