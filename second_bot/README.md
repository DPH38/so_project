# Projeto Second Bot

<div align="center">

![Versão](https://img.shields.io/badge/versão-1.0.0-blue.svg)
![Status](https://img.shields.io/badge/status-estável-success)
![Linguagem](https://img.shields.io/badge/linguagem-rust-orange)
![Plataformas](https://img.shields.io/badge/plataformas-linux%20%7C%20windows-lightgrey)

<img src="https://rustacean.net/assets/rustacean-flat-happy.png" width="200" alt="Ferris the Rustacean">

**Sistema de Monitoramento Distribuído com Agentes Paralelos**

</div>

O `second_bot` é uma solução completa para monitoramento e coleta de snapshots de múltiplas máquinas em ambientes distribuídos, desenvolvida em Rust para máxima performance e confiabilidade. O sistema utiliza processamento paralelo para coleta eficiente de dados e oferece interface centralizada para gerenciamento de todos os agentes remotos.

## 📋 Índice
- [Visão Geral](#visão-geral)
- [Arquitetura](#arquitetura)
- [Componentes](#componentes)
- [Fluxo de Funcionamento](#fluxo-de-funcionamento)
- [Instalação e Compilação](#instalação-e-compilação)
- [Guia de Uso Rápido](#guia-de-uso-rápido)
- [Recursos Técnicos](#recursos-técnicos)
- [Considerações de Segurança](#considerações-de-segurança)
- [Desenvolvimento e Contribuições](#desenvolvimento-e-contribuições)
- [FAQ](#faq)
- [Licença](#licença)

## 🔍 Visão Geral

O projeto `second_bot` foi desenvolvido para monitorar e coletar dados de performance de múltiplos sistemas operacionais em ambientes de rede heterogêneos. Com uma arquitetura distribuída baseada em agentes, o sistema oferece:

- 🖥️ **Suporte multi-plataforma** para ambientes Linux e Windows
- 🔄 **Processamento paralelo** para coleta eficiente de dados
- 🔌 **Deploy automatizado** de agentes via SSH
- 📊 **Dados detalhados** sobre CPU, memória, disco, processos e arquivos
- 🧠 **Inteligência centralizada** para análise e gerenciamento
- 🛠️ **Interface intuitiva** via menu de terminal interativo

## 🏛️ Arquitetura

O sistema utiliza uma arquitetura cliente-servidor distribuída, composta por:

<div align="center">
<pre>
┌─────────────────────┐             ┌─────────────────────┐
│                     │             │                     │
│   Central Monitor   │◄───SSH──────┤   VM Linux #1       │
│                     │    SCP      │   ┌─────────────┐   │
│  ┌───────────────┐  │             │   │ Snapshot    │   │
│  │ Menu          │  │             │   │ Agent Linux │   │
│  │ Interativo    │  │             │   └─────────────┘   │
│  └───────────────┘  │             │                     │
│                     │             └─────────────────────┘
│  ┌───────────────┐  │
│  │ Gerenciador   │  │             ┌─────────────────────┐
│  │ de Agentes    │  │             │                     │
│  └───────────────┘  │◄───SSH──────┤   VM Windows #2     │
│                     │    SCP      │   ┌─────────────┐   │
│  ┌───────────────┐  │             │   │ Snapshot    │   │
│  │ Analisador    │  │             │   │ Agent Win   │   │
│  │ de Dados      │  │             │   └─────────────┘   │
│  └───────────────┘  │             │                     │
│                     │             └─────────────────────┘
└─────────────────────┘
       Controle                     Agentes Distribuídos
</pre>
</div>

## 🧩 Componentes

O sistema é composto por dois componentes principais, cada um com seu propósito específico:

### 1. Central Monitor

<img align="right" width="80" src="https://raw.githubusercontent.com/tokio-rs/website/master/public/img/icons/tokio.svg">

**Função**: Hub central que gerencia, monitora e interage com todos os agentes.

**Recursos**:
- ✨ Interface interativa baseada em menu de terminal
- 🔌 Gerenciamento automático de conexões SSH
- 🔍 Detecção automática de SO e hostname
- 📦 Deploy de binários compatíveis com cada SO
- 📊 Visualização de logs e status centralizados
- 🔄 Operações de ciclo de vida (iniciar, parar, reiniciar)

**Tecnologias**: Rust, tokio (assíncrono), colored, dialoguer, anyhow

**Mais detalhes**: [`central_monitor/README.md`](central_monitor/README.md)

### 2. Snapshot Agent

<img align="right" width="80" src="https://upload.wikimedia.org/wikipedia/commons/thumb/0/0f/Original_Ferris.svg/512px-Original_Ferris.svg.png">

**Função**: Agentes leves executados nas máquinas monitoradas que coletam dados do sistema.

**Recursos**:
- ⚡ Coleta de dados paralela via múltiplas threads
- 📊 Métricas detalhadas de sistema (CPU, memória, disco, etc.)
- 📁 Monitoramento de diretórios específicos
- 📝 Logging robusto com rotação de arquivos
- 🔌 Integração com monitor central
- 🖥️ Versões específicas para Linux e Windows

**Tecnologias**: Rust, threading, APIs nativas de cada SO

**Mais detalhes**: [`snapshot_agent/README.md`](snapshot_agent/README.md)

## ⚙️ Fluxo de Funcionamento

<div align="center">
<img src="https://mermaid.ink/img/pako:eNp1kk9PwzAMxb9K5FOBDzDYBKceuFTiC6ROxsSmjdK4clzQEPtutEALnXaJnPj9_OI_WU9GlZ60SVfXdqBP_8r3pw-7yCAOQNJvpRuoUwH0v_BFWlPQS0DiULBUb4AqxRVLvYOQZsw7Qn-CcjliJDgE77Uqwx62iEzZsr0eMfFmbY5YKfl24QOa90jck59hx62Hpjq3tQdyZaXKQzipb1FaAvS70lLiMBbhic4965Kc0Fgz7BBLXs0gFYWROYY-jzgR1HTwCKr5mQDO48xUWcJqC9O7MlqLPCyKbFtkOOFMfAHlI_0iMbzgDajpKzJXqDPqO5u6qA4txaHhwqd-4A2VqrgResk0DTUos7POZw1F1nDr4CYdNNi7XwUJ1DXcW9Cdjrqh1A15ZVtly31mTbe6A5PbUne0zDrn8ntkFekprTMFGdFTUcfak6rXQMeY_HxL_wnZMeoo?type=png" width="800" alt="Fluxo de Funcionamento">
</div>

1. **Inicialização**: O usuário inicia o `central_monitor` em sua máquina local
2. **Conexão**: O monitor estabelece conexões SSH com as VMs configuradas
3. **Detecção**: O sistema detecta automaticamente o SO de cada VM
4. **Deploy**: O monitor realiza o deploy do agente apropriado para cada SO
5. **Configuração**: Os agentes são configurados e inicializados como serviços locais
6. **Coleta**: Cada agente coleta dados do sistema utilizando threads paralelas:
   - Thread para informações de CPU
   - Thread para informações de memória
   - Thread para informações de disco
   - Thread para listagem de arquivos em diretórios monitorados
7. **Registro**: Os agentes registram snapshots localmente e/ou enviam para o monitor
8. **Monitoramento**: O central_monitor permite consultar status e logs dos agentes
9. **Gerenciamento**: O usuário pode iniciar, parar, reiniciar ou remover agentes via menu

## 🛠️ Instalação e Compilação

### Pré-requisitos

- **Rust** 1.60 ou superior: `rustup update stable`
- **Cargo**: Geralmente instalado com Rust
- **OpenSSH**: Para conexões SSH e transferência de arquivos
- **GCC MinGW-w64**: Para cross-compilation Windows (opcional)

### Compilação Completa

```bash
# Clonar o repositório
git clone https://github.com/username/second_bot.git
cd second_bot

# Compilar o monitor central
cd central_monitor
cargo build --release --bin central_monitor

# Compilar os agentes
cd ../snapshot_agent

# Agente Linux (nativo)
cargo build --release --bin snapshot_agent_linux

# Agente Windows (cross-compilation)
rustup target add x86_64-pc-windows-gnu
sudo apt-get install gcc-mingw-w64-x86-64
cargo build --release --target x86_64-pc-windows-gnu --bin snapshot_agent_windows

# OU, se preferir compilação nativa em cada SO:
# No Windows:
# cargo build --release --bin snapshot_agent_windows
# No Linux:
# cargo build --release --bin snapshot_agent_linux
```

### Diretórios de Saída

- **Monitor Central**: `central_monitor/target/release/central_monitor`
- **Agente Linux**: `snapshot_agent/target/release/snapshot_agent_linux`
- **Agente Windows**: `snapshot_agent/target/x86_64-pc-windows-gnu/release/snapshot_agent_windows.exe`

## 🚀 Guia de Uso Rápido

### 1. Iniciar o Monitor Central
```bash
cd central_monitor
./target/release/central_monitor
```

### 2. Utilizar o Menu Interativo

1. Selecione "Testar conexão com VM" para verificar a conectividade SSH
2. Escolha "Instalar/Atualizar agente" para fazer deploy nas VMs
3. Use "Listar status" para verificar o funcionamento dos agentes
4. Acesse "Ver logs" para conferir a atividade dos agentes

### 3. Monitoramento de Dados

- Os logs dos agentes são armazenados em:
  - Linux: `~/.snapshot_agent/snapshot.log`
  - Windows: `C:\Users\<usuário>\.snapshot_agent\snapshot.log`
- Os snapshots são armazenados em:
  - Linux: `~/.snapshot_agent/data/`
  - Windows: `C:\Users\<usuário>\.snapshot_agent\data\`

## 💡 Recursos Técnicos

### Performance e Escalabilidade

- **Processamento Paralelo**: Coleta dados via múltiplas threads para eficiência
- **Baixo Consumo**: Agentes otimizados para mínimo impacto no sistema hospedeiro
- **Throttling Inteligente**: Ajuste automático de intervalo baseado em carga do sistema
- **Compilação Nativa**: Binários otimizados para cada plataforma

### Segurança

- **Autenticação SSH**: Utiliza configuração SSH padrão com suporte a chaves
- **Sem Dados Sensíveis**: Não coleta informações confidenciais do sistema
- **Logs Seguros**: Não registra senhas ou dados de autenticação
- **Permissões Mínimas**: Execução com privilégios mínimos necessários

### Compatibilidade

- **Linux**: Testado em Ubuntu, Debian, CentOS e Fedora
- **Windows**: Testado em Windows 10, Windows Server 2016/2019/2022
- **Arquiteturas**: x86_64 (outras podem ser suportadas via compilação)

## 🔒 Considerações de Segurança

- **Configuração SSH**: Recomenda-se usar autenticação por chave, não por senha
- **Firewall**: Os agentes não abrem portas de rede (comunicação iniciada pelo central_monitor)
- **Permissões**: Revise os diretórios e arquivos criados para garantir segurança
- **Isolamento**: Em ambientes de produção, considere isolar o central_monitor em rede segura

## 💻 Desenvolvimento e Contribuições

### Estrutura do Repositório
```
second_bot/
├── central_monitor/       # Componente central de gerenciamento
│   ├── src/
│   │   ├── main.rs        # Lógica principal e conexões SSH
│   │   └── cli/           # Interface de linha de comando
│   ├── Cargo.toml         # Dependências
│   └── README.md          # Documentação específica
│
├── snapshot_agent/        # Agentes para coleta de dados
│   ├── src/
│   │   └── bin/
│   │       ├── linux.rs   # Implementação específica para Linux
│   │       └── windows.rs # Implementação específica para Windows
│   ├── Cargo.toml         # Dependências
│   └── README.md          # Documentação específica
│
├── Cargo.lock             # Versões fixas das dependências
└── README.md              # Este documento
```

### Como Contribuir

1. Faça um fork do repositório
2. Crie sua branch de feature (`git checkout -b feature/nova-funcionalidade`)
3. Faça commits de suas alterações (`git commit -m 'Adiciona nova funcionalidade'`)
4. Envie para a branch (`git push origin feature/nova-funcionalidade`)
5. Abra um Pull Request

### Áreas para Contribuição
- Suporte para mais sistemas operacionais (macOS, FreeBSD)
- Interface web para o monitor central
- Visualização de dados e gráficos
- Melhorias de segurança e criptografia
- Otimizações de performance

## ❓ FAQ

### É possível monitorar outros sistemas operacionais além de Linux e Windows?
Atualmente, o sistema suporta apenas Linux e Windows. Contribuições para suporte a outros sistemas são bem-vindas.

### Qual o impacto de performance dos agentes no sistema?
Os agentes foram projetados para uso mínimo de recursos, tipicamente consumindo menos de 1% de CPU e ~30MB de RAM.

### O sistema requer acesso root/administrador?
Para coleta completa de métricas, recomenda-se execução com privilégios elevados, mas a maioria das funcionalidades opera com usuário regular.

### Os dados coletados são criptografados?
A transferência ocorre via SSH, que é criptografada. O armazenamento local não é criptografado por padrão.

### Como expandir o conjunto de métricas coletadas?
Modifique os arquivos `linux.rs` ou `windows.rs` para adicionar novas métricas e contribua com um pull request.

## 📄 Licença

Este projeto está licenciado sob a licença MIT - veja o arquivo LICENSE para mais detalhes.

---

<div align="center">
<p><b>Desenvolvido pelo Grupo de Sistemas Operacionais - 2025</b></p>
<p><small>Powered by Rust 🦀</small></p>
</div>
