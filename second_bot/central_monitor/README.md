# central_monitor

O `central_monitor` é o componente central do projeto responsável por monitorar, gerenciar e interagir com múltiplos agentes de snapshot (Linux e Windows) em diferentes máquinas virtuais ou físicas.

## Funcionalidades
- Testa a conexão SSH com múltiplas VMs.
- Detecta automaticamente o sistema operacional remoto (Linux ou Windows).
- Realiza deploy, parada, remoção e reinício do agente de snapshot nas VMs.
- Exibe status e logs dos agentes de cada VM.
- Interface de menu interativo no terminal para facilitar a operação.

## Como funciona
- O monitor central utiliza SSH para se conectar às VMs e executar comandos remotos.
- O deploy do agente é feito via `scp` (Linux e Windows) e o serviço é iniciado automaticamente.
- No Windows, o agente é iniciado via tarefa agendada (`schtasks`) e roda como binário GUI (sem abrir terminal).
- O status e logs dos agentes podem ser consultados diretamente pelo menu.

## Como usar

### Compilação

```bash
cargo build --release --bin central_monitor
```

### Execução

```bash
./target/release/central_monitor
```

### Menu interativo
Ao iniciar, o sistema apresenta um menu com opções para:
- Testar conexão com as VMs
- Instalar/atualizar agente
- Listar status dos agentes
- Ver logs
- Reiniciar, parar ou remover agentes

Basta selecionar a VM e a ação desejada.

## Pré-requisitos
- Rust 1.60 ou superior
- Acesso SSH configurado para as VMs (Linux e Windows)
- Permissões para executar comandos remotos e copiar arquivos via SSH/SCP
- Para Windows: agente deve ser compilado como GUI e permissões para criar tarefas agendadas

## Estrutura do projeto
- `src/main.rs`: lógica principal, conexão e deploy dos agentes
- `src/cli/`: implementação do menu interativo e comandos auxiliares

## Observações
- Certifique-se de que as VMs estejam acessíveis via SSH.
- O agente Windows não exibirá terminal ao ser iniciado pelo agendador.
- Consulte os logs das VMs para diagnóstico detalhado.

---

Para mais detalhes, consulte o código-fonte e os comentários nos arquivos principais.
