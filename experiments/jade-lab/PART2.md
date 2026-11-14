# Jade Lab — Parte 2: cinco entregas verificáveis

Este projeto continua como aplicativo Android **dark-only**, separado do Horizon
principal e do Glass Lab. O objetivo é um laboratório de materiais e fundos
inspirado na **linguagem de consoles da era Xbox 360**, sem copiar marcas,
assets, interface ou sons proprietários.

| Tarefa | Escopo | Status |
| --- | --- | --- |
| 1. Identidade e seleção | Remover barra lateral fixa, implementar seleção luminosa GPU, base de sete presets completos | **Implementada — validar no Android** |
| 2. Fundo atmosférico | Ondas em camadas, gradientes, textura de fundo, halo volumétrico e controles próprios | **Implementada — validar no Android** |
| 3. Materiais de vidro fosco | Frosted Dark, Tinted Frost, Dense Matte; blur amostrado, borda física e luz transmitida; cena de console mais espaçosa | **Implementada — validar no Android** |
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

## Tarefa 3 — entregue na branch `feature/jade-lab-part2-frosted-materials`

- **Três acabamentos ópticos reais** no WebGL2: `frosted` (difusão atmosférica
  translúcida), `tinted` (vidro colorido mais encorpado) e `matte` (acabamento
  espesso com menor transmissão); opção `solid` para referência de antes/depois.
- `frostedBackdrop` amostra **o fundo procedural gerado na GPU** em um padrão
  de até nove pontos, preservando o texto da interface em resolução nativa.
  Não simula borrão com mera opacidade; tampouco promete desfocar texto DOM
  ou aplicações Android externas.
- Nova aba **Vidro**: densidade do pigmento, difusão, transmissão, cor, espalhamento,
  Fresnel, luz de contorno, grão, reflexo superior, sombra, qualidade de amostras
  e opções de aplicação em hero/cards/barras/CTA.
- Cada uma das sete paletas possui um acabamento calibrado independente.
- **Recomposição solicitada após inspeção da UI anterior:** o dashboard antigo
  cobria quase todo o fundo com seis cards e um painel grande. Foi substituído
  por uma **cena aberta** com um avatar geométrico original feito com CSS,
  um hero compacto e **três cards sobrepostos em perspectiva** (não uma grade).
  O foco muda por toque, setas e gesto lateral, com animação de profundidade.
  Os seis módulos seguem alcançáveis, mas apenas três aparecem simultaneamente.
- O avatar **não** utiliza skins Minecraft/Xbox nem representa um jogador real;
  é um marcador de referência original para testes de movimento e composição.
- Cartões inclinados usam material de vidro CSS/compositor (backdrop-filter)
  para acompanhar sua perspectiva; renderizar por trás deles um material
  retangular no shader provocava caixas fantasmas. Corrigimos a associação
  fixa dos índices `uRects` e ignoramos no shader de placas os cards transformados.
  Os painéis alinhados continuam sendo renderizados no shader GPU.
- Controles extras **Cena 3D**: profundidade, espaçamento, inclinação e movimento
  do avatar, preservados no preset completo e na exportação da aba Componentes.
- `glassChrome` inicia desligado para **deixar o fundo aparente**. O usuário
  continua podendo ativá-lo. Editor recolhível, modo escuro e exportação por
  aba mantidos. APK de testes em versão `1.3.0-dark`.

A profundidade dos cards e do avatar é realizada com perspectiva tridimensional
no compositor WebView/CSS; **não é uma engine 3D de malhas e esqueletos**. A física
óptica dos painéis alinhados e o fundo são calculados no WebGL2. A aparência
deve ser avaliada no celular antes de considerá-la final.

## Limitações e fronteiras

A base atual usa **WebGL2 / GLSL ES 3.00**, não o WGSL nativo do launcher Rust.
O pack exportado é uma referência técnica; a adaptação para a engine Rust será
um trabalho separado. A interface é uma prévia interativa do console; não inicia
o Minecraft. Não se considera a aparência validada antes de inspeção no celular.

As Tarefas 1 e 2 introduzem seleção dinâmica, paletas e fundo atmosférico.
A Tarefa 3 implementa vidro fosco e reorganiza a cena de console para evitar
ocultar o fundo. Polimento fino de materiais, ícones e microtexturas continua
na Tarefa 4; validação visual física e desempenho ficam para a Tarefa 5.
