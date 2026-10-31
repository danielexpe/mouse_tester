# 10 — Checklist de release

> Nenhum item é marcado por suposição. Itens não executados ficam como
> **pendentes**, com motivo. Hardware ausente não é “validado”.

## 1. Funcional

- [ ] Dashboard completo em TUI com evdev real.
- [ ] Movimento, rastro, velocidade e counts corretos.
- [ ] Botões L/R/M, laterais, extras e desconhecidos.
- [ ] Scroll vertical/horizontal e hi-res sem dupla contagem.
- [ ] Taxa estimada, jitter e confiança.
- [ ] Desafio de botões conclui e lista pendentes.
- [ ] Desafio de scroll conclui as direções suportadas.
- [ ] Alvos/arraste/duplo clique (Fase 3), se no escopo.
- [ ] Log bruto limitado, sem teclado.
- [ ] Seletor de dispositivo e hotplug.
- [ ] Modo degradado com `crossterm` sinalizado.
- [ ] `--require-evdev` retorna 4 quando aplicável.
- [ ] Captura PNG determinística.
- [ ] `--list-devices` e `--diagnostics`.
- [ ] i18n pt-BR/en em todas as telas.
- [ ] `--no-animations`, `--no-trail`, `NO_COLOR`, mono.
- [ ] `Q`/`Esc`/`Ctrl+C` restauram o terminal.
- [ ] Binário funciona copiado para `/tmp`, sem assets externos.

## 2. Framebuffer

- [ ] `--backend fb` valida `/dev/fb0`, VT, permissão e formato.
- [ ] `tty` = `/dev/ttyN`, não `/dev/pts/N`.
- [ ] `/sys/class/graphics/fb0/blank` chega a `0`; primeira tela aparece sem
      `Ctrl+C`, inclusive com console previamente blanked/DPMS off.
- [ ] Cursor software visível e clampado nas bordas.
- [ ] `yoffset`, `yres_virtual`, `ypanstep`, `line_length`, `smem_len`
      registrados.
- [ ] Pan honrado; fallback para buffer visível quando não honrado.
- [ ] Falha de `FBIOPAN_DISPLAY` não é fatal.
- [ ] 1920×1080 32 bpp validado.
- [ ] 24/16 bpp validados ou **pendentes** com motivo.
- [ ] `KD_TEXT`, cursor e limpeza restaurados em saída normal, erro, `Ctrl+C`,
      SIGTERM e pânico.
- [ ] Nenhum resíduo visual ou cursor piscando.
- [ ] `testes_debug.sh` executado em VT e anexado quando houver bug.

## 3. Qualidade

- [ ] `cargo fmt --check`.
- [ ] `cargo clippy --workspace --all-targets -- -D warnings`.
- [ ] `cargo clippy --workspace --all-targets --no-default-features -- -D warnings`.
- [ ] `cargo test`.
- [ ] `cargo test --no-default-features`.
- [ ] `cargo test --release`.
- [ ] `cargo xtask assets --check`.
- [ ] `cargo build --release`.
- [ ] `cargo build --release --no-default-features`.
- [ ] MSRV verificada, se possível.
- [ ] Snapshots revisados (versão no rodapé pode mudar).
- [ ] Nenhum `unsafe` fora de `platform/`.
- [ ] Testes de ABI de evdev/fb verdes.
- [ ] Overflow, `SYN_DROPPED` e dispositivo removido testados.
- [ ] Nenhum keycode de teclado em log/teste.

## 4. Manual

- [ ] Terminal(is) principais: GNOME Terminal, Konsole, Alacritty, Kitty, xterm.
- [ ] SSH em modo degradado.
- [ ] 80×24, 100×30, 120×36 e Minimal.
- [ ] Mouse USB básico.
- [ ] Mouse com botões laterais.
- [ ] Mouse com wheel horizontal.
- [ ] Mouse hi-res.
- [ ] 125 Hz e 1000 Hz (ou taxas disponíveis).
- [ ] Mouse Bluetooth/combo sem gravar teclas.
- [ ] Hotplug: conectar, remover, remover primário.
- [ ] Grab ligado/desligado, saída e SIGTERM.
- [ ] Permissão negada em TUI e fb.
- [ ] Sem mouse: espera e hotplug.
- [ ] Idiomas, reset, ajuda, troca de modo.
- [ ] Captura PNG e diagnóstico.
- [ ] Console: resolução e bpp alvo.
- [ ] Troca de VT; registrar limitações.
- [ ] Nenhum terminal/console corrompido.

## 5. Documentação e licenças

- [ ] `README.md` sem drift com CLI/atalhos.
- [ ] `docs/01`..`docs/10` atualizados.
- [ ] `AGENTS.md` com status, pendências, data e registro.
- [ ] `LICENSE` (MIT).
- [ ] `THIRD_PARTY_NOTICES.md` com todas as crates.
- [ ] `assets/fonts/OFL.txt`.
- [ ] Créditos de arte (CC BY 4.0) e fonte.
- [ ] Limitações de DPI/polling e de TUI/fb documentadas.
- [ ] Matriz manual com resultados e pendências.

## 6. Pacote e release

- [ ] Versão em `Cargo.toml`/`Cargo.lock` congelada.
- [ ] Pacote: binário, README, LICENSE, notices, OFL, notas, créditos,
      `install.sh`, `testes_debug.sh`.
- [ ] `tar.gz` + `.sha256`.
- [ ] Smoke test em `/tmp` sem assets.
- [ ] `install.sh` testado com `PREFIX=/tmp/... SKIP_GROUPS=1`.
- [ ] `install.sh` testado com sudo/grupos reais (ou pendente com motivo).
- [ ] Aviso de segurança do grupo `input` no instalador/docs.
- [ ] Checksum validado no diretório do pacote.
- [ ] Notas de release **sem** o hash do próprio pacote.
- [ ] Tag e release com notas + artefatos, se solicitado.
- [ ] Hash e URL registrados em `AGENTS.md`.

## 7. Critérios de bloqueio

Não publicar se:

- [ ] TUI corrompe terminal em algum caminho.
- [ ] fb deixa console em `KD_GRAPHICS`/cursor escondido.
- [ ] Algum portão automatizado falha.
- [ ] Binário depende da pasta `assets/`.
- [ ] Licença/aviso de terceiro ausente.
- [ ] Há overclaim de DPI/polling ou validação de hardware não executada.
- [ ] Privacidade: teclado lido/armazenado.
