# 08 — Catálogo i18n

## 1. Regras

- Idiomas da v1: **pt-BR** (padrão quando `LANG` for pt) e **en**.
- Código, identificadores, flags e ajuda do CLI em inglês.
- Interface, mensagens, ajuda interna e diagnóstico em pt-BR/en.
- Nenhuma string visível hardcoded; todas passam pelo catálogo.
- Chaves em inglês, `snake.case` por grupo.
- Placeholders no formato `{name}`; nunca concatenar frases para formar outras.
- Nomes de dispositivos, `phys`, caminhos e códigos de botão são dados e não
  são traduzidos.
- Fallback: se uma chave faltar em `en`, usar pt-BR e falhar em teste de
  completude.
- `F2` alterna o idioma em qualquer tela, inclusive ajuda, modais e pausa.
- Idioma inicial: `--lang` > `LANG`/`LC_ALL` > pt-BR.
- Ajuda do CLI (clap) é em inglês na v1.

## 2. Formatação

| Item | pt-BR | en |
| --- | --- | --- |
| Separador decimal | vírgula (`1.000,5`) | ponto (`1,000.5`) |
| Milhar | ponto | vírgula |
| Data/hora | ISO 8601 (`2026-10-31T14:05:00-03:00`) | idem |
| Unidades | `Hz`, `ms`, `µs`, `counts`, `px` | idem |
| Direção | `↑ ↓ ← →` ou `^ v < >` | idem |
| Sem valor | `—` | `—` |
| Plural | forma única com número explícito | idem |

- Contadores brutos podem usar separador de milhar do locale.
- Textos de acessibilidade/ASCII usam `^ v < >` quando a fonte não tem setas.

## 3. Catálogo — geral

| Chave | pt-BR | en |
| --- | --- | --- |
| `app.title` | MouseScope | MouseScope |
| `app.subtitle` | Laboratório visual de mouse | Visual mouse lab |
| `backend.tui` | TUI | TUI |
| `backend.fb` | Framebuffer | Framebuffer |
| `backend.degraded` | Degradado | Degraded |
| `source.evdev` | evdev | evdev |
| `source.tui_mouse` | Mouse do terminal | Terminal mouse |
| `state.connected` | Conectado | Connected |
| `state.waiting` | Aguardando mouse | Waiting for mouse |
| `state.no_device` | Nenhum mouse encontrado | No mouse found |
| `state.permission_denied` | Permissão negada | Permission denied |
| `state.reconnecting` | Reconectando | Reconnecting |
| `state.grabbed` | Captura exclusiva ativa | Exclusive grab active |
| `state.overflow` | Eventos descartados | Dropped events |
| `state.extra` | Extra | Extra |

## 4. Rótulos

| Chave | pt-BR | en |
| --- | --- | --- |
| `label.device` | Dispositivo | Device |
| `label.path` | Caminho | Path |
| `label.phys` | Porta | Port |
| `label.class` | Classe | Class |
| `label.vendor` | Fabricante | Vendor |
| `label.product` | Produto | Product |
| `label.capabilities` | Capacidades | Capabilities |
| `label.backend` | Backend | Backend |
| `label.status` | Estado | Status |
| `label.version` | Versão | Version |
| `label.confidence` | Confiança | Confidence |
| `label.samples` | Amostras | Samples |
| `label.position` | Posição | Position |
| `label.raw` | Bruto | Raw |
| `label.dx` | dx | dx |
| `label.dy` | dy | dy |
| `label.speed` | Velocidade | Speed |
| `label.max_speed` | Vel. máx. | Max speed |
| `label.distance` | Distância | Distance |
| `label.events` | Eventos | Events |
| `label.reports` | Relatórios | Reports |
| `label.direction` | Direção | Direction |
| `label.last_event` | Último evento | Last event |

## 5. Painéis e unidades

| Chave | pt-BR | en |
| --- | --- | --- |
| `panel.buttons` | Botões | Buttons |
| `panel.scroll` | Scroll | Scroll |
| `panel.motion` | Movimento | Motion |
| `panel.polling` | Taxa | Rate |
| `panel.log` | Log bruto | Raw log |
| `panel.devices` | Dispositivos | Devices |
| `panel.help` | Ajuda | Help |
| `unit.counts` | counts | counts |
| `unit.px` | px | px |
| `unit.hz` | Hz | Hz |
| `unit.us` | µs | µs |
| `unit.ms` | ms | ms |
| `unit.s` | s | s |
| `unit.per_second` | /s | /s |
| `unit.notch` | notch | notch |
| `unit.none` | — | — |

## 6. Botões

| Chave | pt-BR | en |
| --- | --- | --- |
| `button.left` | Esquerdo | Left |
| `button.right` | Direito | Right |
| `button.middle` | Meio | Middle |
| `button.side` | Lateral 1 | Side 1 |
| `button.extra` | Lateral 2 | Side 2 |
| `button.forward` | Avançar | Forward |
| `button.back` | Voltar | Back |
| `button.task` | Tarefa | Task |
| `button.unknown` | Desconhecido | Unknown |
| `button.pressed` | Pressionado | Pressed |
| `button.released` | Solto | Released |
| `button.count` | Contagem | Count |
| `button.duration` | Duração | Duration |
| `button.double` | Duplo clique | Double click |
| `button.drag` | Arraste | Drag |

Rótulos curtos no canvas: `[L]`, `[R]`, `[M]`, `[S1]`, `[S2]`, `[F]`, `[B]`,
`[T]`, `[0]`..`[9]`, `[0xNNN]`. Não traduzir.

## 7. Scroll

| Chave | pt-BR | en |
| --- | --- | --- |
| `scroll.up` | Cima | Up |
| `scroll.down` | Baixo | Down |
| `scroll.left` | Esquerda | Left |
| `scroll.right` | Direita | Right |
| `scroll.vertical` | Vertical | Vertical |
| `scroll.horizontal` | Horizontal | Horizontal |
| `scroll.ticks` | Ticks | Ticks |
| `scroll.hires` | Alta resolução | High resolution |
| `scroll.invert` | Invertido | Inverted |
| `scroll.none` | Sem scroll | No scroll |

## 8. Taxa de relatório

| Chave | pt-BR | en |
| --- | --- | --- |
| `polling.title` | Taxa de relatório estimada | Estimated report rate |
| `polling.measuring` | Medindo... mova o mouse | Measuring... move the mouse |
| `polling.insufficient` | Amostras insuficientes | Not enough samples |
| `polling.jitter` | Jitter | Jitter |
| `polling.hint` | Mova o mouse por 2 s | Move the mouse for 2 s |
| `polling.confidence.low` | Baixa | Low |
| `polling.confidence.medium` | Média | Medium |
| `polling.confidence.high` | Alta | High |

## 9. Modos e desafios

| Chave | pt-BR | en |
| --- | --- | --- |
| `mode.dashboard` | Painel | Dashboard |
| `mode.buttons` | Botões | Buttons |
| `mode.scroll` | Scroll | Scroll |
| `mode.polling` | Taxa | Rate |
| `mode.targets` | Alvos | Targets |
| `mode.devices` | Dispositivos | Devices |
| `mode.help` | Ajuda | Help |
| `mode.buttons.prompt` | Pressione: {button} | Press: {button} |
| `mode.buttons.progress` | {done}/{total} | {done}/{total} |
| `mode.buttons.missing` | Faltam: {list} | Missing: {list} |
| `mode.buttons.skip` | Pular | Skip |
| `mode.scroll.prompt` | Role para {direction} | Scroll {direction} |
| `mode.scroll.progress` | {done}/{total} | {done}/{total} |
| `mode.targets.prompt.click` | Clique no alvo | Click the target |
| `mode.targets.prompt.drag` | Pressione no alvo e arraste | Press on the target and drag |
| `mode.targets.prompt.double` | Clique duas vezes no alvo | Double-click the target |
| `mode.targets.progress` | {done}/{total} | {done}/{total} |
| `mode.targets.accuracy` | Acerto | Accuracy |
| `mode.targets.error` | Erro | Error |
| `mode.targets.time` | Tempo | Time |
| `mode.targets.hit` | Acerto! | Hit! |
| `mode.targets.miss` | Erro | Miss |
| `mode.targets.done` | Alvos concluídos | Targets complete |

## 10. Ajuda e rodapé

| Chave | pt-BR | en |
| --- | --- | --- |
| `help.title` | Ajuda | Help |
| `help.navigation` | Navegação | Navigation |
| `help.keys` | 1-5 modos, TAB alterna, T dispositivo, L log, R reset | 1-5 modes, TAB cycles, T device, L log, R reset |
| `help.exit` | Q, Esc ou Ctrl+C | Q, Esc or Ctrl+C |
| `help.language` | F2 alterna pt-BR/en | F2 toggles pt-BR/en |
| `help.limitations` | Limitações | Limitations |
| `help.degraded` | Sem movimento bruto nem taxa | No raw motion or rate |
| `help.mouse_ui` | O mouse não controla a interface | Mouse does not control the UI |
| `help.fb` | VT de texto; cursor desenhado pelo app | Text VT; cursor drawn by the app |
| `action.quit` | Sair | Quit |
| `action.back` | Voltar | Back |
| `action.close` | Fechar | Close |
| `action.reset` | Reiniciar | Reset |
| `action.select` | Selecionar | Select |
| `action.next` | Próximo | Next |
| `action.previous` | Anterior | Previous |

## 11. Classes de dispositivo

| Chave | pt-BR | en |
| --- | --- | --- |
| `class.mouse` | Mouse | Mouse |
| `class.touchpad` | Touchpad | Touchpad |
| `class.tablet` | Mesa digitalizadora | Graphics tablet |
| `class.keyboard` | Teclado | Keyboard |
| `class.joystick` | Joystick | Joystick |
| `class.combo` | Combo | Combo |
| `class.unknown` | Desconhecido | Unknown |

## 12. Banners

| Chave | pt-BR | en |
| --- | --- | --- |
| `banner.hotplug.added` | Mouse conectado: {name} | Mouse connected: {name} |
| `banner.hotplug.removed` | Mouse removido: {name} | Mouse removed: {name} |
| `banner.hotplug.switched` | Alternado para {name} | Switched to {name} |
| `banner.degraded` | Modo degradado: mouse do terminal | Degraded mode: terminal mouse |
| `banner.permission` | Permissão negada em /dev/input; veja a ajuda ou use o mouse do terminal | Permission denied on /dev/input; see help or use the terminal mouse |
| `banner.overflow` | {count} eventos descartados | {count} events dropped |
| `banner.no_device` | Conecte um mouse para começar | Connect a mouse to start |

## 13. Erros

| Chave | pt-BR | en |
| --- | --- | --- |
| `error.input.permission` | Permissão negada em /dev/input/event*. Adicione o usuário ao grupo input e entre novamente. | Permission denied on /dev/input/event*. Add the user to the input group and log in again. |
| `error.input.not_found` | Nenhum mouse compatível encontrado. | No compatible mouse found. |
| `error.input.required` | evdev é obrigatório neste modo. | evdev is required in this mode. |
| `error.device_open` | Falha ao abrir {path}: {reason}. | Failed to open {path}: {reason}. |
| `error.fb.missing` | /dev/fb0 não encontrado. | /dev/fb0 not found. |
| `error.fb.permission` | Permissão negada em /dev/fb0. Adicione o usuário ao grupo video. | Permission denied on /dev/fb0. Add the user to the video group. |
| `error.fb.not_vt` | Execute em um VT de texto local (tty = /dev/ttyN). | Run in a local text VT (tty = /dev/ttyN). |
| `error.fb.format` | Formato de framebuffer não suportado: {bpp} bpp. | Unsupported framebuffer format: {bpp} bpp. |
| `error.fb.memory` | smem_len insuficiente para o buffer visível. | smem_len too small for the visible buffer. |
| `error.tty` | A TUI exige um terminal interativo. | TUI requires an interactive terminal. |
| `error.capture` | Falha ao gravar captura: {path}. | Failed to write capture: {path}. |
| `error.diagnostics` | Falha ao gravar diagnóstico: {path}. | Failed to write diagnostics: {path}. |
| `error.combo_grab` | Grab recusado: dispositivo combo. Use --grab-force. | Grab refused: combo device. Use --grab-force. |
| `error.overflow` | Eventos descartados; métricas podem estar imprecisas. | Events dropped; metrics may be inaccurate. |

## 14. Captura e diagnóstico

| Chave | pt-BR | en |
| --- | --- | --- |
| `capture.title` | MouseScope — captura | MouseScope — capture |
| `capture.note` | Estado sintético | Synthetic state |
| `diag.title` | MouseScope — diagnóstico | MouseScope — diagnostics |
| `diag.permissions` | Permissões | Permissions |
| `diag.environment` | Ambiente | Environment |
| `diag.framebuffer` | Framebuffer | Framebuffer |
| `diag.input` | Entrada | Input |
| `diag.notes` | Notas | Notes |
| `diag.no_permission` | sem permissão | no permission |
| `diag.readable` | legível | readable |

## 15. Limites de layout

| Contexto | Limite | Regra |
| --- | --- | --- |
| Rodapé | 1 linha | encurtar atalhos, nunca ocultar sair |
| Banner | 2 linhas | quebrar; não truncar causa |
| Nome de dispositivo | 28 colunas (Wide) | truncar no meio com `…` |
| Caminho | 32 colunas | truncar início com `…` |
| Cartão de botão | 5–7 colunas | usar rótulo curto |
| Painel de taxa | 18 colunas | Hz antes de jitter |
| Ajuda | 80 colunas | quebrar por palavra |
| Minimal | 60×20 | sem parágrafos; só valores |

- Testes de snapshot devem usar strings longas em pt-BR e en.
- Mensagens de erro podem ser longas no stderr; na TUI, usar banner curto +
  detalhe na ajuda.
- Não usar abreviações ambíguas; `Vel.` só quando o espaço exigir.

## 16. Completude

- [ ] Toda chave usada no código existe nos dois idiomas.
- [ ] Nenhuma string visível fora do catálogo.
- [ ] Placeholders iguais entre idiomas.
- [ ] Snapshots pt-BR/en aprovados.
- [ ] Textos longos não estouram.
- [ ] `F2` funciona em todas as telas.
- [ ] Nomes de dispositivo não são traduzidos.

## 17. Chaves adicionadas na Fase 2

| Chave | pt-BR | en |
| --- | --- | --- |
| `help.device_picker` | T fecha, setas escolhem, Enter aplica | T closes, arrows select, Enter applies |
| `device.ok` | ok | ok |
| `device.denied` | sem permissão | denied |
| `device.primary` | primário | primary |
| `banner.waiting` | Aguardando um mouse legível | Waiting for a readable mouse |
| `polling.histogram` | Histograma | Histogram |

As chaves acima são expostas na implementação como variantes de `i18n::Key`;
os nomes semânticos deste documento continuam sendo a referência de tradução.
