use sysinfo::{System, SystemExt, CpuExt, DiskExt};
use serde::Serialize;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::time::{SystemTime, Duration};
use std::path::PathBuf;
use dirs_next::home_dir;
use std::thread::sleep;
use chrono::{Utc, SecondsFormat};

#[derive(Serialize)]
struct Snapshot {
    timestamp: u64,
    datetime: String,
    total_memory: u64,
    used_memory: u64,
    cpu_usage_percent: f32,
    total_disk: u64,
    used_disk: u64,
    folder_files: Vec<String>,
}

#[derive(Serialize)]
struct LogError {
    datetime: String,
    timestamp: u64,
    error: String,
}

fn get_log_path() -> PathBuf {
    let mut path = home_dir().expect("Não foi possível obter a home do usuário");
    path.push(".snapshot_agent");
    path.push("snapshot.log");
    path
}

fn get_folder_to_monitor() -> PathBuf {
    home_dir().expect("Não foi possível obter a home do usuário")
}

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

fn executar_snapshot() {
    let mut sys = System::new_all();
    sys.refresh_all();

    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("Erro ao obter timestamp");
    let timestamp = now.as_secs();
    let datetime = Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true);

    let total_memory = sys.total_memory();
    let used_memory = sys.used_memory();
    let cpu_usage_percent = sys.global_cpu_info().cpu_usage();

    let total_disk: u64 = sys.disks().iter().map(|d| d.total_space()).sum();
    let used_disk: u64 = sys.disks().iter().map(|d| d.total_space() - d.available_space()).sum();

    let folder_path = get_folder_to_monitor();
    let folder_files = match fs::read_dir(&folder_path) {
        Ok(entries) => entries
            .filter_map(|entry| entry.ok().map(|e| e.file_name().to_string_lossy().into_owned()))
            .collect(),
        Err(e) => {
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

    let json = serde_json::to_string(&snapshot).expect("Erro ao serializar snapshot");
    append_to_log(&json);

    println!("Snapshot salvo em {}", get_log_path().display());
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

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

    loop {
        executar_snapshot();
        sleep(Duration::from_secs(30));
    }
}
