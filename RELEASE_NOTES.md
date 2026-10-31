# Mouse Tester 1.0.0

Primeira release estável do teste visual de mouse para console Linux.

## Destaques

- TUI com evdev real, descoberta de dispositivos, hotplug e fallback degradado.
- Dashboard, desafios de botões e scroll, taxa estimada, log bruto e modo Alvos.
- Captura PNG determinística de QA (1344×756) sem abrir TUI, evdev ou framebuffer.
- Backend framebuffer com 32/24/16 bpp, pan/fallback, cursor software, sinais e
  restauração do console.
- Interface pt-BR/en, temas truecolor/256/16/mono, `NO_COLOR` e `--no-animations`.

## Validado

- Mouse HID `1bcf:08a0` via evdev em terminal GNOME.
- Backend framebuffer em Zorin OS 18.1 (base Ubuntu), VT `tty3`, 1920×1080 32 bpp.
- Botões extras, scroll hi-res/horizontal e hotplug.

## Limitações

- 24/16 bpp têm testes de parse/packing, mas validação em hardware pendente.
- A taxa é estimada; o app não afirma DPI nem polling nominal.
- Sem áudio, rede, telemetria ou persistência.
- O fb exige VT local e permissões em `/dev/fb0`; não funciona em X11/Wayland/SSH.
- O texto do fb usa fonte 5×7 interna (D-016).

## Instalação

```bash
tar -xzf mouse_tester-1.0.0-linux-x86_64.tar.gz
cd mouse_tester-1.0.0-linux-x86_64
./install.sh --prefix "$HOME/.local"
```

Para evdev, o usuário precisa de leitura em `/dev/input/event*` (grupo `input`,
com aviso de segurança). Para framebuffer, `/dev/fb0` e VT local.
