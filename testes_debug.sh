#!/bin/sh
# Coleta ambiente e estado do mouse/framebuffer em um único arquivo.
# Não instala nada e não requer root. Execute de um VT para o fb ser relevante.

set -u

WITH_FB=0
if [ "${1:-}" = "--with-fb" ]; then
    WITH_FB=1
    shift
fi
OUT=${1:-testes_debug.txt}
BIN=${MOUSE_TEST_BIN:-}

if [ -z "$BIN" ]; then
    for candidate in ./target/release/mouse_test ./target/debug/mouse_test; do
        if [ -x "$candidate" ]; then
            BIN=$candidate
            break
        fi
    done
fi
if [ -z "$BIN" ]; then
    BIN=$(command -v mouse_test 2>/dev/null || true)
fi

say() {
    printf '%s\n' "$*" >>"$OUT"
}

progress() {
    printf 'testes_debug: %s\n' "$*" >&2
}

section() {
    say ""
    say "==== $* ===="
}

run() {
    say "\$ $*"
    "$@" >>"$OUT" 2>&1 || say "(exit $?)"
}

run_bounded() {
    say "\$ $*"
    timeout -k 2 8 "$@" >>"$OUT" 2>&1 || say "(exit $?)"
}

: >"$OUT"
say "mouse_test debug report"
say "date=$(date -Is 2>/dev/null || date)"
say "cwd=$(pwd)"
say "binary=${BIN:-not found}"

section "os-release"
if [ -r /etc/os-release ]; then cat /etc/os-release >>"$OUT"; else say "missing"; fi

section "kernel and session"
progress "sistema e sessão"
run uname -a
run id
run_bounded loginctl
run sh -c 'cat /proc/self/status | grep -E "^(Name|Pid|PPid):"'
run sh -c 'readlink -f /proc/self/fd/0'
run sh -c 'cat /sys/class/tty/tty0/active 2>/dev/null || true'
if [ -r /dev/tty ] && [ -t 0 ]; then
    run tty
else
    say "stdin is not a TTY"
fi
case $(tty 2>/dev/null || true) in
    /dev/tty[0-9]*) say "vt=yes" ;;
    *) say "vt=no (execute este script em um VT local para testar o fb)" ;;
esac

progress "grupos e permissões"
section "groups and permissions"
run sh -c 'id -nG'
run sh -c 'ls -l /dev/input/event* 2>/dev/null || true'
run sh -c 'ls -l /dev/fb* 2>/dev/null || true'
run sh -c 'ls -l /dev/dri 2>/dev/null || true'

progress "dispositivos de entrada"
section "input devices"
if [ -r /proc/bus/input/devices ]; then
    cat /proc/bus/input/devices >>"$OUT"
else
    say "cannot read /proc/bus/input/devices"
fi
for path in /sys/class/input/event*; do
    [ -e "$path" ] || continue
    say "--- $path ---"
    for file in name phys uniq; do
        [ -r "$path/$file" ] && say "$file=$(cat "$path/$file")"
    done
done

progress "framebuffer"
section "framebuffer"
if [ -r /proc/fb ]; then cat /proc/fb >>"$OUT"; else say "/proc/fb unavailable"; fi
for path in /sys/class/graphics/fb*; do
    [ -e "$path" ] || continue
    say "--- $path ---"
    for file in name virtual_size stride bits_per_pixel blank state; do
        [ -r "$path/$file" ] && say "$file=$(cat "$path/$file")"
    done
done
if command -v fbset >/dev/null 2>&1; then
    run_bounded fbset -i
else
    say "fbset not installed"
fi
if command -v dmesg >/dev/null 2>&1; then
    run_bounded sh -c 'dmesg | tail -n 80'
else
    say "dmesg unavailable"
fi

if [ -n "$BIN" ] && [ -x "$BIN" ]; then
    progress "coletando mouse_test"
    section "mouse_test"
    run "$BIN" --version
    run "$BIN" --list-devices --all-devices
    TMP=$(mktemp 2>/dev/null || echo "/tmp/mouse_test_diag.$$")
    if "$BIN" --diagnostics "$TMP" >>"$OUT" 2>&1; then
        say "--- diagnostics ---"
        cat "$TMP" >>"$OUT"
    else
        say "diagnostics failed (exit $?)"
    fi
    rm -f "$TMP"
    if [ "$WITH_FB" = 1 ]; then
        progress "preflight do framebuffer (até 5 s)"
        section "framebuffer preflight"
        if timeout -k 2 5 "$BIN" --backend fb --demo >>"$OUT" 2>&1; then
            say "fb preflight exited 0"
        else
            say "fb preflight exit=$? (124=timeout; 3=sem VT; 4=sem entrada)"
        fi
        stty sane 2>/dev/null || true
    else
        section "framebuffer preflight"
        say "skipped; use --with-fb para executar o backend fb por até 5 s"
    fi
else
    section "mouse_test"
    say "binary not found; build with cargo build --release"
fi

say ""
say "report written to $OUT"
progress "relatório concluído"
printf 'report: %s\n' "$OUT"
