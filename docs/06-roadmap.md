# 06 — Roadmap

## 1. Regras

- Cada fase termina com critérios **verificáveis**; não avançar por otimismo.
- A ordem do framework é respeitada: TUI funcional antes do framebuffer.
- Nenhuma fase declara validação de hardware sem execução registrada.
- Decisões congeladas em `docs/07-decisions-and-risks.md`.
- O escopo só cresce com pedido explícito do usuário.

## 2. Visão geral

| Fase | Objetivo | Critério de saída |
| --- | --- | --- |
| 0 — Descoberta | escopo, arquitetura, riscos, licenças | specs revisadas e aprovadas |
| 1 — Fundação | crate, domínio puro, mock, CLI, TUI placeholder | testes de domínio + TUI abre/fecha |
| 2 — TUI funcional | evdev real, dashboard, modos, snapshots | fluxo completo + snapshots |
| 3 — Modos ricos | alvos/arraste/duplo clique, acessibilidade, captura | modos + captura determinística |
| 4 — Framebuffer | fbdev, cursor, sinais, validação em VT | fb validado ou pendência registrada |
| 5 — Release | pacote, instalador, diagnóstico, docs | release v1.0 validado |
| 6 — Backlog | DRM/KMS, hidraw, exportação, touchpad | priorizado pelo usuário |

## 3. Fase 0 — Descoberta e especificação

**Status:** concluída e aprovada em 2026-10-31.

Entregáveis:

- `README.md`, `AGENTS.md`;
- `docs/01`..`docs/10`;
- decisões D-001..D-015;
- questões Q-001..Q-008 com defaults.

Critérios de saída:

- [x] Usuário aprova escopo e nome provisório.
- [ ] Usuário confirma se quer release pública (Q-002; default: sim).
- [x] Usuário confirma hardware/distro alvo de fbdev (Q-004; Zorin OS 18.1, VT 1920×1080 32 bpp).
- [x] Nenhuma decisão congelada em conflito.
- [x] Nenhum código escrito na Fase 0.

## 4. Fase 1 — Fundação

**Status:** concluída em 2026-10-31.

Objetivo: provar o domínio sem hardware.

Entregáveis:

- `Cargo.toml`, `src/lib.rs`, `src/main.rs`, `src/cli.rs`;
- `core/pointer`, `core/buttons`, `core/scroll`, `core/effects`,
  `core/metrics`;
- `state/session`, `state/device`;
- fonte de entrada mock com timestamps;
- CLI com `--help`, `--version`, `--list-devices` (mock), `--capture-frame`
  ainda não;
- TUI placeholder que abre, mostra estado mock e fecha;
- guardas RAII de terminal;
- i18n com strings mínimas;
- testes de domínio e PTY básico;
- `AGENTS.md` atualizado.

Critérios de saída:

- [x] `cargo fmt`, `clippy`, `test` verdes (com e sem default features).
- [x] TUI abre e fecha sem corromper terminal (validada em PTY com `q`).
- [x] Eventos mock movem cursor, acendem botões, contam scroll.
- [x] Estimador de taxa com fixtures de 125/1000 Hz.
- [x] Sem evdev real e sem fb.
- [x] Nenhum `unsafe` no código.
- [x] 78 testes (74 unitários + 4 de integração de CLI).

Evidência: `cargo test` = 74 unitários + 4 integração; smoke em `script` PTY
120×36 com `--demo` e saída por `q`; build release de 1,1 MB.

## 5. Fase 2 — TUI funcional com evdev

**Status:** implementada; leitura evdev, extras, scroll hi-res e hotplug
validados em hardware.

Objetivo: o produto principal funcionando no terminal.

Entregáveis:

- `platform/evdev.rs` e `input/evdev.rs`;
- descoberta por sysfs/proc/ioctls;
- classificação mouse/touchpad/tablet/teclado/joystick/combo;
- thread evdev, canal limitado, overflow;
- hotplug por varredura de 1 s;
- seletor de dispositivo;
- fallback `crossterm`;
- dashboard completo;
- desafio de botões e scroll;
- taxa com histograma/confiança;
- log bruto;
- temas truecolor/256/16/mono;
- i18n completa;
- snapshots;
- `--grab` opt-in e recusa em combo;
- mensagens de permissão;
- `--diagnostics` textual inicial.

Critérios de saída:

- [x] Fluxo básico com mouse real na TUI (mouse HID 1bcf:08a0, fonte `evdev`, terminal GNOME).
- [x] Botões advertising e extras (validado pelo usuário).
- [x] Scroll vertical/horizontal e hi-res sem dupla contagem (validado pelo usuário).
- [x] Hotplug ≤ 2 s (validado pelo usuário).
- [x] Degradado sem evdev, sem travar (validado em PTY).
- [ ] Snapshots de todas as telas/idiomas/layouts (há testes de render, sem suíte de snapshots completa).
- [x] Restauração em saída normal, `Ctrl+C` e SIGTERM (SIGTERM validado em PTY).
- [x] Testes automatizados verdes com e sem features.

Evidência: 115 unitários + 5 de integração; descoberta real lista
`/dev/input/event3` como mouse `denied`; TUI degradada validada em PTY 120×36 e
80×24, inclusive clique L via protocolo SGR; `--diagnostics` gerado;
`--require-evdev` retorna 4. Leitura evdev validada com o mouse HID
`1bcf:08a0` (fonte `evdev`) em terminal GNOME. O usuário validou extras, scroll
hi-res/horizontal e hotplug em hardware; snapshots completos seguem pendentes.

## 6. Fase 3 — Modos ricos, acessibilidade e captura

**Status:** implementada; testada manualmente; performance de TUI ainda não
medida.

Objetivo: completar a experiência e o QA visual.

Entregáveis:

- modo Alvos (clique, arraste, duplo clique);
- métricas de precisão/tempo;
- rastro e efeitos polidos;
- `--no-animations`, `--no-trail`, captura determinística 1344×756;
- snapshots de captura;
- revisão de contraste e modo mono;
- documentação visual final.

Critérios de saída:

- [x] Modos ricos funcionam com seed reproduzível (testes de domínio).
- [x] Captura PNG determinística e usada em testes.
- [x] Acessibilidade mínima verificada (contraste dos tokens e snapshot mono).
- [x] Nenhuma regressão nos snapshots de TUI (Wide/Normal, pt-BR/en, mono, degradado).
- [ ] Performance dentro do orçamento na TUI.

Evidência: `core/modes/targets.rs` cobre clique, arraste e duplo clique com
`--seed`; `--capture-frame` gera PNG 1344×756 determinístico sem TTY/evdev;
`tests/snapshots.rs` compara texto; teste de contraste valida ≥ 4,5:1 para os
tokens de texto sobre `surface`/`bg`.

## 7. Fase 4 — Framebuffer

**Status:** implementada e validada em VT real (`tty3`, 1920×1080 32 bpp);
24/16 bpp e caminhos de sinal/pânico do fb ainda pendentes.

Objetivo: experiência idêntica em VT de texto.

Entregáveis:

- `platform/fb.rs`: validação, mmap, formatos 32/24/16, pan/fallback;
- `render/fb.rs`: canvas, escala, blend, cursor software;
- `render/fb.rs` + `render/raster.rs`: pintor RGBA e fonte 5×7 (D-016);
- `KD_GRAPHICS`, `FBIOBLANK`, `KD_TEXT`, cursor;
- sinais e panic hook;
- teclado via crossterm em `/dev/tty`;
- `--backend fb` com erros acionáveis;
- `testes_debug.sh`;
- validação em hardware ou pendência registrada.

Critérios de saída:

- [x] Primeiro frame sem `Ctrl+C`, inclusive console blanked.
- [x] 1920×1080 32 bpp validado em VT real.
- [ ] 24/16 bpp validados ou explicitamente pendentes (parse/packing testados).
- [x] Pan e fallback testados em unidade.
- [ ] Restauração em saída normal/erro/sinal/pânico (saída normal validada; sinais/pânico pendentes).
- [x] `testes_debug.sh` gera relatório único (testado em VT pelo usuário).
- [ ] Se não houver hardware: fase marcada **pendente**, não “concluída”.

Evidência: testes de ABI 160/80, validação 32/24/16 bpp, packing, stride,
pan/fallback, detecção de VT e frame 1920×1080 não vazio. Validação do usuário
em `tty3` (Zorin OS 18.1, 1920×1080 32 bpp) com cursor, botões, modos e saída
normal; o eixo Y invertido e a detecção de VT foram corrigidos nessa rodada.

## 8. Fase 5 — Release

**Status:** concluída; release 1.0.0 publicada.

Objetivo: entregar um pacote reproduzível.

Entregáveis:

- versão e lock congelados;
- todos os portões verdes;
- snapshots revisados;
- `LICENSE`, `THIRD_PARTY_NOTICES.md`, `OFL.txt`;
- `README` final e docs sem drift;
- `install.sh` (binário, docs, grupos com aviso, `--skip-groups`);
- `testes_debug.sh` no pacote e como asset;
- `tar.gz` + `.sha256`;
- smoke em `/tmp` sem assets;
- tag e release (se Q-002 confirmar público).

Critérios de saída:

- [x] Matriz manual executada; pendências registradas em `docs/11-manual-matrix.md`.
- [x] Instalador testado com prefixo temporário; sudo/grupos reais pendente.
- [x] Binário copiado funciona sozinho.
- [x] Checksum validado.
- [x] `AGENTS.md` com hash/URL da release.
- [x] Nenhum terminal/console corrompido nos caminhos validados.

Evidência: `https://github.com/danielexpe/mouse_tester/releases/tag/v1.0.0`;
SHA-256 `dacbd8b736b6027663528bfca52fa8d827f8d00de0b7759f8b3ad24f4ddb97f3`.

## 9. Fase 6 — Backlog

Ordenar com o usuário:

1. DRM/KMS como backend alternativo.
2. hidraw para reports, DPI e latência.
3. Exportação de sessão (CSV/JSON).
4. Touchpad completo (gestos, pressão, palm rejection).
5. Câmera infinita/mundo no canvas.
6. Temas e perfis persistentes.
7. Áudio de clique/scroll.
8. Medição de DPI com distância física.
9. Teste de latência wireless.
10. Suporte a teclado de atalhos configuráveis.

## 10. Marcos

| Marco | Definição |
| --- | --- |
| M0 | specs aprovadas, zero código |
| M1 | TUI mock com domínio testado |
| M2 | TUI real com evdev e modos principais |
| M3 | modos ricos + captura determinística |
| M4 | fbdev validado em VT (ou pendente formal) |
| M5 | release v1.0 empacotada |

## 11. Dependências entre fases

- Fase 2 depende de Fase 1.
- Fase 3 depende de Fase 2, mas Alvos pode ser cortado sem bloquear.
- Fase 4 depende de Fase 2; não iniciar antes da TUI funcional.
- Fase 5 depende de 2, 3 e 4 (ou pendência formal de fb).
- Backlog não bloqueia release.
