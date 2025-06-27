mod vm_map;

// Importa crates necessárias para menus interativos e execução de comandos
use dialoguer::{Input, Select, theme::ColorfulTheme};
use std::process::Command;

fn main() {
    // Variável para armazenar o comando SSH ativo
    let mut active_ssh_cmd: Option<String> = None;
    // Loop principal do menu
    loop {
        // Menu principal: opção de conexão SSH, consultar sistema de arquivos e sair
        let menu_items = vec![
            "🔗 Conexão por SSH",
            "🗂️ Consultar sistema de arquivos",
            "🚪 Sair",
        ];
        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("==============================\n   🖥️  MENU PRINCIPAL\n==============================")
            .items(&menu_items)
            .default(0)
            .interact()
            .unwrap();

        match selection {
            0 => {
                // Solicita o comando SSH (alias ou comando completo)
                let ssh_cmd: String = Input::with_theme(&ColorfulTheme::default())
                    .with_prompt("Digite o comando SSH (ex: ssh usuario@host)")
                    .interact_text()
                    .unwrap();
                println!("\n🔗 Tentando conectar com: {}\n", ssh_cmd);
                // Executa comandos para obter informações da VM
                let info_cmd = format!(
                    "{ssh} hostname && {ssh} whoami && {ssh} hostname -I && {ssh} curl -s ifconfig.me",
                    ssh = ssh_cmd
                );
                let output = Command::new("bash").arg("-c").arg(&info_cmd).output();
                match output {
                    Ok(out) if out.status.success() => {
                        let result = String::from_utf8_lossy(&out.stdout);
                        let mut lines = result.lines();
                        let hostname = lines.next().unwrap_or("");
                        let user = lines.next().unwrap_or("");
                        let ip = lines.next().unwrap_or("");
                        let public_ip = lines.next().unwrap_or("");
                        println!(
                            "✅ Conexão bem-sucedida!\n  🖥️ VM: {}\n  👤 Usuário: {}\n  🌐 IP: {}\n  🌍 IP Público: {}\n",
                            hostname, user, ip, public_ip
                        );
                        active_ssh_cmd = Some(ssh_cmd.clone());
                        post_connection_menu(&ssh_cmd);
                    }
                    Ok(out) => {
                        let err = String::from_utf8_lossy(&out.stderr);
                        println!("❌ Falha na conexão: {}", err);
                    }
                    Err(e) => println!("❌ Erro ao tentar conectar: {}", e),
                }
            }
            1 => {
                // Consultar sistema de arquivos: verifica se já há conexão ativa
                if let Some(ref ssh_cmd) = active_ssh_cmd {
                    post_connection_menu(ssh_cmd);
                } else {
                    // Não há conexão ativa, solicita conexão
                    println!("\nNenhuma conexão SSH ativa. Realize a conexão primeiro.\n");
                    let ssh_cmd: String = Input::with_theme(&ColorfulTheme::default())
                        .with_prompt("Digite o comando SSH (ex: ssh usuario@host)")
                        .interact_text()
                        .unwrap();
                    println!("\n🔗 Tentando conectar com: {}\n", ssh_cmd);
                    let info_cmd = format!(
                        "{ssh} hostname && {ssh} whoami && {ssh} hostname -I && {ssh} curl -s ifconfig.me",
                        ssh = ssh_cmd
                    );
                    let output = Command::new("bash").arg("-c").arg(&info_cmd).output();
                    match output {
                        Ok(out) if out.status.success() => {
                            let result = String::from_utf8_lossy(&out.stdout);
                            let mut lines = result.lines();
                            let hostname = lines.next().unwrap_or("");
                            let user = lines.next().unwrap_or("");
                            let ip = lines.next().unwrap_or("");
                            let public_ip = lines.next().unwrap_or("");
                            println!(
                                "✅ Conexão bem-sucedida!\n  🖥️ VM: {}\n  👤 Usuário: {}\n  🌐 IP: {}\n  🌍 IP Público: {}\n",
                                hostname, user, ip, public_ip
                            );
                            active_ssh_cmd = Some(ssh_cmd.clone());
                            post_connection_menu(&ssh_cmd);
                        }
                        Ok(out) => {
                            let err = String::from_utf8_lossy(&out.stderr);
                            println!("❌ Falha na conexão: {}", err);
                        }
                        Err(e) => println!("❌ Erro ao tentar conectar: {}", e),
                    }
                }
            }
            2 => {
                println!("\n👋 Saindo... Até logo!\n");
                break;
            }
            _ => unreachable!(),
        }
    }
}

// Menu de pós-conexão bem-sucedida
fn post_connection_menu(ssh_cmd: &str) {
    loop {
        let post_items = vec![
            "🗂️ Mapear sistema de arquivos da VM (remoto)",
            "📜 Ver último registro de mapeamento",
            "📝 Verificar alterações no sistema de arquivos (NÃO IMPLEMENTADO)",
            "📄 Verificar alterações em arquivos .pdf da VM (NÃO IMPLEMENTADO)",
            "📑 Resumo do conteúdo de um arquivo (NÃO IMPLEMENTADO)",
            "🔙 Voltar ao menu principal",
        ];
        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("==============================\n   🟢 CONEXÃO ESTABELECIDA\n==============================")
            .items(&post_items)
            .default(0)
            .interact()
            .unwrap();
        match selection {
            0 => {
                // Lista discos e partições remotos via SSH
                match vm_map::get_lsblk_info_via_ssh(ssh_cmd) {
                    Ok((_disks, part_mounts)) => {
                        // Descobre índice da partição montada em '/'
                        let root_index = part_mounts
                            .iter()
                            .position(|(_, mnt)| mnt == "/")
                            .unwrap_or(0);
                        // Monta lista de partições para seleção, destacando a '/'
                        let part_options: Vec<String> = part_mounts
                            .iter()
                            .map(|(part, mnt)| {
                                if mnt == "/" {
                                    format!("{} [ROOT]", part)
                                } else if !mnt.is_empty() {
                                    format!("{} [mnt: {}]", part, mnt)
                                } else {
                                    part.clone()
                                }
                            })
                            .collect();
                        let selection = Select::with_theme(&ColorfulTheme::default())
                            .with_prompt("Selecione a partição para mapear (a '/' está destacada como [ROOT])")
                            .items(&part_options)
                            .default(root_index)
                            .interact()
                            .unwrap();
                        let (part_to_map, _mnt) =
                            part_mounts.iter().nth(selection).expect("Índice inválido");
                        println!("Partição selecionada: {}", part_options[selection]);
                        let remote_cmd =
                            format!("sudo dd if=/dev/{} bs=1 count=4096 | xxd -p", part_to_map);
                        let full_ssh_cmd = format!("{} {}", ssh_cmd, remote_cmd);
                        let output = Command::new("bash").arg("-c").arg(&full_ssh_cmd).output();
                        match output {
                            Ok(out) if out.status.success() => {
                                let hex_data = String::from_utf8_lossy(&out.stdout);
                                match vm_map::save_mapping_result(part_to_map, hex_data.trim()) {
                                    Ok(path) => {
                                        println!("\n✅ Mapeamento remoto salvo em {}\n", path);
                                        if let Err(e) = vm_map::print_saved_mapping_localtime() {
                                            println!(
                                                "❌ Erro ao exibir conteúdo do mapeamento: {}",
                                                e
                                            );
                                        }
                                    }
                                    Err(e) => println!("❌ Erro ao salvar JSON: {}", e),
                                }
                            }
                            Ok(out) => {
                                let err = String::from_utf8_lossy(&out.stderr);
                                println!("❌ Erro ao mapear partição remota: {}", err);
                            }
                            Err(e) => println!("❌ Erro ao executar comando remoto: {}", e),
                        }
                    }
                    Err(e) => println!("❌ Erro ao listar partições na VM: {}", e),
                }
            }
            1 => {
                // Ver último registro de mapeamento
                if let Err(e) = vm_map::print_last_mapping_log() {
                    println!("❌ Erro ao ler o registro de mapeamento: {}", e);
                }
            }
            2 | 3 | 4 => {
                println!("\n⚠️  Esta funcionalidade ainda não foi implementada.\n");
            }
            5 => break, // Voltar ao menu principal
            _ => unreachable!(),
        }
    }
}
