# Jade Lab — Parte 2: cinco entregas verificáveis

Este projeto continua como aplicativo Android **dark-only**, separado do Horizon
principal e do Glass Lab. O objetivo é um laboratório de materiais e fundos
inspirado na **linguagem de consoles da era Xbox 360**, sem copiar marcas,
assets, interface ou sons proprietários.

| Tarefa | Escopo | Status |
| --- | --- | --- |
| 1. Identidade e seleção | Remover barra lateral fixa, implementar seleção luminosa GPU, base de sete presets completos | **Implementada — validar no Android** |
| 2. Fundo atmosférico | Ondas em camadas, gradientes, textura de fundo, halo volumétrico e controles próprios | **Implementada — validar no Android** |
| 3. Materiais de vidro fosco | Frosted Dark, Tinted Frost, Dense Matte; blur amostrado, borda física e luz transmitida | Pendente |
| 4. Hero, cards e microtexturas | Detalhes de relevo, contraste por módulo, reflexos locais, ícones e ajuste por componente | Pendente |
| 5. Integração e desempenho | Regressão visual, benchmarks GPU, modos de economia, exportação, APK final de testes | Pendente |

## Tarefa 1 — entregue na branch `feature/jade-lab-part2-visual-foundation`

- **Removida de fato** a regra CSS `.hero:before` que criava uma barra verde fixa
  ao longo do painel principal. Não foi apenas escondida sob outro elemento.
- Seleção de página agora muda o **índice da superfície iluminada pelo GPU
  shader**. Os modos são `halo`, `rim`, `underline` e `quiet`.
  A seleção não depende de qualquer barra **lateral** permanente.
- Paletas completas `Jade`, `Amethyst`, `Azure`, `Graphite`,
  `Obsidian`, `Cobalt`, `Smoke`, definidas em `palettes.js`.
  Cada uma tem cinco cores, iluminação, material GGX e atmosfera próprios.
- Interface HTML e controles do editor passam a usar as **mesmas variáveis
  cromáticas que o shader**. Além de trocar a paleta, todos os cinco tons ainda
  podem ser editados; edições de cor ficam marcadas como `custom`.
- Presets importados da versão anterior continuam aceitos. A exportação por aba
  inclui `Paletas` e `Componentes`; o ZIP geral adiciona
  `palettes/catalog.json` com o catálogo completo.
- Mantidos **editor recolhível, exportações de cada aba, ZIP, JSON, qualidade
  GPU, FPS e redutor de movimento**. Aplicar uma nova paleta não redefine FPS,
  escala de renderização nem o modo de movimento reduzido.
- Testes automáticos verificam identidade dark, contraste das sete paletas,
  importação da versão anterior, troca de paleta, exportação de abas e ausência
  da barra fixa.
- A build usa `dev.horizon.jadelab` versão `1.1.0-dark`; o APK do GitHub
  Actions utiliza uma chave temporária diferente em cada execução. Se o
  usuário instalou a versão de teste anterior, é preciso **exportar seu preset
  para Downloads/JadeLab antes de desinstalá-la**, pois as assinaturas
  temporárias não permitem instalação por cima.

## Tarefa 2 — entregue na branch `feature/jade-lab-part2-atmospheric-background`

- `renderer.js` agora calcula um **fundo multi-camadas diretamente no WebGL2**.
  A função `renderBackdrop` permite cinco estilos distintos: `ribbons`,
  `aurora`, `satin`, `halo`, `minimal`. As curvas usam distâncias suaves,
  faixas internas de brilho, névoa radial, degradês e grain estável.
- Uma nova aba **Fundo** tem 16 controles com efeito em tempo real na GPU:
  estilo, intensidade, escala, velocidade, número de faixas, curvatura,
  largura, difusão, reflexo acetinado, halo, profundidade, luz secundária,
  textura, posição de foco X/Y e paralaxe.
- Os fundos são **concebidos por paleta**. Por exemplo, Amethyst tem
  `aurora` etérea, Azure tem `satin`, Obsidian tem halos e Graphite fica
  minimalista. Não há recoloração automática de um único fundo verde.
- A nova configuração não altera a ABI de GGX dos painéis: o shader de
  fundo recebe quatro uniformes vetoriais próprios (`uBackdrop*`).
- O tempo respeita **Reduzir movimento**. O grão é espacialmente estável
  para evitar brilho intermitente. O número de camadas pode cair para uma
  em aparelhos mais limitados; o FPS e escala de resolução já são editáveis.
- A aba Fundo pode ser exportada/importada em JSON, e o ZIP geral contém
  `tabs/fundo.json`, os valores dos uniformes e o shader realmente utilizado.
- A versão é `1.2.0-dark`; testes Node e validação GLSL fazem parte do
  GitHub Actions antes da assinatura do APK.

## Limitações e fronteiras

A base atual usa **WebGL2 / GLSL ES 3.00**, não o WGSL nativo do launcher Rust.
O pack exportado é uma referência técnica; a adaptação para a engine Rust será
um trabalho separado. A interface é uma prévia interativa do console; não inicia
o Minecraft. Não se considera a aparência validada antes de inspeção no celular.

As Tarefas 1 e 2 introduzem seleção dinâmica, paletas e fundo atmosférico.
**Vidro fosco com material próprio**, reflexos transmitidos e blur de cena
continuam reservados para a Tarefa 3. Polimento fino de cards e texturas
fica para a Tarefa 4; validação visual física para a Tarefa 5.
