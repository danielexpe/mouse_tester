# 07 — Decisões, riscos e questões abertas

## 1. Formato de ADR

```markdown
### D-0NN — Título curto

- **Status:** proposta | aceita | adiada | substituída.
- **Contexto:** o problema e as forças em jogo.
- **Decisão:** o que será feito.
- **Consequências:** custos, riscos e o que fica fora.
```

## 2. ADRs

### D-001 — Rust, Linux x86_64, TUI + fbdev

- **Status:** aceita.
- **Contexto:** o framework da pasta define Rust, console Linux, TUI e
  framebuffer; o produto é um teste visual de mouse.
- **Decisão:** usar Rust stable (edição 2021, MSRV 1.88), TUI como padrão e
  fbdev como backend explícito.
- **Consequências:** ótimo encaixe em console e offline; exige permissões de
  evdev/fb; Windows/macOS ficam fora.

### D-002 — evdev primário, crossterm fallback

- **Status:** aceita.
- **Contexto:** eventos de mouse do terminal não fornecem movimento relativo
  bruto, botões extras confiáveis nem taxa de relatório. O produto precisa
  desses dados.
- **Decisão:** ler `/dev/input/event*` via evdev como fonte primária. Na TUI,
  usar `crossterm` apenas como fallback degradado quando evdev não estiver
  acessível ou quando `--tui-mouse on` for explícito.
- **Consequências:** exige grupo `input`/permissão; o fallback é útil em SSH mas
  não mede movimento bruto nem taxa; a UI deve sinalizar o modo degradado.

### D-003 — UI somente por teclado

- **Status:** aceita.
- **Contexto:** o mouse é o objeto do teste; usar o mouse para navegar
  contaminaria o experimento e poderia acionar ações por acidente.
- **Decisão:** todos os controles são de teclado; eventos de mouse nunca mudam de
  modo, selecionam dispositivo ou saem.
- **Consequências:** usuário precisa do teclado; documentação e ajuda destacam
  atalhos.

### D-004 — Um estado, dois backends

- **Status:** aceita.
- **Contexto:** TUI e fbdev devem mostrar o mesmo comportamento.
- **Decisão:** um `App`/sessão e uma cena lógica compartilhada; `render/tui.rs` e
  `render/fb.rs` traduzem a mesma cena.
- **Consequências:** menos divergência; pequenas diferenças de layout são
  aceitas e documentadas.

### D-005 — Efeitos procedurais, sem arte bitmap

- **Status:** aceita.
- **Contexto:** o produto precisa de resposta visual rápida e binário pequeno;
  arte bitmap adiciona peso e pipeline.
- **Decisão:** desenhar cursor, rastro, anéis, pulsos, grade e painéis por
  primitivas. Nenhum sprite/textura na v1.
- **Consequências:** visual consistente e leve; menos “personalidade” gráfica;
  qualquer arte futura exigirá licença e pipeline.

### D-006 — Sem grab por padrão

- **Status:** aceita.
- **Contexto:** `EVIOCGRAB` captura o dispositivo exclusivamente, impedindo o uso
  normal do mouse e, em combo, podendo bloquear teclado.
- **Decisão:** grab é opt-in (`--grab`), com aviso; recusado em dispositivo combo
  salvo `--grab-force`. Sempre liberar em `Drop`/saída.
- **Consequências:** o cursor do sistema pode se mover durante o teste; grab
  melhora o isolamento, mas é risco operacional.

### D-007 — `auto` escolhe TUI

- **Status:** aceita.
- **Contexto:** detecção automática de fb é frágil (SSH, X/Wayland, container,
  `/dev/fb0` presente sem VT ativo).
- **Decisão:** `--backend auto` sempre escolhe TUI; fb só com `--backend fb`.
- **Consequências:** usuário de console precisa ser explícito; evita falsos
  positivos e telas invisíveis.

### D-008 — Sem áudio, rede, telemetria ou persistência na v1

- **Status:** aceita.
- **Contexto:** o foco é feedback visual e diagnóstico simples; cada item extra
  adiciona dependência e risco.
- **Decisão:** não implementar áudio, rede, telemetria, perfis ou histórico na
  v1. Feature `audio` fica reservada e desligada.
- **Consequências:** binário menor e offline; feedback sonoro e exportação vão
  para o backlog.

### D-009 — Sem `ratatui-image`

- **Status:** aceita.
- **Contexto:** não há arte bitmap; efeitos são procedurais. Protocolos de imagem
  no terminal adicionam complexidade e variação por terminal.
- **Decisão:** TUI usa `ratatui` Canvas com Braille; fb usa `tiny-skia`.
- **Consequências:** funciona em qualquer terminal; sem preview de imagens, o
  que não é necessário.

### D-010 — Hotplug por varredura periódica

- **Status:** aceita.
- **Contexto:** `inotify` exigiria mais código/descritores e tratamento de
  eventos parciais; 1 s é suficiente para o caso de uso.
- **Decisão:** varrer `/dev/input` a cada 1 s e comparar dispositivos; sem
  `inotify` na v1.
- **Consequências:** latência de até 1 s; menos complexidade; `inotify` pode
  voltar no backlog se necessário.

### D-011 — Idiomas

- **Status:** aceita.
- **Contexto:** o usuário fala português; o framework separa código e UI.
- **Decisão:** documentação em pt-BR; código, identificadores e ajuda do CLI em
  inglês; interface em pt-BR e en, com `F2` e detecção por `LANG`.
- **Consequências:** catálogo obrigatório; snapshots por idioma; ajuda do CLI não
  acompanha `--lang` na v1.

### D-012 — Métricas honestas

- **Status:** aceita.
- **Contexto:** DPI e polling nominal não são observáveis de forma confiável via
  evdev; drivers podem coalescer eventos.
- **Decisão:** reportar taxa de relatório **estimada**, jitter e confiança; nunca
  afirmar DPI ou polling configurado. Medição de DPI/latência fica no backlog.
- **Consequências:** menos marketing, mais confiança; usuário precisa de teste
  manual controlado para comparar valores.

### D-013 — Diagnóstico em texto simples

- **Status:** aceita.
- **Contexto:** diagnóstico precisa ser fácil de anexar em bug report e não deve
  adicionar `serde`.
- **Decisão:** `--diagnostics` grava texto `chave=valor` legível; sem JSON na v1.
- **Consequências:** parsing manual por humanos; se automação exigir, JSON entra
  no backlog com aprovação de dependência.

### D-014 — Modelo de permissões

- **Status:** aceita.
- **Contexto:** evdev e fb exigem grupos; rodar como root é inaceitável.
- **Decisão:** nunca root; documentar grupos `input` (ou dono real do
  dispositivo) e `video`; instalador apenas instrui por padrão e oferece flag
  opcional para adicionar grupos com aviso de segurança.
- **Consequências:** entrar no grupo `input` permite keylogging por qualquer
  processo do grupo; o aviso é obrigatório. Sem permissão, TUI degrada e fb
  falha com código 4.

### D-015 — Framebuffer na Fase 4

- **Status:** aceita.
- **Contexto:** fbdev tem armadilhas de console e exige hardware real; a TUI
  entrega valor antes.
- **Decisão:** implementar fbdev somente após a TUI funcional; se não houver
  hardware, registrar pendência formal em vez de declarar validação.
- **Consequências:** release pode sair com fb “pendente de validação”; o backend
  ainda deve compilar e passar testes sem hardware.

## 3. Opções de arquitetura (resumo)

Detalhes em `docs/02-technical-spec.md`.

| Opção | Motivo da decisão |
| --- | --- |
| Rust TUI + fbdev | escolhida; alinhada ao framework, offline, um domínio |
| Web app | rejeitada; sem evdev bruto, depende de navegador |
| GUI nativa | rejeitada; não atende console/VT |
| CLI de log | rejeitada; não é teste visual |
| SDL2/FFI | rejeitada; dependência C e fora do template |
| evdev direto | escolhida; dados brutos e capacidades |
| `crossterm` | fallback; limitado |
| libinput | rejeitada; dependência C e abstração |
| hidraw | backlog; parsing por fornecedor |

## 4. Riscos

| ID | Risco | Prob. | Impacto | Mitigação |
| --- | --- | --- | --- | --- |
| R-001 | Sem permissão em `/dev/input` | alta | alto | banner acionável, fallback TUI, diagnóstico, instalador com grupos |
| R-002 | Terminal não fornece movimento bruto | alta | médio | evdev primário; fallback explicitamente degradado |
| R-003 | fbdev falha em hardware/driver | alta | alto | validação em VT real, `testes_debug.sh`, pan fallback, fase separada |
| R-004 | Taxa estimada imprecisa | média | médio | timestamps do kernel, mediana, confiança, rótulo “estimado” |
| R-005 | Hi-res duplica notch | média | médio | política única + testes de fixture |
| R-006 | Botões extras variam por fornecedor | média | médio | rótulos por código, extras detectados, teste manual |
| R-007 | Hotplug/troca de fd com corrida | média | médio | thread única, comparação por identidade, testes |
| R-008 | Grupo `input` permite keylogging | média | alto | aviso obrigatório, sem root, opção de não adicionar |
| R-009 | Grab bloqueia mouse/teclado | média | alto | opt-in, recusa em combo, liberação em Drop |
| R-010 | Mouse do terminal conflita com seleção de texto | baixa | baixo | desabilitar quando evdev ativo; documentar Shift |
| R-011 | falso negativo de fb em Wayland/SSH | média | médio | diagnóstico exige `tty=/dev/ttyN` |
| R-012 | Overclaim de DPI/polling | média | alto | D-012; nunca afirmar nominal |
| R-013 | Overflow em 8 kHz | média | médio | canal 8192, drenagem por frame, contador visível |
| R-014 | Dependências crescem | média | médio | política mínima; sem serde/tokio/libinput |
| R-015 | `unsafe` de ioctl incorreto | média | alto | ABI testada, isolamento, comentários `// SAFETY:` |
| R-016 | Console não restaurado | baixa | alto | RAII, sinais, panic hook, testes PTY |
| R-017 | Privacidade em log bruto | baixa | alto | log só de mouse; teclado descartado antes do domínio |
| R-018 | Touchpad tratado como mouse | média | baixo | classificação por ABS/INPUT_PROP; seleção explícita |

## 5. Questões abertas

| ID | Questão | Default proposto | Prazo | Responsável |
| --- | --- | --- | --- | --- |
| Q-001 | Nome/binary definitivos? | `MouseScope`/`mousescope` | antes da Fase 1 | usuário |
| Q-002 | Distribuição pessoal ou release pública? | release pública | antes da Fase 5 | usuário |
| Q-003 | Instalador adiciona ao grupo `input`? | apenas instruir; flag `--add-input-group` | antes da Fase 5 | usuário |
| Q-004 | Hardware/distro para validar fb? | resolvida: Zorin OS 18.1 (base Ubuntu), VT `tty2`, 1920×1080 32 bpp | antes da Fase 4 | usuário |
| Q-005 | Touchpad na v1? | seleção explícita básica | antes da Fase 3 | usuário |
| Q-006 | Exportar métricas? | backlog CSV/JSON | Fase 6 | usuário |
| Q-007 | Medir DPI? | backlog | Fase 6 | usuário |
| Q-008 | Áudio? | fora da v1 | Fase 6 | usuário |

## 6. Premissas a validar antes da implementação

- [ ] O usuário tem ou conseguirá permissão de leitura em `/dev/input/event*`.
- [ ] O terminal alvo suporta TUI e, no modo degradado, mouse do terminal.
- [ ] Existe um VT de texto local para validar fb (ou aceitar pendência).
- [ ] O nome provisório é aceitável.
- [ ] MIT + CC BY 4.0 + OFL são aceitáveis.
- [ ] Não há requisito oculto de Windows/macOS, rede ou áudio.
- [ ] O escopo “teste visual + diagnóstico” é suficiente; não é ferramenta de
      configuração.

## 7. Política de mudança

- Mudança em decisão congelada exige novo ADR com status `proposta`, aprovação
  do usuário e atualização de `AGENTS.md`.
- ADRs substituídos não são apagados; recebem status `substituída` e referência
  ao novo ID.
- Riscos são revisados ao final de cada fase.
- Nenhuma decisão é alterada silenciosamente em código ou docs.
