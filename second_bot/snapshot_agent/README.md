# snapshot_agent

O `snapshot_agent` é um componente do projeto `second_bot` responsável por capturar snapshots do sistema operacional, com implementações específicas para Linux e Windows. Cada sistema operacional gera um executável distinto, aproveitando as particularidades de cada ambiente.

## O que faz

- Coleta informações detalhadas do sistema, como processos, uso de memória, disco, etc.
- Implementação específica para cada SO:
  - **Linux**: utiliza o arquivo `linux.rs` para acessar informações do sistema via `/proc`, comandos nativos e APIs do Linux.
  - **Windows**: utiliza o arquivo `windows.rs` para acessar informações via APIs do Windows.
- Gera arquivos de snapshot para análise posterior.
- Pode enviar os snapshots para o `central_monitor` via socket, arquivo ou outro mecanismo definido.

## Como funciona

- O código detecta o sistema operacional em tempo de compilação e utiliza o módulo correspondente (`linux.rs` ou `windows.rs`).
- Cada executável é otimizado para o SO alvo, garantindo coleta eficiente e precisa dos dados.
- O formato dos dados pode ser JSON, texto ou binário, conforme implementado.

## Exemplos de uso

### Execução padrão (Linux)
Após compilar:
```bash
./target/release/snapshot_agent_linux
```

### Execução padrão (Windows)
Após compilar para Windows:
```powershell
snapshot_agent_windows.exe
```

### Parâmetros de linha de comando (exemplo Linux)
```bash
./target/release/snapshot_agent_linux --interval 60 --output /tmp/snapshots/
```
- `--interval`: intervalo em segundos entre snapshots.
- `--output`: diretório onde os snapshots serão salvos.

### Integração com o monitor central
```bash
./target/release/snapshot_agent_linux --monitor 127.0.0.1:9000
```
- `--monitor`: endereço IP e porta do monitor central.

## Como compilar

### Para Linux (nativo)
```bash
cargo build --release --bin snapshot_agent_linux
```
O executável será gerado em `target/release/snapshot_agent_linux`.

### Para Windows (em máquina Linux)
É necessário instalar o [cross](https://github.com/cross-rs/cross):
```bash
cargo install cross
cross build --release --target x86_64-pc-windows-gnu --bin snapshot_agent_windows
```
O executável será gerado em `target/x86_64-pc-windows-gnu/release/snapshot_agent_windows.exe`.

> **Nota:** O agente Windows é compilado como binário GUI (não abre terminal ao ser iniciado por agendador ou duplo clique).

## Requisitos
- Rust 1.60 ou superior
- Sistema operacional Linux ou Windows
- Para compilar para Windows em Linux: ferramenta `cross` e toolchain apropriado

## Observações
- Certifique-se de que o `central_monitor` esteja em execução se for utilizar integração.
- Consulte o código-fonte dos arquivos `linux.rs` e `windows.rs` para detalhes das informações coletadas em cada SO.
- Os logs e mensagens de erro são gravados em arquivo na pasta do usuário (`~/.snapshot_agent/snapshot.log`).
- No Windows, prints no terminal não aparecerão se o binário for executado como GUI.
