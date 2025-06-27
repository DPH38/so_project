mod vm_map;

// Importa crates necessárias para menus interativos e execução de comandos
use dialoguer::{Input, Select, theme::ColorfulTheme};
use std::process::Command;
use vm_map::get_remote_home_tree_json;
use vm_map::send_fs_tree_bin_to_vm;

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
            "🚀 Enviar binário fs_tree_bin para a VM",
            "🗂️ Mapear sistema de arquivos da VM (remoto)",
            "📜 Ver último registro de mapeamento",
            "📝 Verificar alterações no sistema de arquivos",
            "📑 Resumo do conteúdo de um arquivo (NÃO IMPLEMENTADO)",
            "🔙 Voltar ao menu principal",
        ];
        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("==============================\n   🟢 CONEXÃO ESTABELECIDA\n==============================")
            .items(&post_items)
            .default(1)
            .interact()
            .unwrap();
        match selection {
            0 => {
                // Envia o binário fs_tree_bin para a VM
                match send_fs_tree_bin_to_vm(ssh_cmd) {
                    Ok(_) => {
                        println!("\n✅ Binário fs_tree_bin enviado e configurado com sucesso!\n")
                    }
                    Err(e) => println!("❌ Erro ao enviar binário: {}", e),
                }
            }
            1 => {
                // Executa o binário fs_tree_bin na VM e salva o JSON no log
                match get_remote_home_tree_json(ssh_cmd) {
                    Ok(fs_repr) => {
                        let part_to_map = "~"; // Indica home
                        let hex_data = ""; // Não aplicável
                        match vm_map::save_mapping_result(part_to_map, hex_data, &fs_repr) {
                            Ok(path) => {
                                println!("\n✅ Mapeamento remoto salvo em {}\n", path);
                                if let Err(e) = vm_map::print_saved_mapping_localtime() {
                                    println!("❌ Erro ao exibir conteúdo do mapeamento: {}", e);
                                }
                            }
                            Err(e) => println!("❌ Erro ao salvar JSON: {}", e),
                        }
                    }
                    Err(e) => {
                        println!("❌ Erro ao obter árvore remota: {}", e);
                        println!(
                            "Sugestão: utilize a opção 'Enviar binário fs_tree_bin para a VM' antes de mapear."
                        );
                    }
                }
            }
            2 => {
                // Ver último registro de mapeamento e exibir estrutura amigável
                if let Err(e) = vm_map::print_last_mapping_log() {
                    println!("❌ Erro ao exibir estrutura do sistema de arquivos: {}", e);
                }
            }
            3 => {
                // Verificar alterações no sistema de arquivos
                match vm_map::compare_with_last_snapshot(ssh_cmd) {
                    Ok(Some(report)) => println!("{}", report),
                    Ok(None) => println!(
                        "Nenhuma alteração constatada no sistema de arquivos desde o último mapeamento."
                    ),
                    Err(e) => {
                        println!("❌ Erro ao comparar snapshots: {}", e);
                        if let Some(msg) = e
                            .to_string()
                            .to_lowercase()
                            .find("no such file or directory")
                        {
                            println!(
                                "Sugestão: utilize a opção 'Enviar binário fs_tree_bin para a VM' para reimplantar o bot na VM."
                            );
                        }
                    }
                }
            }
            4 => {
                println!("\n⚠️  Esta funcionalidade ainda não foi implementada.\n");
            }
            5 => break, // Voltar ao menu principal
            _ => unreachable!(),
        }
    }
}
