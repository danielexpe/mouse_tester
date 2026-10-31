# 04 — Assets e licenças

## 1. Princípios

- **Quase nenhum asset.** O visual é procedural: cursor, rastro, anéis, pulsos,
  grade e painéis são desenhados por código.
- Nenhuma imagem bitmap de arte, textura, ícone ou logo na v1.
- A única dependência visual embutida é uma fonte para o framebuffer.
- Tudo é embutido no binário com `include_bytes!`; o binário deve funcionar
  copiado sozinho.
- Nada é baixado em runtime.
- Assets originais e derivados são versionados; nunca editar derivado à mão.

## 2. Inventário

| Asset | Formato | Origem | Tamanho alvo | Uso |
| --- | --- | --- | --- | --- |
| JetBrains Mono Regular | TTF | projeto JetBrains | ~250 KB | texto do fb |
| JetBrains Mono Bold | TTF | projeto JetBrains | ~250 KB | títulos do fb |
| Licença OFL | TXT | SIL/JetBrains | ~4 KB | obrigação legal |
| Manifesto | TXT | próprio projeto | ~1 KB | verificação |
| Capturas de QA | PNG | gerado em teste | variável | docs/testes, não embutido |

- TUI usa a fonte do terminal; nenhuma fonte é embutida para ela.
- Não há áudio.
- Não há cursores bitmap; o cursor é desenhado.
- Capturas PNG de QA são artefatos de teste, não assets de runtime.

### 2.1 Fonte

- **Família:** JetBrains Mono.
- **Pesos:** Regular e Bold.
- **Licença:** SIL Open Font License 1.1 (`OFL.txt`).
- **Motivo:** mono, tabular, alta legibilidade em console, licença compatível.
- **Fallback:** se a decodificação falhar no startup, erro claro de asset; não
  usar fonte do sistema silenciosamente.
- **Cobertura:** latim básico e acentos pt-BR; validar `ã`, `ç`, `õ`, `é`, `—`,
  `×`, `µ`. Não usar glifos decorativos fora da cobertura.

## 3. Estrutura

```text
assets/
  fonts/
    JetBrainsMono-Regular.ttf
    JetBrainsMono-Bold.ttf
    OFL.txt
  manifest.txt
```

`assets/manifest.txt` em formato simples `chave=valor`:

```text
format=1
font.regular=fonts/JetBrainsMono-Regular.ttf
font.regular.size=...
font.regular.license=OFL-1.1
font.bold=fonts/JetBrainsMono-Bold.ttf
font.bold.size=...
font.bold.license=OFL-1.1
license.file=fonts/OFL.txt
```

- Sem hashing na v1 para evitar dependência criptográfica; o git é a fonte de
  integridade.
- Tamanhos são verificados para detectar arquivo truncado.
- Se no futuro houver mais assets, adicionar SHA-256 ao manifesto.

## 4. Pipeline

`xtask` é um workspace member mínimo:

```bash
cargo xtask assets --check
```

Verifica:

1. Todos os arquivos do manifesto existem.
2. Tamanhos batem.
3. `OFL.txt` está presente e não vazio.
4. As duas fontes decodificam com `fontdue` (quando a dependência estiver
   disponível no xtask; caso contrário, um teste de unidade faz o decode).
5. Nenhum asset não declarado em `assets/` (exceto o próprio manifesto).

Regras:

- `cargo xtask assets` sem `--check` não gera nada na v1 (não há derivados).
- Nunca editar TTF à mão.
- Atualizar o manifesto na mesma tarefa em que um asset muda.
- O portão `assets --check` roda no CI e antes de release.

## 5. Embutir e carregar

- `assets.rs` usa `include_bytes!` para as duas fontes e para `OFL.txt`.
- Decode das fontes uma vez no startup do backend fb; cache `OnceLock`.
- TUI não carrega fontes.
- Teste de unidade valida que cada fonte decodifica e que os glifos usados
  existem.
- Teste de integração copia o binário para `/tmp` e executa `--capture-frame`
  sem a pasta `assets/` para provar que está embutido.

## 6. Licenças

| Componente | Licença | Obrigação |
| --- | --- | --- |
| Código do Mouse Tester | MIT | incluir `LICENSE` |
| Efeitos e arte original | CC BY 4.0 | crédito ao projeto |
| JetBrains Mono | OFL 1.1 | incluir `OFL.txt` e crédito |
| Crates Rust | MIT/Apache-2.0 etc. | `THIRD_PARTY_NOTICES.md` |

Regras:

- Não embutir asset de terceiro sem licença compatível e crédito.
- O pacote de release inclui `LICENSE`, `THIRD_PARTY_NOTICES.md`,
  `assets/fonts/OFL.txt` e créditos.
- A licença do código não cobre a fonte; a OFL é distribuída junto.
- Se um asset for substituído, remover a licença antiga e atualizar os avisos.

## 7. Créditos

Texto planejado para README/release:

```text
Mouse Tester
Código: MIT.
Efeitos visuais e arte original: CC BY 4.0.
Fonte JetBrains Mono: Copyright JetBrains, SIL Open Font License 1.1.
Crates de terceiros: ver THIRD_PARTY_NOTICES.md.
```

## 8. O que não será feito

- Baixar fontes, ícones ou temas da internet em runtime.
- Embutir arte de terceiros sem licença.
- Usar `ratatui-image`, Kitty/Sixel ou protocolos de imagem na v1.
- Usar sprites ou texturas para efeitos.
- Versionar binários, capturas de QA grandes ou caches de build.
- Incluir dados reais de dispositivos do usuário no repositório.

## 9. Testes de assets

- [ ] As duas fontes decodificam.
- [ ] Glifos usados na UI existem em ambos os pesos.
- [ ] Manifesto cobre todos os arquivos.
- [ ] `OFL.txt` presente e íntegro.
- [ ] `cargo xtask assets --check` falha se um arquivo for removido/truncado.
- [ ] Binário copiado para `/tmp` renderiza texto sem assets externos.
- [ ] `THIRD_PARTY_NOTICES.md` gerado/atualizado na Fase 5.
