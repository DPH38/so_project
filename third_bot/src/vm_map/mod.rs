//! Módulo para mapeamento binário do sistema de arquivos de um dispositivo (ex: /dev/sda1)
//! Gera um arquivo JSON com informações do mapeamento e logs de data/hora.

use chrono::{DateTime, Local};
use std::fs;

pub fn save_mapping_result(
    device: &str,
    hex_data: &str,
    fs_repr: &serde_json::Value,
) -> std::io::Result<String> {
    let now = chrono::Utc::now().to_rfc3339();
    let log = serde_json::json!({
        "datetime": now,
        "device": device,
        "data_hex": hex_data,
        "fs_repr": fs_repr,
    });
    let output_path = dirs::home_dir()
        .unwrap()
        .join("scaner_file_sistem/mapeamento_remoto.json");
    // Garante que o diretório existe
    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&output_path, serde_json::to_vec_pretty(&log)?)?;
    Ok(output_path.to_string_lossy().to_string())
}

pub fn print_saved_mapping_localtime() -> Result<(), Box<dyn std::error::Error>> {
    let path = dirs::home_dir()
        .unwrap()
        .join("scaner_file_sistem/mapeamento_remoto.json");
    if path.exists() {
        let content = fs::read_to_string(&path)?;
        let mut json: serde_json::Value = serde_json::from_str(&content)?;
        if let Some(datetime_str) = json.get("datetime").and_then(|v| v.as_str()) {
            if let Ok(dt_utc) = DateTime::parse_from_rfc3339(datetime_str) {
                let dt_local = dt_utc.with_timezone(&Local);
                json["datetime"] =
                    serde_json::Value::String(dt_local.format("%Y-%m-%d %H:%M:%S %z").to_string());
            }
        }
        println!(
            "\n✅ Mapeamento realizado com sucesso!\nSugestão: utilize a opção 'Ver último registro de mapeamento' para consultar os detalhes.\n"
        );
    } else {
        println!("Nenhum mapeamento salvo encontrado.");
    }
    Ok(())
}

pub fn print_last_mapping_log() -> Result<(), Box<dyn std::error::Error>> {
    let path = dirs::home_dir()
        .unwrap()
        .join("scaner_file_sistem/mapeamento_remoto.json");
    if path.exists() {
        let content = std::fs::read_to_string(&path)?;
        let json: serde_json::Value = serde_json::from_str(&content)?;
        println!("\n📜 Último registro de mapeamento:\n");
        if let Some(datetime) = json.get("datetime").and_then(|v| v.as_str()) {
            println!("  Data/hora: {}", datetime);
        }
        if let Some(device) = json.get("device").and_then(|v| v.as_str()) {
            println!("  Dispositivo: {}", device);
        }
        if let Some(fs_repr) = json.get("fs_repr") {
            println!("\n  Estrutura do sistema de arquivos (tree):\n");
            print_json_tree_friendly(fs_repr, 0, true);
        }
        return Ok(());
    } else {
        println!(
            "Nenhum registro de mapeamento encontrado.\nSugestão: utilize a opção 'Mapear sistema de arquivos da VM (remoto)' para criar o primeiro registro."
        );
        return Ok(());
    }
}

fn print_json_tree_friendly(node: &serde_json::Value, depth: usize, is_last: bool) {
    let indent = if depth == 0 {
        String::new()
    } else {
        let mut s = String::new();
        for _ in 1..depth {
            s.push_str("│   ");
        }
        s.push_str(if is_last { "└── " } else { "├── " });
        s
    };
    let is_dir = node
        .get("is_dir")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let name = node.get("name").and_then(|v| v.as_str()).unwrap_or("");
    let size = node.get("size").and_then(|v| v.as_u64()).unwrap_or(0);
    let modified = node.get("modified").and_then(|v| v.as_u64()).unwrap_or(0);
    let tipo_str = if is_dir {
        "[DIR]"
    } else if name.ends_with(".pdf") {
        "[PDF]"
    } else {
        "[ARQ]"
    };
    let modified_str = if modified > 0 {
        use chrono::{Local, Utc};
        match chrono::DateTime::<Utc>::from_timestamp(modified as i64, 0) {
            Some(dt_utc) => {
                let dt_local = dt_utc.with_timezone(&Local);
                dt_local.format("%Y-%m-%d %H:%M:%S").to_string()
            }
            None => "-".to_string(),
        }
    } else {
        "-".to_string()
    };
    println!(
        "{}{} {} ({} bytes, modificado: {})",
        indent, tipo_str, name, size, modified_str
    );
    if let Some(children) = node.get("children").and_then(|v| v.as_array()) {
        let len = children.len();
        for (i, child) in children.iter().enumerate() {
            print_json_tree_friendly(child, depth + 1, i == len - 1);
        }
    }
}

use serde_json::Value;
use std::io;
use std::process::Command;

/// Executa o binário fs_tree_bin na VM via SSH e retorna o JSON capturado do stdout.
pub fn get_remote_home_tree_json(ssh_cmd: &str) -> io::Result<Value> {
    // Executa: ssh usuario@host ./fs_tree_bin
    let remote_cmd = format!("{} ./fs_tree_bin", ssh_cmd);
    let output = Command::new("bash").arg("-c").arg(&remote_cmd).output()?;
    if output.status.success() {
        let json_str = String::from_utf8_lossy(&output.stdout);
        let json: Value = serde_json::from_str(&json_str)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        Ok(json)
    } else {
        let err = String::from_utf8_lossy(&output.stderr);
        Err(io::Error::new(
            io::ErrorKind::Other,
            format!("Erro SSH/binário: {}", err),
        ))
    }
}

use std::path::Path;

/// Envia o binário fs_tree_bin para a home do usuário remoto via scp.
pub fn send_fs_tree_bin_to_vm(ssh_cmd: &str) -> std::io::Result<()> {
    // Extrai usuário e host do comando ssh (ex: "ssh usuario@host")
    let mut parts = ssh_cmd.split_whitespace();
    let ssh = parts.next().unwrap_or("");
    let user_host = parts.next().unwrap_or("");
    if ssh != "ssh" || user_host.is_empty() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Comando SSH inválido para envio do binário",
        ));
    }
    // Caminho local do binário
    let local_bin = Path::new("./target/release/fs_tree_bin");
    if !local_bin.exists() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Binário fs_tree_bin não encontrado em ./target/release/",
        ));
    }
    // Comando scp para enviar o binário para a home do usuário remoto
    let scp_cmd = format!("scp {} {}:~/fs_tree_bin", local_bin.display(), user_host);
    let status = std::process::Command::new("bash")
        .arg("-c")
        .arg(&scp_cmd)
        .status()?;
    if !status.success() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Falha ao enviar binário via scp: {}", scp_cmd),
        ));
    }
    // Torna o binário executável na VM
    let chmod_cmd = format!("{} 'chmod +x ~/fs_tree_bin'", ssh_cmd);
    let status = std::process::Command::new("bash")
        .arg("-c")
        .arg(&chmod_cmd)
        .status()?;
    if !status.success() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Falha ao tornar binário executável na VM",
        ));
    }
    Ok(())
}

use std::collections::{HashMap, HashSet};

/// Compara o snapshot atual do sistema de arquivos remoto com o último registro salvo.
/// Retorna Ok(Some(relatório)) se houver diferenças, Ok(None) se não houver, ou Err em caso de erro.
pub fn compare_with_last_snapshot(
    ssh_cmd: &str,
) -> Result<Option<String>, Box<dyn std::error::Error>> {
    // 1. Lê o último registro salvo
    let path = dirs::home_dir()
        .unwrap()
        .join("scaner_file_sistem/mapeamento_remoto.json");
    if !path.exists() {
        return Ok(Some("Nenhum registro de mapeamento encontrado.\nSugestão: realize primeiro uma leitura do sistema de arquivos (mapeamento remoto).".to_string()));
    }
    let content = std::fs::read_to_string(&path)?;
    let json: serde_json::Value = serde_json::from_str(&content)?;
    let old_fs = match json.get("fs_repr") {
        Some(fs) => fs,
        None => {
            return Ok(Some(
                "Registro salvo não possui estrutura de arquivos válida.".to_string(),
            ));
        }
    };
    let last_date = json.get("datetime").and_then(|v| v.as_str()).unwrap_or("-");
    // 2. Faz snapshot instantâneo via SSH
    let new_fs = super::get_remote_home_tree_json(ssh_cmd)?;
    // 3. Compara as árvores
    let mut report = String::new();
    let mut changes = false;
    compare_fs_tree_recursive(old_fs, &new_fs, "", &mut report, &mut changes);
    if changes {
        report = format!("Último mapeamento salvo em: {}\n{}", last_date, report);
        Ok(Some(report))
    } else {
        Ok(Some(format!(
            "Nenhuma alteração constatada no sistema de arquivos desde o último mapeamento em {}.",
            last_date
        )))
    }
}

fn compare_fs_tree_recursive(
    old: &serde_json::Value,
    new: &serde_json::Value,
    parent: &str,
    report: &mut String,
    changes: &mut bool,
) {
    let old_map = build_fs_map(old, parent);
    let new_map = build_fs_map(new, parent);
    let old_keys: HashSet<_> = old_map.keys().collect();
    let new_keys: HashSet<_> = new_map.keys().collect();
    // Arquivos/pastas removidos
    for missing in old_keys.difference(&new_keys) {
        *changes = true;
        report.push_str(&format!("Removido: {}\n", missing));
    }
    // Arquivos/pastas adicionados
    for added in new_keys.difference(&old_keys) {
        *changes = true;
        report.push_str(&format!("Adicionado: {}\n", added));
    }
    // Arquivos/pastas modificados
    for key in old_keys.intersection(&new_keys) {
        let old_meta = &old_map[*key];
        let new_meta = &new_map[*key];
        let old_mod = old_meta.modified;
        let new_mod = new_meta.modified;
        if old_mod != new_mod {
            *changes = true;
            use chrono::{Local, TimeZone};
            let old_dt = Local.timestamp_opt(old_mod as i64, 0).unwrap();
            let new_dt = Local.timestamp_opt(new_mod as i64, 0).unwrap();
            report.push_str(&format!(
                "Alterado: {} (modificado de {} para {})\n",
                key,
                old_dt.format("%Y-%m-%d %H:%M:%S"),
                new_dt.format("%Y-%m-%d %H:%M:%S")
            ));
        }
    }
}

#[derive(Debug)]
struct FsMeta {
    modified: u64,
}

fn build_fs_map(node: &serde_json::Value, parent: &str) -> HashMap<String, FsMeta> {
    let mut map = HashMap::new();
    let name = node.get("name").and_then(|v| v.as_str()).unwrap_or("");
    let path = if parent.is_empty() {
        name.to_string()
    } else {
        format!("{}/{}", parent, name)
    };
    let modified = node.get("modified").and_then(|v| v.as_u64()).unwrap_or(0);
    map.insert(path.clone(), FsMeta { modified });
    if let Some(children) = node.get("children").and_then(|v| v.as_array()) {
        for child in children {
            map.extend(build_fs_map(child, &path));
        }
    }
    map
}

/// Lista todos os arquivos .pdf do último mapeamento salvo.
/// Retorna Ok(Some(lista)) se houver, Ok(None) se não houver mapeamento, ou Err em caso de erro.
pub fn list_pdfs_from_last_mapping() -> Result<Option<Vec<String>>, Box<dyn std::error::Error>> {
    let path = dirs::home_dir()
        .unwrap()
        .join("scaner_file_sistem/mapeamento_remoto.json");
    if !path.exists() {
        return Ok(None);
    }
    let content = std::fs::read_to_string(&path)?;
    let json: serde_json::Value = serde_json::from_str(&content)?;
    let fs_repr = match json.get("fs_repr") {
        Some(fs) => fs,
        None => return Ok(Some(vec![])),
    };
    let mut pdfs = Vec::new();
    collect_pdfs(fs_repr, &mut pdfs);
    Ok(Some(pdfs))
}

fn collect_pdfs(node: &serde_json::Value, pdfs: &mut Vec<String>) {
    let name = node.get("name").and_then(|v| v.as_str()).unwrap_or("");
    let path = node.get("path").and_then(|v| v.as_str()).unwrap_or("");
    if name.to_lowercase().ends_with(".pdf") {
        pdfs.push(path.to_string());
    }
    if let Some(children) = node.get("children").and_then(|v| v.as_array()) {
        for child in children {
            collect_pdfs(child, pdfs);
        }
    }
}

/// Baixa um PDF da VM via SCP, extrai o texto e exibe ao usuário. Remove o arquivo temporário após o uso.
pub async fn summarize_pdf_from_vm(ssh_cmd: &str, remote_pdf_path: &str) -> Result<String, String> {
    use pdf_extract::extract_text;
    use std::env;
    use std::fs;
    use std::path::PathBuf;
    use std::process::Command;

    // Extrai usuário e host do comando ssh (ex: "ssh usuario@host")
    let mut parts = ssh_cmd.split_whitespace();
    let ssh = parts.next().unwrap_or("");
    let user_host = parts.next().unwrap_or("");
    if ssh != "ssh" || user_host.is_empty() {
        return Err("Comando SSH inválido para download do PDF".to_string());
    }

    // Caminho temporário na VM
    let remote_tmp = "/tmp/file.pdf";
    // Renomeia o arquivo na VM para o nome temporário
    let mv_to_tmp = format!(
        r#"{} "mv {} {}""#,
        ssh_cmd,
        shell_escape::unix::escape(remote_pdf_path.into()),
        shell_escape::unix::escape(remote_tmp.into())
    );
    let mv_back = format!(
        r#"{} "mv {} {}""#,
        ssh_cmd,
        shell_escape::unix::escape(remote_tmp.into()),
        shell_escape::unix::escape(remote_pdf_path.into())
    );

    // Renomeia para /tmp/file.pdf
    let status_mv = Command::new("bash")
        .arg("-c")
        .arg(&mv_to_tmp)
        .status()
        .map_err(|e| format!("Erro ao renomear PDF na VM: {}", e))?;
    if !status_mv.success() {
        return Err(format!(
            "Falha ao renomear PDF na VM para download: {}",
            mv_to_tmp
        ));
    }

    // Cria arquivo temporário local
    let tmp_dir = env::temp_dir();
    let mut local_pdf = PathBuf::from(&tmp_dir);
    local_pdf.push("file_tmp.pdf");

    // Baixa o arquivo renomeado
    let scp_cmd = format!("scp {}:{} '{}'", user_host, remote_tmp, local_pdf.display());
    let status_scp = Command::new("bash")
        .arg("-c")
        .arg(&scp_cmd)
        .status()
        .map_err(|e| format!("Erro ao executar SCP: {}", e))?;

    // Sempre tenta restaurar o nome original
    let _ = Command::new("bash").arg("-c").arg(&mv_back).status();
    // Remove o arquivo temporário na VM
    let rm_tmp = format!("{} 'rm -f {}'", ssh_cmd, remote_tmp);
    let _ = Command::new("bash").arg("-c").arg(&rm_tmp).status();

    if !status_scp.success() {
        let _ = fs::remove_file(&local_pdf);
        return Err(format!(
            "Falha ao baixar PDF via SCP: {}\nVerifique se o caminho está correto e se o usuário remoto possui permissão de leitura no arquivo e diretório.",
            scp_cmd
        ));
    }

    // Tenta extrair texto do PDF
    let result = extract_text(&local_pdf);
    // Remove o arquivo temporário local
    let _ = fs::remove_file(&local_pdf);

    match result {
        Ok(text) => {
            if text.trim().is_empty() {
                Err("O PDF não contém texto extraível ou está protegido/não textual.".to_string())
            } else {
                // Processa o conteúdo e gera o resumo
                match super::pdf_processor::process_pdf_content_with_summary(
                    text,
                    remote_pdf_path.to_string(),
                )
                .await
                {
                    Ok(summary) => Ok(summary),
                    Err(e) => Err(format!("Erro ao processar o PDF: {}", e)),
                }
            }
        }
        Err(e) => Err(format!("Erro ao extrair texto do PDF: {}", e)),
    }
}
