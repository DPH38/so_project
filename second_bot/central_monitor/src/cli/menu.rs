use crate::VMConnection;
use anyhow::{Context, Result};
use colored::*;
use dialoguer::{theme::ColorfulTheme, Select};
use std::process::Command;

pub struct Menu {
    vms: Vec<VMConnection>,
}

impl Menu {
    pub fn new(vms: Vec<VMConnection>) -> Self {
        Self { vms }
    }

    fn test_vm_connection(&mut self, vm_idx: usize) -> Result<()> {
        let vm = &mut self.vms[vm_idx];
        match vm.test_connection() {
            Ok(true) => println!("✅ Conectado com sucesso a {}", vm.name.green()),
            Ok(false) => println!("❌ Falha ao conectar com {}", vm.name.red()),
            Err(e) => println!("❌ Erro ao testar {}: {}", vm.name.yellow(), e),
        }
        Ok(())
    }

    pub fn run(&mut self) -> Result<()> {
        loop {
            match self.show_main_menu()? {
                0 => {
                    // Testar conexão
                    let vm_idx = self.select_vm("Selecione a VM para testar conexão")?;
                    if let Err(e) = self.test_vm_connection(vm_idx) {
                        println!("Erro: {}", e);
                    }
                }
                1 => {
                    // Instalar/Atualizar
                    let vm_idx =
                        self.select_vm("Selecione a VM para instalar/atualizar o agente")?;
                    let vm = &self.vms[vm_idx];
                    println!(
                        "\n🚀 Instalando/Atualizando agente em {}...",
                        vm.name.cyan()
                    );
                    if let Err(e) = vm.deploy_snapshot_agent() {
                        println!("❌ Erro ao instalar/atualizar agente: {}", e);
                    } else {
                        println!("✅ Agente instalado/atualizado com sucesso!");
                    }
                }
                2 => {
                    // Listar status
                    let vm_idx = self.select_vm("Selecione a VM para verificar o status")?;
                    if let Err(e) = self.show_agent_status(vm_idx) {
                        println!("Erro: {}", e);
                    }
                }
                3 => {
                    // Ver logs
                    let vm_idx = self.select_vm("Selecione a VM para ver os logs")?;
                    if let Err(e) = self.show_agent_logs(vm_idx) {
                        println!("Erro: {}", e);
                    }
                }
                4 => {
                    // Reiniciar
                    let vm_idx = self.select_vm("Selecione a VM para reiniciar o agente")?;
                    if let Err(e) = self.restart_agent(vm_idx) {
                        println!("Erro: {}", e);
                    }
                }
                5 => {
                    // Parar
                    let vm_idx = self.select_vm("Selecione a VM para parar o agente")?;
                    if let Err(e) = self.stop_agent(vm_idx) {
                        println!("Erro: {}", e);
                    }
                }
                6 => {
                    // Remover
                    let vm_idx = self.select_vm("Selecione a VM para remover o agente")?;
                    if let Err(e) = self.remove_agent(vm_idx) {
                        println!("Erro: {}", e);
                    }
                }
                7 => break, // Sair
                _ => unreachable!(),
            }

            println!("\nPressione Enter para continuar...");
            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
        }

        Ok(())
    }

    fn show_main_menu(&self) -> Result<usize> {
        let options = vec![
            "Testar conexão com VM",
            "Instalar/Atualizar agente em uma VM",
            "Listar status dos agentes",
            "Ver logs de um agente",
            "Reiniciar um agente",
            "Parar um agente",
            "Remover agente",
            "Sair",
        ];

        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Selecione uma opção")
            .items(&options)
            .default(0)
            .interact()?;

        Ok(selection)
    }

    fn select_vm(&self, prompt: &str) -> Result<usize> {
        let vm_names: Vec<_> = self.vms.iter().map(|vm| vm.name.as_str()).collect();

        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt(prompt)
            .items(&vm_names)
            .default(0)
            .interact()?;

        Ok(selection)
    }

    fn show_agent_status(&self, vm_idx: usize) -> Result<()> {
        let vm = &self.vms[vm_idx];

        println!("\n🔍 Verificando status do agente em {}...", vm.name.cyan());

        match &vm.os {
            Some(crate::OperatingSystem::Linux(_)) => {
                // Usa systemctl para status real do serviço e PID
                let output = Command::new("ssh")
                    .args([
                        &vm.ssh_config,
                        "systemctl --user is-active snapshot-agent.service && systemctl --user show -p MainPID snapshot-agent.service && systemctl --user status snapshot-agent.service | head -20"
                    ])
                    .output()
                    .context("Falha ao verificar status do serviço systemd")?;

                if let Ok(stdout) = String::from_utf8(output.stdout) {
                    if stdout.contains("active") {
                        println!("\n✅ Serviço snapshot-agent.service está ativo");
                        // Mostra PID principal
                        for line in stdout.lines() {
                            if line.starts_with("MainPID=") {
                                let pid = line.trim_start_matches("MainPID=");
                                if pid != "0" {
                                    println!("PID principal: {}", pid.green());
                                } else {
                                    println!("PID principal: não encontrado");
                                }
                            }
                        }
                        // Mostra resumo do status
                        println!("\nResumo do status:");
                        for line in stdout.lines().take(20) {
                            println!("{}", line);
                        }
                    } else {
                        println!("\n❌ Serviço snapshot-agent.service não está ativo");
                    }
                }
            }
            Some(crate::OperatingSystem::Windows(_)) => {
                let output = Command::new("ssh")
                    .args([&vm.ssh_config, "tasklist | findstr snapshot_agent"])
                    .output()
                    .context("Falha ao verificar status do processo")?;

                if let Ok(stdout) = String::from_utf8(output.stdout) {
                    if stdout.contains("snapshot_agent") {
                        println!("\n✅ Agente está rodando");
                        println!("{}", stdout);
                    } else {
                        println!("\n❌ Agente não está rodando");
                    }
                }
            }
            _ => println!("❌ Sistema operacional não suportado"),
        }

        Ok(())
    }

    fn show_agent_logs(&self, vm_idx: usize) -> Result<()> {
        let vm = &self.vms[vm_idx];

        println!("\n📖 Obtendo logs do agente em {}...", vm.name.cyan());

        let log_command = match &vm.os {
            Some(crate::OperatingSystem::Linux(_)) => "tail -n 50 ~/.snapshot_agent/snapshot.log",
            Some(crate::OperatingSystem::Windows(_)) => {
                "type \"%USERPROFILE%\\.snapshot_agent\\snapshot.log\""
            }
            _ => return Ok(()),
        };

        let output = Command::new("ssh")
            .args([&vm.ssh_config, log_command])
            .output()
            .context("Falha ao obter logs")?;

        if let Ok(stdout) = String::from_utf8(output.stdout) {
            println!("\nÚltimos logs:");
            println!("{}", stdout);
        }

        Ok(())
    }

    fn restart_agent(&self, vm_idx: usize) -> Result<()> {
        let vm = &self.vms[vm_idx];

        println!("\n🔄 Reiniciando agente em {}...", vm.name.cyan());

        match &vm.os {
            Some(crate::OperatingSystem::Linux(_)) => {
                Command::new("ssh")
                    .args([
                        &vm.ssh_config,
                        "systemctl --user restart snapshot-agent.service",
                    ])
                    .status()
                    .context("Falha ao reiniciar serviço")?;
                println!("✅ Agente reiniciado com sucesso");
            }
            Some(crate::OperatingSystem::Windows(_)) => {
                Command::new("ssh")
                    .args([
                        &vm.ssh_config,
                        "taskkill /F /IM snapshot_agent.exe && \
                         start \"\" \"C:\\Program Files\\SnapshotAgent\\snapshot_agent.exe\"",
                    ])
                    .status()
                    .context("Falha ao reiniciar processo")?;
                println!("✅ Agente reiniciado com sucesso");
            }
            _ => println!("❌ Sistema operacional não suportado"),
        }

        Ok(())
    }

    fn stop_agent(&self, vm_idx: usize) -> Result<()> {
        let vm = &self.vms[vm_idx];

        println!("\n🛑 Parando agente em {}...", vm.name.cyan());

        match &vm.os {
            Some(crate::OperatingSystem::Linux(_)) => {
                Command::new("ssh")
                    .args([
                        &vm.ssh_config,
                        "systemctl --user stop snapshot-agent.service",
                    ])
                    .status()
                    .context("Falha ao parar serviço")?;
                println!("✅ Agente parado com sucesso");
            }
            Some(crate::OperatingSystem::Windows(_)) => {
                // Para o processo e desabilita o agendamento
                Command::new("ssh")
                    .args([
                        &vm.ssh_config,
                        "schtasks /End /TN SnapshotAgent & taskkill /F /IM snapshot_agent.exe",
                    ])
                    .status()
                    .context("Falha ao parar processo e agendamento")?;
                println!("✅ Agente e agendamento parados com sucesso");
            }
            _ => println!("❌ Sistema operacional não suportado"),
        }

        Ok(())
    }

    fn remove_agent(&self, vm_idx: usize) -> Result<()> {
        let vm = &self.vms[vm_idx];

        println!("\n🗑️ Removendo agente de {}...", vm.name.cyan());

        match &vm.os {
            Some(crate::OperatingSystem::Linux(_)) => {
                Command::new("ssh")
                    .args([
                        &vm.ssh_config,
                        "systemctl --user stop snapshot-agent.service && \
                         systemctl --user disable snapshot-agent.service && \
                         rm -f ~/.config/systemd/user/snapshot-agent.service && \
                         sudo rm -rf /opt/snapshot_agent && \
                         rm -rf ~/.snapshot_agent",
                    ])
                    .status()
                    .context("Falha ao remover agente Linux")?;
                println!("✅ Agente removido com sucesso");
            }
            Some(crate::OperatingSystem::Windows(_)) => {
                // Para o processo, deleta o agendamento e remove arquivos
                Command::new("ssh")
                    .args([
                        &vm.ssh_config,
                        "schtasks /End /TN SnapshotAgent & schtasks /Delete /TN SnapshotAgent /F & taskkill /F /IM snapshot_agent.exe 2>NUL & del C:\\Users\\Public\\snapshot_agent.exe & rmdir /S /Q \"C:\\Program Files\\SnapshotAgent\" & rmdir /S /Q \"%USERPROFILE%\\.snapshot_agent\""
                    ])
                    .status()
                    .context("Falha ao remover agente Windows")?;
                println!("✅ Agente e agendamento removidos com sucesso");
            }
            _ => println!("❌ Sistema operacional não suportado"),
        }

        Ok(())
    }
}

pub fn run_menu(vms: Vec<VMConnection>) -> Result<()> {
    let mut menu = Menu::new(vms);
    menu.run()
}
