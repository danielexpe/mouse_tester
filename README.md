# MouseScope

> **Status: Fase 1 concluída.** O crate, o domínio puro, o CLI e uma TUI
> placeholder com entrada mock já existem. evdev, framebuffer, desafios guiados e
> captura ainda não foram implementados.
>
> Nome provisório: **MouseScope** (`mousescope`). Veja a questão aberta Q-001 em
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

## Estado atual (Fase 1)

Implementado:

- domínio puro de ponteiro, botões, scroll, efeitos e métricas;
- estimador de taxa com fixtures de 125 Hz e 1000 Hz;
- entrada mock e demo sintética (`--demo`);
- CLI com `--backend`, `--mode`, `--lang`, `--color`, `--list-devices`,
  `--no-animations`, `--no-trail`, `--raw-log`, `--seed` e `--demo`;
- TUI `ratatui` responsiva com dashboard placeholder, painéis, log e ajuda;
- guarda RAII do terminal e panic hook;
- 78 testes (74 unitários + 4 de integração de CLI).

Ainda não implementado: evdev, seleção real de dispositivo, hotplug, desafios
guiados, alvos, framebuffer, captura PNG e diagnóstico textual.

## Rodando a Fase 1

```bash
cargo run -- --demo          # TUI com eventos sintéticos
cargo run -- --list-devices  # lista o dispositivo mock
cargo test                   # testes de domínio, TUI e CLI
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
| VT de texto com evdev | fbdev | evdev | completo |
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
| `L` | Alternar log bruto |
| `R` | Reiniciar modo/contadores |
| `F2` | Alternar idioma (pt-BR/en) |
| `F1` ou `?` | Ajuda |
| `Q`, `Esc` ou `Ctrl+C` | Sair |
| Setas / `Enter` | Navegar em seletores e ajuda (Fase 2) |

`T` (seletor de dispositivo) e os desafios guiados pertencem à Fase 2; na Fase 1
o dashboard usa o dispositivo mock.

## Requisitos

- Linux x86_64, Rust stable (MSRV 1.88, edição 2021).
- Terminal local com TTY para a TUI; SSH funciona de forma degradada.
- Para evdev: permissão de leitura em `/dev/input/event*` (tipicamente grupo
  `input`; alguns sistemas usam `plugdev`).
- Para framebuffer: `/dev/fb0`, VT de texto ativo (`tty` = `/dev/ttyN`) e
  permissão (tipicamente grupo `video`).
- Sem rede, sem telemetria, sem fontes online.

## Instalação planejada

```text
mousescope                 # TUI, detecção automática de dispositivo
mousescope --backend fb    # console, exige VT e permissões
mousescope --list-devices
mousescope --diagnostics mousescope-debug.txt
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
  relatório; não há hover em muitos terminais.
- A taxa de relatório é uma **estimativa** medida, não a configuração nominal do
  mouse; coalescência do kernel/driver pode reduzi-la.
- Não mede DPI, latência de clique ponta a ponta, latência wireless ou qualidade
  do sensor. Isso é backlog.
- Sem framebuffer em Wayland/X11/SSH; diagnóstico recusa esses ambientes.
- Touchpad/tablet têm suporte básico apenas quando selecionados explicitamente.
- Não altera configurações do mouse (DPI, polling, RGB, macros).

## Licenças planejadas

- Código: MIT.
- Arte original e efeitos: CC BY 4.0.
- Fonte embutida: JetBrains Mono, OFL 1.1.
- Avisos de terceiros em `THIRD_PARTY_NOTICES.md` na implementação.
