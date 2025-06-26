#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

use chrono::{SecondsFormat, Utc};
use dirs_next::home_dir;
use serde::Serialize;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::mpsc;
use std::thread::{sleep, spawn};
use std::time::{Duration, SystemTime};
use sysinfo::{CpuExt, DiskExt, System, SystemExt};

/// =========================================================================
/// ESTRUTURAS DE DADOS
/// =========================================================================
/// Essas estruturas definem os formatos de dados utilizados pelo snapshot
/// agent para armazenar informações do sistema coletadas.

/// Estrutura principal que representa um snapshot completo do sistema
/// Todos os campos são serializados para JSON e armazenados no log
#[derive(Serialize)]
struct Snapshot {
    timestamp: u64,            // Timestamp UNIX em segundos desde epoch
    datetime: String,          // Data/hora em formato ISO8601 (UTC)
    total_memory: u64,         // Memória total do sistema em KB
    used_memory: u64,          // Memória em uso em KB
    cpu_usage_percent: f32,    // Porcentagem de uso da CPU (0-100)
    total_disk: u64,           // Espaço total em disco em bytes
    used_disk: u64,            // Espaço utilizado em disco em bytes
    folder_files: Vec<String>, // Arquivos na pasta monitorada (C:\Users\Public ou fallback)
}

/// =========================================================================
/// UTILITÁRIOS DE CAMINHOS
/// =========================================================================
/// Estas funções auxiliares retornam caminhos de arquivos e diretórios
/// usados pelo sistema de snapshots

/// Retorna o caminho completo para o arquivo de log
/// Por padrão: ~/.snapshot_agent/snapshot.log
fn get_log_path() -> PathBuf {
    let mut path = home_dir().expect("Não foi possível obter a home do usuário");
    path.push(".snapshot_agent");
    path.push("snapshot.log");
    path
}

/// Retorna o caminho da pasta a ser monitorada no Windows
/// Monitoramos a pasta Public por padrão, com fallback para C:\Users\so
fn get_folder_to_monitor() -> PathBuf {
    PathBuf::from("C:\\Users\\Public")
}

/// Retorna o caminho para o arquivo temporário onde os dados
/// da lista de arquivos são armazenados entre threads
fn get_tmp_file() -> PathBuf {
    let mut tmp = std::env::temp_dir();
    tmp.push("snapshot_folder_files_win.tmp");
    tmp
}

/// =========================================================================
/// SISTEMA DE LOG
/// =========================================================================

/// Adiciona uma entrada JSON ao arquivo de log
/// Cria o diretório e arquivo se não existirem
/// Formato: uma entrada JSON por linha
fn append_to_log(json: &str) {
    let log_path = get_log_path();
    if let Some(dir) = log_path.parent() {
        fs::create_dir_all(dir).expect("Erro ao criar diretório do log");
    }
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
        .expect("Erro ao abrir arquivo de log");
    writeln!(file, "{}", json).expect("Erro ao escrever log");
}

/// =========================================================================
/// EXECUÇÃO DE SNAPSHOT
/// =========================================================================
/// Função principal que orquestra a coleta paralela de dados do sistema
/// utilizando threads para maximizar a eficiência

/// Coleta dados do sistema via threads paralelas e gera um snapshot
fn executar_snapshot() {
    let (tx, rx) = mpsc::channel();

    // =====================================================================
    // THREAD DE COLETA DE MEMÓRIA
    // =====================================================================
    // Coleta total e uso de memória RAM do sistema em uma thread dedicada
    let tx_mem = tx.clone();
    spawn(move || {
        let mut sys = System::new_all();
        sys.refresh_memory();
        tx_mem
            .send(("memory", sys.total_memory(), sys.used_memory()))
            .unwrap();
    });

    // =====================================================================
    // THREAD DE COLETA DE USO DE CPU
    // =====================================================================
    // Realiza duas leituras com intervalo para obter taxa de uso real
    // A primeira leitura apenas inicializa o contador interno
    let tx_cpu = tx.clone();
    spawn(move || {
        let mut sys = System::new_all();
        sys.refresh_cpu();
        sleep(Duration::from_millis(200));
        sys.refresh_cpu();
        tx_cpu
            .send(("cpu", sys.global_cpu_info().cpu_usage() as u64, 0))
            .unwrap();
    });

    // =====================================================================
    // THREAD DE COLETA DE INFORMAÇÕES DE DISCO
    // =====================================================================
    // Soma o espaço total e usado de todos os discos/volumes
    let tx_disk = tx.clone();
    spawn(move || {
        let mut sys = System::new_all();
        sys.refresh_disks_list();
        let total_disk: u64 = sys.disks().iter().map(|d| d.total_space()).sum();
        let used_disk: u64 = sys
            .disks()
            .iter()
            .map(|d| d.total_space() - d.available_space())
            .sum();
        tx_disk.send(("disk", total_disk, used_disk)).unwrap();
    });

    // =====================================================================
    // THREAD DE COLETA DE ARQUIVOS DA PASTA MONITORADA
    // =====================================================================
    // Tenta ler C:\Users\Public primeiro, com fallback para C:\Users\so
    // Salva os resultados em arquivo temporário para transferência entre threads
    let tx_files = tx.clone();
    let folder_path = get_folder_to_monitor();
    let tmp_file_read = get_tmp_file(); // fica no thread principal
    let tmp_file_thread = tmp_file_read.clone(); // cópia para mover na thread

    spawn(move || {
        let folder_files = match fs::read_dir(&folder_path) {
            Ok(entries) => entries
                .filter_map(|e| e.ok().map(|f| f.file_name().to_string_lossy().into_owned()))
                .collect::<Vec<String>>(),
            Err(_) => {
                let fallback = PathBuf::from("C:\\Users\\so");
                match fs::read_dir(&fallback) {
                    Ok(entries) => entries
                        .filter_map(|e| {
                            e.ok().map(|f| f.file_name().to_string_lossy().into_owned())
                        })
                        .collect::<Vec<String>>(),
                    Err(_) => vec!["<pasta não encontrada>".to_string()],
                }
            }
        };

        if let Some(dir) = tmp_file_thread.parent() {
            let _ = fs::create_dir_all(dir);
        }
        let _ = fs::write(
            &tmp_file_thread,
            serde_json::to_string(&folder_files).unwrap(),
        );
        tx_files.send(("folder_files_done", 0, 0)).unwrap();
    });

    // =====================================================================
    // CONSOLIDAÇÃO DOS RESULTADOS DAS THREADS
    // =====================================================================
    // Aguarda os resultados de todas as 4 threads e consolida os dados
    let mut total_memory = 0;
    let mut used_memory = 0;
    let mut cpu_usage_percent = 0.0;
    let mut total_disk = 0;
    let mut used_disk = 0;

    for _ in 0..4 {
        let (kind, v1, v2) = rx.recv().unwrap();
        match kind {
            "memory" => {
                total_memory = v1;
                used_memory = v2;
            }
            "cpu" => {
                cpu_usage_percent = v1 as f32;
            }
            "disk" => {
                total_disk = v1;
                used_disk = v2;
            }
            _ => (), // token da thread de arquivos
        }
    }

    // Recupera a lista de arquivos do arquivo temporário
    let folder_files: Vec<String> = match fs::read(&tmp_file_read) {
        Ok(data) => serde_json::from_slice(&data)
            .unwrap_or_else(|_| vec!["<erro ao ler arquivos>".to_string()]),
        Err(_) => vec!["<pasta não encontrada>".to_string()],
    };
    let _ = fs::remove_file(&tmp_file_read);

    // =====================================================================
    // SERIALIZAÇÃO E ARMAZENAMENTO DO SNAPSHOT
    // =====================================================================
    // Monta o objeto Snapshot, serializa para JSON e grava no log
    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let datetime = Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true);

    let snapshot = Snapshot {
        timestamp,
        datetime,
        total_memory,
        used_memory,
        cpu_usage_percent,
        total_disk,
        used_disk,
        folder_files,
    };

    let json = serde_json::to_string(&snapshot).unwrap();
    append_to_log(&json);
    println!("Snapshot salvo em {}", get_log_path().display());
}

/// =========================================================================
/// FUNÇÃO PRINCIPAL
/// =========================================================================
/// Ponto de entrada do programa, analisa argumentos e inicia monitoramento

fn main() {
    let args: Vec<String> = std::env::args().collect();

    // Modo de reset: limpa o log e encerra o programa
    if args.len() > 1 && args[1] == "--reset" {
        let log_path = get_log_path();
        if let Some(dir) = log_path.parent() {
            fs::create_dir_all(dir).unwrap();
        }
        fs::write(&log_path, b"").unwrap();
        println!("Log resetado em {}", log_path.display());
        return;
    }

    // Modo padrão: loop de snapshots a cada 30 segundos
    loop {
        executar_snapshot();
        sleep(Duration::from_secs(30));
    }
}
