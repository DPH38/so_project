# Central Monitor

<div align="center">
  
![Versão](https://img.shields.io/badge/versão-1.0.0-blue.svg)
![Plataforma](https://img.shields.io/badge/plataforma-linux-lightgrey)
![Licença](https://img.shields.io/badge/licença-MIT-green)

</div>

O `central_monitor` é o componente central do projeto responsável por monitorar, gerenciar e interagir com múltiplos agentes de snapshot (Linux e Windows) em diferentes máquinas virtuais ou físicas através de conexões SSH.

## 📋 Índice
- [Visão Geral](#visão-geral)
- [Funcionalidades](#funcionalidades)
- [Arquitetura](#arquitetura)
- [Instalação e Compilação](#instalação-e-compilação)
- [Configuração](#configuração)
- [Uso](#uso)
- [Troubleshooting](#troubleshooting)
- [Desenvolvimento](#desenvolvimento)
- [Contribuição](#contribuição)
- [Licença](#licença)

## 🔍 Visão Geral

O Central Monitor funciona como um hub de gerenciamento para os agentes de snapshot que coletam métricas de sistemas remotos. Ele facilita:
- O monitoramento unificado de ambientes heterogêneos (Linux e Windows)
- O gerenciamento do ciclo de vida dos agentes (instalação, atualização, remoção)
- A coleta e visualização de dados de desempenho do sistema remoto

## ✨ Funcionalidades

### Gerenciamento de VMs
- ✅ Testa a conexão SSH com múltiplas VMs em paralelo
- 🔍 Detecta automaticamente o sistema operacional remoto (Linux ou Windows)
- 📊 Exibe status de conectividade em tempo real

### Gerenciamento de Agentes
- 📦 Deploy automatizado dos agentes para Linux e Windows
- 🚀 Configuração automática de inicialização (systemd no Linux, scheduled tasks no Windows)
- 🛑 Controle de ciclo de vida (iniciar, parar, reiniciar, remover)
- 🔄 Atualização simplificada de versões

### Monitoramento
- 📝 Visualização de logs centralizada
- 📈 Consulta de status de execução dos agentes
- 🧹 Limpeza de logs e dados antigos

### Interface
- 🖥️ Menu interativo no terminal com navegação intuitiva
- 🎨 Feedback visual com códigos de cores para estados e resultados
- ⌨️ Atalhos de teclado para operações comuns

## 🏛️ Arquitetura

O sistema é composto por três componentes principais:

1. **Central Monitor** (este componente)
   - Escrito em Rust utilizando tokio para operações assíncronas
   - Interface CLI com dialoguer e colored para melhor experiência do usuário
   - Comunica-se via SSH com as máquinas remotas

2. **Agentes de Snapshot**
   - Binários nativos para Linux e Windows
   - Execução como serviço em segundo plano
   - Coleta de dados de sistema em paralelo usando threads

3. **Infraestrutura de Comunicação**
   - Baseada em SSH para comandos e transferência de arquivos
   - Utiliza arquivos de configuração SSH padrão

## 🛠️ Instalação e Compilação

### Pré-requisitos
- Rust 1.60 ou superior (`rustup update stable`)
- OpenSSH client (`ssh`, `scp`)
- Acesso SSH configurado para as VMs alvo
- Git (para clonar o repositório)

### Dependências do Cargo
- tokio (runtime assíncrona)
- colored (colorização do terminal)
- dialoguer (menus interativos)
- anyhow (tratamento de erros)
- chrono (manipulação de datas)
- serde_json (processamento JSON)

### Compilação

```bash
# Compilação padrão
cargo build --release --bin central_monitor

# Compilação com otimizações adicionais
RUSTFLAGS="-C target-cpu=native" cargo build --release --bin central_monitor
```

O binário será gerado em `./target/release/central_monitor`.

## ⚙️ Configuração

### Configuração SSH
Certifique-se de que seu arquivo `~/.ssh/config` está configurado corretamente com entradas para cada VM:

```
Host so-lin
    HostName 192.168.1.1
    User so
    IdentityFile ~/.ssh/id_rsa_so

Host so-win
    HostName 192.168.1.2
    User so
    IdentityFile ~/.ssh/id_rsa_so
```

### Personalização das VMs
Você pode modificar a lista de VMs editando o arquivo `src/main.rs`:

```rust
let mut vms = vec![
    VMConnection::new("computer 1", "so-lin", "192.168.1.1"),
    VMConnection::new("computer 2", "so-win", "192.168.1.2"),
    VMConnection::new("computer 3", "so-lin2", "192.168.1.3"),
];
```

## 📱 Uso

### Execução

```bash
./target/release/central_monitor
```

### Menu Interativo
Ao iniciar, o sistema apresenta um menu com as seguintes opções:

1. **Testar conexão com VM**
   - Verifica a conectividade SSH
   - Detecta automaticamente o sistema operacional

2. **Instalar/Atualizar agente em uma VM**
   - Copia o binário apropriado para a VM
   - Configura a inicialização automática
   - Inicia o agente imediatamente

3. **Listar status dos agentes**
   - Verifica se os agentes estão em execução
   - Exibe detalhes do processo

4. **Ver logs de um agente**
   - Exibe os logs mais recentes
   - Opções para filtrar e analisar

5. **Reiniciar um agente**
   - Para e inicia o serviço do agente
   - Verifica se o reinício foi bem-sucedido

6. **Parar um agente**
   - Interrompe a execução do agente
   - Mantém a configuração intacta

7. **Remover agente**
   - Remove o agente e suas configurações
   - Opção para manter ou apagar logs

8. **Apagar logs de um agente**
   - Limpa os arquivos de log
   - Mantém o agente em execução

9. **Sair**
   - Encerra o programa

### Configuração de Execução Automática (opcional)

Para configurar o central_monitor para execução automática no login:

**Systemd (Linux):**
```bash
mkdir -p ~/.config/systemd/user/
cat > ~/.config/systemd/user/central-monitor.service << EOL
[Unit]
Description=Central Monitor Service
After=network.target

[Service]
Type=simple
ExecStart=/caminho/para/central_monitor
Restart=on-failure

[Install]
WantedBy=default.target
EOL

systemctl --user enable central-monitor.service
systemctl --user start central-monitor.service
```

## 🔧 Troubleshooting

### Problemas Comuns

#### Falha na Conexão SSH
- Verifique se o serviço SSH está ativo na VM alvo
- Confirme se as chaves SSH estão corretamente configuradas
- Teste manualmente com `ssh nome-config`

#### Agente não Inicia no Windows
- Verifique permissões de administrador
- Confirme que o binário foi compilado com configuração GUI
- Verifique logs do Windows em `C:\Users\so\.snapshot_agent\`

#### Agente não Inicia no Linux
- Verifique logs do systemd: `systemctl --user status snapshot-agent.service`
- Confirme permissões de execução: `ls -la ~/snapshot_agent`
- Teste execução manual: `~/snapshot_agent`

## 💻 Desenvolvimento

### Estrutura do Projeto
```
central_monitor/
├── src/
│   ├── main.rs       # Lógica principal e gerenciamento de VMs
│   └── cli/          # Interface de linha de comando
│       ├── mod.rs    # Exportação do módulo
│       └── menu.rs   # Implementação do menu interativo
├── Cargo.toml        # Dependências e metadados
└── README.md         # Documentação
```

### Extensão e Personalização

Para adicionar suporte a novos sistemas operacionais:
1. Expanda o enum `OperatingSystem` em `main.rs`
2. Adicione lógica de detecção no método `detect_os()`
3. Implemente deploy específico em `deploy_snapshot_agent()`

## 🤝 Contribuição

Contribuições são bem-vindas! Para contribuir:

1. Fork o repositório
2. Crie uma branch para sua feature (`git checkout -b feature/nova-funcionalidade`)
3. Implemente suas mudanças
4. Execute os testes e lints (`cargo test && cargo clippy`)
5. Commit suas alterações (`git commit -m 'Adiciona nova funcionalidade'`)
6. Envie para a branch (`git push origin feature/nova-funcionalidade`)
7. Abra um Pull Request

## 📄 Licença

Este projeto está licenciado sob a licença MIT. Veja o arquivo LICENSE para mais detalhes.

---

**Desenvolvido pelo grupo de Sistemas Operacionais - 2025**
