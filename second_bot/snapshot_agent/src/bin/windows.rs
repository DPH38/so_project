use sysinfo::{System, SystemExt, CpuExt, DiskExt};
use serde::Serialize;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::time::{SystemTime, Duration};
use std::path::PathBuf;
use dirs_next::home_dir;
use std::thread::sleep;
use chrono::{Utc, SecondsFormat};

/// Estrutura que representa um snapshot do sistema
#[derive(Serialize)]
struct Snapshot {
    timestamp: u64,           // Timestamp UNIX do momento do snapshot
    datetime: String,         // Data/hora legível (UTC, formato ISO8601)
    total_memory: u64,        // Memória total do sistema (em KB)
    used_memory: u64,         // Memória usada (em KB)
    cpu_usage_percent: f32,   // Uso global da CPU em porcentagem
    total_disk: u64,          // Espaço total em disco (em bytes)
    used_disk: u64,           // Espaço usado em disco (em bytes)
    folder_files: Vec<String>,// Lista de arquivos encontrados na pasta monitorada
}

/// Estrutura para logar erros
#[derive(Serialize)]
struct LogError {
    datetime: String, // Data/hora legível (UTC, formato ISO8601)
    timestamp: u64,   // Timestamp UNIX
    error: String,    // Mensagem de erro
}

/// Retorna o caminho do arquivo de log na home do usuário
fn get_log_path() -> PathBuf {
    let mut path = home_dir().expect("Não foi possível obter a home do usuário");
    path.push(".snapshot_agent");
    path.push("snapshot.log");
    path
}

/// Retorna o caminho da pasta 'usuarios' na home do usuário
fn get_folder_to_monitor() -> PathBuf {
    home_dir().expect("Não foi possível obter a home do usuário")
}

/// Função para gravar qualquer mensagem JSON no log
fn append_to_log(json: &str) {
    let log_path = get_log_path();
    let log_dir = log_path.parent().expect("Erro ao obter diretório do log");
    fs::create_dir_all(log_dir).expect("Erro ao criar diretório do log");
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
        .expect("Erro ao abrir arquivo de log");
    writeln!(file, "{}", json).expect("Erro ao escrever log");
}

/// Executa a coleta de dados do sistema e salva um snapshot no arquivo de log
fn executar_snapshot() {
    // Inicializa e atualiza as informações do sistema
    let mut sys = System::new_all();
    sys.refresh_all();

    // Obtém o timestamp atual e data/hora legível
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("Erro ao obter timestamp");
    let timestamp = now.as_secs();
    let datetime = Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true);

    // Coleta informações de memória e CPU
    let total_memory = sys.total_memory();
    let used_memory = sys.used_memory();
    let cpu_usage_percent = sys.global_cpu_info().cpu_usage();

    // Coleta informações de disco (usando disks() com feature disk)
    let total_disk: u64 = sys.disks().iter().map(|d| d.total_space()).sum();
    let used_disk: u64 = sys.disks().iter().map(|d| d.total_space() - d.available_space()).sum();

    // Lista arquivos da pasta monitorada (usuarios)
    let folder_path = get_folder_to_monitor();
    let folder_files = match fs::read_dir(&folder_path) {
        Ok(entries) => entries
            .filter_map(|entry| entry.ok().map(|e| e.file_name().to_string_lossy().into_owned()))
            .collect(),
        Err(e) => {
            // Loga o erro de leitura da pasta
            let log_error = LogError {
                datetime: datetime.clone(),
                timestamp,
                error: format!("Erro ao ler pasta {:?}: {}", folder_path, e),
            };
            let json = serde_json::to_string(&log_error).expect("Erro ao serializar erro");
            append_to_log(&json);
            vec!["<pasta não encontrada>".to_string()]
        }
    };

    // Monta o snapshot
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

    // Serializa e grava no log
    let json = serde_json::to_string(&snapshot).expect("Erro ao serializar snapshot");
    append_to_log(&json);

    println!("Snapshot salvo em {}", get_log_path().display());
}

/// Função principal: interpreta argumentos e executa ações
fn main() {
    let args: Vec<String> = std::env::args().collect();

    // Se a flag --reset for usada, limpa o conteúdo do log e encerra
    if args.len() > 1 && args[1] == "--reset" {
        let log_path = get_log_path();

        if let Some(parent) = log_path.parent() {
            fs::create_dir_all(parent).expect("Erro ao criar pasta do log");
        }

        match fs::write(&log_path, b"") {
            Ok(_) => println!("Arquivo de log resetado em {}", log_path.display()),
            Err(e) => eprintln!("Erro ao resetar o log: {}", e),
        }

        return;
    }

    // Caso contrário, inicia o monitoramento a cada 30 segundos
    loop {
        executar_snapshot();
        sleep(Duration::from_secs(30));
    }
}
