# 04 — Assets e licenças

## 1. Princípios

- **Sem assets de arte.** O visual é procedural: cursor, rastro, anéis, pulsos,
  grade e painéis são desenhados por código.
- Nenhuma imagem bitmap, textura, ícone ou logo na v1.
- Nenhuma fonte externa embutida: o fb usa a fonte 5×7 interna (D-016).
- O binário funciona copiado sozinho; nada é baixado em runtime.
- Capturas PNG de QA são artefatos de teste, não assets de runtime.

## 2. Inventário

| Asset | Formato | Origem | Uso |
| --- | --- | --- | --- |
| Fonte 5×7 | código | próprio projeto | texto do fb |
| Capturas de QA | PNG gerado | teste/captura | docs/testes, não embutido |

- TUI usa a fonte do terminal.
- Não há áudio, sprites, cursores bitmap ou temas externos.
- `--capture-frame` gera PNG sob demanda; o arquivo não é necessário para rodar.

## 3. Pipeline

Não há pipeline de assets nem manifesto. O portão `cargo xtask assets --check`
não se aplica e não existe workspace `xtask` na v1.

Regras:

- Não introduzir asset de terceiro sem licença compatível, crédito e aviso.
- Se uma fonte vetorial for adotada no futuro, reabrir D-016 e adicionar
  manifesto, checksum e licença ao pacote.

## 4. Licenças

| Componente | Licença | Obrigação |
| --- | --- | --- |
| Código do Mouse Tester | MIT | incluir `LICENSE` |
| Efeitos e arte procedural | MIT (parte do código) | sem licença separada |
| Crates Rust | MIT/Apache-2.0, BSL-1.0, Unicode-3.0, Zlib | `THIRD_PARTY_NOTICES.md` + `licenses/` |

O pacote inclui `LICENSE`, `THIRD_PARTY_NOTICES.md` e os textos em `licenses/`.
Não há `OFL.txt` porque nenhuma fonte externa é distribuída.

## 5. Créditos

```text
Mouse Tester
Código e efeitos procedurais: MIT, Copyright (c) 2026 Daniel (danielexpe).
Crates de terceiros: ver THIRD_PARTY_NOTICES.md.
```

## 6. O que não será feito

- Baixar fontes, ícones ou temas da internet em runtime.
- Embutir arte de terceiros sem licença.
- Usar `ratatui-image`, Kitty/Sixel ou protocolos de imagem na v1.
- Usar sprites ou texturas para efeitos.
- Versionar binários, capturas de QA grandes ou caches de build.
- Incluir dados reais de dispositivos do usuário no repositório.
