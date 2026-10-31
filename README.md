# Mouse Test

> **Status: Fase 4 implementada e validada em VT.** A TUI lê evdev quando há
> permissão, descobre dispositivos, faz hotplug, oferece fallback degradado,
> desafios, Alvos e captura PNG. O backend framebuffer foi validado em VT
> local 1920×1080 32 bpp.
>
> Nome e binário: **Mouse Test** (`mouse_test`). Q-001 resolvida em
> [`docs/07-decisions-and-risks.md`](docs/07-decisions-and-risks.md).

Aplicativo Rust para console Linux que transforma cada ação de um mouse em
feedback visual imediato: movimento, botões, scroll vertical/horizontal, botões
extras, arraste e taxa de relatório. O objetivo é ser um **teste visual e
diagnóstico rápido** de mouse, com dois backends que compartilham o mesmo
domínio:

- **TUI** (`ratatui` + `crossterm`) no terminal, inclusive em SSH de forma
  degradada;
- **framebuffer** (`/dev/fb0`) em VT de texto, sem X11/Wayland, com cursor
  desenhado pelo próprio app.

A entrada principal é **evdev** (`/dev/input/event*`), que fornece movimento
relativo real, códigos de botão, wheel de alta resolução e timestamps do kernel.
A TUI pode cair para eventos de mouse do terminal (`crossterm`) quando não há
permissão de evdev, com limitações explicitamente sinalizadas.

## Estado atual (Fase 4)

Implementado:

- descoberta de dispositivos via `/proc/bus/input/devices` e classificação;
- leitura evdev em thread, com hotplug por varredura de 1 s;
- normalização de movimento, botões, scroll normal/hi-res e absolutos básicos;
- fallback degradado com mouse do terminal quando evdev é negado;
- seletor de dispositivo, desafio de botões e desafio de scroll;
- modo Alvos com clique, arraste e duplo clique, seed reproduzível e métricas;
- captura PNG determinística 1344×756 (`--capture-frame`), sem evdev ou TTY;
- backend framebuffer (`--backend fb`): 32/24/16 bpp, pan/fallback, cursor
  software, `KD_GRAPHICS`, sinais e restauração do console;
- snapshots de texto da TUI para alvos, mono e modo degradado;
- estimador de taxa com histograma, jitter e confiança;
- diagnóstico textual (`--diagnostics`) e lista de dispositivos;
- CLI com `--device`, `--fb-device`, `--grab`, `--grab-force`,
  `--require-evdev`, `--tui-mouse`, `--mode`, `--lang`, `--color`,
  `--list-devices`, `--capture-frame`;
- TUI `ratatui` com banner, log, ajuda e layouts Wide/Normal/Compact/Minimal;
- 164 testes (152 unitários + 7 de CLI + 5 snapshots).

Leitura evdev validada com o mouse HID `1bcf:08a0` (fonte `evdev`). O usuário
validou extras, scroll hi-res/horizontal, hotplug e o backend framebuffer em VT
1920×1080 32 bpp. Pendentes: 24/16 bpp em hardware, sinais/pânico do fb e
performance medida da TUI.

## Rodando

```bash
cargo run                          # TUI, evdev ou fallback
cargo run -- --demo                # TUI com eventos sintéticos
cargo run -- --list-devices        # lista pontos de entrada e permissões
cargo run -- --diagnostics diag.txt
cargo run -- --capture-frame captura.png --seed 7
cargo run -- --device /dev/input/event5
cargo run -- --backend fb          # requer VT local; validado 1920×1080 32 bpp
./testes_debug.sh                  # relatório único (fb preflight opcional)
./testes_debug.sh --with-fb        # inclui execução de 5 s no fb
cargo test                         # testes de domínio, TUI, input e CLI
```

## O que o app mostra

- **Dashboard ao vivo:** cursor virtual, rastro de movimento, velocidade,
  deslocamento bruto, contagem de eventos e relatórios.
- **Botões:** estado pressionado, contagem, duração, clique duplo, arraste e
  botões extras identificados por capacidade do dispositivo.
- **Scroll:** ticks verticais e horizontais, wheel de alta resolução, direção e
  pulsos visuais.
- **Taxa de relatório:** estimativa em Hz, jitter e confiança da amostra.
- **Desafio de botões:** guia o usuário a pressionar todos os botões suportados
  e marca os que faltam.
- **Desafio de scroll:** confirma cima/baixo/esquerda/direita.
- **Alvos:** nove rodadas de clique, arraste e duplo clique com acerto, erro,
  tempo e seed reproduzível.
- **Log bruto:** últimos eventos normalizados, com filtro por tipo.
- **Dispositivos:** descoberta, escolha do dispositivo primário, capacidades e
  hotplug.
- **Diagnóstico:** relatório textual de ambiente, permissões, dispositivos e
  framebuffer para anexar em bugs.

## Backends e modos de operação

| Situação | Backend | Entrada de mouse | Nível |
| --- | --- | --- | --- |
| Terminal com evdev acessível | TUI | evdev | completo |
| Terminal sem evdev | TUI | `crossterm` (limitado) | degradado |
| VT de texto com evdev | fbdev | evdev | completo (validado 1920×1080 32 bpp) |
| VT de texto sem evdev | fbdev | nenhuma | bloqueado com erro acionável |

`--backend auto` é conservador e escolhe **TUI**. O framebuffer só é usado com
`--backend fb` explícito.

## Controles

A navegação da interface é **somente por teclado**. O mouse é o objeto do teste,
nunca um controle da UI, para evitar que um clique de teste altere o aplicativo.

| Tecla | Ação |
| --- | --- |
| `1`..`5` | Dashboard, Botões, Scroll, Taxa, Alvos |
| `Tab` / `Shift+Tab` | Próximo/anterior modo |
| `T` | Seletor de dispositivo |
| `↑` / `↓` / `Enter` | Navegar e aplicar no seletor |
| `L` | Alternar log bruto |
| `R` | Reiniciar modo/contadores |
| `F2` | Alternar idioma (pt-BR/en) |
| `F1` ou `?` | Ajuda |
| `Q`, `Esc` ou `Ctrl+C` | Sair |
`T` abre o seletor de dispositivo. Os desafios de botões e scroll estão nos
modos 2 e 3; o modo Alvos (clique, arraste e duplo clique) é o modo 5.

## Requisitos

- Linux x86_64, Rust stable (MSRV 1.88, edição 2021). Validado em Zorin OS
  18.1; alvo de compatibilidade: distribuições base Ubuntu.
- Terminal local com TTY para a TUI; SSH funciona de forma degradada.
- Para evdev: permissão de leitura em `/dev/input/event*` (tipicamente grupo
  `input`; alguns sistemas usam `plugdev`). Sem ela, a TUI cai para o mouse do
  terminal; use `--require-evdev` para falhar em vez de degradar.
- Para framebuffer: `/dev/fb0`, VT de texto ativo (`tty` = `/dev/ttyN`) e
  permissão (tipicamente grupo `video`). O fb só é usado com `--backend fb`.
- Sem rede, sem telemetria, sem fontes online.

## Instalação planejada

```text
mouse_test                          # TUI, evdev ou fallback
mouse_test --backend fb             # console, exige VT e permissões
mouse_test --list-devices           # lista pontos de entrada
mouse_test --diagnostics diag.txt   # ambiente, permissões e capacidades
mouse_test --capture-frame qa.png   # PNG determinístico de QA
```

O pacote futuro incluirá binário, `README`, licenças, `install.sh` e
`testes_debug.sh`. O instalador nunca roda o app como root e avisa sobre o
impacto de segurança de entrar no grupo `input` (qualquer processo desse grupo
pode ler teclado).

## Índice de especificações

| Documento | Conteúdo |
| --- | --- |
| [`docs/01-product.md`](docs/01-product.md) | visão, fluxos, requisitos, aceite |
| [`docs/02-technical-spec.md`](docs/02-technical-spec.md) | arquitetura, módulos, CLI, desempenho |
| [`docs/03-visual-design.md`](docs/03-visual-design.md) | layout, paleta, efeitos, acessibilidade |
| [`docs/04-assets-and-licenses.md`](docs/04-assets-and-licenses.md) | fontes, pipeline, licenças |
| [`docs/05-test-plan.md`](docs/05-test-plan.md) | testes automatizados e matriz manual |
| [`docs/06-roadmap.md`](docs/06-roadmap.md) | fases e critérios de saída |
| [`docs/07-decisions-and-risks.md`](docs/07-decisions-and-risks.md) | ADRs, riscos e questões abertas |
| [`docs/08-i18n-catalog.md`](docs/08-i18n-catalog.md) | catálogo de strings pt-BR/en |
| [`docs/09-input-spec.md`](docs/09-input-spec.md) | especificação detalhada de evdev e fallback |
| [`docs/10-release-checklist.md`](docs/10-release-checklist.md) | checklist de release |
| [`AGENTS.md`](AGENTS.md) | memória operacional para agentes |

## Decisões principais

1. Um único estado de aplicação e dois backends de desenho.
2. **evdev é a fonte primária**; `crossterm` é fallback degradado na TUI.
3. Navegação da UI apenas por teclado; mouse é entrada sob teste.
4. Efeitos visuais procedurais; sem bitmaps de arte.
5. Sem áudio, sem rede e sem persistência na v1.
6. `auto` nunca escolhe framebuffer.
7. Sem `EVIOCGRAB` por padrão; grab é opt-in e recusado em dispositivos combo.

## Limitações conhecidas (por design ou ambiente)

- A TUI não consegue esconder o ponteiro do sistema operacional; no fbdev o
  cursor desenhado é o único.
- O fallback `crossterm` não fornece movimento relativo bruto nem taxa de
  relatório; não há hover em muitos terminais. Ele é ativado apenas quando evdev
  não está disponível (ou com `--tui-mouse on`).
- A leitura evdev foi validada com o mouse HID `1bcf:08a0`; dispositivos sem
  permissão em `/dev/input` caem no fallback degradado ou exigem o grupo
  `input`.
- A taxa de relatório é uma **estimativa** medida, não a configuração nominal do
  mouse; coalescência do kernel/driver pode reduzi-la.
- Não mede DPI, latência de clique ponta a ponta, latência wireless ou qualidade
  do sensor. Isso é backlog.
- Sem framebuffer em Wayland/X11/SSH; diagnóstico recusa esses ambientes.
- Touchpad/tablet têm suporte básico apenas quando selecionados explicitamente.
- A captura PNG e o framebuffer usam o mesmo pintor RGBA e fonte 5×7 embutidos
  (D-016); o texto é mais simples que uma fonte vetorial.
- O fb foi validado em VT 1920×1080 32 bpp; 24/16 bpp têm testes de parse e
  packing, mas ainda não foram exercitados em hardware.
- Não altera configurações do mouse (DPI, polling, RGB, macros).

## Licenças planejadas

- Código: MIT.
- Arte original e efeitos: CC BY 4.0.
- Fonte embutida: JetBrains Mono, OFL 1.1.
- Avisos de terceiros em `THIRD_PARTY_NOTICES.md` na implementação.
