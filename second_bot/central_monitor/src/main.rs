use anyhow::{Context, Result};
use colored::*;

use std::process::Command;
use std::str;

mod cli;

pub enum OperatingSystem {
    Linux(String),   // String contains the detailed version
    Windows(String), // String contains the detailed version
    Unknown,
}

struct VMConnection {
    name: String,
    ssh_config: String,
    ip: String,               // This will be replaced with the hostname eventually
    hostname: Option<String>, // Real hostname from SSH config
    os: Option<OperatingSystem>,
}

impl VMConnection {
    fn new(name: &str, ssh_config: &str, ip: &str) -> Self {
        Self {
            name: name.to_string(),
            ssh_config: ssh_config.to_string(),
            ip: ip.to_string(),
            hostname: None,
            os: None,
        }
    }

    // Function to read HostName from the SSH config file
    fn get_ssh_hostname(&mut self) -> Result<String> {
        // Use ssh -G to print the effective configuration for the host
        let output = Command::new("ssh")
            .args(["-G", &self.ssh_config])
            .output()
            .context(format!("Failed to read SSH config for {}", self.name))?;

        let config = String::from_utf8_lossy(&output.stdout);

        // Look for the line that starts with "hostname "
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

        // If not found, return the current IP as a fallback
        Ok(self.ip.clone())
    }

    fn test_connection(&mut self) -> Result<bool> {
        println!("🔄 Testing connection to {}...", self.name.cyan());

        // Get real hostname before testing
        if self.hostname.is_none() {
            let _ = self.get_ssh_hostname();
        }

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
                // LINUX (kept the same)
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

                // Configurar serviço e iniciar
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
                // Copy the executable to the VM
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

                // Ensure the log directory exists
                Command::new("ssh")
        .args([
            &self.ssh_config,
            "powershell -Command \"New-Item -ItemType Directory -Path C:\\Users\\so\\.snapshot_agent -Force | Out-Null\"",
        ])
        .status()
        .context("Failed to create log directory in Windows VM")?;

                // Create the scheduled task with schtasks
                Command::new("ssh")
        .args([
            &self.ssh_config,
            "schtasks /Create /TN SnapshotAgent /TR \"C:\\Users\\Public\\snapshot_agent.exe\" /SC ONCE /ST 00:00 /RL HIGHEST /F"
        ])
        .status()
        .context("Failed to create scheduled task for agent")?;

                // Run the task
                Command::new("ssh")
                    .args([&self.ssh_config, "schtasks /Run /TN SnapshotAgent"])
                    .status()
                    .context("Failed to run scheduled task for agent")?;

                // Wait 2 seconds to ensure startup
                std::thread::sleep(std::time::Duration::from_secs(2));

                // Check if the process is active
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

    fn get_current_hostname(&self) -> &str {
        self.hostname.as_deref().unwrap_or(&self.ip)
    }

    fn is_connected(&self) -> bool {
        let output = Command::new("ssh").args(["-T", &self.ssh_config]).output();

        match output {
            Ok(result) => result.status.success(),
            Err(_) => false,
        }
    }

    fn stop_linux_agent(&self) -> Result<()> {
        println!(
            "🛑 Stopping Linux agent on {}...",
            self.get_current_hostname().cyan()
        );

        // Stop the service using systemctl
        Command::new("ssh")
            .args([
                &self.ssh_config,
                "systemctl --user stop snapshot-agent.service",
            ])
            .status()
            .context("Failed to stop Linux agent service")?;

        // Check if the process is still running
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

    fn restart_linux_agent(&self) -> Result<()> {
        println!(
            "🔄 Restarting Linux agent on {}...",
            self.get_current_hostname().cyan()
        );

        // Stop the service
        Command::new("ssh")
            .args([
                &self.ssh_config,
                "systemctl --user stop snapshot-agent.service",
            ])
            .status()
            .context("Failed to stop Linux agent service")?;

        // Short pause to ensure process has stopped
        std::thread::sleep(std::time::Duration::from_secs(2));

        // Start the service
        Command::new("ssh")
            .args([
                &self.ssh_config,
                "systemctl --user start snapshot-agent.service",
            ])
            .status()
            .context("Failed to start Linux agent service")?;

        // Check if the process is running
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

#[tokio::main]
async fn main() -> Result<()> {
    println!("{}", "🤖 VM Connection Bot Starting...".bright_blue());
    println!("{}", "==============================".bright_blue());
    println!("\n🔄 Initializing SSH connections...");

    let mut vms = vec![
        VMConnection::new("computer 1", "so-lin", "192.168.1.1"),
        VMConnection::new("computer 2", "so-win", "192.168.1.2"),
        VMConnection::new("computer 3", "so-lin2", "192.168.1.3"),
    ];

    // Initialize the real hostnames of the VMs
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
    println!(); // Blank line for better visibility

    // Start interactive menu
    cli::run_menu(vms)?;

    Ok(())
}
