#!/bin/sh
set -eu

ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
cd "$ROOT"

VERSION=$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)
NAME="mouse_tester-$VERSION-linux-x86_64"
DIST="$ROOT/dist"
STAGE="$DIST/$NAME"

[ -n "$VERSION" ] || { echo "package.sh: versão não encontrada" >&2; exit 1; }
cargo build --release

rm -rf "$STAGE"
mkdir -p "$STAGE"
install -m 0755 target/release/mouse_tester "$STAGE/mouse_tester"
for file in README.md LICENSE THIRD_PARTY_NOTICES.md RELEASE_NOTES.md install.sh testes_debug.sh; do
    install -m 0644 "$file" "$STAGE/$file"
done
chmod 0755 "$STAGE/install.sh" "$STAGE/testes_debug.sh"
cp -R docs "$STAGE/docs"
cp -R licenses "$STAGE/licenses"

tar --sort=name --mtime='UTC 2026-10-31' --owner=0 --group=0 --numeric-owner \
    -cf - -C "$DIST" "$NAME" | gzip -n >"$DIST/$NAME.tar.gz"
(cd "$DIST" && sha256sum "$NAME.tar.gz" >"$NAME.tar.gz.sha256")

echo "pacote:  $DIST/$NAME.tar.gz"
echo "sha256:  $DIST/$NAME.tar.gz.sha256"
