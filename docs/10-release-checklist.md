# 10 — Checklist de release

> Nenhum item é marcado por suposição. Itens não executados ficam como
> **pendentes**, com motivo. Hardware ausente não é “validado”.

## 1. Funcional

- [x] Dashboard completo em TUI com evdev real.
- [x] Movimento, rastro, velocidade e counts corretos.
- [x] Botões L/R/M, laterais, extras e desconhecidos.
- [x] Scroll vertical/horizontal e hi-res sem dupla contagem.
- [x] Taxa estimada, jitter e confiança.
- [x] Desafio de botões conclui e lista pendentes.
- [x] Desafio de scroll conclui as direções suportadas.
- [x] Alvos/arraste/duplo clique (Fase 3), se no escopo.
- [x] Log bruto limitado, sem teclado.
- [x] Seletor de dispositivo e hotplug.
- [x] Modo degradado com `crossterm` sinalizado.
- [x] `--require-evdev` retorna 4 quando aplicável.
- [x] Captura PNG determinística.
- [x] `--list-devices` e `--diagnostics`.
- [x] i18n pt-BR/en em todas as telas.
- [x] `--no-animations`, `--no-trail`, `NO_COLOR`, mono.
- [x] `Q`/`Esc`/`Ctrl+C` restauram o terminal.
- [x] Binário funciona copiado para `/tmp`, sem assets externos.

## 2. Framebuffer

- [x] `--backend fb` valida `/dev/fb0`, VT, permissão e formato.
- [x] `tty` = `/dev/ttyN`, não `/dev/pts/N`.
- [x] `/sys/class/graphics/fb0/blank` chega a `0`; primeira tela aparece sem
      `Ctrl+C`, inclusive com console previamente blanked/DPMS off.
- [x] Cursor software visível e clampado nas bordas.
- [x] `yoffset`, `yres_virtual`, `ypanstep`, `line_length`, `smem_len`
      registrados.
- [ ] Pan honrado; fallback para buffer visível quando não honrado.
- [x] Falha de `FBIOPAN_DISPLAY` não é fatal.
- [x] 1920×1080 32 bpp validado.
- [ ] 24/16 bpp validados ou **pendentes** com motivo.
- [ ] `KD_TEXT`, cursor e limpeza restaurados em saída normal, erro, `Ctrl+C`,
      SIGTERM e pânico.
- [x] Nenhum resíduo visual ou cursor piscando.
- [x] `testes_debug.sh` executado em VT e anexado quando houver bug.

## 3. Qualidade

- [x] `cargo fmt --check`.
- [x] `cargo clippy --workspace --all-targets -- -D warnings`.
- [x] `cargo clippy --workspace --all-targets --no-default-features -- -D warnings`.
- [x] `cargo test`.
- [x] `cargo test --no-default-features`.
- [x] `cargo test --release`.
- [x] `cargo xtask assets --check` — N/A: sem assets/xtask (D-016).
- [x] `cargo build --release`.
- [x] `cargo build --release --no-default-features`.
- [ ] MSRV 1.88 não instalada; build e testes executados em Rust 1.99.1.
- [x] Snapshots revisados (versão no rodapé pode mudar).
- [x] Nenhum `unsafe` fora de `platform/`.
- [x] Testes de ABI de evdev/fb verdes.
- [x] Overflow, `SYN_DROPPED` e dispositivo removido testados.
- [x] Nenhum keycode de teclado em log/teste.

## 4. Manual

- [x] Terminal(is) principais: GNOME Terminal, Konsole, Alacritty, Kitty, xterm.
- [ ] SSH em modo degradado.
- [ ] 80×24, 100×30, 120×36 e Minimal.
- [x] Mouse USB básico.
- [x] Mouse com botões laterais.
- [x] Mouse com wheel horizontal.
- [x] Mouse hi-res.
- [ ] 125 Hz e 1000 Hz (ou taxas disponíveis).
- [ ] Mouse Bluetooth/combo sem gravar teclas.
- [x] Hotplug: conectar, remover, remover primário.
- [ ] Grab ligado/desligado, saída e SIGTERM.
- [ ] Permissão negada em TUI e fb.
- [ ] Sem mouse: espera e hotplug.
- [x] Idiomas, reset, ajuda, troca de modo.
- [x] Captura PNG e diagnóstico.
- [x] Console: resolução e bpp alvo.
- [x] Troca de VT; registrar limitações.
- [x] Nenhum terminal/console corrompido.

## 5. Documentação e licenças

- [x] `README.md` sem drift com CLI/atalhos.
- [x] `docs/01`..`docs/10` atualizados.
- [x] `AGENTS.md` com status, pendências, data e registro.
- [x] `LICENSE` (MIT).
- [x] `THIRD_PARTY_NOTICES.md` com todas as crates.
- [x] `assets/fonts/OFL.txt` — N/A: sem fonte externa (D-016).
- [x] Créditos de arte (CC BY 4.0) e fonte.
- [x] Limitações de DPI/polling e de TUI/fb documentadas.
- [x] Matriz manual com resultados e pendências (`docs/11-manual-matrix.md`).

## 6. Pacote e release

- [x] Versão em `Cargo.toml`/`Cargo.lock` congelada.
- [x] Pacote: binário, README, LICENSE, notices, OFL, notas, créditos,
      `install.sh`, `testes_debug.sh`.
- [x] `tar.gz` + `.sha256`.
- [x] Smoke test em `/tmp` sem assets.
- [x] `install.sh` testado com `PREFIX=/tmp/... SKIP_GROUPS=1`.
- [ ] `install.sh` testado com sudo/grupos reais (ou pendente com motivo).
- [x] Aviso de segurança do grupo `input` no instalador/docs.
- [x] Checksum validado no diretório do pacote.
- [x] Notas de release **sem** o hash do próprio pacote.
- [x] Tag e release com notas + artefatos: v1.0.0.
- [x] Hash e URL registrados em `AGENTS.md`.

## 7. Critérios de bloqueio

Não publicar se:

- [ ] TUI corrompe terminal em algum caminho.
- [ ] fb deixa console em `KD_GRAPHICS`/cursor escondido.
- [ ] Algum portão automatizado falha.
- [ ] Binário depende da pasta `assets/`.
- [ ] Licença/aviso de terceiro ausente.
- [ ] Há overclaim de DPI/polling ou validação de hardware não executada.
- [ ] Privacidade: teclado lido/armazenado.
