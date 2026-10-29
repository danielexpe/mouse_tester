# Framework — Aplicação Rust para Console Linux com TUI + Framebuffer

> **O que é este arquivo:** um template de arquitetura e de processo para gerar
> especificações e implementações de aplicações Rust que rodam em terminal Linux
> (TUI) e em console local via framebuffer (`/dev/fb0`), com apoio de IA.
>
> **Como usar:** copie este arquivo para a raiz do novo projeto, renomeie-o se
> quiser, e trate cada seção marcada com `[PREENCHER]` como um campo da
> especificação. Mantenha as regras, checklists e anti-patterns: elas são o
> resultado de lições reais de um projeto entregue e validado em console.
>
> **Substituições:** `<APP>` = nome do app; `<BIN>` = nome do binário;
> `<DOMAIN>` = domínio do problema; `<SCREEN>` = tela do app. Não copie nomes,
> textos ou assets do projeto de origem; copie a **forma**.
>
> **Última atualização:** 2026-10-28. Incorpora lições de campo do backend
> fbdev: console blanked/DPMS, `yoffset`/pan, validação de `smem_len`,
> diagnóstico em hardware real e instalador com acesso ao grupo `video`.

---

## 0. Resumo executivo

Uma aplicação deste tipo tem três metas simultâneas:

1. **Lógica independente de interface** — domínio puro, testável sem terminal.
2. **Experiência TUI completa** — `ratatui` + `crossterm`, com fallbacks.
3. **Experiência idêntica no console** — framebuffer fbdev, sem X/Wayland,
   com restauração impecável do console.

A arquitetura de referência usa **um único estado de aplicação** e **dois
backends de saída** que consomem o mesmo modelo:

```text
+-------------------+      +----------------------+
| core (puro)       |      | assets / fontes      |
| regras, modelos   |      | embutidos            |
+---------+---------+      +----------+-----------+
          |                           |
          v                           v
+-------------------+      +----------------------+
| state / sessão    |      | render compartilhado |
| telas, seleção,   |----->| cena, texto, tokens  |
| timers, RNG       |      +----+------------+----+
+---------+---------+           |            |
          |                     v            v
          v              +-----------+  +-----------+
+-------------------+    | TUI       |  | fbdev     |
| app / loop        |--->| ratatui   |  | /dev/fb0  |
| input, áudio, cli |    +-----------+  +-----------+
+-------------------+          |            |
                               v            v
                        terminal ANSI   console KD_GRAPHICS
```

**Regra de ouro:** nenhum módulo de domínio pode conhecer `ratatui`,
`crossterm`, `libc`, framebuffer, áudio ou assets. O `app` orquestra; os
backends apenas desenham e capturam eventos.

---

## 1. Escopo, premissas e não-objetivos

### 1.1 Premissas

- Plataforma oficial: **Linux x86_64 local**.
- Rust stable, edição 2021, MSRV declarada e testada (sugestão: 1.88+).
- TUI funciona em terminal local; SSH é melhor esforço.
- Console alvo tem `/dev/fb0` (efifb, simpledrm, amdgpu, i915 etc.).
- Runtime **sem rede**; nada de telemetria, APIs remotas ou fontes online.
- Entrada por teclado; mouse fora do escopo inicial.
- Assets e fontes embutidos no binário; ele deve funcionar copiado sozinho.
- Áudio opcional, sintetizado em runtime, sem arquivos e sem bloquear o app.

### 1.2 Não-objetivos iniciais

- Windows, macOS, Wayland nativo ou X11 nativo.
- DRM/KMS na primeira entrega do framebuffer (fbdev basta; KMS é backlog).
- Persistência de configurações/histórico, salvo pedido explícito.
- Multiplayer em rede, contas, nuvem.
- Mouse, gamepad, acessibilidade avançada além de contraste/redução de movimento.
- Instalador gráfico; entrega é binário + documento.

### 1.3 Requisitos de produto `[PREENCHER]`

| Campo | Valor |
| --- | --- |
| Nome / marca | `<APP>` |
| Subtítulo | `<...>` |
| Gênero / propósito | `<DOMAIN>` |
| Público | `<...>` |
| Telas | `<SCREEN>`… |
| Fluxo principal | `<...>` |
| Diferenciais | `<...>` |
| Licença do código | MIT (sugestão) |
| Licença da arte | CC BY 4.0 (sugestão) |
| Plataforma | Linux console local |

---

## 2. Arquitetura de referência

### 2.1 Camadas e dependências

```text
main.rs / cli.rs
      |
      v
app/ (máquina de telas, loop, timers, áudio, input)
  |        |              |
  v        v              v
state/   render/        audio/
  |      /      \          |
  v     v        v         v
core/  tui     fb     assets/
(puro)         |
               v
          platform/ (libc, ioctls, mmap, sinais)
```

Regras:

- `core/` não importa nada de UI; é função pura sempre que possível.
- `state/` modela a sessão e as transições; pode depender de `core/`.
- `app/` é o único que chama backend, áudio e relógio.
- `render/tui.rs` pode depender de `ratatui`; `render/fb.rs` de `tiny-skia`,
  `fontdue` e assets; `render/scene.rs`/`text.rs` são compartilhados.
- `platform/fb.rs` é o **único** lugar com `unsafe` de sistema (ioctls,
  `mmap`, sinais), sempre com comentário de segurança.
- `util/` contém utilidades sem estado global (RNG, guardas de terminal).

### 2.2 Módulos mínimos

```text
src/
  main.rs            # parse da CLI, escolha do backend, códigos de saída
  lib.rs             # reexporta módulos (permite testes de integração)
  cli.rs             # flags e enums de opções
  app.rs             # estado, transições, loop TUI e loop fb
  input.rs           # Action + map_key
  core/              # domínio puro + testes
  state/             # sessão, telas, configurações
  render/
    mod.rs
    layout.rs        # breakpoints e geometria TUI
    scene.rs         # cena lógica compartilhada
    text.rs          # texto no fb (fontdue + cache de glifos)
    tui.rs           # desenho ratatui
    fb.rs            # canvas, escala, blend, telas do fb
    image.rs         # protocolos de imagem e fallback
  platform/
    mod.rs
    fb.rs            # device, console, ioctls, mmap, sinais
  audio.rs           # síntese e backend ALSA dinâmico
  assets.rs          # include_bytes!, decode e cache
  capture.rs         # PNG determinístico de QA
  i18n/              # catálogo de strings
  theme.rs           # tokens de cor e detecção de cor
  util/
    rng.rs           # RNG determinístico
    tty.rs           # guardas de terminal
assets/
  src/               # fontes originais (SVG etc.)
  generated/         # derivados versionados
  fonts/             # TTF + licenças
xtask/               # pipeline de assets
tests/snapshots/     # snapshots de TUI
docs/                # especificações e checklists
AGENTS.md            # memória operacional para IA
```

### 2.3 Contrato conceitual dos backends

Não é obrigatório usar `trait Backend`; dois loops podem compartilhar o mesmo
`App`. O contrato é:

1. **Entrar** com guarda RAII (raw mode, alternate screen ou `KD_GRAPHICS`).
2. **Desenhar** a partir de `&App` sem mutar estado.
3. **Apresentar** (flush TUI ou presente no fb).
4. **Ler eventos** com timeout, mapeando para `Action`.
5. **Restaurar** terminal/console em saída normal, erro, pânico e sinais.
6. **Nunca** imprimir logs no meio da tela; erros vão para stderr ou diário.

---

## 3. Stack, dependências e features

### 3.1 Stack recomendada

| Área | Crate | Observação |
| --- | --- | --- |
| CLI | `clap` (derive) | help/version e validação |
| TUI | `ratatui` + `crossterm` | crossterm lê `/dev/tty` quando necessário |
| Imagens no terminal | `ratatui-image` | Kitty/Sixel/iTerm2 + fallback |
| Raster | `tiny-skia` | cena, composição, PNG de QA |
| Imagens | `image` (png) | decode/encode |
| Texto no fb | `fontdue` | rasterização + `layout` com wrap |
| Sistema | `libc` | ioctls, termios, sinais, `mmap` |
| Erros | `thiserror` | erros de domínio/backends |
| Áudio opcional | `libloading` | ALSA dinâmico, feature `audio` |
| Assets | `xtask` (workspace) | SVG → PNG, manifesto, `--check` |

### 3.2 Política de dependências

- **Menos é mais.** Só adicione crate após verificar se o cache local já tem e
  se a licença é compatível. Download exige aprovação explícita.
- Evite `memmap2` se já há `libc`: `mmap`/`munmap` direto é pequeno e testável.
- Evite dependências de rede em runtime, mesmo opcionais.
- Prefira crates maduros e puramente Rust quando possível.
- Toda dependência nova entra em `THIRD_PARTY_NOTICES.md` e no orçamento de
  tamanho.
- `--no-default-features` deve compilar e testar; features opcionais não podem
  vazar para o núcleo.

### 3.3 Features sugeridas

```toml
[features]
default = ["audio"]
audio = ["dep:libloading"]
```

O backend fb é compilado por `cfg(target_os = "linux")`; em outros sistemas a
flag `--backend fb` falha com mensagem clara, sem `cfg` espalhado pelo domínio.

---

## 4. CLI, ambiente e códigos de saída

### 4.1 Template de flags

```text
<BIN> [OPTIONS]

  --lang <pt|en>                     idioma inicial
  --backend <auto|tui|fb>            saída de vídeo
  --fb-device <PATH>                 padrão: /dev/fb0
  --seed <u64>                       determinismo
  --color <auto|truecolor|256|16|never>
  --image-protocol <auto|kitty|sixel|iterm2|halfblocks|none>
  --no-images
  --no-animations
  --no-audio
  --capture-frame <PATH>             PNG determinístico e sai
  -h, --help
  -V, --version
```

### 4.2 Regras

- `--backend auto` é **conservador**: usa TUI; nunca tenta fbdev sozinho.
  Detecção automática de console é frágil (SSH, X/Wayland, containers,
  `/dev/fb0` presente sem VT ativo).
- `--backend fb` é explícito e exige VT local + permissão.
- `--capture-frame` não abre áudio, framebuffer nem terminal interativo.
- `NO_COLOR` tem prioridade; `--color never` força monocromático.
- Determinismo: `--seed` controla o RNG e decisões de timing que afetam jogadas.
- Nada de ler stdin para detectar protocolo de imagem; use ambiente + flags.

### 4.3 Códigos de saída

| Código | Significado |
| --- | --- |
| 0 | sucesso |
| 1 | erro de runtime/IO/captura |
| 2 | erro de CLI (clap) |
| 3 | backend indisponível (fb sem permissão/VT, TUI sem TTY) |

Mensagens de erro devem ser **acionáveis**: caminho, causa e o que fazer.

---

## 5. Máquina de estados, loop e tempo

### 5.1 Estado explícito

Modele telas como enum e transições em métodos nomeados:

```text
Screen = Home | Selecting | Playing | Paused | Finished | ...
Action = Up | Down | Left | Right | Confirm | Back | ToggleLanguage | ...
```

- `handle(Action) -> bool` centraliza o roteamento.
- Seleções por tela são índices com `rem_euclid`.
- Timers usam `Instant` no `App`; **nunca** `sleep` dentro da lógica.
- Ações globais (idioma, som, sair) são tratadas antes da tela.
- Teclas não mapeadas são ignoradas.

### 5.2 Loop

```text
while !quit:
    update(now)                 # vence timers: tarefas, animações, status
    draw()                      # TUI flush ou fb presente
    poll(timeout) -> eventos    # timeout = pacing da animação
    handle(action)
restore()
```

- Tempo ocioso maior (ex.: 100 ms); animação a 16 ms (60 FPS) ou 33 ms.
- Nunca bloqueie por mais de ~100 ms sem tratar sinais/saída.
- Efeitos colaterais (áudio, sons) ficam no `App`, não no renderer.
- Pausa cancela/reagenda tarefas pendentes em vez de matá-las pela metade.
- `Ctrl+C` em raw mode **não** gera SIGINT: mapeie a tecla para `Quit`.

### 5.3 Determinismo e RNG

- RNG próprio (ex.: SplitMix64) com seed explícita.
- Sorteios de desempate devem ser reprodutíveis com a mesma seed.
- Testes usam seed fixa; produção usa entropia do sistema.

---

## 6. Renderização compartilhada

### 6.1 Canvas lógico

- Defina um canvas fixo (referência: **70% de 1920 × 1080 = 1344 × 756**).
- No fb: fundo ocupa a tela inteira; composição é escalada e centralizada.
- Na TUI: layout adaptativo por breakpoints (Wide/Compact/Minimal).
- A cena do domínio deve ser desenhada uma vez e reutilizada pelos dois
  backends (mesmo que a TUI use imagem/blocos e o fb use o pixmap direto).

### 6.2 Assets e cena

- Assets originais em fonte editável (SVG), derivados versionados (PNG).
- Pipeline `xtask` determinístico + comando `--check` como portão.
- Nada de decodificar PNG a cada frame: use caches (`OnceLock`) e pré-renderize
  fundos estáticos.
- Embuta tudo com `include_bytes!` e teste que cada asset decodifica.
- Cores em tokens semânticos (`Surface`, `Text`, `Accent`, `Danger`…), com
  variantes truecolor/256/16/mono.
- Evite glifos que as fontes não têm (✦, ▸, setas exóticas). Prefira desenhar
  primitivas ou verificar cobertura.
- Layout de texto no fb: use `fontdue::layout` (wrap por palavra, alinhamento,
  `line_height`); meça antes de centralizar.
- Alpha: `tiny-skia` trabalha com RGBA **pré-multiplicado**; respeite isso no
  blend manual e nos testes.

### 6.3 Cache e custo por frame

Ordem de otimização que funcionou:

1. Cachear decode de imagens e rasterização de glifos.
2. Pré-renderizar fundo estático no tamanho do dispositivo.
3. Caminho rápido 1:1 para composição sem escala.
4. Escala bilinear apenas quando necessário (não use nearest em texto).
5. Só então considere cache por assinatura da cena.

Referência medida em release 1080p: cena ~12 ms, composição 1:1 ~1,7 ms,
frame de animação a 16 ms. Meça na máquina alvo; não confie só no desktop.

---

## 7. Backend TUI

### 7.1 Ciclo de vida

- Exigir stdin **e** stdout TTY (`ensure_tty`) antes de entrar.
- Guarda RAII: `enable_raw_mode`, `EnterAlternateScreen`, `Hide`.
- Restaurar em `Drop`, em erro e em panic hook.
- Em panique, restaurar antes de imprimir o erro.
- Nunca deixar o cursor escondido ou alternate screen ativo.

### 7.2 Layout e imagens

- Breakpoints com mínimos documentados (ex.: 80 × 24).
- Redimensionar preserva todo o estado.
- Protocolos de imagem em tiers: Kitty/Sixel/iTerm2 → half-blocks → textual.
- Detecção por ambiente (`KITTY_WINDOW_ID`, `TERM`, `TERM_PROGRAM`) + flag
  explícita; fallback obrigatório.
- Imagens automáticas podem exigir truecolor; documente.

### 7.3 Snapshots

- Prefira comparador próprio (texto) a dependência extra.
- `UPDATE_SNAPSHOTS=1 cargo test` regenera; revise o diff.
- Versão da aplicação no rodapé muda snapshots a cada release — esperado.
- Snapshots cobrem telas, idiomas, breakpoints e estados de erro.

---

## 8. Backend framebuffer (fbdev)

### 8.1 Pré-requisitos e diagnóstico

```bash
ls -l /dev/fb0
cat /sys/class/graphics/fb0/name
cat /sys/class/graphics/fb0/virtual_size
cat /sys/class/graphics/fb0/bits_per_pixel
cat /sys/class/graphics/fb0/stride
cat /sys/class/graphics/fb0/blank   # 0 = ligado; qualquer outro = blank/DPMS
cat /proc/fb
cat /sys/class/tty/tty0/active
tty                                 # deve ser /dev/ttyN, nunca /dev/pts/N
id -nG                              # deve conter video
fbset -i 2>/dev/null || echo "fbset ausente"
```

- Permissão típica `root:video 660`; adicionar o usuário ao grupo `video`
  exige **novo login** (ou `newgrp video`).
- `/dev/fb0` existir **não** significa que o processo está em um VT ativo.
- Fora de VT (X/Wayland/SSH), `KDSETMODE` falha com `ENOTTY`: recuse com
  mensagem acionável, sem tentar adivinhar.
- **Rode o diagnóstico de dentro de um VT de texto.** `tty` deve retornar
  `/dev/ttyN`; se retornar `/dev/pts/N`, `XDG_SESSION_TYPE=wayland` ou você
  estiver via SSH, o teste não exercita o framebuffer e produz falso negativo.
- **Console blanked/DPMS é a primeira suspeita quando a tela só aparece ao
  pressionar `Ctrl+C`.** O `Ctrl+C` é entrada de teclado e acorda o console;
  o app não deve depender disso.
- Colete sempre, no hardware afetado: driver, `virtual_size`, `bits_per_pixel`,
  `stride`, `blank`, `yoffset`, `yres_virtual`, `ypanstep`, `line_length`,
  `smem_len`, VT ativo e as chamadas `ioctl` do app.
- `fbset` pode não estar instalado; os valores de `/sys` e `strace` bastam.
- Entregue um script de diagnóstico reproduzível (ver 8.1.1) em vez de pedir
  comandos soltos ao usuário.

#### 8.1.1 Script de diagnóstico no hardware afetado

Inclua no projeto/release um `testes_debug.sh` que grave tudo em
`testes_debug.txt`. Ele deve:

1. Recusar ou avisar se não estiver em um VT de texto (`/dev/ttyN`).
2. Coletar `/etc/os-release`, `uname`, `id`, `loginctl`, `tty`, VT ativo.
3. Coletar `/dev/fb*`, `/proc/fb`, `/sys/class/graphics/fb*/*`, `fbset -i`,
   `vtconsole`, `fbcon`, `/dev/dri`, drivers DRM e `dmesg` (quando acessível).
4. Rodar o app real sob `strace -e ioctl,openat,mmap,munmap,write`, com
   `timeout`, **antes** de qualquer teste que altere o estado do console.
5. Testar `FBIOBLANK`/`FB_BLANK_UNBLANK`, escrita direta de faixas RGB,
   `FBIOPAN_DISPLAY` e restauração de `KD_TEXT`.
6. Rodar o app novamente após o desbloqueio, para comparar.
7. Restaurar `KD_TEXT`/cursor mesmo em erro/interrupção.

O script não deve instalar nada nem rodar como root. O usuário roda no VT e
envia o `testes_debug.txt`.

### 8.2 Abertura e informações

- Abrir com `O_RDWR | O_CLOEXEC`.
- `FBIOGET_VSCREENINFO` (`0x4600`) e `FBIOGET_FSCREENINFO` (`0x4602`).
- Estruturas do kernel: `fb_var_screeninfo` (160 bytes) e
  `fb_fix_screeninfo` (80 bytes) em x86_64. Teste os tamanhos.
- Validar: `xres/yres > 0`, `line_length >= xres * bytes_por_pixel`,
  `yres_virtual >= yres` e `yoffset <= yres_virtual - yres`.
- Ler e guardar o `yoffset` atual. **Nunca assuma que é zero.**
- Validar `smem_len`: ele precisa cobrir o buffer visível
  (`line_length * (yoffset + yres)`) e, se houver pan, os dois buffers
  (`line_length * 2 * yres`). Mapear exatamente o necessário (ou o máximo
  entre visível e pan), nunca um valor arbitrário.
- `yres_virtual == yres` é comum em fbdev sobre DRM (por exemplo `i915drmfb`)
  e significa **sem ypan**; escreva no buffer visível.
- Mapear com `mmap` (`PROT_READ|PROT_WRITE`, `MAP_SHARED`) e desmapear em
  `Drop`. Não é necessário `memmap2`.
- Validar o formato antes de desenhar; erro deve informar bpp e offsets.
- Respeite `line_length` como stride, inclusive no padding da última linha.

### 8.3 Formatos suportados

| bpp | Layout | Aceitação |
| --- | --- | --- |
| 32 | RGBX/BGRX/XRGB/XBGR | R/G/B com 8 bits; alpha 0 ou 8 |
| 24 | RGB/BGR | R/G/B com 8 bits; 3 bytes little-endian |
| 16 | RGB565 | R 5, G 6, B 5 |

Conversão genérica por bitfield:

```text
canal(valor8, offset, len) =
    len == 0 ? 0 : ((valor8 * ((1<<len)-1) + 127) / 255) << offset
```

- Little-endian é a ABI alvo; documente a premissa.
- Respeite `line_length` (padding entre linhas).
- Em 16 bpp a fidelidade é menor; documente.
- Formatos desconhecidos: erro claro, nunca “chutar”.

### 8.4 Apresentação

- `ypan`/double buffer só se **todas** as condições valerem:
  `ypanstep > 0`, `yres_virtual >= 2*yres`, `yres % ypanstep == 0` e
  `smem_len >= line_length * 2*yres`.
- O buffer de destino é o **não visível agora**, derivado do `yoffset` atual:
  `target = current_yoffset == 0 ? yres : 0`.
- Escreva o frame completo no `target`, chame `FBIOPAN_DISPLAY` (`0x4606`) com
  `yoffset = target` e só então atualize `current_yoffset`.
- Se `FBIOPAN_DISPLAY` falhar (`EINVAL`, `ENOTSUP` ou outro erro), **desabilite
  o pan permanentemente** e escreva no buffer visível atual. Não encerre o jogo
  por causa disso.
- Sem `ypan`: escrever o frame completo começando em
  `current_yoffset * line_length`; nunca em `0` por suposição.
- Mapear memória suficiente para o buffer de pan **e** para o buffer visível
  usado no fallback.
- Um driver pode reportar `ypanstep > 0` e ainda assim não honrar o pan; o
  fallback é obrigatório.
- Fundo pré-renderizado no tamanho do dispositivo.
- Composição lógica escalada por fator calculado (70% do menor eixo) e
  centralizada; caminho rápido quando escala ≈ 1.
- Bilinear para escala fracionária; alfa pré-multiplicado.
- Pacing: 60 FPS em animação, ~10 FPS ocioso. Sem vsync garantido.
- DRM/KMS fica no backlog; não misture as APIs na primeira versão.

Modelo de referência:

```text
current = var.yoffset
pan_ok = ypanstep > 0
         && yres_virtual >= 2*yres
         && yres % ypanstep == 0
         && smem_len >= line_length * 2*yres

map_len = pan_ok
    ? max(line_length * 2*yres, line_length * (current + yres))
    : line_length * (current + yres)

present(frame):
    if pan_ok:
        target = current == 0 ? yres : 0
        write_frame(frame, target * line_length)
        if pan(target):
            current = target
            return
        pan_ok = false
    write_frame(frame, current * line_length)
```

### 8.5 Console e entrada

- Abrir `/dev/tty` para ioctls e saída ANSI (cursor/limpeza).
- Guardar o modo anterior com `KDGETMODE` (`0x4B3B`) e entrar em
  `KDSETMODE` (`0x4B3A`) com `KD_GRAPHICS` (`1`).
- **Desbloquear o console** logo depois de entrar em `KD_GRAPHICS`:
  `FBIOBLANK` (`0x4611`) com `FB_BLANK_UNBLANK` (`0`). Ignorar falha é
  aceitável; muitos drivers não implementam blanking.
- `KD_GRAPHICS` **não** desbloqueia um console que chegou blanked/DPMS off.
  Sem esse ioctl, o app desenha no framebuffer invisível até uma tecla (como
  `Ctrl+C`) acordar o console.
- Restaurar `KD_TEXT` (`0`) na saída, erro, pânico e sinais.
- Raw mode via `crossterm`; ele prefere `/dev/tty`, não dependa de stdin.
- Esconder cursor e limpar a tela ao entrar; mostrar cursor e limpar ao sair.
- Teste o cenário real: chegue ao VT, espere o console blankar e só então
  inicie o app; a primeira tela deve aparecer sem tocar no teclado.

### 8.6 Ordem das guardas

```text
1. Abrir e validar /dev/fb0, validar yoffset/smem_len e mapear memória
2. Abrir /dev/tty, salvar KDGETMODE, entrar em KD_GRAPHICS
3. FBIOBLANK(FB_BLANK_UNBLANK) — desbloquear blank/DPMS
4. enable_raw_mode
5. primeiro frame imediatamente (sem esperar tecla)
6. loop
7. disable_raw_mode
8. restaurar KD_TEXT, mostrar cursor, limpar
9. munmap e fechar descritores
```

Se qualquer passo falhar, os passos anteriores devem ser desfeitos. Use RAII
(`Drop`) e não confie apenas no caminho feliz.

### 8.7 Sinais e pânico

- `SIGINT`, `SIGTERM`, `SIGHUP` com `sigaction` e flag atômica; `SA_RESTART`.
- Handler só marca a flag; restauração acontece no loop/`Drop`.
- `Ctrl+C` em raw mode chega como evento de tecla, não como sinal.
- Panic hook restaura console **antes** de imprimir o erro.
- `unsafe` só no módulo de plataforma, com comentário `// SAFETY:`.

### 8.8 Limitações a documentar

- Troca de VT durante a execução pode deixar resíduos; sair e reiniciar.
- Sem vsync: tearing leve aceitável.
- Sem DRM/KMS; hardware sem `/dev/fb0` não é suportado.
- `auto` não escolhe fb.
- Resolução lida no início; mudanças de modo exigem reinício.
- O console pode chegar blanked/DPMS off; o app deve desbloquear no início.
- O blank por inatividade pode voltar depois; entrada de teclado acorda, mas o
  jogo não deve depender disso para o primeiro frame.
- Alguns drivers anunciam ypan e não o honram; o fallback para o buffer visível
  é parte do contrato.

---

## 9. Entrada, acessibilidade e i18n

### 9.1 Entrada

- Mapeamento único de teclas para `Action`: setas/WASD, Enter/Espaço, Esc,
  atalhos globais (idioma, som, sair), `R` para repetir quando fizer sentido.
- Ignorar `KeyEventKind::Release` (use `Press`).
- Debounce de confirmação para evitar duplo toque (ex.: 150 ms).
- Sem mouse na v1.

### 9.2 Acessibilidade mínima

- Contraste adequado em fundos escuros e claros.
- `NO_COLOR` e modo monocromático.
- `--no-animations` reduz movimento e encurta atrasos.
- Controles visíveis no rodapé/pausa; sem tutorial separado obrigatório.
- Nada de depender só de cor para indicar estado (use marcador/forma).

### 9.3 i18n

- Catálogo com todas as strings e idiomas suportados.
- `F2` alterna idioma em todas as telas, inclusive pausa/resultado.
- Teste que strings longas não estouram o layout.
- Marca pode permanecer no idioma original; traduza subtítulos/descrições.
- Versão e status de som aparecem de forma consistente.

---

## 10. Áudio

- Síntese em runtime (ondas simples/envelopes), sem arquivos e sem música.
- Volume padrão baixo (ex.: 0,32).
- Backend ALSA carregado dinamicamente (`libasound.so.2` + `libloading`);
  ausência de biblioteca/dispositivo = silêncio, nunca erro fatal.
- Feature `audio` desligável; `--no-audio` e `M` alternam mudo.
- Eventos em thread separada com canal limitado; não bloquear o loop.
- Teste síntese (sem NaN, pico ≤ 1,0, duração) sem precisar de hardware.
- Em console, valide o áudio real; em SSH, degrade em silêncio.

---

## 11. Assets, fontes e licenças

### 11.1 Pipeline

- Fontes originais versionadas (`assets/src/`), derivados em
  `assets/generated/` com manifesto.
- `cargo xtask assets` gera; `cargo xtask assets --check` falha se divergir.
- Nunca editar PNG gerado à mão.
- Validar dimensões e decode em testes.
- Embutir no binário; testar execução copiada para `/tmp` sem a pasta de assets.

### 11.2 Licenças

- Código: MIT (ou compatível).
- Arte original: CC BY 4.0 com crédito.
- Fontes: OFL 1.1, incluindo textos de licença no pacote.
- `THIRD_PARTY_NOTICES.md` com todas as crates e licenças.
- Não embutir asset de terceiro sem licença compatível e crédito.

---

## 12. Testes e qualidade

### 12.1 Pirâmide

1. **Unitários de domínio**: regras, modelos, transições, RNG, placar.
2. **Propriedades/invariantes**: estados impossíveis, somas, simetria,
   invariantes de jogo/IA (se houver), limites.
3. **Snapshots de TUI**: texto por tela/idioma/breakpoint.
4. **Integração/PTY**: entrada real, resize, saída limpa.
5. **Sem hardware**: formatos, ABI, conversão, geometria, blend, telas.
6. **Manual em hardware**: TUI em terminais reais e fb no VT.

### 12.2 Testes específicos do framebuffer (sem `/dev/fb0`)

- Parse de formato 32/24/16 e rejeição de desconhecidos.
- Tamanho das structs ABI (160/80).
- Empacotamento de pixels e escrita de linha (incluindo padding).
- Geometria 70% e offsets em 1080p/1440p/768p.
- Blending com alpha 0, opaco e parcial; escala 1:1 e fracionária.
- Renderização das telas gera frame do tamanho esperado e não vazio.
- Captura determinística 1344 × 756 (arte + texto).
- Erro de permissão/console com mensagem e código 3.
- Função pura de escolha do buffer: `current == 0 -> yres`;
  `current == yres -> 0`; `current inesperado -> 0`.
- Validação de `yoffset > yres_virtual - yres`, stride menor que a linha,
  `smem_len` insuficiente e geometria virtual inconsistente.
- Plano de mapeamento: sem ypan usa `(yoffset + yres)` linhas; com ypan cobre
  os dois buffers e o buffer visível usado no fallback.
- Pan com memória só para um buffer deve ser desabilitado, não fatal.

### 12.3 Portões automatizados

```bash
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo clippy --workspace --all-targets --no-default-features -- -D warnings
cargo test
cargo test --no-default-features
cargo xtask assets --check
cargo build --release
cargo build --release --no-default-features
```

Adicione, se possível, `cargo test --release` e verificação de MSRV.

### 12.4 Matriz manual mínima

- TUI: terminais principais, 80 × 24 e Wide.
- fb: 1920 × 1080 em 32 bpp; 24/16 bpp se houver hardware/VM.
- Fluxos completos em todos os modos.
- Redimensionar (TUI) e resoluções alternativas (fb).
- Idiomas, som, pausa, reinício, saída.
- Restauração: menu, `Ctrl+C`, SIGTERM, pânico.
- Troca de VT; registrar limitações.
- Nenhum resíduo visual ou cursor piscando.
- Rodar em **VT de texto real**: `tty` = `/dev/ttyN`, não `/dev/pts/N`;
  não usar Wayland, terminal gráfico ou SSH.
- Verificar `/sys/class/graphics/fb0/blank` antes do app e confirmar que ele
  chega a `0` após o startup; a primeira tela deve aparecer **sem `Ctrl+C`**.
- Testar com o console previamente blanked/DPMS off.
- Coletar driver, `virtual_size`, bpp, `stride`, `yoffset`, `yres_virtual`,
  `ypanstep`, `line_length`, `smem_len` e `strace -e ioctl` no hardware afetado.
- Se houver/for possível, validar em uma segunda máquina/driver; se não,
  registrar explicitamente como pendente.
- Testar o instalador em prefixo temporário (`PREFIX=/tmp/... SKIP_GROUP=1`) e
  o fluxo real com `sudo`/grupo `video`.

**Não declare validado o que não foi executado.** Marque como pendente.

---

## 13. Desempenho e orçamentos

| Métrica | Alvo sugerido |
| --- | --- |
| Startup até primeira tela | < 400 ms |
| Frame de animação (fb 1080p) | ≤ 16 ms |
| Frame ocioso | ≤ 33 ms, ~10 FPS |
| RSS | < 120 MB |
| CPU ocioso | < 2% |
| Binário release | < 15 MB |
| Assets embutidos | orçamento explícito (ex.: < 8 MB) |
| Latência de resposta a tecla | < 50 ms |
| Captura de frame | determinística e < 1 s |

Meça em release, na máquina alvo. Registre números e desvios nos documentos.

---

## 14. Segurança e robustez

- `unsafe` restrito a plataforma/TTY, com `// SAFETY:` e testes de ABI.
- Validar todos os valores vindos do driver antes de mapear/escrever.
- Não logar segredos; não armazenar credenciais.
- Não executar comandos externos em runtime.
- Não depender de rede.
- Restaurar terminal/console em **todos** os caminhos, inclusive pânico.
- Erros de backend são fatais e passam pela restauração.
- Ler entradas com limites; ignorar teclas desconhecidas.
- Arquivos de captura: caminho escolhido pelo usuário; reportar falhas claras.

---

## 15. Empacotamento e release

### 15.1 Sequência

1. Congelar escopo e atualizar versão (`Cargo.toml` + `Cargo.lock`).
2. Rodar todos os portões (com e sem features).
3. Regenerar snapshots e revisar (mudança de versão afeta rodapé).
4. Validar manualmente o máximo possível; registrar pendências.
5. `cargo build --release`.
6. Montar pasta do pacote: binário, README, licenças, notas, créditos,
   licenças de fontes, `install.sh` e `testes_debug.sh`.
7. `tar.gz` + `.sha256`.
8. Smoke test **fora do projeto** (`/tmp`), sem assets; testar o `install.sh`
   com `PREFIX=/tmp/... SKIP_GROUP=1`.
9. Tag e release no GitHub com notas + artefatos, incluindo os dois scripts
   como assets separados.
10. Registrar hash e URL na memória operacional (`AGENTS.md`).

### 15.2 Regras

- `dist/` e `*.tar.gz` no `.gitignore`; não versionar binários.
- Notas de release **não** devem conter o hash do próprio pacote (referência
  circular: o hash muda ao alterar as notas). Coloque o hash no checklist e no
  `AGENTS.md`/página da release.
- Valide o `.sha256` no diretório do pacote.
- Releases públicas: licença e créditos devem viajar junto.
- Nunca publique um pacote cujo binário não foi testado copiado sozinho.

### 15.3 Instalador (`install.sh`)

Inclua um instalador no pacote **e** como asset da release. Requisitos:

- Instalar binário em `/usr/local/bin` e documentos em
  `/usr/local/share/<APP>`; aceitar `--prefix` e `PREFIX=...`.
- Adicionar o usuário que executou o script ao grupo `video` (criar o grupo se
  faltar) e avisar que é preciso sair/entrar ou usar `newgrp video`.
- Usar `sudo` de forma interativa quando necessário; **nunca** rodar o app como
  root para “resolver” permissão.
- Aceitar o `.tar.gz` como argumento, encontrar um pacote adjacente ou usar o
  diretório já extraído; validar o `.sha256` quando disponível.
- Restaurar nada no sistema além dos arquivos instalados e do grupo.
- Oferecer `--skip-group` para testes/CI; testar com prefixo temporário.
- Incluir `testes_debug.sh` no pacote e como asset separado.

---

## 16. Documentação e memória operacional (AGENTS.md)

Todo projeto deve ter um índice e uma memória persistente:

```text
README.md                       # visão do usuário, instalação, controles
docs/01-<produto>.md            # regras, fluxos, aceite
docs/02-technical-spec.md       # arquitetura, crates, CLI, desempenho
docs/03-visual-design.md        # paleta, tipografia, layout
docs/04-assets-and-licenses.md  # pipeline, manifesto, licenças
docs/05-test-plan.md            # testes e matriz manual
docs/06-roadmap.md              # fases e critérios de saída
docs/07-decisions-and-risks.md  # ADRs, premissas, questões
docs/08-i18n-catalog.md         # strings e limites
docs/09..12-*                   # notas e checklists de release
AGENTS.md                       # estado operacional para IA
```

`AGENTS.md` deve conter:

1. Retomada rápida (ler arquivo, `git status`, docs).
2. Status atual (tabela por fase/item).
3. Resumo do produto.
4. Decisões congeladas (não reabrir sem o usuário).
5. Fontes de verdade (docs).
6. Pendências e questões abertas com prazo.
7. Comandos e portões.
8. Regras de trabalho do agente.
9. Protocolo de atualização do próprio arquivo.
10. Registro de atividades (data, resumo, arquivos, commits).
11. Fases.
12. Notas para a próxima sessão.

Regras: conciso, sem segredos, atualizar ao final da tarefa, registrar bloqueios
em vez de inventar progresso, registrar hash de commits e releases.

---

## 17. Processo de trabalho com IA

### 17.1 Fases

| Fase | Objetivo | Critério de saída |
| --- | --- | --- |
| 0 — Descoberta | escopo, regras, telas, hardware, licenças | specs aprovadas em escopo |
| 1 — Fundação | crate, núcleo puro, i18n, guards, CLI | testes de domínio + TUI placeholder |
| 2 — TUI funcional | fluxo principal completo | fluxo completo + snapshots |
| 3 — Lógica rica | regras/IA avançadas, determinismo | invariantes e seed |
| 4 — Arte/som/i18n | assets, pipeline, áudio, acessibilidade | tier de imagem + áudio |
| 5 — Release desktop | licenças, docs, medições, pacote | release vX.0 validado |
| 6 — Framebuffer | fbdev, texto, scaling, sinais | console validado + pacote |
| 7 — Backlog | DRM/KMS, persistência, temas | priorizado pelo usuário |

### 17.2 Regras para o agente

- Ler `AGENTS.md` e `git status` antes de agir.
- Não reimplementar fases concluídas.
- Não reabrir decisões congeladas sem pedido; proponha ADR.
- Não adicionar dependência/rede sem aprovação explícita.
- Não fazer commit, push, amend ou PR sem pedido explícito.
- Não configurar remote sem pedido.
- Manter `unsafe` isolado e comentado.
- Testar antes de declarar concluído; rodar portões.
- Não inventar validação manual; marcar pendente.
- Preservar terminal/console em qualquer saída.
- Atualizar `AGENTS.md` ao final de tarefas relevantes.
- Código/identificadores em inglês; UI nos idiomas do produto.
- Sem comentários desnecessários; siga o estilo existente.
- Diante de hardware/permissões indisponíveis, registrar bloqueio e seguir com
  o que for testável sem hardware.

### 17.3 Ordem de trabalho recomendada

1. Especificar antes de codar; validar escopo com o usuário.
2. Fundação e domínio puro com testes.
3. TUI funcional sem arte final.
4. Conteúdo/lógica avançada.
5. Arte, som, i18n e acessibilidade.
6. Fechar release desktop.
7. Só então framebuffer.
8. Validar em hardware real, incluindo uma segunda máquina/driver quando
   possível; se a tela falhar, rodar `testes_debug.sh` no VT antes de mudar o
   código.
9. Empacotar com `install.sh` e `testes_debug.sh`; publicar.

---

## 18. Templates

### 18.1 ADR

```markdown
### D-0NN — Título curto

- **Status:** proposta | aceita | adiada | substituída.
- **Contexto:** o problema e as forças em jogo.
- **Decisão:** o que será feito.
- **Consequências:** custos, riscos e o que fica fora.
```

### 18.2 Checklist de release

```markdown
## Funcional
- [ ] fluxo principal completo em todos os modos
- [ ] idiomas, som, atalhos e pausa
- [ ] captura determinística
- [ ] binário funciona copiado, sem assets

## Qualidade
- [ ] fmt / clippy / clippy no-default / test / assets --check
- [ ] build release com e sem features
- [ ] snapshots revisados
- [ ] MSRV verificada (se possível)

## Manual
- [ ] terminal(is) principais
- [ ] console: resolução e bpp alvo
- [ ] console em VT de texto real (`tty` = /dev/ttyN), não pts/Wayland/SSH
- [ ] `/sys/class/graphics/fb0/blank` chega a 0 e a primeira tela aparece sem Ctrl+C
- [ ] yoffset/yres_virtual/ypanstep/smem_len registrados
- [ ] fallback sem pan e com pan não honrado
- [ ] restauração: menu, Ctrl+C, SIGTERM, pânico
- [ ] redimensionar / trocar resolução
- [ ] áudio ou silêncio sem dispositivo
- [ ] diagnósticos (`testes_debug.txt`) anexados quando houver bug de hardware

## Release
- [ ] versão e lock atualizados
- [ ] pacote + sha256
- [ ] `install.sh` + `testes_debug.sh` no pacote e como assets
- [ ] smoke em /tmp
- [ ] instalação testada com prefixo temporário e com sudo/grupo video
- [ ] tag + release com notas e artefatos
- [ ] hash e URL no AGENTS.md
```

### 18.3 Prompt para gerar especificações a partir deste framework

```text
Você é o arquiteto de um novo projeto Rust para console Linux com TUI e
framebuffer. Use o arquivo framework_app_rust_console_linux_TUI_framebuffer.md
como template obrigatório.

Produto: <descreva o app, público, fluxos e telas>.
Hardware alvo: <resolução, bpp, GPU/driver, console local>.
Restrições: <licenças, offline, acessibilidade, idiomas, prazo>.

Gere:
1. README.md com visão do usuário e índice.
2. docs/01 a docs/08 seguindo o template, com decisões explícitas.
3. AGENTS.md com estado inicial, decisões congeladas e protocolo.
4. Roadmap por fases com critérios de saída verificáveis.
5. Perguntas abertas apenas onde houver risco real; proponha defaults.

Não implemente código nesta etapa. Aponte conflitos e lacunas.
```

### 18.4 Prompt para implementar uma fase

```text
Leia AGENTS.md, git status e os docs relevantes. Implemente somente a Fase <N>
com escopo <...>. Respeite as decisões congeladas e o template de arquitetura.

Antes de editar: liste arquivos e plano curto.
Depois: rode fmt, clippy (com e sem features), testes e assets --check.
Atualize docs e AGENTS.md. Não faça commit/push sem pedido.
Se faltar hardware/permissão, registre o bloqueio e valide o que for possível
sem ele.
```

### 18.5 Prompt de validação final

```text
Revise o repositório como se fosse um release iminente. Verifique:
- portões automatizados e MSRV;
- restauração em saída normal, erro, pânico, Ctrl+C e SIGTERM;
- console: permissões, VT, KD_TEXT, cursor, frame completo;
- formatos 24/16 bpp e resoluções alternativas (ou marque pendente);
- snapshots, licenças, notices e docs sem drift;
- pacote em /tmp sem assets e checksum.
Entregue uma lista objetiva de bloqueios e pendências, sem inventar validação.
```

---

## 19. Anti-patterns e lições aprendidas

### 19.1 Caso real — tela só aparece ao pressionar `Ctrl+C`

**Sintoma:** no VT, o app roda, mas a tela fica em branco; ao pressionar
`Ctrl+C`, a imagem aparece por instantes antes de o console ser restaurado.
Funcionava em uma máquina e falhava em outra.

**Diagnóstico:** o `testes_debug.txt` mostrou `i915drmfb`, 1920 × 1080, 32 bpp,
sem ypan (`yres_virtual == yres`) e `/sys/class/graphics/fb0/blank: 4` antes do
app. O app entrava em `KD_GRAPHICS` e desenhava, mas nunca desbloqueava o
console. `FBIOBLANK`/`FB_BLANK_UNBLANK` levava o estado para `0`; o `Ctrl+C`
(entrada de teclado) acordava o console.

**Correção:** após `KDSETMODE(KD_GRAPHICS)`, chamar
`FBIOBLANK(FB_BLANK_UNBLANK)`; falha é não fatal. O teste de escrita direta com
faixas RGB e o strace confirmaram que o framebuffer e o mapeamento estavam
corretos.

**Lição:** não presuma `yoffset`, permissão, Wayland ou instalação. Em caso de
tela em branco, confirme primeiro o VT de texto, o `blank`/DPMS e os ioctls
`KDSETMODE`/`FBIOBLANK`/`FBIOPAN_DISPLAY` no hardware afetado.

1. **Decodificar PNG ou rasterizar glifos a cada frame.** Ordens de magnitude
   mais lento. Cacheie decode, fundo e glifos.
2. **Escalar texto com nearest neighbor.** Use bilinear (ou renderize no
   tamanho final); texto serrilhado é inaceitável.
3. **Achar que `/dev/fb0` basta.** Sem VT e sem grupo `video`, não funciona.
   Trate os dois erros separadamente.
4. **Usar `memmap2` sem necessidade.** `libc::mmap` já resolve e reduz deps.
5. **Esquecer `line_length` e padding.** A imagem “torta” por stride errado.
6. **Ignorar ordem de canais/bitfields.** Um formato pode ser BGRX.
7. **Assumir `ypan`.** Só use double buffer se o driver reportar suporte.
8. **Esquecer alpha pré-multiplicado.** Blending escurece/clareia errado.
9. **Depender de SIGINT em raw mode.** `Ctrl+C` chega como tecla.
10. **Restaurar na ordem errada ou só no caminho feliz.** Use RAII + sinais +
    panic hook.
11. **Imprimir logs na tela.** Corrompe a TUI e o console gráfico.
12. **Auto-detectar framebuffer.** É frágil; mantenha `auto` na TUI.
13. **Colocar o hash do pacote nas próprias notas.** Referência circular;
    o hash muda quando as notas mudam.
14. **Esquecer a versão nos snapshots.** Bump de versão quebra testes; é
    esperado, revise o diff.
15. **Confiar em glifos exóticos.** Verifique cobertura das fontes; desenhe
    primitivas quando necessário.
16. **Ajustar arte por “fit” quando a intenção era “cover”.** Um painel
    retrato num card paisagem deixa faixas vazias; use fundo sólido ou cover.
17. **Declarar validação manual sem hardware.** Registre explicitamente
    “pendente” e o que falta.
18. **Empacotar antes de testar o binário copiado.** Sempre teste em `/tmp`.
19. **Deixar docs e decisões divergirem do código.** Atualize spec/ADR/README
    na mesma tarefa; o próximo agente confia neles.
20. **Adicionar dependências por conveniência.** Cada crate custa licença,
    tamanho, MSRV e risco.
21. **Assumir que `KD_GRAPHICS` desbloqueia o console.** Verifique
    `/sys/class/graphics/fb0/blank` e chame
    `FBIOBLANK(FB_BLANK_UNBLANK)` após entrar em `KD_GRAPHICS`.
22. **Diagnosticar a partir de Wayland/SSH/`/dev/pts/N`.** O teste não exercita
    o VT e gera falso negativo; confirme `tty` = `/dev/ttyN` antes de concluir.
23. **Escolher o buffer de pan por contador fixo.** Use o `yoffset` atual e
    valide `yoffset <= yres_virtual - yres`.
24. **Mapear só `line_length * yres`.** Cubra o buffer visível e, com pan, os
    dois buffers; valide `smem_len`.
25. **Tratar falha de `FBIOPAN_DISPLAY` como fatal.** Desabilite o pan e escreva
    no buffer visível; o jogo deve continuar.
26. **Ignorar que um driver pode anunciar ypan sem honrá-lo.** Tenha fallback.
27. **Instalar sem liberar o framebuffer.** Adicione o usuário ao grupo
    `video`; não peça para rodar como root.
28. **Pedir diagnóstico por comandos soltos.** Entregue um script
    reproduzível que gera um arquivo único.

---

## 20. Definition of Done

Uma entrega só está concluída quando:

- [ ] Escopo da fase atendido e decisões registradas.
- [ ] Domínio puro com testes; invariantes cobertas.
- [ ] TUI completa e snapshots aprovados.
- [ ] Framebuffer (quando na fase): 32 bpp validado no console; 24/16 bpp
      validados ou explicitamente pendentes.
- [ ] Primeira tela aparece sem `Ctrl+C` em VT de texto real, inclusive com
      console previamente blanked/DPMS off.
- [ ] `yoffset`, `smem_len`, ypan e fallback validados/testados.
- [ ] Restauração validada em saída normal; `Ctrl+C`/SIGTERM/pânico testados ou
      registrados como pendentes.
- [ ] `testes_debug.sh` disponível e `install.sh` testado.
- [ ] Portões verdes com e sem features; builds release.
- [ ] Assets/manifesto/licenças/notices consistentes.
- [ ] README e docs atualizados; sem drift.
- [ ] `AGENTS.md` atualizado com status, pendências e próximos passos.
- [ ] Pacote `tar.gz` + `.sha256` e smoke em `/tmp` sem assets.
- [ ] Tag/release publicados, se solicitado.
- [ ] Nenhum terminal/console corrompido em nenhum caminho de saída.

---

## 21. Parâmetros iniciais sugeridos `[PREENCHER]`

| Parâmetro | Default sugerido | Seu valor |
| --- | --- | --- |
| Canvas lógico | 1344 × 756 (70% de 1080p) | |
| Resolução alvo | 1920 × 1080 | |
| bpp alvo | 32 (24/16 como bônus) | |
| Frame animação | 16 ms (60 FPS) | |
| Frame ocioso | 100 ms | |
| Atalho de idioma | `F2` | |
| Atalho de som | `M` | |
| Saída | `Q`/`Esc`/`Ctrl+C` | |
| Volume | 0,32 | |
| Seed padrão | entropia; flag `--seed` | |
| Licença código | MIT | |
| Licença arte | CC BY 4.0 | |
| Fontes | OFL 1.1 | |
| Tamanho do pacote | < 15 MB | |
| Cobertura de testes | 150+ testes no total | |

---

## 22. Primeiro comando após copiar este template

```bash
mkdir -p docs assets/src assets/generated assets/fonts xtask/src tests/snapshots
git init -b main
# crie AGENTS.md, README.md e docs/01..08 a partir deste framework
# preencha os campos [PREENCHER] e congele as decisões com o usuário
cargo init --name <BIN> .
```

Depois siga a Fase 0: **especificar, revisar e congelar** antes de escrever a
primeira linha de código. A qualidade do console vem de guardas, testes e
validação real — não de otimismo.
