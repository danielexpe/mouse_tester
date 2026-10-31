#!/bin/sh
set -eu

PREFIX=${PREFIX:-/usr/local}
DESTDIR=${DESTDIR:-}
SKIP_GROUPS=${SKIP_GROUPS:-0}
ADD_GROUP=0

usage() {
    cat <<'USAGE'
Uso: ./install.sh [opções]

  --prefix DIR        instala sob DIR (padrão: /usr/local)
  --skip-groups       não mostra/altera instruções de grupo
  --add-input-group   tenta adicionar o usuário ao grupo input
  -h, --help          mostra esta ajuda

Variáveis: PREFIX, DESTDIR, SKIP_GROUPS=1
USAGE
}

while [ $# -gt 0 ]; do
    case "$1" in
        --prefix)
            [ $# -ge 2 ] || { echo "install.sh: --prefix exige um diretório" >&2; exit 2; }
            PREFIX=$2
            shift 2
            ;;
        --skip-groups)
            SKIP_GROUPS=1
            shift
            ;;
        --add-input-group)
            ADD_GROUP=1
            shift
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        *)
            echo "install.sh: opção desconhecida: $1" >&2
            usage >&2
            exit 2
            ;;
    esac
done

SCRIPT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)

if [ -x "$SCRIPT_DIR/mouse_tester" ]; then
    BIN="$SCRIPT_DIR/mouse_tester"
elif [ -x "$SCRIPT_DIR/target/release/mouse_tester" ]; then
    BIN="$SCRIPT_DIR/target/release/mouse_tester"
else
    BIN=$(command -v mouse_tester 2>/dev/null || true)
fi

if [ -z "$BIN" ] || [ ! -x "$BIN" ]; then
    echo "install.sh: binário mouse_tester não encontrado" >&2
    echo "compile com 'cargo build --release' ou execute dentro do pacote" >&2
    exit 1
fi

DEST="$DESTDIR$PREFIX"
BINDIR="$DEST/bin"
DOCDIR="$DEST/share/doc/mouse_tester"
SHAREDIR="$DEST/share/mouse_tester"

install -d "$BINDIR"
install -m 0755 "$BIN" "$BINDIR/mouse_tester"
install -d "$DOCDIR" "$SHAREDIR"

for file in README.md LICENSE THIRD_PARTY_NOTICES.md RELEASE_NOTES.md; do
    if [ -f "$SCRIPT_DIR/$file" ]; then
        install -m 0644 "$SCRIPT_DIR/$file" "$DOCDIR/$file"
    fi
done
if [ -d "$SCRIPT_DIR/docs" ]; then
    cp -R "$SCRIPT_DIR/docs" "$DOCDIR/docs"
fi
if [ -d "$SCRIPT_DIR/licenses" ]; then
    cp -R "$SCRIPT_DIR/licenses" "$DOCDIR/licenses"
fi
if [ -f "$SCRIPT_DIR/testes_debug.sh" ]; then
    install -m 0755 "$SCRIPT_DIR/testes_debug.sh" "$SHAREDIR/testes_debug.sh"
fi

echo "instalado: $BINDIR/mouse_tester"
echo "docs:     $DOCDIR"
echo "debug:    $SHAREDIR/testes_debug.sh"

if [ "$SKIP_GROUPS" != 1 ]; then
    cat <<'WARN'
AVISO: ler /dev/input/event* normalmente exige o grupo `input`.
Qualquer processo nesse grupo pode ler teclado (keylogging). Prefira conceder
acesso apenas quando necessário.

Para conceder acesso manualmente:
  sudo usermod -aG input "$USER"
  # faça logout e login novamente

Para framebuffer, o usuário precisa de acesso a /dev/fb0 (tipicamente grupo
`video`) e deve executar o app em um VT local.
WARN

    if [ "$ADD_GROUP" = 1 ]; then
        TARGET_USER=${SUDO_USER:-}
        if [ "$(id -u)" -eq 0 ]; then
            if [ -z "$TARGET_USER" ] || [ "$TARGET_USER" = root ]; then
                echo "install.sh: rode com sudo a partir do usuário alvo ou use usermod manualmente" >&2
            else
                usermod -aG input "$TARGET_USER"
                echo "usuário $TARGET_USER adicionado ao grupo input; faça logout/login."
            fi
        else
            echo "execute: sudo usermod -aG input $(id -un)"
            echo "depois faça logout e login."
        fi
    else
        echo "Nenhum grupo foi alterado. Use --add-input-group ou o comando acima."
    fi
else
    echo "Grupos não alterados (--skip-groups)."
fi
