# 09 — Especificação de entrada (evdev e fallback)

## 0. Status de implementação (Fase 2)

Implementado: parser de `/proc/bus/input/devices`, descoberta por sysfs,
classificação por capacidades, ioctls de nome/IDs/capacidades/abs/estado de
teclas, leitura com `poll`, thread, canal limitado, hotplug de 1 s, normalização
de movimento/botões/wheel hi-res, absolutos básicos, fallback `crossterm` e
recusa de grab em combo.

Validação real: leitura evdev validada com o mouse HID `1bcf:08a0` (fonte
`evdev`) em terminal GNOME. Botões extras, scroll hi-res e hotplug ainda não
foram exercitados ponta a ponta. O fallback foi validado em PTY para clique
esquerdo; wheel/scroll do terminal tem teste unitário de mapeamento, mas não foi
validado ponta a ponta.

## 1. Objetivos

1. Ler movimento relativo, botões, scroll e timestamps diretamente do kernel.
2. Identificar capacidades e classificar o dispositivo sem “adivinhar”.
3. Nunca ler nem armazenar teclas de dispositivos combo.
4. Funcionar com múltiplos dispositivos e hotplug.
5. Degradar de forma explícita na TUI quando evdev não está disponível.
6. Sobreviver a erros de permissão, remoção, overflow e `SYN_DROPPED`.

## 2. Fonte de entrada

| Fonte | Uso | Fidelidade |
| --- | --- | --- |
| evdev | primária em TUI e fb | movimento, botões, wheel hi-res, taxa, capacidades |
| `crossterm` | fallback na TUI | clique, scroll, arraste; posição em células |
| mock | testes | sintética e determinística |

- Framebuffer **não** usa fallback: sem evdev não há cursor nem dados de mouse.
- `--tui-mouse on` força fallback na TUI; `off` desativa; `auto` usa se evdev
  não estiver disponível.
- Quando evdev está ativo, o mouse do terminal fica desabilitado para evitar
  duplicidade e conflito com seleção de texto.

## 3. Descoberta e identidade

### 3.1 Fontes

- `/sys/class/input/eventN/` para nome, `phys`, `uniq`, IDs e caminho.
- `/proc/bus/input/devices` como fonte legível sem permissão.
- `/dev/input/eventN` para abrir e consultar capacidades via ioctl.
- `/dev/input/by-id/` e `/dev/input/by-path/` para nomes estáveis, se existirem.

### 3.2 Identidade

```text
DeviceId(u32)        # atribuído na descoberta, único na sessão
DeviceInfo {
  path: PathBuf,
  sysfs: Option<PathBuf>,
  name: String,
  phys: Option<String>,
  uniq: Option<String>,
  ids: { bustype, vendor, product, version },
  class: DeviceClass,
  capabilities: Capabilities,
  permissions: Readable | Denied | Unknown,
}
```

- `DeviceId` não é persistido.
- A reconexão pode ser reconhecida por `uniq`/`phys`/IDs, mas a v1 pode tratar
  como dispositivo novo; o seletor mostra duplicatas se o usuário reconectar.
- Ordenação estável: mouse-like primeiro, depois absolutos, depois outros;
  dentro do grupo, por caminho.
- Nunca abrir o mesmo `path` duas vezes na mesma sessão.

### 3.3 Abrir

```text
open(path, O_RDONLY | O_NONBLOCK | O_CLOEXEC)
```

- `EACCES` ⇒ permissão negada, não “dispositivo ausente”.
- `ENOENT`/`ENODEV` ⇒ removido; ignorar e reescanear.
- `EBUSY` (grab por outro processo) ⇒ mostrar motivo; não insistir.
- Após abrir, consultar nome/IDs/capacidades; se falhar, fechar e marcar como
  ilegível.

## 4. Classificação

### 4.1 Capacidades

- `EVIOCGBIT(0, len)` para tipos (`EV_KEY`, `EV_REL`, `EV_ABS`, …).
- `EVIOCGBIT(EV_KEY, len)`, `EVIOCGBIT(EV_REL, len)`, `EVIOCGBIT(EV_ABS, len)`.
- `EVIOCGPROP(len)` para propriedades (`INPUT_PROP_POINTER`, `DIRECT`, …).
- `EVIOCGID`, `EVIOCGNAME`, `EVIOCGPHYS`, `EVIOCGUNIQ`.

### 4.2 Regras

| Classe | Regra |
| --- | --- |
| Mouse | `EV_REL` com `REL_X` e `REL_Y` **e** `EV_KEY` com `BTN_LEFT` |
| Touchpad | `EV_ABS` com `ABS_X`/`ABS_Y` e (`BTN_TOOL_FINGER` ou `BTN_TOUCH` ou `INPUT_PROP_BUTTONPAD`) |
| Tablet | `EV_ABS` com `ABS_X`/`ABS_Y` e `INPUT_PROP_DIRECT` ou `BTN_TOOL_PEN` |
| Teclado | `EV_KEY` com `KEY_A` e `KEY_Z`, sem `REL_X`/`REL_Y` |
| Joystick | `EV_ABS` com `ABS_X`/`ABS_Y` e `BTN_TRIGGER`/`BTN_SOUTH` |
| Combo | regras de mouse **e** teclado no mesmo nó |
| Desconhecido | nenhuma das anteriores |

- Mouse-like é auto-selecionado.
- Touchpad/tablet **não** são auto-selecionados; só com `--device` explícito.
- Teclado/joystick não entram na lista padrão; `--all-devices` mostra tudo.
- Combo é mostrado com aviso e grab é recusado por padrão.
- Um dispositivo pode ser mouse-like mesmo sem wheel; wheel é opcional.
- Trackball/trackpoint são classificados como mouse se tiverem `REL_X/Y` e
  `BTN_LEFT`.

### 4.3 Escolha automática

1. Filtrar mouse-like legíveis.
2. Preferir o primeiro por ordem de caminho.
3. Se `--device` for nome/caminho, tentar correspondência exata; se ambígua,
   abrir o seletor e pedir confirmação por teclado.
4. `--device all` agrega todos os mouse-like; cursor combina o movimento e cada
   evento carrega `DeviceId` para colorir o rastro.
5. Sem candidato: estado de espera + hotplug.

## 5. ABI evdev

Alvo: Linux x86_64, little-endian.

```c
struct input_event {
    struct timeval time;  /* 16 bytes em x86_64 */
    __u16 type;           /* 2 */
    __u16 code;           /* 2 */
    __s32 value;          /* 4 */
};                        /* 24 bytes */
```

- Testar `size_of::<InputEvent>() == 24`.
- `timeval` = segundos + microssegundos.
- `EVIOCSCLOCKID` com `CLOCK_MONOTONIC` (melhor esforço) para timestamps
  monotônicos.
- Se o driver não honrar clockid ou retornar timestamps não monotônicos, usar
  `Instant` capturado na leitura.
- Validar o tamanho lido; eventos truncados são descartados e contabilizados.

Constantes usadas:

| Nome | Valor | Uso |
| --- | --- | --- |
| `EV_SYN` | 0x00 | sincronização |
| `EV_KEY` | 0x01 | botões |
| `EV_REL` | 0x02 | movimento e wheel |
| `EV_ABS` | 0x03 | absolutos |
| `EV_MSC` | 0x04 | scan/timestamp (ignorar na v1) |
| `SYN_REPORT` | 0x00 | fim de pacote |
| `SYN_DROPPED` | 0x03 | perda de eventos |
| `REL_X` / `REL_Y` | 0x00 / 0x01 | movimento |
| `REL_HWHEEL` | 0x06 | scroll horizontal |
| `REL_WHEEL` | 0x08 | scroll vertical |
| `REL_WHEEL_HI_RES` | 0x0b | scroll vertical hi-res |
| `REL_HWHEEL_HI_RES` | 0x0c | scroll horizontal hi-res |
| `ABS_X` / `ABS_Y` | 0x00 / 0x01 | posição absoluta |
| `BTN_LEFT` | 0x110 | botão esquerdo |
| `BTN_RIGHT` | 0x111 | botão direito |
| `BTN_MIDDLE` | 0x112 | botão do meio |
| `BTN_SIDE` | 0x113 | lateral 1 |
| `BTN_EXTRA` | 0x114 | lateral 2 |
| `BTN_FORWARD` | 0x115 | avançar |
| `BTN_BACK` | 0x116 | voltar |
| `BTN_TASK` | 0x117 | tarefa |
| `BTN_0..9` | 0x100..0x109 | extras |
| `BTN_TOOL_FINGER` | 0x145 | touchpad |
| `BTN_TOUCH` | 0x14a | toque |
| `BTN_TOOL_PEN` | 0x140 | caneta |
| `BTN_TRIGGER` | 0x120 | joystick |

ioctls:

| Nome | Uso |
| --- | --- |
| `EVIOCGNAME(len)` | nome |
| `EVIOCGID` | bustype/vendor/product/version |
| `EVIOCGPHYS(len)` | porta física |
| `EVIOCGUNIQ(len)` | identificador único |
| `EVIOCGPROP(len)` | propriedades |
| `EVIOCGBIT(type,len)` | capacidades |
| `EVIOCGABS(abs)` | faixa absoluta |
| `EVIOCGKEY(len)` | estado atual dos botões (ressincronização) |
| `EVIOCGRAB` | grab exclusivo |
| `EVIOCSCLOCKID` | clock dos timestamps |

- `struct input_absinfo` = 24 bytes (`value`, `minimum`, `maximum`, `fuzz`,
  `flat`, `resolution`), testar tamanho.
- `struct input_id` = 8 bytes (4 × `u16`).
- Limitar buffers de nome a 256 bytes e tratar ausência de `\0`.

## 6. Normalização de eventos

### 6.1 Pipeline

```text
bytes → input_event → filtro mouse-only → acumulação por SYN_REPORT
      → InputEvent → canal → domínio
```

- `EV_KEY`: aceitar apenas códigos de botão de mouse conhecidos
  (`0x100..0x15f` e `0x2c0..0x2e7`, incluindo `BTN_TRIGGER_HAPPY`) e ignorar
  `value == 2` (autorepeat).
- Qualquer `KEY_*` de teclado é descartado **antes** de entrar no domínio; não é
  armazenado, contado nem logado.
- `EV_REL`: aceitar `REL_X/Y`, `REL_WHEEL`, `REL_HWHEEL`,
  `REL_WHEEL_HI_RES`, `REL_HWHEEL_HI_RES`; ignorar demais.
- `EV_ABS`: só para dispositivo absoluto selecionado explicitamente.
- `EV_MSC`, `EV_SW`, `EV_LED`, etc.: ignorar.
- `SYN_REPORT`: fechar o pacote, emitir `Report` e liberar o estado de
  acumulação.
- `SYN_DROPPED`: limpar acumuladores, consultar `EVIOCGKEY` e reconciliar
  botões, emitir `Overflow`/aviso; não gerar movimento falso.

### 6.2 Botões

| Código | Rótulo | Nome |
| --- | --- | --- |
| 0x110 | `[L]` | esquerdo |
| 0x111 | `[R]` | direito |
| 0x112 | `[M]` | meio |
| 0x113 | `[S1]` | side |
| 0x114 | `[S2]` | extra |
| 0x115 | `[F]` | forward |
| 0x116 | `[B]` | back |
| 0x117 | `[T]` | task |
| 0x100..0x109 | `[0]`..`[9]` | BTN_0..BTN_9 |
| 0x2c0..0x2e7 | `[0xNNN]` | BTN_TRIGGER_HAPPY (extras) |
| outro | `[0xNNN]` | desconhecido |

- `value`: `0` = solto, `1` = pressionado, `2` = repetição (ignorar).
- Rótulos são visuais; a identidade é o código.
- O app não traduz códigos para “botão 4/5” de navegador; mostra o código e um
  nome estável.

### 6.3 Wheel

- `REL_WHEEL`/`REL_HWHEEL` são passos de notch (normalmente ±1).
- `REL_WHEEL_HI_RES`/`REL_HWHEEL_HI_RES` usam 120 unidades por notch na maioria
  dos dispositivos; o valor é tratado como sub-unidade.
- Regra por `SYN_REPORT`:
  1. somar `wheel_normal` e `wheel_hires` do pacote;
  2. se `wheel_normal != 0`: notch = `wheel_normal`; hi-res serve só para
     intensidade visual;
  3. senão: acumular `hires_remainder += wheel_hires`; notch = truncamento
     inteiro de `remainder / 120`; `remainder %= 120`;
  4. emitir um `Wheel` normalizado com `notch`, `remainder` e `hires`.
- Nunca contar normal e hi-res como dois notches.
- Coalescer pulsos visuais por frame (máx. 8).
- Scroll horizontal segue a mesma regra.

### 6.4 Movimento

- `REL_X`/`REL_Y` são counts relativos; somar ao total e aplicar ao cursor.
- Não aplicar aceleração, DPI ou escala do sistema; mostrar counts brutos.
- Movimento negativo é válido; não usar `abs()` para totais.
- Se o dispositivo também emitir `EV_ABS` (combo estranho), priorizar `REL`
  quando a classe for mouse.

### 6.5 Absolutos

- Para touchpad/tablet selecionado:
  - ler `EVIOCGABS(ABS_X/Y)` no início;
  - normalizar `(value - min) / (max - min)` para o canvas;
  - rejeitar faixa zero/invertida com aviso;
  - pressão (`ABS_PRESSURE`) e multitouch ficam no backlog.
- O modo absoluto é rotulado no cabeçalho; métricas de taxa continuam válidas.

## 7. Thread, poll e overflow

```text
loop:
  poll(fds, timeout=100ms)
  para cada fd pronto:
    ler até EAGAIN
    parsear eventos completos
    acumular
    em SYN_REPORT: enviar InputEvent(s)
    em erro ENODEV: marcar remoção
  a cada 1 s: reescanear /dev/input
  se stop: sair
```

- Fd `O_NONBLOCK` + `poll`; nunca bloquear em `read`.
- Buffer de leitura de 4096 bytes (múltiplo de 24); manter sobra entre leituras.
- Canal limitado de 8192; se cheio, descartar o mais antigo, incrementar
  `dropped` e enviar `Overflow` quando houver espaço.
- Um único thread controla todos os fds; não há thread por dispositivo.
- `join` no encerramento; fds fechados após parar.
- Erros de leitura são reportados como `SourceError`, não pânico.

## 8. Hotplug

- Varredura a cada 1 s.
- Comparar por caminho; abrir novos, fechar removidos.
- `POLLHUP`/`POLLERR`/`ENODEV` também indicam remoção.
- Se o primário sumir:
  1. escolher o próximo mouse-like legível;
  2. emitir `DeviceRemoved` + `DeviceAdded`;
  3. recentralizar o cursor e preservar contadores por dispositivo;
  4. se não houver, estado de espera.
- Dispositivo novo não muda o primário automaticamente, exceto se não houver
  primário.
- Sem `inotify` na v1 (D-010).

## 9. Grab

- Padrão: desligado.
- `--grab`: `EVIOCGRAB(1)` após abrir; liberar com `EVIOCGRAB(0)` em `Drop`.
- Efeito: o sistema deixa de receber o mouse; o cursor do SO congela.
- Combo: recusar com mensagem; `--grab-force` permite, com aviso de que o
  teclado do mesmo nó também será capturado.
- Se o processo morrer, o kernel libera o grab ao fechar o fd; ainda assim,
  liberar explicitamente.
- Grab não é necessário para medir; é uma opção de isolamento.

## 10. Permissões e privacidade

- `/dev/input/event*` tipicamente `root:input 660`.
- Verificar `getgroups`/acesso; não executar `id` via shell.
- Mensagem: comando sugerido (`usermod -aG input $USER`), necessidade de novo
  login e aviso de que o grupo `input` permite keylogging.
- Nunca rodar como root; nunca alterar permissões/udev automaticamente.
- Dispositivos combo: apenas eventos de mouse são normalizados; teclas são
  descartadas imediatamente.
- Log bruto nunca inclui keycodes de teclado.
- Diagnóstico lista capacidades, não conteúdo de teclas.

## 11. Fallback `crossterm`

- Habilitado apenas na TUI quando evdev indisponível ou `--tui-mouse on`.
- `EnableMouseCapture` ao entrar; `DisableMouseCapture` ao sair.
- Mapeamento:
  - `MouseEventKind::Down/Up(Left|Right|Middle)` → `Button` (0x110/0x111/0x112);
  - `ScrollUp/Down/Left/Right` → `Wheel` com notch ±1;
  - `Drag`/`Moved` → `Move` em células, se o terminal reportar;
  - posição `column/row` → coordenada do canvas.
- Limitações:
  - sem movimento relativo bruto; sem counts;
  - sem botões laterais em muitos terminais;
  - sem hover contínuo; alguns terminais só reportam com botão pressionado;
  - sem taxa de relatório confiável;
  - captura de mouse pode conflitar com seleção de texto (Shift).
- UI mostra etiqueta `TUI-MOUSE`/degradado e esconde métricas impossíveis.
- Não habilitar quando evdev estiver ativo.
- Frame buffer não usa este fallback.

## 12. Estimador de taxa de relatório

- Base: timestamps de `SYN_REPORT` que contenham evento de mouse.
- Intervalo válido: `0 < interval_us <= 250_000`.
- Ring de 256 intervalos; mediana; `hz = 1_000_000 / median_us`.
- Jitter: `p95 - p50` em µs.
- Confiança:
  - baixa: < 20 amostras;
  - média: 20–99;
  - alta: ≥ 100.
- Sem relatórios por > 2 s: manter último valor esmaecido, reduzir confiança e
  mostrar dica de movimento.
- Nunca afirmar polling nominal; texto sempre “estimada”.
- Dispositivos que coalescem ou enviam múltiplos relatórios por pacote podem
  apresentar valores diferentes do configurado; documentar.

## 13. Diagnóstico de entrada

`--list-devices` (sem sessão interativa):

```text
PATH            CLASS      NAME                 PHYS        PERM
/dev/input/event5 mouse    Logitech G Pro       usb-...     ok
/dev/input/event6 touchpad  SynPS/2 Synaptics   ...         denied
```

`--diagnostics out.txt`:

```text
mouse_tester diagnostics
version=...
kernel=...
os=...
session=...
tty=...
groups=...
backend=...
fb.device=/dev/fb0
fb.readable=yes
fb.format=...
input.devices=2
device.0.path=/dev/input/event5
device.0.name=...
device.0.class=mouse
device.0.phys=...
device.0.uniq=...
device.0.ids=0003:046d:c08b
device.0.permissions=readable
device.0.caps.key=...
device.0.caps.rel=...
device.0.caps.abs=...
device.0.abs.x=0..32767:res=...
notes=...
```

- Se não puder abrir, `permissions=denied` e o motivo.
- Não listar keycodes pressionados.
- O relatório é texto simples (D-013).

## 14. Matriz de falhas

| Caso | Detecção | Comportamento | Saída |
| --- | --- | --- | --- |
| evdev sem permissão, TUI | `EACCES` | banner + fallback crossterm | 0 ao sair |
| evdev sem permissão, `--require-evdev` | `EACCES` | erro acionável | 4 |
| evdev sem permissão, fb | `EACCES` | erro antes de `KD_GRAPHICS` | 4 |
| sem mouse conectado, TUI | lista vazia | espera + hotplug | 0 |
| sem mouse conectado, fb | lista vazia | espera + hotplug | 0 |
| dispositivo removido | `ENODEV`/`POLLHUP` | troca ou espera | 0 |
| grab em combo | classe combo | recusa | 0 (continua) |
| `SYN_DROPPED` | code 3 | ressincroniza botões | 0 |
| overflow de canal | canal cheio | descarta + contador | 0 |
| evento truncado | tamanho inválido | descarta e conta | 0 |
| fd ocupado | `EBUSY` | mostra motivo, não insiste | 0 |

## 15. Testes e fixtures

- Fixtures sintéticas de 24 bytes por evento.
- Casos em `docs/05-test-plan.md`.
- Testar:
  - parse little-endian e tamanho;
  - `SYN_DROPPED` + `EVIOCGKEY` simulado;
  - botões desconhecidos;
  - hi-res com/sem normal;
  - intervalos de 125/1000 Hz;
  - remoção durante leitura;
  - overflow;
  - classificação de combo e descarte de teclas.
- Nenhuma fixture contém dados reais de usuário.

## 16. Pseudocódigo de referência

```text
fn evdev_thread(paths, tx, stop):
    fds = open_all_readable(paths)
    while !stop:
        ready = poll(fds, 100ms)
        for fd in ready:
            data = read_until_eagain(fd)
            for ev in parse_events(data):
                if ev is keyboard_key:
                    continue                 # privacidade
                if ev.type == EV_KEY and is_mouse_button(ev.code):
                    emit(Button{ev.code, ev.value == 1})
                elif ev.type == EV_REL:
                    accumulate(ev)
                elif ev.type == EV_ABS and device.absolute:
                    emit(Absolute{...})
                elif ev.type == EV_SYN and ev.code == SYN_REPORT:
                    flush_wheel_and_motion()
                    emit(Report{interval})
                elif ev.type == EV_SYN and ev.code == SYN_DROPPED:
                    reset_accumulators()
                    reconcile_buttons(EVIOCGKEY)
                    emit(Overflow)
        every 1s: rescan_devices()
    close_all(fds)
```
