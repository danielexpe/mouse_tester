# 05 — Plano de testes

## 1. Estratégia

O app deve ser testável **sem mouse e sem framebuffer**. A entrada evdev é
isolada atrás de uma fonte de eventos; o domínio recebe eventos normalizados com
timestamps. Testes automatizados cobrem regras, parsers, geometria, efeitos,
estimador de taxa e telas. Hardware real é validado manualmente e registrado;
nunca é presumido.

Pirâmide:

1. Unitários de domínio (maior volume).
2. Propriedades/invariantes.
3. Testes de parser evdev com fixtures sintéticas.
4. Snapshots de TUI.
5. Integração/PTY com entrada sintética.
6. Testes de fb sem `/dev/fb0`.
7. Matriz manual em hardware real (menor volume, obrigatória).

## 2. Testes de domínio

### 2.1 Ponteiro

- Soma de `REL_X`/`REL_Y` em `raw_total`.
- Clamp nas quatro bordas; flag `clamped` ligada/desligada.
- `R` recentraliza sem zerar totais no dashboard.
- Rastro limitado a 96 pontos e janela de 600 ms.
- Rastro termina sempre na posição atual.
- Velocidade EWMA: subida, descida e repouso.
- Distância acumulada nunca diminui.
- Dispositivo absoluto: mapeamento de faixa para canvas, inclusive mínimos
  negativos e faixa zero (rejeitar).
- Troca de dispositivo recentraliza cursor/rastro e mantém totais separados por
  dispositivo.

### 2.2 Botões

- Press/release atualiza `pressed_mask` e `ButtonRuntime`.
- Contagem incrementa apenas em press.
- Duração = release − press.
- Duplo clique dentro de 400 ms; fora, não.
- Botão desconhecido recebe rótulo `[0xNNN]`.
- Extras detectados entram no desafio.
- Códigos altos de mouse (0x2c0..0x2e7) não colidem no conjunto de pressionados.
- Release sem press não corrompe estado.
- Combo: eventos de teclado são descartados; nenhum keycode armazenado.

### 2.3 Scroll

- Sinal vertical/horizontal correto.
- Hi-res + wheel normal não duplica notch.
- `hires` acumula até 120 e gera notch; resto preservado.
- Inversão é exibida como recebida.
- Coalescência: no máximo 8 pulsos por frame.
- Scroll sem botão pressionado funciona.
- Scroll horizontal opcional quando não anunciado.

### 2.4 Efeitos

- Efeito expira exatamente em `start + duration`.
- Máximo de 128; descarta o mais antigo.
- `--no-animations` mantém estado e remove animação.
- Progresso de anel em 0, 0,5 e 1,0.
- Duplo clique gera segundo anel defasado.
- Pulsos de scroll usam direção e cor corretas.

### 2.5 Métricas

- Estimador com intervalos fixos de 1.000 µs ⇒ ~1.000 Hz.
- Intervalos de 8.000 µs ⇒ ~125 Hz.
- Ignora intervalos 0, negativos e > 250 ms.
- Jitter p95−p50 conhecido.
- Confiança por número de amostras (0/19/20/99/100).
- Overflow incrementa contador e gera evento.
- Sparkline de 5 s em 20 bins.
- Sem dados ⇒ `None`, UI mostra `—`.

### 2.6 Modos

- Desafio de botões: esperado derivado de capacidades; extras; conclusão.
- Desafio de scroll: 4 etapas; horizontal condicional; reset.
- Taxa: amostra insuficiente; resultado; reinício.
- Alvos (Fase 3): seed reproduz posições; acerto/erro; arraste.
- Troca de modo preserva estado global; reset é explícito.

### 2.7 Sessão e i18n

- Troca de idioma em todas as telas, inclusive overlay e ajuda.
- Nenhuma string hardcoded fora do catálogo.
- Fallback para pt-BR se chave faltar em en (ou erro em teste).
- Textos longos não estouram layout (snapshot).
- `NO_COLOR` e `--color never` geram tema mono.

## 3. Fixtures de evdev

- `tests/fixtures/evdev/` com arquivos binários **sintéticos**, não capturas de
  usuários reais.
- Cada fixture: sequência de `struct input_event` de 24 bytes + metadados em
  texto (nome, capacidades, intervalos).
- Casos:
  - movimento simples com `SYN_REPORT`;
  - clique L/R/M;
  - botões laterais 0x113..0x117 e BTN_0..9;
  - wheel normal;
  - wheel hi-res + normal;
  - scroll horizontal;
  - intervalos de 1.000 µs (1000 Hz) e 8.000 µs (125 Hz);
  - sequência truncada no meio de um evento;
  - evento de teclado (deve ser descartado);
  - dispositivo combo;
  - `SYN_DROPPED`.
- Testes:
  - tamanho de `input_event` = 24 e `input_absinfo` = 24 em x86_64;
  - parse little-endian;
  - eventos truncados não causam pânico;
  - `SYN_DROPPED` força ressincronização/limpeza de estado;
  - capacidade `EV_REL`/`EV_KEY` decodificada por bit.

## 4. Testes de descoberta e classificação

- `is_mouse_like`: REL_X+REL_Y+BTN_LEFT.
- Touchpad (ABS + BTN_TOOL_FINGER) não é auto-selecionado.
- Tablet (ABS + INPUT_PROP_DIRECT) classificado como absoluto.
- Teclado e joystick não entram na lista padrão.
- Combo é sinalizado e grab é recusado.
- Dispositivo sem `EVIOCGNAME` recebe nome fallback.
- Permissão negada não é confundida com “sem dispositivo”.
- Lista ordenada de forma estável.
- Hotplug: adicionar, remover e trocar primário.
- Remoção do primário escolhe próximo mouse-like.
- Sem mouse restante: estado de espera, sem pânico.
- Varredura não abre o mesmo fd duas vezes.

## 5. Testes de TUI

- Snapshots de texto para:
  - Dashboard Wide/Normal/Compact/Minimal;
  - pt-BR e en;
  - sem dispositivo;
  - degradado (TUI mouse);
  - permissão negada;
  - overflow;
  - ajuda;
  - desafio de botões em andamento e concluído;
  - scroll 2/4;
  - taxa sem dados e com 1000 Hz;
  - modo alvos (Fase 3).
- Comparador próprio de texto; `UPDATE_SNAPSHOTS=1` regenera e o diff é
  revisado.
- Versão no cabeçalho muda snapshots; esperado por release.
- Teste de resize preserva estado.
- Teste de terminal < 60×20 mostra layout Insuficiente.
- Nenhuma string fora do catálogo.

## 6. Integração/PTY

- Feature `mock-input` injeta eventos sintéticos com timestamps.
- Executar a TUI em PTY de teste:
  - abre, desenha primeiro frame, processa `Q`, restaura;
  - `Ctrl+C` em raw mode sai e restaura;
  - resize;
  - idioma;
  - captura de frame;
  - permissão simulada.
- Testar `--require-evdev` sem evdev ⇒ código 4 e mensagem.
- Testar `--backend fb` sem `/dev/fb0` ⇒ código 3 e mensagem.
- Testar `--capture-frame` sem TTY ⇒ sucesso e PNG válido.
- Testar binário copiado para `/tmp` sem assets.

## 7. Testes de framebuffer sem hardware

- Parse de formato 32/24/16 e rejeição de desconhecido.
- Tamanhos ABI: `fb_var_screeninfo` 160, `fb_fix_screeninfo` 80 (x86_64).
- Empacotamento de pixel e escrita de linha com `line_length`/padding.
- Conversão por bitfield para 32/24/16.
- Geometria de canvas 70% em 1080p/1440p/768p.
- Escala 1:1 e fracionária; bilinear não borra texto além do esperado.
- Blend com alpha 0, opaco e parcial (RGBA pré-multiplicado).
- Escolha de buffer de pan:
  - `current == 0 -> yres`;
  - `current == yres -> 0`;
  - `current` inesperado -> 0.
- Validações:
  - `yoffset > yres_virtual - yres` ⇒ erro claro;
  - `line_length < xres * bytes_por_pixel` ⇒ erro;
  - `smem_len` insuficiente ⇒ erro/pan desabilitado conforme caso;
  - geometria virtual inconsistente ⇒ erro.
- Plano de mapeamento:
  - sem pan: `(yoffset + yres)` linhas;
  - com pan: dois buffers + buffer visível de fallback.
- Pan com memória só para um buffer é desabilitado, não fatal.
- Falha de `FBIOPAN_DISPLAY` desabilita pan e continua.
- Cursor desenhado dentro/fora das bordas é clampado corretamente.
- Frame não vazio e do tamanho esperado.

## 8. Testes de captura e determinismo

- `--capture-frame` com seed fixa e estado fixo gera PNG idêntico no mesmo
  build.
- Dimensões 1344×756.
- Frame contém cursor, rastro, anel e texto (verificação por pixels não vazios
  em regiões conhecidas).
- Captura não abre evdev, fb, áudio ou TUI.
- Tempo lógico fixo; nenhum `SystemTime`.
- Falha de caminho inválido ⇒ código 1 e mensagem.

## 9. Matriz manual mínima

Registrar em `docs/06-roadmap.md` ou relatório de release; não inventar.

### 9.1 Ambiente

- [ ] Ubuntu/Debian em VT de texto.
- [ ] Fedora ou Arch (se disponível).
- [ ] Terminal local (GNOME Terminal, Konsole, Alacritty, Kitty, xterm).
- [ ] SSH (degradado, sem fb).

### 9.2 Mouse

- [ ] Mouse USB básico 3 botões + wheel.
- [ ] Mouse com botões laterais (voltar/avançar).
- [ ] Mouse com wheel horizontal.
- [ ] Mouse hi-res (REL_WHEEL_HI_RES).
- [ ] Mouse 125 Hz e 1000 Hz (ou taxas disponíveis).
- [ ] Mouse Bluetooth (uniq preenchido).
- [ ] Dispositivo combo (teclado + mouse) sem gravar teclas.
- [ ] Touchpad selecionado explicitamente (se Q-005 aprovado).
- [ ] Hotplug: conectar, remover, remover primário.
- [ ] Grab: ligar/desligar, sair, matar com SIGTERM.

### 9.3 Framebuffer

- [ ] `tty` = `/dev/ttyN`, não `/dev/pts/N`.
- [ ] `/sys/class/graphics/fb0/blank` chega a 0 e a primeira tela aparece sem
      `Ctrl+C`.
- [ ] Console previamente blanked/DPMS off.
- [ ] 1920×1080 32 bpp.
- [ ] 24/16 bpp se houver hardware/VM; senão marcar pendente.
- [ ] `yoffset`, `yres_virtual`, `ypanstep`, `line_length`, `smem_len`
      registrados.
- [ ] Pan honrado e fallback quando não honrado.
- [ ] Restauração: saída normal, `Ctrl+C`, SIGTERM, pânico.
- [ ] Nenhum cursor piscando/resíduo.

### 9.4 Fluxos

- [ ] Dashboard completo.
- [ ] Desafio de botões: todos os botões advertising.
- [ ] Desafio de scroll: 4 direções quando suportadas.
- [ ] Taxa: 2 s de movimento, confiança média/alta, ±10% de 125/500/1000 Hz.
- [ ] Alvos/arraste/duplo clique (Fase 3).
- [ ] Log bruto.
- [ ] Seletor de dispositivo.
- [ ] Troca de idioma em todas as telas.
- [ ] `--no-animations`, `NO_COLOR`, mono.
- [ ] Captura PNG.
- [ ] Diagnóstico textual.
- [ ] Saída limpa em terminal e console.

## 10. Diagnóstico e bugs de hardware

- `testes_debug.sh` deve gerar um único arquivo `testes_debug.txt` com:
  - versão, `os-release`, kernel, `id`, `loginctl`, `tty`, VT ativo;
  - grupos e permissões de `/dev/input/event*` e `/dev/fb*`;
  - `/proc/bus/input/devices`, `/sys/class/input/*`;
  - `/proc/fb`, `/sys/class/graphics/fb*/*`, `fbset -i` (se existir);
  - `/dev/dri`, driver DRM, `dmesg` (se acessível);
  - `mousescope --list-devices --all-devices`;
  - `mousescope --diagnostics`;
  - execução com `timeout` e, quando possível, `strace -e ioctl,openat,read`;
  - restauração de `KD_TEXT`/cursor mesmo em erro.
- O script não instala nada, não roda como root e avisa se não estiver em VT.
- Para bug de fb, coletar antes de mudar o código.
- Para bug de evdev, anexar lista de dispositivos, capacidades e permissões
  (sem dados de teclado).

## 11. Portões automatizados

```bash
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo clippy --workspace --all-targets --no-default-features -- -D warnings
cargo test
cargo test --no-default-features
cargo test --release
cargo xtask assets --check
cargo build --release
cargo build --release --no-default-features
```

Adicionar verificação de MSRV quando o CI existir.

## 12. Critérios de conclusão de teste

- [ ] Unitários de domínio, parser e métricas verdes.
- [ ] Snapshots revisados.
- [ ] PTY cobre entrada, saída e restauração.
- [ ] Testes de fb sem hardware verdes.
- [ ] Captura determinística.
- [ ] Matriz manual executada e registrada; itens não executados marcados como
      **pendentes** com motivo.
- [ ] Nenhuma validação de hardware declarada sem evidência.
- [ ] `testes_debug.sh` testado em VT.
- [ ] Nenhum terminal/console corrompido.

## 13. Estado da Fase 2

- [x] 115 testes unitários: domínio, parser evdev, descoberta por
      `/proc/bus/input/devices`, classificação, desafios, mapeamento do mouse do
      terminal, TUI (TestBackend) e diagnóstico.
- [x] 5 testes de integração de CLI (`--list-devices`, `--all-devices`,
      `--backend fb`, TUI sem TTY, `--version`).
- [x] `cargo fmt --check`, `clippy` (com e sem default features), `test` e
      `test --release` verdes.
- [x] Smoke manual em PTY: TUI degradada, banner de permissão, seletor de
      dispositivo, saída por `q` em 120×36 e 80×24 e clique esquerdo via
      protocolo SGR do mouse do terminal.
- [x] Descoberta real no ambiente: `/dev/input/event3` listado como mouse
      `denied`; modo degradado ativado.
- [x] `--require-evdev` retorna 4; `--diagnostics` gera relatório.
- [ ] Leitura real de eventos, botões extras, scroll hi-res e hotplug:
      pendentes de sessão com permissão em `/dev/input`.
- [ ] fbdev: pendente (Fase 4).
