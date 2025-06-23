# Projeto second_bot

O `second_bot` é uma solução completa para monitoramento e coleta de snapshots de múltiplas máquinas (Linux e Windows) em ambientes distribuídos, com foco em automação, centralização de logs e facilidade de operação.

## Visão Geral
O projeto é composto por dois principais componentes:

- **central_monitor**: Aplicação central que gerencia, monitora e interage com todos os agentes instalados nas máquinas remotas.
- **snapshot_agent**: Agente leve, com versões específicas para Linux e Windows, responsável por coletar informações do sistema e enviar/registrar snapshots periodicamente.

## Componentes

### 1. central_monitor
- Gerencia múltiplas VMs via SSH.
- Detecta o sistema operacional remoto automaticamente.
- Realiza deploy, parada, reinício e remoção dos agentes.
- Exibe status e logs dos agentes de cada VM.
- Interface de menu interativo no terminal.

Mais detalhes: [`central_monitor/README.md`](central_monitor/README.md)

### 2. snapshot_agent
- Coleta informações detalhadas do sistema (memória, CPU, disco, arquivos, etc).
- Implementação separada para Linux (`snapshot_agent_linux`) e Windows (`snapshot_agent_windows`).
- Gera logs locais e pode integrar com o monitor central.
- No Windows, roda como binário GUI (não abre terminal).

Mais detalhes: [`snapshot_agent/README.md`](snapshot_agent/README.md)

## Fluxo de Funcionamento
1. O usuário executa o `central_monitor` e seleciona as VMs a serem monitoradas.
2. O monitor central realiza deploy e gerenciamento dos agentes via SSH/SCP.
3. Os agentes coletam snapshots periodicamente e registram logs.
4. O monitor central pode consultar status, logs e controlar os agentes remotamente.

## Como compilar e rodar

### Compilar todos os componentes
```bash
# Compilar monitor central
cd central_monitor
cargo build --release --bin central_monitor

# Compilar agentes
cd ../snapshot_agent
cargo build --release --bin snapshot_agent_linux
cross build --release --target x86_64-pc-windows-gnu --bin snapshot_agent_windows
```

### Executar monitor central
```bash
cd ../central_monitor
./target/release/central_monitor
```

## Pré-requisitos
- Rust 1.60 ou superior
- Ferramenta [cross](https://github.com/cross-rs/cross) para compilar para Windows em Linux
- Acesso SSH configurado para as VMs

## Estrutura do repositório
- `central_monitor/` — Monitor central e menu interativo
- `snapshot_agent/` — Agentes para Linux e Windows

## Observações
- Consulte os READMEs de cada componente para detalhes de uso e configuração.
- O projeto é modular e pode ser expandido para outros sistemas operacionais ou integrações.

---

Para dúvidas ou contribuições, consulte os comentários no código-fonte ou abra uma issue no repositório.
