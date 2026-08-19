#!/usr/bin/env bash
# Build a native Debian/Ubuntu package without requiring cargo-deb.
#
#   bash packaging/build-deb.sh
#   sudo apt install ./dist/etamil_0.2.0_amd64.deb

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CRATE="$ROOT/etamil_compiler"
VERSION="$(grep -m1 '^version' "$CRATE/Cargo.toml" | sed 's/.*"\(.*\)".*/\1/')"
ARCH="$(dpkg --print-architecture)"
PACKAGE="etamil_${VERSION}_${ARCH}"
DIST="$ROOT/dist"
STAGE="$(mktemp -d)"
trap 'rm -rf "$STAGE"' EXIT

cd "$CRATE"
cargo build --release

mkdir -p \
    "$STAGE/DEBIAN" \
    "$STAGE/usr/bin" \
    "$STAGE/usr/share/etamil/nUlakam" \
    "$STAGE/usr/share/etamil/examples"

install -m 755 "$CRATE/target/release/etamil" "$STAGE/usr/bin/etamil"
cp -r "$ROOT/nUlakam/." "$STAGE/usr/share/etamil/nUlakam/"
cp -r "$ROOT/examples/." "$STAGE/usr/share/etamil/examples/"

cat >"$STAGE/DEBIAN/control" <<EOF
Package: etamil
Version: $VERSION
Section: devel
Priority: optional
Architecture: $ARCH
Maintainer: eTamil contributors
Description: Tamil programming language compiler
 Bilingual Tamil/English compiler and VM for FinTech programs.
Depends: libc6, libgcc-s1
EOF

rm -f "$DIST/$PACKAGE.deb"
mkdir -p "$DIST"
dpkg-deb --build --root-owner-group "$STAGE" "$DIST/$PACKAGE.deb" >/dev/null

echo "$DIST/$PACKAGE.deb"
ls -lh "$DIST/$PACKAGE.deb"
sha256sum "$DIST/$PACKAGE.deb" | tee "$DIST/$PACKAGE.deb.sha256"