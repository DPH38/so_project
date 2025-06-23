use anyhow::{Context, Result};
use colored::*;
use std::process::Command;

mod cli;

enum OperatingSystem {
    Linux(String),   // String contains the detailed version
    Windows(String), // String contains the detailed version
    Unknown,
}

struct VMConnection {
    name: String,
    ssh_config: String,
    os: Option<OperatingSystem>,
}

impl VMConnection {
    fn new(name: &str, ssh_config: &str) -> Self {
        Self {
            name: name.to_string(),
            ssh_config: ssh_config.to_string(),
            os: None,
        }
    }

    fn test_connection(&mut self) -> Result<bool> {
        println!("🔄 Testing connection to {}...", self.name.cyan());

        // Use ssh -T to test the connection
        let output = Command::new("ssh")
            .args(["-T", &self.ssh_config])
            .output()
            .context(format!("Failed to execute SSH command for {}", self.name))?;

        let success = output.status.success();
        if success {
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

    fn detect_os(&mut self) -> Result<()> {
        // Try Linux command first
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

        // If Linux check failed, try Windows
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

        self.os = Some(OperatingSystem::Unknown);
        Ok(())
    }

    fn deploy_snapshot_agent(&self) -> Result<()> {
        println!("🚀 Deploying snapshot agent to {}...", self.name.cyan());

        match &self.os {
            Some(OperatingSystem::Linux(_)) => {
                // LINUX (mantido igual)
                let status = Command::new("scp")
                    .args([
                        "/home/drp/my/so/boots/second_bot/target/release/snapshot_agent_linux",
                        &format!("{}:~/snapshot_agent", self.ssh_config),
                    ])
                    .status()
                    .context("Failed to copy Linux agent")?;

                if !status.success() {
                    return Err(anyhow::anyhow!("Failed to copy Linux agent"));
                }

                let service_content = format!(
                    "[Unit]\n\
                Description=Snapshot Agent Service\n\
                \n\
                [Service]\n\
                ExecStart=/home/%u/snapshot_agent\n\
                Restart=always\n\
                \n\
                [Install]\n\
                WantedBy=default.target"
                );

                Command::new("ssh")
                    .args([
                        &self.ssh_config,
                        &format!(
                            "mkdir -p ~/.config/systemd/user && \
                        echo '{}' > ~/.config/systemd/user/snapshot-agent.service && \
                        chmod +x ~/snapshot_agent && \
                        systemctl --user enable snapshot-agent.service && \
                        systemctl --user start snapshot-agent.service && \
                        sleep 2 && \
                        systemctl --user status snapshot-agent.service",
                            service_content
                        ),
                    ])
                    .status()
                    .context("Failed to set up Linux autostart")?;

                Command::new("ssh")
                    .args([
                        &self.ssh_config,
                        "nohup ~/snapshot_agent > ~/.snapshot_agent/snapshot.log 2>&1 &",
                    ])
                    .status()
                    .context("Failed to start Linux agent in background")?;

                println!(
                    "✅ Successfully deployed and started Linux agent to {}",
                    self.name.green()
                );
            }

            Some(OperatingSystem::Windows(_)) => {
                // Copia o executável para a VM
                let temp_dest = format!("{}:C:/Users/Public/snapshot_agent.exe", self.ssh_config);
                let status = Command::new("scp")
        .args([
            "/home/drp/my/so/boots/second_bot/target/x86_64-pc-windows-gnu/release/snapshot_agent_windows.exe",
            &temp_dest,
        ])
        .status()
        .context("Failed to copy Windows agent to public location")?;

                if !status.success() {
                    return Err(anyhow::anyhow!(
                        "Failed to copy Windows agent to public location"
                    ));
                }

                // Garante que o diretório de log existe
                Command::new("ssh")
        .args([
            &self.ssh_config,
            "powershell -Command \"New-Item -ItemType Directory -Path C:\\Users\\so\\.snapshot_agent -Force | Out-Null\"",
        ])
        .status()
        .context("Failed to create log directory in Windows VM")?;

                // Cria a tarefa agendada com schtasks
                Command::new("ssh")
        .args([
            &self.ssh_config,
            "schtasks /Create /TN SnapshotAgent /TR \"C:\\Users\\Public\\snapshot_agent.exe\" /SC ONCE /ST 00:00 /RL HIGHEST /F"
        ])
        .status()
        .context("Failed to create scheduled task for agent")?;

                // Executa a tarefa
                Command::new("ssh")
                    .args([&self.ssh_config, "schtasks /Run /TN SnapshotAgent"])
                    .status()
                    .context("Failed to run scheduled task for agent")?;

                // Aguarda 2 segundos para garantir startup
                std::thread::sleep(std::time::Duration::from_secs(2));

                // Verifica se o processo está ativo
                let check = Command::new("ssh")
                    .args([
                        &self.ssh_config,
                        "wmic process where \"Name='snapshot_agent.exe'\" get ProcessId",
                    ])
                    .output()
                    .context("Failed to check agent process via WMIC")?;

                if String::from_utf8_lossy(&check.stdout).contains("ProcessId") {
                    println!("🟢 Agente Windows rodando na VM {}", self.name.green());
                } else {
                    println!(
                        "🟡 Agente Windows NÃO detectado como processo ativo na VM {}",
                        self.name.yellow()
                    );
                }

                println!(
                    "✅ Successfully deployed and started Windows agent to {} (via Scheduled Task)",
                    self.name.green()
                );
            }

            _ => {
                println!("❌ Cannot deploy agent to {} - Unknown OS", self.name.red());
                return Err(anyhow::anyhow!("Unknown OS type"));
            }
        }

        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("{}", "🤖 VM Connection Bot Starting...".bright_blue());
    println!("{}", "==============================".bright_blue());

    let vms = vec![
        VMConnection::new("computer 1", "so-lin"),
        VMConnection::new("computer 2", "so-win"),
        VMConnection::new("computer 3", "so-lin2"),
    ];

    // Iniciar menu interativo
    cli::run_menu(vms)?;

    Ok(())
}
