# Horizon Nexo — abertura e vistoria do launcher

## Abertura

A versão anterior colocava o WebView dentro da janela de `NativeActivity`, cuja superfície pertence ao motor nativo. Além disso, a apresentação do launcher esperava a criação do renderizador Vulkan e do atlas do jogo. O launcher agora usa uma janela Android própria, ligada à mesma Activity, e abre sem inicializar esses recursos de Minecraft. A ponte de domínio continua nativa e só libera as ações depois de receber o primeiro estado real.

Falhas de página, encerramento do processo WebView e ausência do primeiro estado acionam uma tela de recuperação Nexo. A perda do contexto WebGL durante o uso abre uma apresentação Nexo de compatibilidade, com as operações do launcher disponíveis. A recuperação não troca para a interface antiga.

## Acabamento e navegação

Os painéis 3D mantêm o carrossel aprovado e passam a ter cantos discretos. O conteúdo recebe margem interna constante e o cabeçalho usa colunas próprias para o ícone, o título e as ações. A posição do ícone 3D é calculada a partir do espaço efetivamente reservado no cabeçalho. A linha ornamental do painel fechado desaparece antes da entrada dos controles, evitando cruzar textos e botões em telas altas.

Michroma continua nos títulos. Nexo Read, uma derivação renomeada da fonte Rajdhani sob SIL OFL, é usada em textos, campos, listas, botões e legendas das abas. A fonte e a licença são locais. A interface não depende de baixar fontes para abrir.

Os controles compartilham estados de toque, foco, seleção e desativação. Os seletores de modo e de atalho do editor usam painéis próprios. O retorno das abas tem um único comando por toque; o aviso tem um botão de fechar com espaço próprio. Os sliders têm trilha e marcador Nexo. O tamanho dos textos respeita a preferência da interface.

## Cobertura da vistoria

| Área | Verificação |
| --- | --- |
| Menu e abas | Seleção, arraste, abertura no próprio painel, retorno integrado e Jogar direto |
| Mods, pacotes e shaders | Busca, fontes, paginação, detalhes, versões, coleção, importação e estados de instalação |
| Ajustes | Atmosfera Nexo e os cinco grupos nativos, opções, campos, sliders e chaves |
| Arquivos | Pastas, seleção, ferramentas, formulário de renomeação e confirmação |
| Personagem | Skins, capas, modelo, prévia, importação e ações da conta |
| Identidade | Provedores, perfil local, Microsoft, Ely.by e contas salvas, com campos secretos protegidos |
| Editor | Modo, posicionamento, medidas, novo atalho, restauração e salvamento |
| Painéis auxiliares | Escolhas, confirmação, formulário, detalhes, projeto, avisos e fechamento |
| Recuperação | Perda de WebGL e reabertura da janela Android |

O ensaio de geometria usa dados de domínio de teste, incluindo nomes longos, e verifica quatro áreas visíveis: 852 × 393, 393 × 852, 360 × 640 e 568 × 320. Ele mede margens, sobreposições de controles, fonte, posição dos símbolos e limites do ícone 3D; também cobre texto ampliado e recuperação de WebGL. Um ensaio separado mede a biblioteca em 1280 × 720.

O teste Android instala a aplicação release x86_64 em Android 14, abre a `NativeActivity`, espera o estado nativo de esquema 2 e exige uma captura com conteúdo visível. Exercita bibliotecas, configurações, criação real de pasta, formulário de perfil local, skins/capas, mudança de orientação e retorno do segundo plano. Os registros e capturas ficam no artefato `nexo-android-release-boot` do fluxo de compilação. O APK de entrega é ARM64, auditado separadamente, com os mesmos arquivos de interface e código de domínio.

Esses ensaios não demonstram autenticação em serviços externos, entrada numa partida ou desempenho no Redmi. A conferência desses fluxos permanece necessária no aparelho. O motor, FirstPerson e a implementação de áudio do jogo não fazem parte desta alteração.

## Instalação de desenvolvimento

O pacote continua `dev.horizon.nexo`. As compilações do CI usam chaves temporárias e podem exigir reinstalação do Nexo entre versões. Antes de remover uma instalação com conteúdo, exporte os arquivos desejados. A chave estável de distribuição deve ser definida antes de um canal contínuo de atualizações. As outras instalações do Horizon usam pacotes distintos.
