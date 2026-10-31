# 01 — Produto

## 1. Visão

Mouse Test é um **laboratório visual de mouse** para console Linux. Cada ação
física do mouse vira um efeito gráfico imediato e legível. O usuário consegue
verificar em segundos se movimento, botões, scroll, botões laterais e taxa de
relatório estão funcionando, sem instalar driver, abrir navegador ou depender de
servidor gráfico.

O aplicativo tem dois modos de saída, com o mesmo estado e o mesmo domínio:

- **TUI** — terminal, backend padrão, inclusive remoto (degradado);
- **fbdev** — console local em VT de texto, com cursor próprio.

## 2. Público e casos de uso

| Público | Uso típico |
| --- | --- |
| Entusiasta/usuário doméstico | conferir mouse novo, botões extras e scroll |
| Técnico de manutenção | diagnosticar botão que falha, scroll invertido, duplo clique |
| Desenvolvedor de periféricos | observar eventos brutos e taxa de relatório |
| Usuário de console Linux | testar mouse em VT sem X/Wayland |
| Acessibilidade | validar botões e scroll com feedback visual reforçado |

## 3. Metas

1. Feedback visual imediato para todo evento de mouse suportado.
2. Diagnóstico honesto: mostrar o que foi observado, não o que o mouse promete.
3. Funcionar no terminal e no console local.
4. Não exigir root, rede, telemetria ou persistência.
5. Domínio puro e testável sem hardware.

## 4. Não-objetivos

- Alterar DPI, polling rate, RGB, macros ou firmware do mouse.
- Medir DPI, latência ponta a ponta, latência wireless ou qualidade do sensor.
- Ser driver, daemon, libinput frontend ou ferramenta de configuração.
- Suportar Windows/macOS, X11/Wayland nativo ou DRM/KMS na v1.
- Ler teclado de dispositivos combo para qualquer finalidade.
- Exportar sessão, nuvem ou telemetria na v1.
- Áudio na v1.
- Touchpad/tablet completo na v1; apenas seleção explícita e modo absoluto
  básico.
- Interface gráfica com mouse; a UI é de teclado.

## 5. Premissas

- Linux x86_64 local; Rust stable, edição 2021, MSRV 1.88.
- O usuário tem permissão de leitura em `/dev/input/event*` para o modo completo.
- A TUI exige TTY; SSH funciona com limitações.
- O framebuffer exige `/dev/fb0`, VT de texto ativo e permissão.
- O app nunca roda como root.
- Sem rede em runtime.
- O mouse é a entrada sob teste; o teclado controla a UI.

## 6. Jornadas principais

### 6.1 Primeiro uso na TUI

1. Usuário executa `mouse_test`.
2. O app descobre dispositivos, abre o primeiro mouse compatível e mostra o
   dashboard.
3. Ao mover o mouse, o cursor virtual e o rastro respondem.
4. Ao clicar, o painel de botões acende e um anel aparece no canvas.
5. Ao rolar, o contador e os pulsos de scroll aparecem.
6. `Tab` percorre modos; `T` troca de dispositivo; `Q` sai e restaura o terminal.

### 6.2 TUI sem permissão de evdev

1. O app detecta `EACCES` ao abrir `/dev/input/event*`.
2. Mostra banner acionável com o comando sugerido para o grupo `input` e o aviso
   de segurança.
3. Se o terminal suportar, ativa mouse do terminal (`crossterm`) em modo
   degradado: clique, scroll e arraste, sem movimento relativo bruto nem taxa.
4. O usuário pode continuar testando ou sair. `--require-evdev` transforma isso
   em erro com código 4.

### 6.3 Framebuffer

1. Usuário muda para um VT de texto e executa `mouse_test --backend fb`.
2. O app valida `/dev/fb0`, VT ativo, permissões, `yoffset`, `smem_len` e
   formato.
3. Entra em `KD_GRAPHICS`, desbloqueia o console com `FBIOBLANK`, desenha o
   primeiro frame imediatamente e lê mouse via evdev.
4. Ao sair, restaura `KD_TEXT`, cursor e terminal mesmo em erro/sinal/pânico.

### 6.4 Hotplug

1. O app varre `/dev/input` periodicamente.
2. Um mouse novo aparece no seletor e num banner.
3. Se o primário for removido, o app escolhe o próximo mouse compatível; se não
   houver, mostra tela de espera e continua permitindo sair pelo teclado.

### 6.5 Captura e diagnóstico

1. `--capture-frame out.png` renderiza um frame determinístico e sai.
2. `--list-devices` lista caminhos, nomes, classes, capacidades e permissão.
3. `--diagnostics out.txt` grava ambiente, grupos, dispositivos, fb e
   permissões, sem abrir sessão interativa.

## 7. Modos e telas

| ID | Modo | Objetivo | Fase |
| --- | --- | --- | --- |
| M-01 | Dashboard | visão ao vivo de movimento, botões, scroll e métricas | 2 |
| M-02 | Botões | desafio guiado para pressionar todos os botões suportados | 2 |
| M-03 | Scroll | desafio guiado de scroll vertical/horizontal | 2 |
| M-04 | Taxa | estimar taxa de relatório, jitter e confiança | 2 |
| M-05 | Alvos | precisão, arraste, duplo clique (opcional) | 3 |
| M-06 | Dispositivos | escolher primário e inspecionar capacidades | 2 |
| M-07 | Ajuda | atalhos, limitações e estado do backend | 2 |
| O-01 | Log bruto | sobreposição com últimos eventos normalizados | 2 |
| O-02 | Banner | avisos de permissão, hotplug, overflow, degradado | 2 |

`Alvos` é um modo rico, não essencial para o teste visual básico. Se o prazo
apertar, ele fica para a Fase 3 sem bloquear a Fase 2.

## 8. Requisitos funcionais

| ID | Requisito | Prioridade |
| --- | --- | --- |
| FR-001 | Descobrir dispositivos de ponteiro via evdev e classificar mouse, touchpad, tablet, teclado, joystick. | Must |
| FR-002 | Selecionar automaticamente um mouse e permitir troca manual. | Must |
| FR-003 | Mostrar cursor virtual com posição, rastro, velocidade e deslocamento bruto. | Must |
| FR-004 | Mostrar estado, contagem e duração de cada botão, incluindo extras. | Must |
| FR-005 | Mostrar scroll vertical/horizontal, inclusive hi-res, sem dupla contagem. | Must |
| FR-006 | Estimar taxa de relatório e jitter com indicador de confiança. | Must |
| FR-007 | Exibir log bruto normalizado, limitado e filtrável. | Should |
| FR-008 | Oferecer desafio de botões baseado nas capacidades do dispositivo. | Should |
| FR-009 | Oferecer desafio de scroll vertical/horizontal. | Should |
| FR-010 | Detectar hotplug e mudar de dispositivo sem reiniciar. | Should |
| FR-011 | Operar em TUI degradada com mouse do terminal. | Must |
| FR-012 | Capturar frame PNG determinístico para QA/docs. | Should |
| FR-013 | Gerar diagnóstico textual de ambiente e permissões. | Must |
| FR-014 | Alternar pt-BR/en em todas as telas. | Should |
| FR-015 | Respeitar `--no-animations`, `NO_COLOR` e modo monocromático. | Must |
| FR-016 | Restaurar terminal/console em saída normal, erro, pânico e sinais. | Must |
| FR-017 | Suportar backend fbdev com cursor, escala e fallback de pan. | Must (Fase 4) |
| FR-018 | Nunca usar mouse para acionar controles da UI. | Must |
| FR-019 | Recusar grab em dispositivo combo e exigir opt-in. | Must |
| FR-020 | Não ler nem registrar teclas de dispositivos combo. | Must |

## 9. Requisitos não funcionais

| ID | Requisito | Alvo |
| --- | --- | --- |
| NFR-001 | Latência evdev→frame | p95 ≤ 16 ms, p99 ≤ 33 ms |
| NFR-002 | Startup até primeiro frame | < 300 ms |
| NFR-003 | Frame TUI | < 8 ms |
| NFR-004 | Frame fb 1080p | ≤ 16 ms |
| NFR-005 | RSS | < 80 MB |
| NFR-006 | CPU ocioso | < 1% |
| NFR-007 | Binário release | < 12 MB |
| NFR-008 | Throughput | 10.000 eventos/s sem travar; overflow contado |
| NFR-009 | Offline | zero chamadas de rede |
| NFR-010 | Privilégio | nunca root |
| NFR-011 | Determinismo | captura e modos com `--seed` reproduzíveis |
| NFR-012 | Acessibilidade | contraste ≥ 4,5:1; estado não depende só de cor |
| NFR-013 | Restauração | 100% dos caminhos de saída |
| NFR-014 | Privacidade | nenhum keycode de teclado é lido/armazenado |

## 10. Efeitos visuais por evento (resumo)

| Evento | Efeito | Também mostrado como |
| --- | --- | --- |
| movimento relativo | cursor, rastro, vetor de velocidade | dx/dy, counts, velocidade |
| botão esquerdo | anel verde + cartão `[L]` | contagem e duração |
| botão direito | anel vermelho + cartão `[R]` | contagem e duração |
| botão do meio | anel âmbar + cartão `[M]` | contagem e duração |
| botão lateral/extra | anel violeta/rosa + cartão `[S1]` | código bruto e contagem |
| scroll vertical | pulsos para cima/baixo + contador | ticks, hi-res, direção |
| scroll horizontal | pulsos laterais + contador | ticks, hi-res, direção |
| arraste | linha elástica do ponto de pressão | tempo e distância |
| duplo clique | anel duplo + rótulo | intervalo em ms |
| relatório | histograma e Hz estimado | jitter e confiança |
| hotplug | banner e atualização do seletor | estado do dispositivo |

Detalhes de geometria, duração e cor ficam em `docs/03-visual-design.md`.

## 11. Critérios de aceite

### 11.1 TUI completa

- [ ] Mover o mouse atualiza cursor/rastro em ≤ 100 ms percebidos.
- [ ] Todos os botões advertising no dispositivo acendem o cartão correto.
- [ ] Botões não anunciados, mas recebidos, aparecem como extras.
- [ ] Scroll vertical e horizontal contam na direção correta.
- [ ] Hi-res não duplica a contagem de notch.
- [ ] Taxa estimada fica dentro de ±10% de 125/500/1000 Hz em teste manual
      controlado, com confiança média/alta após 2 s.
- [ ] Desafio de botões lista pendentes e conclui quando todos são vistos.
- [ ] Hotplug adiciona/remove dispositivo em ≤ 2 s.
- [ ] Sem permissão, a TUI explica o que fazer e não trava.
- [ ] `Q`/`Esc`/`Ctrl+C` restauram o terminal sem cursor escondido.
- [x] Captura PNG com seed fixa é byte-a-byte estável no mesmo build.

### 11.2 Framebuffer

- [ ] Primeiro frame aparece sem pressionar tecla, inclusive com console
      previamente blanked/DPMS off.
- [ ] Cursor desenhado aparece e não some nas bordas.
- [ ] `yoffset`, `smem_len` e fallback de pan são respeitados.
- [ ] Resolução 1920×1080 32 bpp validada; 24/16 bpp validados ou pendentes.
- [ ] `KD_TEXT`, cursor e tela limpa restaurados em saída normal, erro, pânico,
      `Ctrl+C` e SIGTERM.
- [ ] Rodar em VT de texto real (`tty` = `/dev/ttyN`), não pts/Wayland/SSH.

### 11.3 Release

- [ ] Portões verdes com e sem features.
- [ ] Binário testado copiado para `/tmp`, sem pasta de assets.
- [ ] `install.sh` testado com prefixo temporário e com sudo/grupos reais.
- [ ] `testes_debug.sh` coleta ambiente e dispositivos sem root.
- [ ] Docs, licenças e `AGENTS.md` sem drift.
- [ ] Nenhum terminal/console corrompido em nenhum caminho.

## 12. Escopo v1 e backlog

| Item | v1 | Backlog |
| --- | --- | --- |
| Dashboard, botões, scroll, taxa | sim | — |
| Desafio de botões/scroll | sim | — |
| Alvos/arraste/duplo clique | Fase 3 | — |
| TUI e fbdev | sim | — |
| Multi-dispositivo | primário + lista | agregação avançada |
| Touchpad | seleção explícita básica | gestos, palm rejection |
| Raw HID/hidraw | não | inspeção de reports, DPI, latência |
| DPI | não | medição com distância física |
| Exportação | não | CSV/JSON |
| Áudio | não | cliques/scroll |
| DRM/KMS | não | backend alternativo |
| Persistência | não | perfis e último dispositivo |
| Temas | tokens fixos | temas customizados |

## 13. Glossário

| Termo | Significado |
| --- | --- |
| evdev | interface de eventos de entrada do Linux (`/dev/input/event*`) |
| TUI | interface de texto no terminal |
| fbdev | framebuffer (`/dev/fb0`) |
| raw counts | unidades relativas do sensor, antes de aceleração do sistema |
| notch | um passo de scroll, normalmente ±120 em hi-res |
| report | pacote de eventos terminado em `SYN_REPORT` |
| polling rate | taxa de relatórios do dispositivo, medida como estimativa |
| degradado | operação com menos dados/fidelidade, sempre sinalizada |
| grab | `EVIOCGRAB`, captura exclusiva do dispositivo |
| combo | dispositivo que expõe teclado e mouse no mesmo nó evdev |
