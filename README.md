# Projeto de Sistemas Operacionais - Bots de Monitoramento e Interação com SO

<div align="center">

![Versão](https://img.shields.io/badge/versão-1.0.0-blue.svg)
![Status](https://img.shields.io/badge/status-completo-success)
![Linguagem](https://img.shields.io/badge/linguagem-rust-orange)
![Plataformas](https://img.shields.io/badge/plataformas-linux%20%7C%20windows-lightgrey)

</div>

## 📌 Visão Geral

Este projeto consiste em três bots escritos em Rust, cada um com propósitos distintos, demonstrando diferentes níveis de interação com sistemas operacionais. Os bots progridem em complexidade desde manipulação local de arquivos até monitoramento remoto com análise de conteúdo usando IA.

## 🤖 Os Três Bots

### 1️⃣ First Bot - Gerenciamento Local de Usuários
Um utilitário simples para cadastro e consulta de usuários via interface de terminal, armazenando dados em arquivos locais.

**Conceitos de SO explorados:**
- Manipulação de arquivos
- Interface de linha de comando
- Gerenciamento de diretórios e arquivos locais

### 2️⃣ Second Bot - Monitoramento Distribuído com Agentes Paralelos
Sistema avançado para monitoramento e coleta de snapshots de múltiplas máquinas em ambientes distribuídos, utilizando processamento paralelo.

**Conceitos de SO explorados:**
- Deployment remoto de executáveis via SSH
- Paralelismo e concorrência
- Comunicação entre processos
- Monitoramento de recursos do sistema (CPU, memória, disco)
- Suporte cross-platform (Linux e Windows)

### 3️⃣ Third Bot - Scanner Não-Intrusivo com Análise de Conteúdo
Sistema não-invasivo para mapear sistemas de arquivos remotos e analisar conteúdo de PDFs usando OpenAI GPT.

**Conceitos de SO explorados:**
- Injeção de binários independentes para evitar dependência de ferramentas do SO
- Transferência e processamento seguro de arquivos
- Scan recursivo de sistemas de arquivos
- Integração com APIs de IA para análise de conteúdo
- Operações temporárias sem rastros permanentes

## 🔍 Características Principais do Projeto

1. **Evolução de Complexidade**: Os bots evoluem de simples (local) para complexo (distribuído com IA).

2. **Uso de Rust**: Escolhemos Rust pela segurança de memória, performance e capacidade de criar binários compactos e independentes.

3. **Não-Intrusividade**: Especialmente no third_bot, desenvolvemos técnicas para monitorar sistemas sem depender de ferramentas nativas, usando apenas binários Rust auto-contidos.

4. **Técnicas de Sistemas Distribuídos**: O second_bot e third_bot implementam técnicas avançadas de comunicação e coleta paralela de dados.

5. **Integração com IA**: O third_bot integra a API OpenAI para análise de conteúdo de documentos, demonstrando possibilidades de automação inteligente.

## 🔧 Arquitetura Técnica

### Interação com o Sistema Operacional
- **first_bot**: Manipulação direta de arquivos locais usando APIs padrão de Rust
- **second_bot**: Execução remota de processos, monitoramento de métricas do sistema, comunicação segura via SSH
- **third_bot**: Injeção de binários independentes, scanning não-invasivo de sistemas de arquivos, processamento de arquivos PDF

### Fluxo de Processamento
1. **Coleta de Dados**: Local (first_bot), remota com agentes distribuídos (second_bot), ou remota via binário temporário (third_bot)
2. **Processamento**: Desde o simples (armazenamento em arquivo) até complexo (análise paralela e processamento com IA)
3. **Visualização**: Interfaces de terminal interativas em todos os bots

## 📊 Comparação entre os Bots

| Característica                   | First Bot | Second Bot | Third Bot |
|----------------------------------|-----------|------------|-----------|
| Complexidade                     | Baixa     | Média      | Alta      |
| Escopo                           | Local     | Distribuído| Distribuído |
| Dependência de ferramentas do SO | Alta      | Média      | Baixa     |
| Processamento Paralelo           | Não       | Sim        | Parcial   |
| Integração com IA                | Não       | Não        | Sim (OpenAI) |
| Cross-platform                   | Parcial   | Total      | Total     |
| Persistência no sistema remoto   | N/A       | Permanente | Temporária |

## 🚀 Como Executar

Cada bot possui seu próprio README com instruções específicas. Em geral, para compilar e executar:

```bash
# Navegue para o diretório do bot
cd first_bot  # ou second_bot, ou third_bot

# Compile o projeto
cargo build --release

# Execute o binário
cargo run --release
```

## 🔐 Requisitos de Segurança

O projeto implementa várias medidas de segurança:

- **Autenticação SSH**: Para operações remotas (second_bot e third_bot)
- **Manipulação temporária**: No third_bot, arquivos são processados localmente e removidos após uso
- **Proteção de credenciais**: API keys são armazenadas em variáveis de ambiente ou arquivos .env
- **Operações sem privilégios**: Os bots foram projetados para operar sem necessidade de privilégios administrativos

## 📝 Conclusão

Este conjunto de bots demonstra diferentes abordagens para interagir com sistemas operacionais, desde operações locais básicas até monitoramento remoto avançado com análise de conteúdo via IA. A progressão dos bots ilustra como aplicações modernas podem escalar de simples utilitários para sistemas distribuídos complexos, mantendo princípios de segurança e eficiência.

O uso do Rust permitiu criar soluções de alto desempenho com segurança de memória garantida em tempo de compilação, essencial para ferramentas de sistema confiáveis. A abordagem não-intrusiva do third_bot representa uma evolução importante, permitindo diagnósticos e análises sem depender de ferramentas nativas do sistema operacional.

## 📜 Licença

Este projeto está sob a licença MIT. Consulte o arquivo LICENSE para mais detalhes.
