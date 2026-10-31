# 03 — Design visual

## 1. Direção de arte

**“Instrumento de precisão”, não “jogo”.** Fundo escuro, tipografia mono no
console, números tabulares, alto contraste e movimento contido. O canvas é o
herói; painéis são densos mas legíveis. Cada evento tem uma resposta visual
distinta que não depende apenas de cor.

Princípios:

1. Movimento gera rastro; clique gera anel; scroll gera pulso direcional.
2. O estado atual é sempre legível sem animação (`--no-animations`).
3. Números mostram counts brutos e unidades estimadas separadamente.
4. Cor é reforço, nunca o único indicador.
5. Nada de arte bitmap, textura ou ruído visual.
6. Sem glifos exóticos: ASCII, primitivas e texto.

## 2. Layout TUI

### 2.1 Breakpoints

| Layout | Condição | Conteúdo |
| --- | --- | --- |
| Wide | ≥ 120×36 | canvas + 2 colunas de painéis + log opcional |
| Normal | ≥ 100×30 | canvas + 1 coluna lateral |
| Compact | ≥ 80×24 | canvas reduzido + painéis empilhados |
| Minimal | ≥ 60×20 | cursor, botões e scroll apenas; sem gráficos |
| Insuficiente | < 60×20 | mensagem “redimensione” e atalhos de saída |

- Redimensionar preserva estado, seleção e contadores.
- O canvas nunca some; no Compact ele ocupa a linha central.
- O rodapé de atalhos é fixo e traduzido.

### 2.2 Wide (referência)

```text
+ Mouse Test  v0.1.0 ------------------------------------------------+
| Logitech G Pro | event5 | 1000 Hz estimado (alta) | evdev         |
+------------------------------------+-----------------------------+
|                                    | BOTOES                      |
|                                    | [L]  [R]  [M]  [S1] [S2]    |
|          canvas Braille            |                             |
|      cursor + rastro + aneis       | SCROLL   V +12   H 0        |
|                                    | MOVE     dx +148  dy -32    |
|                                    | VEL      934 counts/s       |
+------------------------------------+-----------------------------+
| MOVIMENTO  total 12.480 counts | eventos 3.214 | reports 2.980    |
| TAXA       mediana 1.000 us | jitter 42 us | amostras 256       |
+------------------------------------------------------------------+
| 1-5 modo  TAB alterna  T dispositivo  L log  R reset  F2 idioma |
| F1 ajuda  Q sair                                                 |
+------------------------------------------------------------------+
```

### 2.3 Compact

```text
+ Mouse Test  event5  1000 Hz (alta) ------------------+
|                                                      |
|                 canvas reduzido                      |
|                                                      |
+-------------------------+----------------------------+
| BOTOES [L][R][M][S1]    | SCROLL V+12 H0             |
| MOVE dx+148 dy-32       | VEL 934 counts/s           |
+-------------------------+----------------------------+
| TAB modo  T disp.  L log  F2 idioma  Q sair          |
+------------------------------------------------------+
```

### 2.4 Minimal

```text
Mouse Test  event5  evdev  1000 Hz (alta)
dx +148  dy -32   vel 934 c/s
[L] [R] [M] [S1] [S2]   scroll V +12  H 0
TAB modo  T disp.  Q sair
```

## 3. Framebuffer

- Tela inteira recebe fundo `Background`.
- Canvas lógico **1344×756** (70% de 1920×1080), escalado por fator calculado a
  partir de 70% do menor eixo e centralizado.
- Caminho rápido 1:1 quando a escala é ≈ 1; bilinear para escala fracionária.
- Cursor desenhado por software como última camada, com anel de contraste para
  qualquer fundo.
- Texto com `fontdue::layout`, `line_height` consistente e alinhamento medido.
- Alpha pré-multiplicado no blend.
- Sem vsync; tearing leve aceitável.
- Fundo estático (grade, painéis) pré-renderizado; apenas canvas e números
  animam.

## 4. Tokens de cor

| Token | Hex | 256 | Uso |
| --- | --- | --- | --- |
| Background | `#0B0F14` | 233 | fundo do app/fb |
| Surface | `#121821` | 234 | painéis |
| SurfaceElevated | `#1A2330` | 235 | cartões e cabeçalho |
| Border | `#2A3646` | 237 | bordas, grade forte |
| Grid | `#1E2836` | 236 | grade do canvas |
| Text | `#E6EDF3` | 255 | texto principal |
| TextMuted | `#9AA7B6` | 247 | unidades, ajuda |
| Accent | `#38BDF8` | 45 | foco, cursor, seleção |
| Success | `#34D399` | 48 | botão esquerdo, conectado |
| Warning | `#FBBF24` | 214 | botão do meio, atenção |
| Danger | `#F87171` | 203 | botão direito, erro, overflow |
| Violet | `#A78BFA` | 141 | botão lateral 1 |
| Pink | `#F472B6` | 212 | botão lateral 2/extra |
| Drag | `#FDE68A` | 222 | linha de arraste |

Fallback:

- **16 cores:** `Accent`→ciano, `Success`→verde, `Warning`→amarelo, `Danger`→
  vermelho, `Violet`/`Pink`→magenta, `Text`→branco, `TextMuted`→cinza.
- **Mono:** diferenciação por forma, rótulo, posição e contorno.
- `NO_COLOR` força mono; `--color never` idem.
- Contraste de texto ≥ 4,5:1 sobre `Surface`.

## 5. Tipografia

| Contexto | Fonte | Tamanho lógico |
| --- | --- | --- |
| TUI | fonte do terminal | — |
| fb: título | JetBrains Mono Bold | 28 px |
| fb: seções | JetBrains Mono Bold | 20 px |
| fb: corpo | JetBrains Mono Regular | 16 px |
| fb: números | JetBrains Mono Regular, tabular | 16 px |

- TUI usa apenas texto; nenhuma fonte é embutida para o terminal.
- fb usa JetBrains Mono (OFL 1.1), embutida.
- Números alinhados à direita nos painéis; unidades em `TextMuted`.
- Evitar glifos não cobertos; usar `->`, `^`, `v`, `[L]`, `(x)`.
- Linhas de ajuda podem quebrar; nunca truncar sem reticências.

## 6. Componentes

| Componente | Conteúdo | Regras |
| --- | --- | --- |
| Cabeçalho | nome, versão, dispositivo, backend, estado | trunca nome com `…` no meio |
| Banner | permissão, hotplug, degradado, overflow | 1–2 linhas, cor + texto |
| Canvas | grade, origem, cursor, rastro, efeitos | foco visual; sem scroll |
| Cartão de botão | rótulo, estado, contagem, duração | `[L]` etc.; contorno quando ativo |
| Painel de scroll | V/H, ticks, hi-res, direção | seta textual + valor |
| Painel de movimento | dx/dy, total, velocidade, distância | counts primeiro |
| Painel de taxa | Hz, jitter, amostras, confiança | “—” sem dados |
| Sparkline | eventos por 250 ms, últimos 5 s | 20 bins |
| Log | tipo, código, valor, delta | ring buffer, sem teclado |
| Rodapé | atalhos contextuais | sempre inclui sair |
| Overlay de ajuda | atalhos, limitações, backend | teclado apenas |

## 7. Canvas e cursor

### 7.1 Sistema de coordenadas

- Canvas lógico fixo, margem de 12 px.
- Cursor inicia no centro; movimento relativo soma counts 1:1 em unidades
  lógicas.
- Ao atingir a borda, a posição é **clampada** e um flash de borda aparece.
- `R` recentraliza e zera o rastro (não zera totais, salvo no modo de desafio).
- Ao trocar de dispositivo, recentralizar cursor e rastro; totais por
  dispositivo são preservados.
- `raw_total_x/y` continuam crescendo mesmo com clamp.
- Dispositivos absolutos mapeiam `ABS_X/ABS_Y` para o canvas.
- Câmera infinita/mundo fica no backlog (Fase 6).

### 7.2 Cursor

| Propriedade | Valor |
| --- | --- |
| Forma | anel externo + ponto central |
| Raio | 10 px (anel), 3 px (ponto) |
| Contorno | 1 px `Background` + 1 px `Accent` |
| Rastro | até 96 pontos, 600 ms, alpha decrescente |
| Cor do rastro | gradiente `Accent`→transparente; espessura por velocidade |
| Indicador de borda | flash de 150 ms na borda tocada |

- Cursor é sempre visível no fb e no canvas da TUI.
- Na TUI, o ponteiro do SO não é escondido; o cursor desenhado é o “cursor do
  teste”.
- Em modo degradado, o cursor da TUI segue a célula do último evento do terminal
  (sem hover contínuo).

### 7.3 Escala de velocidade

- Velocidade em counts/s e px/s lógico.
- Rastro: comprimento e opacidade aumentam até 2.000 counts/s.
- Acima de 4.000 counts/s, cor vai de `Accent` a `Warning`.
- Indicador textual sempre presente; cor não é o único sinal.

## 8. Efeitos

| Evento | Efeito | Duração | Parâmetros |
| --- | --- | --- | --- |
| movimento | rastro + vetor | 600 ms | 96 pontos, alpha 0,8→0 |
| botão esquerdo | anel + cartão | 400 ms | raio 8→72, ease-out cúbico |
| botão direito | anel + cartão | 400 ms | idem, cor `Danger` |
| botão do meio | anel + cartão | 400 ms | idem, cor `Warning` |
| lateral/extra | anel menor + cartão | 320 ms | raio 6→48 |
| scroll vertical | pulsos `^`/`v` | 250 ms | 1–3 chevrons, deslocamento 12 px |
| scroll horizontal | pulsos `<`/`>` | 250 ms | 1–3 chevrons |
| hi-res | sub-pulso | 120 ms | alpha 50%, sem contar notch |
| arraste | linha pressão→cursor | enquanto ativo | 2 px, alpha 0,7 |
| soltura de arraste | linha residual | 300 ms | alpha decrescente |
| duplo clique | segundo anel defasado | 400 ms | atraso 90 ms + rótulo `x2` |
| borda | flash de borda | 150 ms | 2 px |
| hotplug | banner + destaque no seletor | 2 s | sem bloquear |
| overflow | banner + contador | persistente | até reconectar/resetar |

Regras:

- Máximo de 128 efeitos ativos; ao exceder, descartar o mais antigo.
- Efeitos são dados puros com `start_us`/`duration_us`; o render calcula o
  progresso.
- Pulsos de scroll são coalescidos por frame (máx. 8) para não saturar em
  hi-res.
- `--no-animations`: duração 0, sem rastro animado, mas estado e contadores
  continuam visíveis.
- `--no-trail`: remove somente o rastro; cursor e efeitos permanecem.

## 9. Cores por botão

| Botão | Código | Rótulo | Cor |
| --- | --- | --- | --- |
| Esquerdo | 0x110 | `[L]` | Success |
| Direito | 0x111 | `[R]` | Danger |
| Meio | 0x112 | `[M]` | Warning |
| Side | 0x113 | `[S1]` | Violet |
| Extra | 0x114 | `[S2]` | Pink |
| Forward | 0x115 | `[F]` | Accent |
| Back | 0x116 | `[B]` | Violet |
| Task | 0x117 | `[T]` | Pink |
| BTN_0..9 | 0x100..0x109 | `[0]`..`[9]` | alterna Violet/Pink |
| Desconhecido | outro | `[0xNNN]` | `TextMuted` |

- O rótulo textual acompanha a cor.
- O cartão mostra `pressionado`/`solto`, contagem e última duração.
- Botões não anunciados que aparecem são marcados como `extra` e incluídos no
  desafio.

## 10. Scroll

- Contadores `V` e `H` com sinal.
- Última direção com seta textual (`^`, `v`, `<`, `>`).
- `hi-res` mostrado como sub-unidade (ex.: `+3.5` notches) e barra de progresso
  até o próximo notch.
- Inversão: mostrar valor bruto; não “corrigir” a direção.
- Pulsos nascem no cursor; em modo degradado, na posição do terminal.

## 11. Taxa de relatório

- Valor principal em Hz estimado.
- Jitter em µs (p95−p50) e amostras.
- Confiança:
  - `baixa`: < 20 intervalos;
  - `média`: 20–99;
  - `alta`: ≥ 100.
- Sem dados: `—` e instrução “mova o mouse por 2 s”.
- Histograma de 12 bins de intervalo (0–2 ms, 2–4 ms, …).
- Nunca exibir “polling rate configurado”; sempre “estimado”.

## 12. Modos guiados

### 12.1 Desafio de botões

```text
BOTOES 4/7
Pressione: [S1] lateral
[L] ok  [R] ok  [M] ok  [S1] pendente  [S2] pendente
R reinicia  S pula  TAB volta
```

- Lista derivada de `EVIOCGBIT(EV_KEY)`.
- Extras detectados entram no fim.
- Conclusão mostra resumo e botões vistos.

### 12.2 Desafio de scroll

```text
SCROLL 2/4
Role para cima  (recebido: 0)
V +0  H 0  hi-res 0
```

- Etapas: cima, baixo, esquerda, direita (horizontal só se suportado).
- Conta qualquer evento na direção pedida; não exige quantidade exata.

### 12.3 Alvos (Fase 3)

- Alvo circular com anel de acerto e cruz de mira; cor indica o tipo.
- Modos: clique, arraste e duplo clique, em nove rodadas.
- Clique: acerta com o botão esquerdo dentro do raio.
- Arraste: pressão dentro do raio, movimento mínimo e soltura; parado conta
  como erro.
- Duplo clique: duas pressões dentro do raio em até 400 ms.
- Métricas: tempo, erro de distância ao centro, taxa de acerto e últimas
  tentativas.
- Seed controla posições; `R` reinicia com a mesma seed.

## 13. Estados e mensagens

| Estado | Visual | Ação oferecida |
| --- | --- | --- |
| Sem dispositivo | banner + canvas vazio com grade | conecte um mouse; hotplug ativo |
| Permissão evdev negada | banner `Danger`, backend `degradado` | comando de grupo + aviso de segurança |
| Degradado | etiqueta `TUI-MOUSE` no cabeçalho | limitar métricas; explicar ausência |
| Overflow | banner `Danger` + contador | reduzir taxa/verificar sistema |
| Hotplug | banner `Success` 2 s | `T` para escolher |
| Dispositivo removido | banner `Warning` | seleção automática ou espera |
| fb sem VT | erro no stderr antes de desenhar | instrução para VT local |
| fb sem permissão | erro acionável | grupo `video` |
| Terminal pequeno | layout Minimal | redimensione |

Nenhuma mensagem de erro é impressa sobre a TUI/fb; erros fatais vão para
stderr após restaurar.

## 14. Acessibilidade

- Contraste mínimo 4,5:1 para texto; 3:1 para elementos gráficos.
- Estado indicado por texto + forma + posição, além de cor.
- `--no-animations` remove movimento e reduz flashes.
- `NO_COLOR` e `--color never` produzem paleta mono legível.
- Números sempre com unidade explícita.
- Sem dependência de hover; tudo acessível por teclado.
- Textos longos quebram em vez de truncar informações críticas.
- Modo Minimal mantém botões/scroll/velocidade.

## 15. Captura e referência visual

- `--capture-frame` gera 1344×756 com estado sintético:
  - cursor deslocado, rastro, um anel de clique, scroll +3, botões L/R/M,
  - taxa 1000 Hz, jitter 42 µs, confiança alta.
- Captura é usada em testes; nunca depende de hardware.
- Na Fase 3 a captura usa pintor RGBA e fonte 5×7 embutidos; o render final
  com `tiny-skia`/`fontdue` continua previsto para o framebuffer.
- Snapshots de TUI usam layout Wide e Normal, pt-BR e en, estados normal,
  degradado e sem dispositivo.
- Versão no cabeçalho muda snapshots por release; revisar o diff é esperado.

## 16. Limitações visuais documentadas

- TUI: ponteiro do SO visível; sem hover contínuo em muitos terminais; sem
  movimento relativo bruto.
- fb: sem vsync; cursor é software; resolução fixa desde o início.
- Escala fracionária no fb usa bilinear; texto é rasterizado no tamanho final
  quando possível.
- Cores de 16/256 são aproximações.
- Não há tema customizável na v1.
