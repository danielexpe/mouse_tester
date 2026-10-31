# 02 — Especificação técnica

## 1. Opções de arquitetura consideradas

| Opção | Prós | Contras | Decisão |
| --- | --- | --- | --- |
| **Rust TUI + fbdev (escolhida)** | alinhada ao framework; offline; um domínio e dois backends; funciona em terminal e console; baixo consumo | exige evdev/permissões; fbdev é restrito a VT | **adotada** |
| Aplicativo web (Pointer Lock/Wheel) | portátil; visual rico; rápido de desenvolver | não lê evdev bruto; depende de navegador/servidor gráfico; sem dispositivo/capacidades; não é console | rejeitada na v1 |
| GUI nativa (winit/egui) | entrada de mouse rica; janela fácil | não atende console/VT; mais dependências; não é o framework pedido | rejeitada na v1 |
| CLI de log (evdev apenas) | simples e preciso | não é teste visual; sem efeitos | rejeitada como produto |
| SDL2/C + Rust FFI | baixo nível e portátil | dependência C; foge do template Rust/TUI/fb | rejeitada |

### 1.1 Fontes de entrada consideradas

| Fonte | Dados | Limitações | Decisão |
| --- | --- | --- | --- |
| **evdev direto** | movimento relativo, botões, wheel hi-res, timestamps, capacidades, IDs | permissão de grupo; ABI Linux; sem HID bruto | **primária** |
| `crossterm` mouse | clique, scroll, arraste, posição em células | sem movimento relativo bruto; sem extras em muitos terminais; sem taxa | fallback TUI |
| `libinput` | eventos normalizados | dependência C; esconde counts brutos; mais camadas | rejeitada na v1 |
| `hidraw` | reports HID brutos | parsing por fornecedor; alto custo; precisa de udev | backlog |
| `/dev/input/mice` | pacote PS/2 legado | sem dispositivo, sem extras, wheel inconsistente | rejeitada |

## 2. Decisão arquitetural

- Rust stable, edição 2021, MSRV 1.88, Linux x86_64.
- Um único estado de aplicação (`App`) e dois backends de desenho.
- `core` é puro: não conhece `ratatui`, `crossterm`, `libc`, fbdev, evdev ou
  assets.
- Entrada primária: evdev em thread dedicada, canal limitado, eventos
  normalizados.
- Entrada de teclado e fallback de mouse: `crossterm` na thread principal.
- Framebuffer atrás de `platform/fb.rs`, com `unsafe` isolado e comentado.
- Sem áudio, rede, telemetria ou persistência na v1.
- Sem `serde`; diagnóstico em texto simples.

## 3. Diagrama de camadas

```text
+-----------------------------+
| main.rs / cli.rs            |
+--------------+--------------+
               |
               v
+-----------------------------+        +----------------------+
| app/ (loop, teclado, sinais)|------->| input/ (evdev, tui,  |
| estado da sessão            |        | mock, classificação) |
+----+-----------+------------+        +----------+-----------+
     |           |                                |
     v           v                                v
+---------+  +------------------+          +-------------+
| state/  |  | render/          |          | platform/   |
| sessão  |  | scene, tui, fb   |          | evdev, fb,  |
+----+----+  +----+--------+----+          | tty, sinais |
     |            |        |               +-------------+
     v            v        v
+---------+   +--------+ +--------+
| core/   |   | ratatui| | tiny-  |
| puro    |   | + term | | skia   |
+---------+   +--------+ +--------+
```

Regras de dependência:

- `core/` não importa nenhum módulo de UI, plataforma, assets ou tempo de
  parede. Recebe `now`/timestamps e devolve dados.
- `state/` depende de `core/`; modela sessão, modo atual, seleção e flags.
- `app/` é o único que orquestra backends, canais, relógio e teclado.
- `render/scene.rs` é independente de backend; descreve primitivas lógicas.
- `render/tui.rs` depende de `ratatui`; `render/fb.rs` usa o rasterizador
  interno e a fonte 5×7 (D-016).
- `platform/` contém todo `unsafe` de sistema (`ioctl`, `poll`, `read`, `mmap`,
  `sigaction`), sempre com `// SAFETY:`.
- `input/evdev.rs` é seguro; usa a API de `platform/evdev.rs`.

## 4. Módulos planejados

```text
src/
  main.rs              # parse CLI, escolha de backend, códigos de saída
  lib.rs               # reexporta módulos para testes
  cli.rs               # flags, enums, validação
  app.rs               # App, roteamento de ações, loop TUI/fb
  diagnostics.rs       # relatório textual de ambiente/entrada/fb
  input/
    mod.rs             # InputEvent, InputSource
    controller.rs      # evdev ou fallback degradado
    evdev.rs           # descoberta, thread de leitura, normalização
    capabilities.rs    # bits EVIOCGBIT -> capacidades
    classify.rs        # mouse/touchpad/tablet/teclado/joystick/combo
    tui_mouse.rs       # fallback crossterm
    mock.rs            # fonte sintética para testes e --demo
  core/
    pointer.rs         # posição, clamp, rastro, velocidade
    buttons.rs         # estado, contagem, duração, duplo clique
    scroll.rs          # vertical/horizontal/hi-res, coalescência
    effects.rs         # efeitos com lifetime, sem UI
    metrics.rs         # taxa de relatório, jitter, overflow, sparkline
    modes/
      dashboard.rs
      buttons.rs
      scroll.rs
      polling.rs
      targets.rs       # Fase 3
  state/
    session.rs         # modo, flags, seleção, resultados
    device.rs          # DeviceInfo, DeviceId, capacidades
    log.rs             # ring buffer de eventos
  render/
    mod.rs
    scene.rs           # primitivas compartilhadas
    layout.rs          # breakpoints e geometria
    raster.rs          # pintor RGBA, fonte 5×7 e PNG
    tui.rs             # desenho ratatui + Canvas Braille
    fb.rs              # canvas, escala, blend, cursor
    png.rs             # captura determinística
  platform/
    mod.rs
    evdev.rs           # open/ioctl/read/poll/grab (unsafe)
    fb.rs              # /dev/fb0, KD_GRAPHICS, blank, pan (unsafe)
    tty.rs             # guardas de terminal/console
    signals.rs         # SIGINT/SIGTERM/SIGHUP (unsafe)
  assets.rs            # include_bytes!, validação
  i18n/
    mod.rs             # pt-BR/en, fallback
  theme.rs             # tokens de cor e tiers
  capture.rs           # composição de frame de QA
  util/
    rng.rs             # SplitMix64
    time.rs            # conversores de timestamp
xtask/                 # verificação de assets
tests/
  fixtures/            # eventos sintéticos, capturas douradas
  snapshots/           # snapshots de TUI
```

## 5. Modelo de eventos e domínio

### 5.1 Evento normalizado

```text
InputEvent {
  device: DeviceId,
  source: Evdev | TuiMouse | Mock,
  time_us: u64,                # monotônico, microssegundos
  kind: Move { dx, dy } |
        Absolute { x, y, x_range, y_range } |
        Button { code: u16, pressed: bool } |
        Wheel { axis: Vertical|Horizontal, value: i32, hires: bool } |
        Report { interval_us: u32 } |
        DeviceAdded(DeviceInfo) |
        DeviceRemoved(DeviceId) |
        Overflow { dropped: u64 } |
        SourceError(InputError)
}
```

- `Move` usa counts relativos exatamente como recebidos.
- `Report` é emitido em `SYN_REPORT` e alimenta a estimativa de taxa.
- `Absolute` só é usado para touchpad/tablet selecionado explicitamente.
- Eventos de teclado de evdev são **descartados** antes de virar `InputEvent`.

### 5.2 Estado de domínio

```text
PointerState {
  raw_total_x, raw_total_y: i64,
  pos: Point,                  # canvas lógico
  prev_pos: Point,
  velocity: Vec2,              # EWMA
  speed_counts_s: f32,
  distance_counts: u64,
  trail: VecDeque<TrailPoint>, # máx. 96, janela 600 ms
  clamped: bool,
}

ButtonState {
  buttons: BTreeMap<u16, ButtonRuntime>,
  pressed: BTreeSet<u16>,      # qualquer código de botão de mouse
  last_press_us: BTreeMap<u16, u64>,
  dragging: Option<DragState>,
  double_click_us: u64,        # padrão 400 ms
}

ScrollState {
  vertical_ticks: i64,
  horizontal_ticks: i64,
  hires_remainder_v: i32,
  hires_remainder_h: i32,
  last_delta: i32,
  last_at_us: u64,
}

Metrics {
  events: u64,
  reports: u64,
  dropped: u64,
  intervals: Ring<u32, 256>,
  hz_estimate: Option<f32>,
  jitter_us: Option<u32>,
  confidence: Low | Medium | High,
  event_rate_bins: [u32; 20],  # 5 s
}
```

Invariantes:

- `trail` sempre termina na posição atual e tem no máximo 96 pontos.
- Efeitos expirados são removidos em `tick`; máximo de 128 ativos.
- `distance_counts` nunca diminui.
- `pressed` reflete exatamente os botões com `pressed=true`.
- Hi-res e notch nunca contam a mesma rotação duas vezes.
- Métricas só aceitam intervalos em `(0, 250 ms]` para estimar taxa.
- Nenhum estado de domínio usa `SystemTime` ou `Instant` diretamente.

## 6. Loop, threads e tempo

### 6.1 Threads

| Thread | Responsabilidade |
| --- | --- |
| principal | `crossterm` (teclado/fallback), drenar canal, `update`, `draw`, apresentar |
| evdev | `poll` em todos os fds selecionados, ler, normalizar, enviar ao canal |
| assets/captura | não há thread; decode no startup e cache |

- Canal limitado evdev→principal: capacidade 8192.
- O leitor nunca bloqueia no canal: se cheio, descarta o mais antigo, incrementa
  `dropped` e envia `Overflow` no próximo slot disponível.
- A thread evdev também faz a varredura de hotplug a cada 1 s.
- Encerramento: flag atômica + fechar/poll timeout; `join` na saída.
- O canal usa `std::sync::mpsc`; sem `tokio`/`async`.

### 6.2 Loop principal

```text
while !quit:
    action = read_keyboard(timeout)       # crossterm poll
    events = drain_evdev()                # sem bloquear
    feed(events)                          # domínio
    tick(now)                             # expira efeitos, métricas, hotplug
    draw(&app)                            # TUI flush ou fb present
    if exit_flag: break
restore()
```

- Timeout: 16 ms durante animação/atividade; 100 ms ocioso.
- Animação a 60 FPS; modo ocioso a ~10 FPS.
- `Ctrl+C` em raw mode chega como tecla; mapear para `Quit`.
- `SIGINT`/`SIGTERM`/`SIGHUP` marcam flag atômica; restauração no loop/`Drop`.

### 6.3 Relógios

- Animações: `Instant` da thread principal.
- Taxa de relatório: timestamps do kernel convertidos para microssegundos
  monotônicos; se indisponíveis/não monotônicos, `Instant` na leitura.
- Captura determinística: tempo lógico fixo, sem relógio de parede.

## 7. Entrada (resumo; detalhes em `docs/09-input-spec.md`)

- Descoberta por `/sys/class/input`, `/proc/bus/input/devices` e abertura de
  `/dev/input/event*`.
- Classificação por capacidades (`EVIOCGBIT`) e propriedades (`EVIOCGPROP`).
- Mouse-like: `REL_X` + `REL_Y` + `BTN_LEFT`. Wheel é opcional.
- Thread evdev com `poll` + `read` não bloqueante.
- Fallback TUI com `crossterm` apenas quando evdev não está disponível ou quando
  forçado; sem movimento relativo bruto.
- Dispositivo combo: teclas ignoradas e grab recusado.
- Grab opt-in (`--grab`), com aviso e liberação em `Drop`.

## 8. Backends

### 8.1 Contrato

1. Entrar com guarda RAII (raw mode + alternate screen ou `KD_GRAPHICS`).
2. Desenhar a partir de `&App` sem mutar estado.
3. Apresentar (flush TUI ou present fb).
4. Ler teclado com timeout e mapear para `Action`.
5. Restaurar terminal/console em saída normal, erro, pânico e sinais.
6. Nunca imprimir logs no meio da tela; erros vão para stderr.

### 8.2 TUI

- Exige stdin **e** stdout TTY.
- `enable_raw_mode`, `EnterAlternateScreen`, cursor escondido.
- `ratatui` com layout adaptativo; canvas principal em Braille.
- Mouse do terminal desabilitado por padrão quando evdev está ativo, para não
  interferir na seleção de texto; habilitado no modo degradado.
- Snapshots cobrem telas, idiomas, breakpoints, degradado e erros.
- Restauração em `Drop`, erro e panic hook.

### 8.3 Framebuffer

Segue o framework, com adaptações:

- Validar `fb_var_screeninfo`/`fb_fix_screeninfo`, `yoffset`, `smem_len`,
  `line_length`, formato 32/24/16 bpp.
- Entrar em `KD_GRAPHICS`, chamar `FBIOBLANK(FB_BLANK_UNBLANK)`, desenhar o
  primeiro frame imediatamente.
- Pan com fallback; falha de `FBIOPAN_DISPLAY` desabilita pan, não é fatal.
- Cursor desenhado por software, última camada.
- Canvas lógico 1344×756 escalado ao espaço disponível, preservando a proporção
  e centralizado.
- Teclado via `crossterm` em `/dev/tty`.
- Sem evdev acessível: erro 4 antes de entrar em `KD_GRAPHICS`; sem dispositivo:
  tela de espera com hotplug.
- Restaurar `KD_TEXT`, cursor e limpeza em todos os caminhos.

## 9. Renderização compartilhada

- `render/scene.rs` descreve primitivas: retângulo, retângulo arredondado, linha,
  polilinha, círculo, anel, grade, texto e camadas com alpha.
- A cena é calculada uma vez a partir de `&App`; cada backend traduz.
- TUI: primitivas livres no `Canvas` Braille; painéis e texto com widgets.
- fb: rasterizador interno com RGBA e fonte 5×7 (D-016); sem `tiny-skia` na v1.
- Cores por tokens semânticos com variantes truecolor/256/16/mono.
- Sem imagens raster; efeitos e cursor são procedurais.
- Caches: fonte decodificada uma vez; glifos rasterizados em cache LRU; fundo
  estático pré-renderizado no tamanho do fb.
- Frame de QA: 1344×756, seed e tempo lógicos fixos.

## 10. CLI

```text
mouse_test [OPTIONS]

  --backend <auto|tui|fb>          padrão: auto (TUI)
  --fb-device <PATH>               padrão: /dev/fb0
  --device <auto|all|PATH|NAME>    dispositivo primário
  --list-devices                   lista e sai
  --all-devices                    inclui teclado/joystick na lista
  --grab                           captura exclusiva (opt-in)
  --grab-force                     permite grab em combo (perigoso)
  --require-evdev                  falha se evdev indisponível
  --tui-mouse <auto|on|off>        fallback de mouse do terminal
  --mode <dashboard|buttons|scroll|polling|targets>
  --lang <pt-BR|en>                padrão: LANG
  --seed <u64>                     determinismo
  --color <auto|truecolor|256|16|never>
  --no-animations
  --no-trail
  --raw-log
  --demo                           eventos sintéticos (depuração)
  --capture-frame <PATH>           PNG determinístico e sai
  --diagnostics <PATH>             relatório textual e sai
  -h, --help
  -V, --version
```

Regras:

- `auto` nunca escolhe fb.
- `--capture-frame` não abre áudio, framebuffer, teclado interativo ou evdev.
- `--diagnostics` não entra em modo interativo; pode exigir permissão para
  capacidades e deve indicar o que não conseguiu ler.
- `NO_COLOR` tem prioridade; `--color never` força monocromático.
- `--grab` recusado em combo, salvo `--grab-force` com aviso explícito.
- `--device all` agrega movimento de todos os mice, com cor por dispositivo;
  padrão é um primário.
- Ajuda do CLI em inglês; interface em pt-BR/en.

## 11. Códigos de saída

| Código | Significado |
| --- | --- |
| 0 | sucesso |
| 1 | erro de runtime/IO/captura |
| 2 | erro de CLI (clap) |
| 3 | backend indisponível (fb, TTY, tela) |
| 4 | entrada indisponível/permissão quando exigida (evdev/fb) |

Mensagens devem ser acionáveis: caminho, causa, comando sugerido e aviso de
segurança quando aplicável.

## 12. Permissões e ambiente

- evdev: `/dev/input/event*` tipicamente `root:input 660`.
- fb: `/dev/fb0` tipicamente `root:video 660`.
- O app nunca chama `sudo`, nunca altera permissões e nunca roda como root.
- Instalador documenta grupos `input` e `video`; adicionar ao grupo `input`
  permite keylogging por qualquer processo do grupo — aviso obrigatório.
- Verificações de sessão: `XDG_SESSION_TYPE`, `tty`, `/sys/class/tty/tty0/active`.
- O app não usa `inotify` na v1; varredura a cada 1 s.

## 13. Dependências e features

| Área | Crate | Motivo |
| --- | --- | --- |
| CLI | `clap` (derive) | parsing e help |
| TUI | `ratatui` | widgets, Canvas Braille |
| Terminal | `crossterm` | raw mode, teclado, fallback mouse |
| Raster | interno | fb, composição, PNG determinístico |
| Texto fb | fonte 5×7 interna | rasterização e layout simples |
| Sistema | `libc` | ioctls, poll, read, sinais |
| Erros | `thiserror` | erros de domínio/backend |
| Build | `xtask` (workspace) | verificação de assets |

Política:

- Sem `serde`, `tokio`, `libinput`, `memmap2`, `image`, `ratatui-image` ou
  `libloading` na v1.
- `--no-default-features` deve compilar e testar; default vazio.
- Features planejadas: `mock-input` (testes), `audio` (reservada, não implementar
  na v1).
- Toda dependência nova exige aprovação e entrada em `THIRD_PARTY_NOTICES.md`.

## 14. Desempenho e orçamentos

| Métrica | Alvo |
| --- | --- |
| Startup | < 300 ms |
| Latência evdev→frame | p95 ≤ 16 ms |
| Frame TUI | < 8 ms |
| Frame fb 1080p | ≤ 16 ms |
| Frame ocioso | ≤ 33 ms |
| RSS | < 80 MB |
| CPU ocioso | < 1% |
| Binário | < 12 MB |
| Assets | < 2 MB |
| Throughput | 10.000 eventos/s |
| Captura | determinística e < 1 s |

Medir em release, na máquina alvo; registrar números e desvios.

## 15. Determinismo e captura

- RNG SplitMix64 próprio com `--seed`; produção usa entropia do sistema.
- Modos com escolhas aleatórias (alvos) e captura usam seed fixa.
- Captura usa tempo lógico e estado sintético, sem depender de eventos reais.
- Mesmo build + mesma seed + mesmo estado ⇒ PNG idêntico.
- Snapshots de TUI comparam texto, não pixels.

## 16. Segurança e robustez

- Validar todos os valores do driver antes de mapear/escrever.
- `unsafe` restrito a `platform/`, com `// SAFETY:` e testes de ABI.
- Descartar eventos de teclado de combo sem armazenar keycodes.
- Não logar dados sensíveis; log bruto só de mouse.
- Overflow de canal é contado e visível, nunca silencioso.
- Restaurar terminal/console em todos os caminhos, inclusive pânico.
- Erros de backend passam pela restauração antes de sair.
- Não executar comandos externos em runtime.
- Caminhos de captura/diagnóstico são escolhidos pelo usuário; reportar falhas.

## 17. Documentos relacionados

- Fluxos e aceite: `docs/01-product.md`.
- Efeitos e layout: `docs/03-visual-design.md`.
- Testes: `docs/05-test-plan.md`.
- Evdev detalhado: `docs/09-input-spec.md`.
- ADRs e riscos: `docs/07-decisions-and-risks.md`.
