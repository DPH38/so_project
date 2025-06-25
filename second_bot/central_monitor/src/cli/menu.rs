use crate::VMConnection;
use anyhow::{Context, Result};
use chrono::{DateTime, Local, TimeZone, Utc};
use colored::*;
use dialoguer::{theme::ColorfulTheme, Select};
use serde_json;
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
            Ok(true) => {
                println!("\n✅ Conectado com sucesso:");
                println!("   • Host: {}", vm.name.green().bold());
                println!("   • SSH:  {}\n", vm.get_current_hostname().green().bold());
            }
            Ok(false) => {
                println!("\n❌ Falha ao conectar:");
                println!("   • Host: {}", vm.name.red().bold());
                println!("   • SSH:  {}", vm.get_current_hostname().red().bold());
                println!("   • Motivo: Não foi possível estabelecer conexão SSH\n");
            }
            Err(e) => {
                println!("\n❌ Erro ao testar conexão:");
                println!("   • Host: {}", vm.name.yellow().bold());
                println!("   • SSH:  {}", vm.get_current_hostname().yellow().bold());
                println!("   • Erro: {}\n", e.to_string().red());
            }
        }
        Ok(())
    }

    fn select_vm(&self, prompt: &str) -> Result<Option<usize>> {
        let mut vm_names: Vec<_> = self.vms.iter().map(|vm| vm.name.as_str()).collect();
        vm_names.push("« Voltar ao menu principal");

        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt(prompt)
            .items(&vm_names)
            .default(0)
            .interact()?;

        if selection == vm_names.len() - 1 {
            Ok(None) // Usuário escolheu voltar
        } else {
            Ok(Some(selection))
        }
    }

    fn ensure_connection(&self, vm_idx: usize) -> Result<()> {
        let vm = &self.vms[vm_idx];
        if !vm.is_connected() {
            println!(
                "\n❌ Sem conexão SSH com {} (Host: {})",
                vm.name.red(),
                vm.get_current_hostname().red()
            );
            println!(
                "   Execute primeiro a opção {} disponível",
                "'Testar conexão com VM'".green().bold()
            );
            println!("   no menu principal para estabelecer a conexão primeiro.\n");
            return Err(anyhow::anyhow!("Conexão SSH não estabelecida"));
        }
        Ok(())
    }

    pub fn run(&mut self) -> Result<()> {
        loop {
            match self.show_main_menu()? {
                0 => {
                    // Testar conexão
                    if let Some(vm_idx) = self.select_vm("Selecione a VM para testar conexão")? {
                        if let Err(e) = self.test_vm_connection(vm_idx) {
                            println!("❌ Erro ao testar conexão: {}", e);
                        }
                    }
                }
                1 => {
                    // Instalar/Atualizar
                    if let Some(vm_idx) =
                        self.select_vm("Selecione a VM para instalar/atualizar o agente")?
                    {
                        if let Err(_) = self.ensure_connection(vm_idx) {
                            continue;
                        }
                        let vm = &self.vms[vm_idx];
                        if let Err(e) = vm.deploy_snapshot_agent() {
                            println!("❌ Erro ao instalar/atualizar agente: {}", e);
                        }
                    }
                }
                2 => {
                    // Listar status
                    if let Some(vm_idx) =
                        self.select_vm("Selecione a VM para verificar o status")?
                    {
                        if let Err(_) = self.ensure_connection(vm_idx) {
                            continue;
                        }
                        if let Err(e) = self.show_agent_status(vm_idx) {
                            println!("❌ Erro ao verificar status: {}", e);
                        }
                    }
                }
                3 => {
                    // Ver logs
                    if let Some(vm_idx) = self.select_vm("Selecione a VM para ver os logs")? {
                        if let Err(_) = self.ensure_connection(vm_idx) {
                            continue;
                        }
                        if let Err(e) = self.show_agent_logs(vm_idx) {
                            println!("❌ Erro ao obter logs: {}", e);
                        }
                    }
                }
                4 => {
                    // Reiniciar
                    if let Some(vm_idx) =
                        self.select_vm("Selecione a VM para reiniciar o agente")?
                    {
                        if let Err(_) = self.ensure_connection(vm_idx) {
                            continue;
                        }
                        if let Err(e) = self.restart_agent(vm_idx) {
                            println!("❌ Erro ao reiniciar agente: {}", e);
                        }
                    }
                }
                5 => {
                    // Parar
                    if let Some(vm_idx) = self.select_vm("Selecione a VM para parar o agente")? {
                        if let Err(_) = self.ensure_connection(vm_idx) {
                            continue;
                        }
                        if let Err(e) = self.stop_agent(vm_idx) {
                            println!("❌ Erro ao parar agente: {}", e);
                        }
                    }
                }
                6 => {
                    // Remover
                    if let Some(vm_idx) = self.select_vm("Selecione a VM para remover o agente")? {
                        if let Err(_) = self.ensure_connection(vm_idx) {
                            continue;
                        }
                        if let Err(e) = self.remove_agent(vm_idx) {
                            println!("❌ Erro ao remover agente: {}", e);
                        }
                    }
                }
                7 => {
                    // Apagar logs
                    if let Some(vm_idx) = self.select_vm("Selecione a VM para apagar os logs")? {
                        if let Err(_) = self.ensure_connection(vm_idx) {
                            continue;
                        }
                        if let Err(e) = self.clear_agent_logs(vm_idx) {
                            println!("❌ Erro ao apagar logs: {}", e);
                        }
                    }
                }
                8 => break, // Sair
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
            "Apagar logs de um agente",
            "Sair",
        ];

        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Selecione uma opção")
            .items(&options)
            .default(0)
            .interact()?;

        Ok(selection)
    }

    fn show_agent_status(&self, vm_idx: usize) -> Result<()> {
        let vm = &self.vms[vm_idx];

        println!("\n🔍 Verificando status do agente em {}...", vm.name.cyan());

        match &vm.os {
            Some(crate::OperatingSystem::Linux(_)) => {
                vm.check_linux_agent_status()?;
            }
            Some(crate::OperatingSystem::Windows(_)) => {
                println!("\nStatus do agente Windows:");
                println!("═══════════════════════");

                // Verifica processo
                let proc_output = Command::new("ssh")
                    .args([
                        &vm.ssh_config,
                        "tasklist /FI \"IMAGENAME eq snapshot_agent.exe\" /FO LIST",
                    ])
                    .output()
                    .context("Falha ao verificar status do processo")?;

                if let Ok(stdout) = String::from_utf8(proc_output.stdout) {
                    if stdout.contains("snapshot_agent.exe") {
                        println!("• Estado: {}", "ATIVO".green().bold());

                        // Extrai e mostra informações do processo
                        for line in stdout.lines() {
                            if line.starts_with("PID:") {
                                println!(
                                    "• PID: {}",
                                    line.split_once(':').unwrap().1.trim().cyan().bold()
                                );
                            } else if line.starts_with("Mem Usage:") {
                                println!(
                                    "• Memória: {}",
                                    line.split_once(':').unwrap().1.trim().cyan()
                                );
                            }
                        }

                        // Verifica estado da tarefa agendada
                        let task_output = Command::new("ssh")
                            .args([&vm.ssh_config, "schtasks /Query /TN SnapshotAgent /FO LIST"])
                            .output()
                            .context("Falha ao verificar tarefa agendada")?;

                        if let Ok(task_stdout) = String::from_utf8(task_output.stdout) {
                            println!("\nDetalhes da tarefa agendada:");
                            println!("══════════════════════════");
                            for line in task_stdout.lines() {
                                if line.starts_with("Status:")
                                    || line.starts_with("Next Run Time:")
                                    || line.starts_with("Last Run Time:")
                                {
                                    let (key, value) = line.split_once(':').unwrap();
                                    println!("• {}: {}", key, value.trim().cyan());
                                }
                            }
                        }
                    } else {
                        println!("• Estado: {}", "INATIVO".red().bold());
                        println!(
                            "ℹ️  O agente não está em execução. Use a opção {} para iniciar.",
                            "'Instalar/Atualizar agente'".green().bold()
                        );
                    }
                }
            }
            _ => println!(
                "❌ Sistema operacional não detectado. Execute a opção {} primeiro.",
                "'Testar conexão com VM'".green().bold()
            ),
        }

        Ok(())
    }

    fn show_agent_logs(&self, vm_idx: usize) -> Result<()> {
        let vm = &self.vms[vm_idx];

        println!("\n📖 Obtendo logs do agente em {}...", vm.name.cyan());

        let log_command = match &vm.os {
            Some(crate::OperatingSystem::Linux(_)) => "test -f ~/.snapshot_agent/snapshot.log && tail -n 50 ~/.snapshot_agent/snapshot.log || echo ''",
            Some(crate::OperatingSystem::Windows(_)) => {
                "if exist \"%USERPROFILE%\\.snapshot_agent\\snapshot.log\" (type \"%USERPROFILE%\\.snapshot_agent\\snapshot.log\") else (echo.)"
            }
            _ => {
                println!("\n❌ Sistema operacional não detectado. Execute {} primeiro.", "'Testar conexão com VM'".green().bold());
                return Ok(());
            }
        };

        let output = Command::new("ssh")
            .args([&vm.ssh_config, log_command])
            .output()
            .context("Falha ao obter logs")?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        if stdout.trim().is_empty() {
            println!("\n📝 Nenhum log encontrado.");
            println!("ℹ️  Possíveis motivos:");
            println!("   • O agente ainda não foi instalado");
            println!("   • O agente foi instalado mas ainda não gerou logs");
            println!("   • Os logs foram apagados recentemente");
            println!("\nℹ️  Sugestões:");
            println!(
                "   1. Use {} para instalar o agente",
                "'Instalar/Atualizar agente'".green().bold()
            );
            println!(
                "   2. Use {} para verificar se o agente está rodando",
                "'Listar status dos agentes'".green().bold()
            );
            return Ok(());
        }

        println!("\n📄 Últimos logs encontrados:");
        println!("═════════════════════════");

        let mut has_valid_logs = false;
        for line in stdout.lines() {
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(line) {
                has_valid_logs = true;
                if let Some(timestamp) = parsed["timestamp"].as_u64() {
                    let datetime = Utc.timestamp_opt(timestamp as i64, 0).single();
                    if let Some(utc_datetime) = datetime {
                        let local_datetime: DateTime<Local> = DateTime::from(utc_datetime);
                        println!(
                            "\n⏰ Registro em {}",
                            local_datetime
                                .format("%d/%m/%Y %H:%M:%S")
                                .to_string()
                                .cyan()
                        );
                    }
                }

                // Formatação dos valores com unidades apropriadas
                let format_bytes = |bytes: u64| -> String {
                    if bytes >= 1024 * 1024 * 1024 {
                        format!("{:.2} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
                    } else if bytes >= 1024 * 1024 {
                        format!("{:.2} MB", bytes as f64 / (1024.0 * 1024.0))
                    } else if bytes >= 1024 {
                        format!("{:.2} KB", bytes as f64 / 1024.0)
                    } else {
                        format!("{} bytes", bytes)
                    }
                };

                if let (Some(total_mem), Some(used_mem)) = (
                    parsed["total_memory"].as_u64(),
                    parsed["used_memory"].as_u64(),
                ) {
                    let mem_percent = (used_mem as f64 / total_mem as f64 * 100.0).round();
                    println!(
                        "💾 Memória: {} / {} ({}%)",
                        format_bytes(used_mem).cyan(),
                        format_bytes(total_mem),
                        mem_percent
                    );
                }

                if let Some(cpu) = parsed["cpu_usage_percent"].as_f64() {
                    println!("🔄 CPU: {}%", format!("{:.1}", cpu).cyan());
                }

                if let (Some(total_disk), Some(used_disk)) =
                    (parsed["total_disk"].as_u64(), parsed["used_disk"].as_u64())
                {
                    let disk_percent = (used_disk as f64 / total_disk as f64 * 100.0).round();
                    println!(
                        "💿 Disco: {} / {} ({}%)",
                        format_bytes(used_disk).cyan(),
                        format_bytes(total_disk),
                        disk_percent
                    );
                }

                if let Some(files) = parsed["folder_files"].as_array() {
                    if !files.is_empty() {
                        println!("📁 Arquivos na pasta: {}", files.len());
                        for file in files {
                            if let Some(name) = file.as_str() {
                                println!("   • {}", name);
                            }
                        }
                    } else {
                        println!("📁 Pasta vazia");
                    }
                }
                println!("───────────────────────");
            }
        }

        if !has_valid_logs {
            println!("\n⚠️  Arquivo de log existe mas não contém registros válidos.");
            println!("ℹ️  Isso pode indicar que o agente não está funcionando corretamente.");
            println!(
                "   Use {} para verificar o status do agente.",
                "'Listar status dos agentes'".green().bold()
            );
        }

        Ok(())
    }

    fn clear_agent_logs(&self, vm_idx: usize) -> Result<()> {
        let vm = &self.vms[vm_idx];

        println!("\n🗑️ Apagando logs do agente em {}...", vm.name.cyan());

        let clear_command = match &vm.os {
            Some(crate::OperatingSystem::Linux(_)) => "rm -f ~/.snapshot_agent/snapshot.log",
            Some(crate::OperatingSystem::Windows(_)) => {
                "del \"%USERPROFILE%\\.snapshot_agent\\snapshot.log\""
            }
            _ => return Ok(()),
        };

        Command::new("ssh")
            .args([&vm.ssh_config, clear_command])
            .status()
            .context("Falha ao apagar logs")?;

        println!("✅ Logs apagados com sucesso");

        Ok(())
    }

    fn restart_agent(&self, vm_idx: usize) -> Result<()> {
        let vm = &self.vms[vm_idx];

        println!("\n🔄 Reiniciando agente em {}...", vm.name.cyan());

        match &vm.os {
            Some(crate::OperatingSystem::Linux(_)) => {
                vm.restart_linux_agent()?;
            }
            Some(crate::OperatingSystem::Windows(_)) => {
                // Para o processo atual e reagenda a tarefa
                Command::new("ssh")
                    .args([
                        &vm.ssh_config,
                        "taskkill /F /IM snapshot_agent.exe 2>NUL & \
                         schtasks /End /TN SnapshotAgent & \
                         schtasks /Run /TN SnapshotAgent",
                    ])
                    .status()
                    .context("Falha ao reiniciar processo")?;

                // Aguarda um pouco para o processo iniciar
                std::thread::sleep(std::time::Duration::from_secs(2));

                // Verifica se o processo está rodando
                let check = Command::new("ssh")
                    .args([
                        &vm.ssh_config,
                        "tasklist /FI \"IMAGENAME eq snapshot_agent.exe\" /NH",
                    ])
                    .output()
                    .context("Falha ao verificar processo")?;

                if String::from_utf8_lossy(&check.stdout).contains("snapshot_agent.exe") {
                    println!(
                        "✅ Agente Windows reiniciado com sucesso em {}",
                        vm.name.green()
                    );
                } else {
                    println!(
                        "⚠️ Agente reiniciado mas processo não detectado em {}",
                        vm.name.yellow()
                    );
                    println!(
                        "ℹ️ Tente usar a opção 'Instalar/Atualizar agente' se o problema persistir"
                    );
                }
            }
            _ => {
                println!("\n❌ Sistema operacional não detectado");
                println!(
                    "ℹ️  Execute primeiro a opção {} para detectar o sistema operacional.",
                    "'Testar conexão com VM'".green().bold()
                );
            }
        }

        Ok(())
    }

    fn stop_agent(&self, vm_idx: usize) -> Result<()> {
        let vm = &self.vms[vm_idx];

        println!("\n🛑 Parando agente em {}...", vm.name.cyan());

        match &vm.os {
            Some(crate::OperatingSystem::Linux(_)) => {
                vm.stop_linux_agent()?;
            }
            Some(crate::OperatingSystem::Windows(_)) => {
                // Para o processo e a tarefa agendada no Windows
                Command::new("ssh")
                    .args([
                        &vm.ssh_config,
                        "schtasks /End /TN SnapshotAgent && \
                         taskkill /F /IM snapshot_agent.exe",
                    ])
                    .status()
                    .context("Falha ao parar processo no Windows")?;

                // Verifica se o processo realmente parou
                let check = Command::new("ssh")
                    .args([
                        &vm.ssh_config,
                        "tasklist /FI \"IMAGENAME eq snapshot_agent.exe\" /NH",
                    ])
                    .output()
                    .context("Falha ao verificar processo")?;

                if String::from_utf8_lossy(&check.stdout).contains("snapshot_agent.exe") {
                    println!("❌ ERRO: Processo snapshot_agent.exe ainda em execução");
                    println!(
                        "ℹ️  Tente remover e reinstalar o agente usando as opções apropriadas"
                    );
                } else {
                    println!("✅ Processo snapshot_agent.exe encerrado com sucesso");
                }
            }
            _ => {
                println!("\n❌ Sistema operacional não detectado");
                println!(
                    "ℹ️  Execute primeiro a opção {} para detectar o sistema operacional.",
                    "'Testar conexão com VM'".green().bold()
                );
            }
        }

        Ok(())
    }

    fn remove_agent(&self, vm_idx: usize) -> Result<()> {
        let vm = &self.vms[vm_idx];

        println!("\n🗑️ Removendo agente de {}...", vm.name.cyan());

        match &vm.os {
            Some(crate::OperatingSystem::Linux(_)) => {
                // Para o serviço primeiro
                Command::new("ssh")
                    .args([
                        &vm.ssh_config,
                        "systemctl --user stop snapshot-agent.service && \
                         systemctl --user disable snapshot-agent.service && \
                         rm -f ~/.config/systemd/user/snapshot-agent.service && \
                         rm -f ~/snapshot_agent && \
                         rm -rf ~/.snapshot_agent",
                    ])
                    .status()
                    .context("Falha ao remover agente Linux")?;

                // Verifica se o processo ainda está rodando
                let check = Command::new("ssh")
                    .args([&vm.ssh_config, "pgrep -af snapshot_agent"])
                    .output()
                    .context("Falha ao verificar processo")?;

                if check.status.success() {
                    println!("⚠️ Processo ainda detectado após remoção, detalhes:");
                    if let Ok(output) = String::from_utf8(check.stdout) {
                        println!("{}", output);
                    }
                    println!("ℹ️ Tente reiniciar a VM se o processo persistir.");
                } else {
                    println!("✅ Agente removido com sucesso");
                }
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
            _ => {
                println!("\n❌ Sistema operacional não detectado");
                println!(
                    "ℹ️  Execute primeiro a opção {} para detectar o sistema operacional.",
                    "'Testar conexão com VM'".green().bold()
                );
            }
        }

        Ok(())
    }
}

pub fn run_menu(vms: Vec<VMConnection>) -> Result<()> {
    Menu::new(vms).run()
}
