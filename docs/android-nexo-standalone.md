# Horizon Nexo — aplicativo independente

O Horizon Nexo usa o pacote Android `dev.horizon.nexo`. Ele abre diretamente na interface Nexo e pode coexistir com as outras instalações do Horizon. Seus dados ficam no espaço próprio do novo aplicativo.

## Interface e temas

- Carrossel 3D tátil, seleção direta de uma aba e abertura pelo próprio painel. Jogar chama a preparação ou a execução real do jogo.
- Painéis, ícones com formas de Minecraft, iluminação, sombras, reflexos e blur direcional por objeto mantêm a linguagem visual Nexo.
- Abertura animada com opção de pular, rever ou reduzir movimento. Os sons editados do Adobe Firefly permanecem com volume ajustável.
- Os temas Maré, Prisma, Brasa e Mineral variam a paleta e a iluminação dentro da mesma interface. Materiais acetinado, polido e fosco alteram a resposta dos objetos à luz.
- Não há seleção de interface Original, Old School ou Liquid Glass neste aplicativo.

## Telas próprias

O catálogo tem áreas Explorar e Minha coleção, busca, fontes disponíveis, detalhes, versões, importação e controles dos downloads. A coleção usa o estado real de instalação e ativação do motor.

O gerenciador de arquivos tem raízes, pastas, seleção e ações de renomear, copiar, mover, importar, compartilhar e excluir. As confirmações e formulários usam os componentes Nexo.

O ateliê de personagem apresenta as skins e capas locais e da conta, a prévia real do personagem, escolha de modelo, importação e envio pelos serviços existentes.

As telas de identidade apresentam Microsoft, Ely.by, perfil local e contas salvas. A autenticação continua a cargo dos serviços nativos. Senha e código de segundo fator não são refletidos nos dados enviados à interface.

Os ajustes são organizados em Atmosfera Nexo, Imagem e desempenho, Controles e gestos, Jogo e dispositivo, Ajustes avançados e Diagnóstico. O editor Nexo permite arrastar controles, ajustar dimensões e comportamento, adicionar atalhos e salvar o layout real usado pelo jogo.

## Integração

A interface recebe dados de domínio e identificadores de ações pela ponte nativa de esquema 2. Ela não captura nem reproduz a disposição dos componentes visuais antigos. O APK independente exclui as páginas e os scripts do protótipo anterior. O motor, os serviços de contas, o instalador e as operações de conteúdo e arquivos continuam nativos.

O contexto WebGL é destruído antes de entregar a tela à partida, liberando os recursos da interface. A cena pausa quando o aplicativo deixa de estar visível. Há uma apresentação Nexo de compatibilidade quando WebGL não inicia.

O launcher abre numa janela Android própria, sem esperar o renderizador Vulkan ou o atlas do jogo. A vistoria de tipografia, margens, recuperação e navegação está descrita em [Abertura e vistoria do launcher](android-nexo-startup-audit.md).

## Verificação

Os testes automatizados da interface usam dados de domínio simulados, sem contas reais. Eles exercitam busca, detalhes, temas, ajustes, arraste e salvamento do layout, renomeação, campos de autenticação e uma única chamada direta de Jogar. Não substituem um teste de partida no celular.

| Área visível | Itens completos na biblioteca | Corte horizontal |
| --- | ---: | --- |
| 852 × 393 | 6 | Não |
| 393 × 852 | 4 | Não |
| 360 × 640 | 4 | Não |
| 1280 × 720 | 8 | Não |

A compilação Android ARM64, a assinatura e a auditoria do pacote são verificadas pelo fluxo `Horizon Nexo standalone` no GitHub. O artefato inclui a revisão, o hash SHA-256 e os avisos de licença.

## Validação no aparelho

Ainda é necessário testar a instalação final, o teclado e a navegação no Android, autenticação real, acesso aos arquivos, preparação e entrada no jogo e retorno ao launcher. Desempenho e acabamento dos efeitos precisam ser avaliados no aparelho; os testes de navegador não medem FPS do celular.

Como o pacote é independente, contas e arquivos de outro aplicativo não são migrados automaticamente. Esta compilação de teste usa uma chave temporária do CI. Uma chave estável deve ser definida antes de um canal contínuo de atualizações que preserve a instalação entre versões.
