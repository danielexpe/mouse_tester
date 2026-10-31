# 11 — Matriz manual da release 1.0.0

Registro da validação executada em 2026-10-31. Itens não executados ficam
explicitamente pendentes.

## Ambiente

- Zorin OS 18.1 (base Ubuntu noble), kernel 7.0.0-42.
- GNOME/Wayland para a TUI; VT `tty3` para o framebuffer.
- `/dev/fb0` 1920×1080 32 bpp, stride 7680, `blank=0`.
- Mouse HID `1bcf:08a0` via evdev.
- Usuário nos grupos `input` e `video`; app nunca executado como root.

## Executado

| Item | Resultado |
| --- | --- |
| TUI com evdev real no GNOME Terminal | OK |
| Movimento, cursor, rastro e contagem | OK |
| Botões L/R/M, laterais e extras | OK |
| Scroll vertical/horizontal e hi-res | OK |
| Hotplug: adicionar/remover/trocar primário | OK |
| Modo Alvos: clique, arraste e duplo clique | OK |
| Captura PNG determinística 1344×756 | OK |
| Modo degradado sem evdev em PTY | OK |
| Restauração em saída normal, `Ctrl+C` e SIGTERM (TUI) | OK |
| Backend fb em VT `tty3`, 1920×1080 32 bpp | OK após corrigir eixo Y |
| Movimento vertical no fb | OK |
| Saída normal do fb restaura console | OK |
| `testes_debug.sh` gera relatório único | OK |

## Pendente

| Item | Motivo |
| --- | --- |
| 24/16 bpp em hardware | só há 32 bpp disponível; parse/packing testados |
| SIGTERM e pânico no fb | caminho implementado; não exercitado em VT |
| Pan honrado pelo driver | sem confirmação visual do uso de double buffer |
| Konsole, Alacritty, Kitty, xterm | não executados |
| SSH degradado | não executado |
| Matriz 80×24/100×30/120×36/Minimal | coberta por snapshots/telas, não manual completa |
| Grab ligado/desligado | não exercitado com evdev real |
| Mouse Bluetooth/combo | sem hardware disponível |
| Outras máquinas base Ubuntu | alvo declarado, não testado |
| Performance da TUI | não medida |

## Observações

- Durante a validação foram corrigidos o eixo Y invertido no fb e um falso
  negativo na detecção de VT.
- `testes_debug.sh` passou a ter preflight de fb opcional (`--with-fb`) com
  `timeout -k` para não prender terminais.
- Nenhum keycode de teclado é lido ou registrado; dispositivos combo têm teclas
  descartadas.
