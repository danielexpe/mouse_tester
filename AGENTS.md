# AGENTS.md — memória operacional

## 1. Retomada rápida

1. Ler este arquivo, `README.md`, `docs/06-roadmap.md` e
   `docs/07-decisions-and-risks.md`.
2. Rodar `git status --short` e ler o diff antes de editar.
3. Confirmar com o usuário antes de sair da fase de planejamento.
4. Não implementar código enquanto o escopo da Fase 0 não estiver aprovado.

## 2. Status atual

Data de referência: **2026-10-31**.

| Fase | Status | Observação |
| --- | --- | --- |
| 0 — Descoberta e especificação | concluída | escopo aprovado pelo usuário em 2026-10-31 |
| 1 — Fundação | concluída | crate, domínio puro, CLI, mock, TUI placeholder; 78 testes |
| 2 — TUI funcional | implementada; evdev e extras validados | mouse HID 1bcf:08a0; scroll hi-res/hotplug ok; snapshots parciais |
| 3 — Modos ricos | implementada; testada manualmente | performance da TUI ainda não medida |
| 4 — Framebuffer | pendente | fbdev, cursor, sinais, validação em VT |
| 5 — Release | pendente | pacote, instalador, diagnóstico, matriz manual |
| 6 — Backlog | pendente | DRM/KMS, hidraw, exportação, touchpad |

As Fases 1, 2 e 3 foram implementadas. A TUI abre e fecha via PTY; a entrada
evdev foi implementada com descoberta, classificação, thread de leitura, hotplug
e fallback `crossterm`. A leitura real foi validada com o mouse HID `1bcf:08a0`
(fonte `evdev`) em terminal GNOME; o modo degradado também foi validado em PTY.
A Fase 3 adicionou o modo Alvos com seed, captura PNG determinística e snapshots
de texto. O usuário validou extras, scroll hi-res/horizontal e hotplug em
hardware e testou a Fase 3; a performance da TUI ainda não foi medida.
framebuffer ainda não existe.

## 3. Resumo do produto

**MouseScope** é um teste visual de mouse para console Linux. Lê eventos de
mouse via evdev e representa movimento, botões, scroll, botões extras, arraste e
taxa de relatório com efeitos em uma TUI (`ratatui`) ou no framebuffer
(`/dev/fb0`). O mouse é o objeto do teste; a UI é navegada apenas por teclado.
A TUI é o backend padrão; o framebuffer é explícito.

Estado atual: Fase 3 implementada com domínio puro, CLI, evdev, fallback,
desafios, Alvos, captura PNG, seletor de dispositivo e diagnóstico textual.
`--demo` continua disponível para inspeção sem hardware.

## 4. Decisões congeladas

Não reabrir sem pedido explícito do usuário; mudanças exigem novo ADR.

| ID | Decisão |
| --- | --- |
| D-001 | Rust, Linux x86_64, TUI + fbdev, conforme o framework da pasta. |
| D-002 | evdev é a entrada primária; `crossterm` é fallback degradado na TUI. |
| D-003 | Navegação da UI somente por teclado; mouse nunca aciona controles. |
| D-004 | Um único estado de aplicação; dois backends de desenho. |
| D-005 | Efeitos procedurais; sem arte bitmap. |
| D-006 | Sem `EVIOCGRAB` por padrão; grab opt-in e bloqueado em dispositivo combo. |
| D-007 | `--backend auto` escolhe TUI; fb só com flag explícita. |
| D-008 | Sem áudio na v1; sem rede, telemetria ou persistência. |
| D-009 | Sem `ratatui-image`; canvas Braille na TUI e `tiny-skia` no fb. |
| D-010 | Hotplug por varredura periódica, não `inotify` na v1. |
| D-011 | Docs em pt-BR; código/CLI em inglês; UI em pt-BR e en. |
| D-012 | Taxa de relatório é estimativa; o app não afirma DPI nem polling nominal. |
| D-013 | Diagnóstico em texto simples; sem `serde` na v1. |
| D-014 | Nunca rodar como root; permissões via grupos `input`/`video` documentadas. |
| D-015 | Framebuffer é entregue na Fase 4; se não houver hardware, fica pendente. |

## 5. Fontes de verdade

| Assunto | Documento |
| --- | --- |
| Produto, fluxos e aceite | `docs/01-product.md` |
| Arquitetura e CLI | `docs/02-technical-spec.md` |
| Visual e efeitos | `docs/03-visual-design.md` |
| Assets e licenças | `docs/04-assets-and-licenses.md` |
| Testes | `docs/05-test-plan.md` |
| Fases | `docs/06-roadmap.md` |
| ADRs, riscos, questões | `docs/07-decisions-and-risks.md` |
| i18n | `docs/08-i18n-catalog.md` |
| Evdev e entrada | `docs/09-input-spec.md` |
| Release | `docs/10-release-checklist.md` |

## 6. Pendências e questões abertas

| ID | Questão | Default proposto | Prazo |
| --- | --- | --- | --- |
| Q-001 | Confirmar nome/binary `MouseScope`/`mousescope`. | em uso; confirmar antes da release | antes da Fase 5 |
| Q-002 | Distribuição pessoal ou release pública? | release pública | antes da Fase 5 |
| Q-003 | Instalador deve adicionar o usuário ao grupo `input`? | apenas instruir; flag opcional | antes da Fase 5 |
| Q-004 | Hardware/distro alvo para validar fbdev? | resolvida: Zorin OS 18.1 / base Ubuntu, VT 1920×1080 32 bpp | antes da Fase 4 |
| Q-005 | Touchpad na v1? | apenas seleção explícita, básico | antes da Fase 3 |
| Q-006 | Exportar métricas de sessão? | backlog (CSV/JSON) | Fase 6 |
| Q-007 | Medir DPI com deslocamento físico? | backlog | Fase 6 |
| Q-008 | Áudio de clique/scroll? | fora da v1 | Fase 6 |

## 7. Comandos e portões planejados

```bash
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo clippy --workspace --all-targets --no-default-features -- -D warnings
cargo test
cargo test --no-default-features
cargo test --release
cargo build --release
cargo build --release --no-default-features
cargo run -- --demo
```

`cargo xtask assets --check` passa a valer quando `assets/` e `xtask/` existirem
(Fase 4/5); não é aplicável à Fase 1.

Teste manual em VT real é obrigatório para declarar fbdev validado. Nunca
declarar validação manual sem execução registrada.

## 8. Regras de trabalho do agente

- Especificar antes de codar; não implementar sem autorização.
- Respeitar as decisões congeladas; propor ADR para mudanças.
- Não adicionar dependência, rede ou áudio sem aprovação explícita.
- Não fazer commit, push, amend ou PR sem pedido explícito.
- `unsafe` somente em `platform/`, com comentário `// SAFETY:`.
- Preservar terminal/console em saída normal, erro, pânico e sinais.
- Não ler nem registrar teclas de dispositivos combo (privacidade).
- Testar antes de declarar concluído; rodar todos os portões aplicáveis.
- Registrar bloqueios de hardware/permissão em vez de inventar progresso.
- Código e identificadores em inglês; UI nos idiomas do produto.
- Sem comentários desnecessários; seguir o estilo existente.
- Atualizar este arquivo ao final de tarefas relevantes.

## 9. Protocolo de atualização deste arquivo

1. Atualizar a tabela de status e a data de referência.
2. Registrar decisões novas ou substituídas com ID e link ao ADR.
3. Registrar pendências com prazo e responsável.
4. Adicionar entrada no registro de atividades.
5. Nunca apagar histórico; marcar como substituído.
6. Não incluir segredos, tokens ou dados de dispositivos pessoais.

## 10. Registro de atividades

| Data | Resumo | Arquivos |
| --- | --- | --- |
| 2026-10-31 | Leitura do framework e criação das especificações iniciais (Fase 0). Nenhum código. | `README.md`, `AGENTS.md`, `docs/01`..`docs/10` |
| 2026-10-31 | Fase 1 implementada: crate, domínio puro, input mock, CLI, TUI placeholder e guardas. 78 testes; TUI validada em PTY com `q`. Sem evdev/fb. | `Cargo.toml`, `Cargo.lock`, `src/`, `tests/cli.rs`, `README.md`, `AGENTS.md`, `docs/06-roadmap.md` |
| 2026-10-31 | Fase 2 implementada: evdev (descoberta, classificação, thread, hotplug), fallback TUI, seletor, desafios, diagnóstico. 120 testes; modo degradado e clique SGR validados em PTY. Leitura real pendente por falta de permissão em `/dev/input`. | `src/input/`, `src/platform/evdev.rs`, `src/core/modes/`, `src/diagnostics.rs`, `src/app.rs`, `src/render/tui.rs`, docs |
| 2026-10-31 | Leitura evdev validada em hardware com mouse HID `1bcf:08a0` (fonte `evdev`) em terminal GNOME. Modo degradado segue validado em PTY; extras/hotplug/snapshots pendentes. | `AGENTS.md`, `README.md`, `docs/06-roadmap.md`, `docs/09-input-spec.md` |
| 2026-10-31 | Fase 3 implementada: modo Alvos (clique/arraste/duplo clique, seed, métricas), captura PNG determinística 1344×756 sem dependências, snapshots de TUI, snapshot de captura e teste de contraste. 144 testes. Performance de TUI ainda não medida. | `src/core/modes/targets.rs`, `src/render/capture.rs`, `src/state/session.rs`, `src/render/tui.rs`, `tests/snapshots.rs`, docs |
| 2026-10-31 | Validação manual do usuário: botões extras, scroll hi-res/horizontal e hotplug OK; modo Alvos e captura testados. Q-004 respondida: Zorin OS 18.1 (base Ubuntu noble), VT `tty2`, `/dev/fb0` 1920×1080 32 bpp; alvo: distros base Ubuntu. | `AGENTS.md`, `README.md`, `docs/06-roadmap.md`, `docs/07-decisions-and-risks.md` |

## 11. Fases

| Fase | Objetivo | Critério de saída |
| --- | --- | --- |
| 0 | escopo, arquitetura, riscos, licenças | specs revisadas e congeladas |
| 1 | crate, domínio puro, mock input, CLI, TUI placeholder | testes de domínio + TUI abre e fecha |
| 2 | TUI funcional com evdev | fluxo completo + snapshots |
| 3 | modos ricos e acessibilidade | alvos/arraste/clique duplo + captura |
| 4 | framebuffer | fb validado em VT real ou pendência registrada |
| 5 | release | pacote, install, diagnóstico, docs |
| 6 | backlog | itens priorizados pelo usuário |

## 12. Notas para a próxima sessão

- Fases 2 e 3 implementadas; evdev validado com mouse HID `1bcf:08a0`
  (fonte `evdev`) em terminal GNOME.
- Próximo passo: Fase 4 (framebuffer), após definir hardware/distro (Q-004).
- Pendências da Fase 2/3: matriz completa de snapshots e performance medida.
- A sessão do agente continua fora do grupo `input`; para reproduzir testes
  evdev, usar `newgrp input`/`sg input`. O usuário validou em sessão própria.
- VT Linux não recebe mouse sem evdev; o suporte completo a console depende do
  backend framebuffer da Fase 4.
- Q-001..Q-008 continuam em aberto; os defaults estão em uso.
- Definir hardware/distro de validação (Q-004) antes da Fase 4.
