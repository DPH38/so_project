//! Módulo para mapeamento binário do sistema de arquivos de um dispositivo (ex: /dev/sda1)
//! Gera um arquivo JSON com informações do mapeamento e logs de data/hora.

use chrono::{DateTime, Local};
use std::fs;

pub mod ext4;

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
        let content = fs::read_to_string(&path)?;
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
    } else {
        println!(
            "Nenhum registro de mapeamento encontrado.\nSugestão: utilize a opção 'Mapear sistema de arquivos da VM (remoto)' para criar o primeiro registro."
        );
    }
    Ok(())
}

fn print_json_tree(node: &serde_json::Value, depth: usize, is_last: bool) {
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
    let tipo = node.get("file_type").and_then(|v| v.as_str()).unwrap_or("");
    let name = node.get("name").and_then(|v| v.as_str()).unwrap_or("");
    let size = node.get("size").and_then(|v| v.as_u64()).unwrap_or(0);
    let tipo_str = if tipo == "dir" {
        "[DIR]"
    } else if name.ends_with(".pdf") {
        "[PDF]"
    } else {
        "[ARQ]"
    };
    println!("{}{} {} ({} bytes)", indent, tipo_str, name, size);
    if let Some(children) = node.get("children").and_then(|v| v.as_array()) {
        let len = children.len();
        for (i, child) in children.iter().enumerate() {
            print_json_tree(child, depth + 1, i == len - 1);
        }
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
        use chrono::{Local, TimeZone};
        let dt = chrono::NaiveDateTime::from_timestamp(modified as i64, 0);
        let local: chrono::DateTime<Local> = Local.from_utc_datetime(&dt);
        local.format("%Y-%m-%d %H:%M:%S").to_string()
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
