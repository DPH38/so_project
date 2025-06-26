# Snapshot Agent

<div align="center">

![Versão](https://img.shields.io/badge/versão-1.0.0-blue.svg)
![Plataformas](https://img.shields.io/badge/plataformas-linux%20%7C%20windows-lightgrey)
![Paralelo](https://img.shields.io/badge/execução-paralela-orange)
![Licença](https://img.shields.io/badge/licença-MIT-green)

</div>

O `snapshot_agent` é um componente do projeto `second_bot` que captura dados de performance e estado do sistema operacional em tempo real, com implementações específicas e otimizadas para Linux e Windows. Usando processamento paralelo, coleta métricas críticas do sistema para monitoramento e diagnóstico.

## 📋 Índice
- [Visão Geral](#visão-geral)
- [Arquitetura](#arquitetura)
- [Funcionalidades](#funcionalidades)
- [Compilação](#compilação)
- [Configuração](#configuração)
- [Uso](#uso)
- [Performance](#performance)
- [Troubleshooting](#troubleshooting)
- [Desenvolvimento](#desenvolvimento)
- [Segurança](#segurança)
- [Contribuição](#contribuição)
- [Licença](#licença)

## 🔍 Visão Geral

O Snapshot Agent foi projetado para coletar métricas detalhadas de sistemas operacionais em tempo real utilizando APIs nativas de cada plataforma. Principais características:

- 🔄 **Multi-plataforma**: Suporte nativo para Linux e Windows
- ⚡ **Alta Performance**: Coleta paralela de dados usando threads
- 🔌 **Plugável**: Integração fácil com o `central_monitor` ou outros sistemas
- 📊 **Dados Ricos**: Métricas completas de CPU, memória, disco e rede
- 📝 **Logging**: Sistema de logs robusto para diagnóstico
- 🧩 **Modular**: Arquitetura que facilita extensões e personalizações

## 🏛️ Arquitetura

O sistema utiliza uma arquitetura modular com componentes específicos para cada sistema operacional:

### Componentes Principais

```
snapshot_agent/
├── src/
│   └── bin/
│       ├── linux.rs    # Implementação específica para Linux
│       └── windows.rs  # Implementação específica para Windows
```

### Fluxo de Dados
1. **Inicialização**: Configuração baseada em argumentos CLI ou arquivo de configuração
2. **Coleta Paralela**: Threads separadas para CPU, memória, disco e sistema de arquivos
3. **Processamento**: Normalização e formatação dos dados coletados
4. **Armazenamento**: Gravação em disco local e/ou envio ao servidor central
5. **Logging**: Registro de atividades, erros e métricas de performance

### Processamento Paralelo
<div align="center">
<pre>
┌──────────────┐     ┌───────────────────────────────┐
│              │     │  Thread Pool                  │
│  Thread      │     │ ┌─────────┐ ┌─────────┐       │
│  Principal   ├────►│ │CPU Info │ │Memory   │       │
│              │     │ └─────────┘ └─────────┘       │
│  - Coordena  │     │                               │
│  - Configura │     │ ┌─────────┐ ┌─────────┐       │
│  - Reporta   │     │ │Disk I/O │ │File List│       │
└──────────────┘     │ └─────────┘ └─────────┘       │
                     └───────────────────────────────┘
                                   │
                                   ▼
                     ┌───────────────────────────────┐
                     │        Snapshot Final         │
                     │   (Formato JSON ou Binário)   │
                     └───────────────────────────────┘
</pre>
</div>

## ✨ Funcionalidades

### Métricas Coletadas

#### Sistema & Hardware
- 💻 Informações detalhadas do sistema (kernel, versão, arquitetura)
- 🔌 Detecção de hardware (CPUs, núcleos, modelo)
- ⏱️ Tempo de atividade e estatísticas de carga

#### Recursos
- 🧠 **Memória**: Uso total, livre, compartilhada, cache, buffers
- 🔄 **CPU**: Utilização por núcleo, tempos de sistema/usuário/idle, load average
- 💾 **Disco**: Espaço utilizado/disponível, IOPS, throughput, tempos de resposta
- 📁 **Sistema de Arquivos**: Listagem de arquivos em diretórios monitorados
- 🌐 **Rede**: Conexões ativas, tráfego, latência (opcional)

### Recursos Adicionais
- ⏲️ Agendamento flexível (intervalos configuráveis)
- 📦 Suporte para compressão de dados
- 🔒 Obtenção segura de dados sensíveis
- 📈 Estatísticas de performance do próprio agente

## 🛠️ Compilação

### Pré-requisitos
- Rust 1.60 ou superior (`rustup update stable`)
- Cargo e ferramentas padrão de desenvolvimento Rust
- Para compilação cruzada Windows: toolchain específico e compilador MinGW

### Para Linux (nativo)
```bash
# Compilação padrão
cargo build --release --bin snapshot_agent_linux

# Compilação com otimizações específicas para a CPU
RUSTFLAGS="-C target-cpu=native" cargo build --release --bin snapshot_agent_linux
```

### Para Windows (cross-compilation em Linux)
```bash
# Instalar toolchain para Windows
rustup target add x86_64-pc-windows-gnu
sudo apt-get install gcc-mingw-w64-x86-64

# Compilar
cargo build --release --target x86_64-pc-windows-gnu --bin snapshot_agent_windows

# Verificar o executável gerado
file target/x86_64-pc-windows-gnu/release/snapshot_agent_windows.exe
```

### Para Windows (nativo em Windows)
```powershell
# Em PowerShell ou Prompt de Comando do Windows
cargo build --release --bin snapshot_agent_windows

# Para executar imediatamente após compilar
.\target\release\snapshot_agent_windows.exe
```

> **Nota Importante:** O agente Windows é compilado como binário GUI (sem console), permitindo execução silenciosa quando iniciado pelo agendador de tarefas ou central_monitor.

## ⚙️ Configuração

### Parâmetros de Linha de Comando

| Parâmetro | Descrição | Padrão | Exemplo |
|-----------|-----------|--------|---------|
| `--interval` | Intervalo entre snapshots (segundos) | 300 | `--interval 60` |
| `--output` | Diretório para salvar snapshots | `~/.snapshot_agent/data` | `--output /tmp/snapshots` |
| `--monitor` | Endereço do monitor central | N/A | `--monitor 192.168.1.100:9000` |
| `--log-level` | Nível de detalhamento dos logs | `info` | `--log-level debug` |
| `--config` | Arquivo de configuração alternativo | N/A | `--config /etc/snapshot-agent.conf` |

### Arquivo de Configuração (Opcional)

O agente pode ser configurado usando um arquivo JSON:

```json
{
  "interval": 120,
  "output": "/var/log/snapshots",
  "monitor": {
    "address": "10.0.0.5",
    "port": 9000,
    "secure": true
  },
  "collection": {
    "cpu": true,
    "memory": true,
    "disk": true,
    "network": false,
    "processes": true
  },
  "folders": [
    "/var/log",
    "/tmp/app"
  ]
}
```

## 📱 Uso

### Execução Básica

**Linux:**
```bash
# Execução com parâmetros padrão
./target/release/snapshot_agent_linux

# Especificar intervalo e diretório de saída
./target/release/snapshot_agent_linux --interval 60 --output /var/snapshots
```

**Windows:**
```powershell
# Execução com parâmetros padrão
.\snapshot_agent_windows.exe

# Especificar intervalo e diretório de saída
.\snapshot_agent_windows.exe --interval 120 --output C:\Snapshots
```

### Integração com o Central Monitor

```bash
# Conectar ao central_monitor
./target/release/snapshot_agent_linux --monitor 192.168.1.100:9000
```

### Instalação como Serviço

**Linux (systemd):**
```bash
cat > ~/.config/systemd/user/snapshot-agent.service << EOL
[Unit]
Description=Snapshot Agent Service
After=network.target

[Service]
Type=simple
ExecStart=/caminho/para/snapshot_agent_linux --interval 300
Restart=on-failure

[Install]
WantedBy=default.target
EOL

systemctl --user enable snapshot-agent.service
systemctl --user start snapshot-agent.service
```

**Windows (Task Scheduler):**
```powershell
# Criar tarefa agendada via PowerShell
$action = New-ScheduledTaskAction -Execute "C:\path\to\snapshot_agent_windows.exe"
$trigger = New-ScheduledTaskTrigger -AtStartup
Register-ScheduledTask -Action $action -Trigger $trigger -TaskName "SnapshotAgent" -Description "Sistema de monitoramento"
```

## 🚀 Performance

### Consumo de Recursos
- **CPU:** < 1% em idle, picos de 2-5% durante coleta
- **Memória:** ~15-30MB dependendo do sistema e configuração
- **Disco:** ~100KB por snapshot (sem compressão)

### Otimizações
- **Paralelismo:** Uso de threads para coleta simultânea
- **I/O Eficiente:** Buffer pools e operações assíncronas
- **Throttling:** Limites configuráveis de uso de CPU
- **Compressão:** Redução do tamanho dos snapshots (opcional)

### Benchmarks
| Operação | Tempo (Linux) | Tempo (Windows) |
|----------|---------------|-----------------|
| Inicialização | < 100ms | < 150ms |
| Coleta completa | 200-500ms | 300-700ms |
| Escrita em disco | 10-50ms | 20-80ms |
| Envio ao monitor | 50-150ms | 70-200ms |

## 🔧 Troubleshooting

### Logs
Os logs estão disponíveis em:
- **Linux:** `~/.snapshot_agent/snapshot.log`
- **Windows:** `C:\Users\<usuario>\.snapshot_agent\snapshot.log`

### Problemas Comuns

#### Agente não inicia
- Verifique permissões do diretório de saída
- Confirme que não há outra instância em execução
- Verifique logs para erros específicos

#### Falha na coleta de dados
- Verifique permissões (especialmente em `/proc` no Linux)
- Para Windows, confirme que o usuário tem direitos administrativos
- Aumente o log level para debug: `--log-level debug`

#### Uso alto de CPU/memória
- Aumente o intervalo entre coletas
- Desative coleta de métricas desnecessárias
- Verifique se há looping infinito nos logs

## 💻 Desenvolvimento

### Estrutura do Código
```
snapshot_agent/
├── src/
│   └── bin/
│       ├── linux.rs    # Implementação Linux
│       └── windows.rs  # Implementação Windows
├── Cargo.toml          # Dependências do projeto
└── README.md           # Este arquivo
```

### Ampliando o Agente

Para adicionar novas métricas:
1. Identifique a API ou arquivo de sistema para a nova métrica
2. Implemente a função de coleta no arquivo específico do SO
3. Adicione um novo canal (thread) para coleta paralela
4. Integre os dados ao formato final do snapshot

### Diretrizes de Contribuição
- Mantenha a coleta eficiente (evite bloqueios longos)
- Garanta tratamento adequado de erros
- Padronize o formato de saída entre plataformas
- Documente todas as novas métricas e parâmetros

## 🔒 Segurança

### Considerações
- O agente lida com informações potencialmente sensíveis do sistema
- A execução requer permissões elevadas em alguns sistemas
- Os canais de comunicação devem ser protegidos

### Recomendações
- Execute com privilégios mínimos necessários
- Criptografe comunicação com o monitor central
- Não armazene senhas ou tokens em logs
- Valide entrada de usuário em todos os parâmetros

## 🤝 Contribuição

Contribuições são bem-vindas! Para contribuir:

1. Fork o repositório
2. Crie uma branch para sua feature (`git checkout -b feature/nova-metrica`)
3. Implemente suas mudanças
4. Execute os testes (`cargo test`)
5. Verifique o estilo e warnings (`cargo clippy`)
6. Commit suas alterações (`git commit -m 'Adiciona nova métrica'`)
7. Envie para a branch (`git push origin feature/nova-metrica`)
8. Abra um Pull Request

## 📄 Licença

Este projeto está licenciado sob a licença MIT. Veja o arquivo LICENSE para mais detalhes.

---

**Desenvolvido pelo grupo de Sistemas Operacionais - 2025**
