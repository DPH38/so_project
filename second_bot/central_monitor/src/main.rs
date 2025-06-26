// ------------------------------------------------------------------------------
// CENTRAL MONITOR - MAIN MODULE
// ------------------------------------------------------------------------------
// Este é o módulo principal do Central Monitor, responsável por:
// - Inicializar conexões com máquinas virtuais (VMs) via SSH
// - Detectar automaticamente o sistema operacional remoto
// - Gerenciar o deploy e monitoramento de agentes de snapshot
// - Expor funcionalidades via interface de linha de comando (CLI)
// ------------------------------------------------------------------------------

use anyhow::{Context, Result};
use colored::*;

use std::process::Command;
use std::str;

// Importação do módulo CLI, que contém a interface de linha de comando
mod cli;

/// Enumeração que representa os sistemas operacionais suportados
/// Os valores armazenam a versão detalhada do sistema operacional como String
pub enum OperatingSystem {
    Linux(String),   // String contém a versão detalhada do Linux
    Windows(String), // String contém a versão detalhada do Windows
    Unknown,         // Sistema operacional não identificado
}

/// Estrutura que representa uma conexão com uma máquina virtual (VM)
/// Mantém todas as informações necessárias para conexão SSH e metadados da VM
struct VMConnection {
    name: String,            // Nome amigável da VM para exibição ao usuário
    ssh_config: String,      // Nome da configuração SSH no arquivo ~/.ssh/config
    ip: String,              // Endereço IP (será substituído pelo hostname eventualmente)
    hostname: Option<String>, // Hostname real obtido da configuração SSH
    os: Option<OperatingSystem>, // Sistema operacional detectado na VM
}

impl VMConnection {
    /// Cria uma nova instância de VMConnection com valores iniciais
    ///
    /// # Argumentos
    /// * `name` - Nome amigável para a VM
    /// * `ssh_config` - Nome da entrada no arquivo SSH config
    /// * `ip` - Endereço IP da VM (usado como fallback se hostname não for obtido)
    fn new(name: &str, ssh_config: &str, ip: &str) -> Self {
        Self {
            name: name.to_string(),
            ssh_config: ssh_config.to_string(),
            ip: ip.to_string(),
            hostname: None,
            os: None,
        }
    }

    /// Lê o hostname real da configuração SSH
    /// 
    /// Usa o comando `ssh -G` para obter a configuração efetiva do host SSH
    /// e extrai o hostname real da configuração.
    /// 
    /// # Retorna
    /// * `Result<String>` - O hostname obtido ou o IP como fallback em caso de sucesso
    fn get_ssh_hostname(&mut self) -> Result<String> {
        // Usa ssh -G para imprimir a configuração efetiva para o host
        let output = Command::new("ssh")
            .args(["-G", &self.ssh_config])
            .output()
            .context(format!("Failed to read SSH config for {}", self.name))?;

        let config = String::from_utf8_lossy(&output.stdout);

        // Procura pela linha que começa com "hostname "
        for line in config.lines() {
            if line.starts_with("hostname ") {
                let hostname = line
                    .split_whitespace()
                    .nth(1)
                    .unwrap_or(&self.ip)
                    .to_string();
                self.hostname = Some(hostname.clone());
                return Ok(hostname);
            }
        }

        // Se não encontrado, retorna o IP atual como fallback
        Ok(self.ip.clone())
    }

    /// Testa a conexão SSH com a VM e detecta o sistema operacional
    /// 
    /// Este método tenta estabelecer uma conexão SSH com a VM e,
    /// se bem-sucedido, detecta automaticamente o sistema operacional.
    /// 
    /// # Retorna
    /// * `Result<bool>` - true se a conexão foi estabelecida com sucesso
    fn test_connection(&mut self) -> Result<bool> {
        println!("🔄 Testing connection to {}...", self.name.cyan());

        // Obtém o hostname real antes de testar
        if self.hostname.is_none() {
            let _ = self.get_ssh_hostname();
        }

        // Usa ssh -T para testar a conexão
        let output = Command::new("ssh")
            .args(["-T", &self.ssh_config])
            .output()
            .context(format!("Failed to execute SSH command for {}", self.name))?;

        let success = output.status.success();
        if success {
            // Se a conexão teve sucesso, tenta detectar o SO
            self.detect_os()?;
            let os_description = match &self.os {
                Some(OperatingSystem::Linux(version)) => format!("Linux ({})", version),
                Some(OperatingSystem::Windows(version)) => format!("Windows ({})", version),
                Some(OperatingSystem::Unknown) => "Unknown OS".to_string(),
                None => "OS not detected".to_string(),
            };
            println!(
                "✅ Successfully connected to {} - Running {}",
                self.name.green(),
                os_description.cyan()
            );
        } else {
            println!("❌ Failed to connect to {}", self.name.red());
            if let Ok(stderr) = String::from_utf8(output.stderr) {
                if !stderr.trim().is_empty() {
                    println!("Error: {}", stderr.red());
                }
            }
        }

        Ok(success)
    }

    /// Detecta o sistema operacional da VM tentando executar comandos específicos
    /// 
    /// Este método tenta primeiro identificar Linux com 'uname -a',
    /// depois tenta Windows com 'ver'. Se ambos falharem, marca como Unknown.
    /// 
    /// # Retorna
    /// * `Result<()>` - Sucesso ou erro durante a detecção
    fn detect_os(&mut self) -> Result<()> {
        // Tenta primeiro o comando Linux
        let linux_check = Command::new("ssh")
            .args([&self.ssh_config, "uname -a"])
            .output()
            .context("Failed to execute uname command")?;

        if linux_check.status.success() {
            if let Ok(output) = String::from_utf8(linux_check.stdout) {
                self.os = Some(OperatingSystem::Linux(output.trim().to_string()));
                return Ok(());
            }
        }

        // Se a verificação do Linux falhou, tenta Windows
        let windows_check = Command::new("ssh")
            .args([&self.ssh_config, "ver"])
            .output()
            .context("Failed to execute ver command")?;

        if windows_check.status.success() {
            if let Ok(output) = String::from_utf8(windows_check.stdout) {
                self.os = Some(OperatingSystem::Windows(output.trim().to_string()));
                return Ok(());
            }
        }

        // Se ambas as verificações falharem, marca como desconhecido
        self.os = Some(OperatingSystem::Unknown);
        Ok(())
    }

    /// Implanta o agente de snapshot na VM remota
    /// 
    /// Copia o binário apropriado para a VM e configura para execução automática
    /// de forma diferente dependendo do sistema operacional (Linux ou Windows).
    /// 
    /// # Retorna
    /// * `Result<()>` - Sucesso ou erro durante o deploy
    fn deploy_snapshot_agent(&self) -> Result<()> {
        println!("🚀 Deploying snapshot agent to {}...", self.name.cyan());

        match &self.os {
            Some(OperatingSystem::Linux(_)) => {
                // LINUX: Deploy do agente para Linux usando systemd user service
                // 1. Copia o binário Linux para a VM
                let status = Command::new("scp")
                    .args([
                        "/home/drp/my/so/bots/second_bot/target/release/snapshot_agent_linux",
                        &format!("{}:~/snapshot_agent", self.ssh_config),
                    ])
                    .status()
                    .context("Failed to copy Linux agent")?;

                if !status.success() {
                    return Err(anyhow::anyhow!("Failed to copy Linux agent"));
                }

                // 2. Define o conteúdo do arquivo de serviço systemd
                let service_content = format!(
                    "[Unit]\n\
                Description=Snapshot Agent Service\n\
                \n\
                [Service]\n\
                Type=simple\n\
                ExecStart=/home/%u/snapshot_agent\n\
                ExecStop=/bin/kill -TERM $MAINPID\n\
                KillMode=process\n\
                Restart=always\n\
                WorkingDirectory=/home/%u\n\
                \n\
                [Install]\n\
                WantedBy=default.target"
                );

                // 3. Configura e inicia o serviço systemd do usuário
                Command::new("ssh")
                    .args([
                        &self.ssh_config,
                        &format!(
                            "mkdir -p ~/.config/systemd/user ~/.snapshot_agent && \
                            echo '{}' > ~/.config/systemd/user/snapshot-agent.service && \
                            chmod +x ~/snapshot_agent && \
                            systemctl --user daemon-reload && \
                            systemctl --user enable snapshot-agent.service && \
                            systemctl --user start snapshot-agent.service && \
                            sleep 2 && \
                            systemctl --user status snapshot-agent.service",
                            service_content
                        ),
                    ])
                    .status()
                    .context("Failed to set up Linux autostart")?;

                println!(
                    "✅ Successfully deployed and started Linux agent to {}",
                    self.name.green()
                );
            }

            Some(OperatingSystem::Windows(_)) => {
                // WINDOWS: Deploy do agente para Windows usando Scheduled Tasks
                // 1. Copia o executável para uma localização pública na VM
                let temp_dest = format!("{}:C:/Users/Public/snapshot_agent.exe", self.ssh_config);
                let status = Command::new("scp")
        .args([
            "/home/drp/my/so/bots/second_bot/target/x86_64-pc-windows-gnu/release/snapshot_agent_windows.exe",
            &temp_dest,
        ])
        .status()
        .context("Failed to copy Windows agent to public location")?;

                if !status.success() {
                    return Err(anyhow::anyhow!(
                        "Failed to copy Windows agent to public location"
                    ));
                }

                // 2. Garante que o diretório de logs exista
                Command::new("ssh")
        .args([
            &self.ssh_config,
            "powershell -Command \"New-Item -ItemType Directory -Path C:\\Users\\so\\.snapshot_agent -Force | Out-Null\"",
        ])
        .status()
        .context("Failed to create log directory in Windows VM")?;

                // 3. Cria a tarefa agendada com schtasks
                Command::new("ssh")
        .args([
            &self.ssh_config,
            "schtasks /Create /TN SnapshotAgent /TR \"C:\\Users\\Public\\snapshot_agent.exe\" /SC ONCE /ST 00:00 /RL HIGHEST /F"
        ])
        .status()
        .context("Failed to create scheduled task for agent")?;

                // 4. Executa a tarefa
                Command::new("ssh")
                    .args([&self.ssh_config, "schtasks /Run /TN SnapshotAgent"])
                    .status()
                    .context("Failed to run scheduled task for agent")?;

                // Aguarda 2 segundos para garantir o startup
                std::thread::sleep(std::time::Duration::from_secs(2));

                // 5. Verifica se o processo está ativo
                let check = Command::new("ssh")
                    .args([
                        &self.ssh_config,
                        "wmic process where \"Name='snapshot_agent.exe'\" get ProcessId",
                    ])
                    .output()
                    .context("Failed to check agent process via WMIC")?;

                if String::from_utf8_lossy(&check.stdout).contains("ProcessId") {
                    println!("🟢 Windows agent running on VM {}", self.name.green());
                } else {
                    println!(
                        "🟡 Windows agent NOT detected as active process on VM {}",
                        self.name.yellow()
                    );
                }

                println!(
                    "✅ Successfully deployed and started Windows agent to {} (via Scheduled Task)",
                    self.name.green()
                );
            }

            _ => {
                // Sistema operacional não detectado
                println!("❌ Operating system not detected for {}", self.name.red());
                println!(
                    "ℹ️  Please run the {} option first to detect the operating system.",
                    "'Test connection with VM'".green().bold()
                );
                return Err(anyhow::anyhow!("Operating system not detected"));
            }
        }

        Ok(())
    }

    /// Retorna o hostname atual da VM ou o IP como fallback
    /// 
    /// # Retorna
    /// * `&str` - Uma referência ao hostname ou IP
    fn get_current_hostname(&self) -> &str {
        self.hostname.as_deref().unwrap_or(&self.ip)
    }

    /// Verifica se é possível conectar à VM via SSH
    /// 
    /// # Retorna
    /// * `bool` - true se a conexão foi bem-sucedida
    fn is_connected(&self) -> bool {
        let output = Command::new("ssh").args(["-T", &self.ssh_config]).output();

        match output {
            Ok(result) => result.status.success(),
            Err(_) => false,
        }
    }

    /// Para o agente de snapshot em uma VM Linux
    /// 
    /// # Retorna
    /// * `Result<()>` - Sucesso ou erro durante a operação
    fn stop_linux_agent(&self) -> Result<()> {
        println!(
            "🛑 Stopping Linux agent on {}...",
            self.get_current_hostname().cyan()
        );

        // Para o serviço usando systemctl
        Command::new("ssh")
            .args([
                &self.ssh_config,
                "systemctl --user stop snapshot-agent.service",
            ])
            .status()
            .context("Failed to stop Linux agent service")?;

        // Verifica se o processo ainda está em execução
        let check = Command::new("ssh")
            .args([&self.ssh_config, "pgrep -af snapshot_agent"])
            .output()
            .context("Failed to check Linux agent process")?;

        if check.status.success() {
            println!(
                "⚠️ Process still detected on {}, details:",
                self.get_current_hostname().yellow()
            );
            if let Ok(output) = String::from_utf8(check.stdout) {
                println!("{}", output);
            }
            println!("ℹ️ The process might take a few seconds to fully stop.");
        } else {
            println!(
                "✅ Linux agent stopped successfully on {}",
                self.get_current_hostname().green()
            );
        }

        Ok(())
    }

    /// Verifica o status do agente Linux
    /// 
    /// # Retorna
    /// * `Result<()>` - Sucesso ou erro durante a verificação
    fn check_linux_agent_status(&self) -> Result<()> {
        println!(
            "🔍 Checking Linux agent status on {}...",
            self.get_current_hostname().cyan()
        );

        let check = Command::new("ssh")
            .args([&self.ssh_config, "pgrep -af snapshot_agent"])
            .output()
            .context("Failed to check Linux agent process")?;

        if check.status.success() {
            println!(
                "🟢 Linux agent is running on {}",
                self.get_current_hostname().green()
            );
            if let Ok(output) = String::from_utf8(check.stdout) {
                println!("Process details:\n{}", output);
            }
        } else {
            println!(
                "🔴 Linux agent is not running on {}",
                self.get_current_hostname().red()
            );
        }

        Ok(())
    }

    /// Reinicia o agente Linux parando e iniciando o serviço
    /// 
    /// # Retorna
    /// * `Result<()>` - Sucesso ou erro durante o reinício
    fn restart_linux_agent(&self) -> Result<()> {
        println!(
            "🔄 Restarting Linux agent on {}...",
            self.get_current_hostname().cyan()
        );

        // Para o serviço
        Command::new("ssh")
            .args([
                &self.ssh_config,
                "systemctl --user stop snapshot-agent.service",
            ])
            .status()
            .context("Failed to stop Linux agent service")?;

        // Pausa curta para garantir que o processo foi encerrado
        std::thread::sleep(std::time::Duration::from_secs(2));

        // Inicia o serviço
        Command::new("ssh")
            .args([
                &self.ssh_config,
                "systemctl --user start snapshot-agent.service",
            ])
            .status()
            .context("Failed to start Linux agent service")?;

        // Verifica se o processo está em execução
        let check = Command::new("ssh")
            .args([&self.ssh_config, "pgrep -af snapshot_agent"])
            .output()
            .context("Failed to check Linux agent process")?;

        if check.status.success() {
            println!(
                "✅ Linux agent restarted successfully on {}",
                self.get_current_hostname().green()
            );
            if let Ok(output) = String::from_utf8(check.stdout) {
                println!("Process details:\n{}", output);
            }
        } else {
            println!(
                "❌ Failed to restart Linux agent on {}",
                self.get_current_hostname().red()
            );
        }

        Ok(())
    }
}

/// Função principal que inicializa as conexões e inicia o menu interativo
/// Usa tokio para suporte assíncrono, embora a maioria das operações sejam bloqueantes
#[tokio::main]
async fn main() -> Result<()> {
    println!("{}", "🤖 VM Connection Bot Starting...".bright_blue());
    println!("{}", "==============================".bright_blue());
    println!("\n🔄 Initializing SSH connections...");

    // Define as VMs a serem gerenciadas
    let mut vms = vec![
        VMConnection::new("computer 1", "so-lin", "192.168.1.1"),
        VMConnection::new("computer 2", "so-win", "192.168.1.2"),
        VMConnection::new("computer 3", "so-lin2", "192.168.1.3"),
    ];

    // Inicializa os hostnames reais das VMs
    for vm in &mut vms {
        match vm.get_ssh_hostname() {
            Ok(hostname) => {
                println!("✅ {} -> {}", vm.name.green(), hostname.cyan());
            }
            Err(e) => {
                println!(
                    "⚠️  {} -> Using fallback ({}): {}",
                    vm.name.yellow(),
                    vm.ip.yellow(),
                    e.to_string().red()
                );
            }
        }
    }
    println!(); // Linha em branco para melhor visibilidade

    // Inicia o menu interativo
    cli::run_menu(vms)?;

    Ok(())
}
