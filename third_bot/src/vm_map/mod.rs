//! Módulo para mapeamento binário do sistema de arquivos de um dispositivo (ex: /dev/sda1)
//! Gera um arquivo JSON com informações do mapeamento e logs de data/hora.

use chrono::{DateTime, Local};
use std::collections::HashMap;
use std::fs;

pub fn save_mapping_result(device: &str, hex_data: &str) -> std::io::Result<String> {
    let now = chrono::Utc::now().to_rfc3339();
    let log = serde_json::json!({
        "datetime": now,
        "device": device,
        "data_hex": hex_data,
    });
    let output_path = dirs::home_dir()
        .unwrap()
        .join("scaner_file_sistem/mapeamento_remoto.json");
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
            "\nConteúdo do mapeamento salvo:\n{}\n",
            serde_json::to_string_pretty(&json)?
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
        println!(
            "\nÚltimo registro de mapeamento:\n{}\n",
            serde_json::to_string_pretty(&json)?
        );
    } else {
        println!(
            "Nenhum registro de mapeamento encontrado.\nSugestão: utilize a opção 'Mapear sistema de arquivos da VM (remoto)' para criar o primeiro registro."
        );
    }
    Ok(())
}

/// Executa o comando `lsblk` na VM via SSH e retorna os discos e partições com seus pontos de montagem.
pub fn get_lsblk_info_via_ssh(
    ssh_command: &str,
) -> std::io::Result<(Vec<String>, HashMap<String, String>)> {
    let lsblk_command = format!("{} lsblk -o NAME,TYPE,MOUNTPOINT", ssh_command);
    let output = std::process::Command::new("bash")
        .arg("-c")
        .arg(&lsblk_command)
        .output()?;

    if output.status.success() {
        let result = String::from_utf8_lossy(&output.stdout);
        let mut disks = Vec::new();
        let mut part_mounts = HashMap::new();
        for line in result.lines().skip(1) {
            let fields: Vec<&str> = line.split_whitespace().collect();
            if fields.len() >= 2 {
                let name = fields[0].to_string();
                let typ = fields[1];
                let mount = if fields.len() >= 3 { fields[2] } else { "" };
                if typ == "disk" {
                    disks.push(name.clone());
                }
                if typ == "part" {
                    part_mounts.insert(name.clone(), mount.to_string());
                }
            }
        }
        Ok((disks, part_mounts))
    } else {
        Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Erro ao executar lsblk na VM",
        ))
    }
}
