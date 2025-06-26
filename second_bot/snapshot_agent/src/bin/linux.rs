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
use uuid::Uuid;

/// =========================================================================
/// ESTRUTURAS DE DADOS
/// =========================================================================
/// Definições das estruturas serializáveis para armazenamento de dados

/// Registra mudanças de estado do agente (iniciado/parado)
/// Permite rastrear ciclos de execução e interrupções
#[derive(Serialize)]
struct ServiceStatus {
    agent_id: String,     // Identificador único do agente
    datetime: String,     // Data/hora formatada ISO8601
    timestamp: u64,       // Timestamp UNIX em segundos
    status: &'static str, // Status: "STARTED" ou "STOPPED"
    hostname: String,     // Nome do host onde o agente está rodando
}

/// Estrutura principal para armazenar informações coletadas
/// Contém todos os dados de uso do sistema em um momento específico
#[derive(Serialize)]
struct Snapshot {
    agent_id: String,          // Identificador único do agente (UUID)
    hostname: String,          // Nome do host onde o agente está rodando
    timestamp: u64,            // Timestamp UNIX em segundos
    datetime: String,          // Data/hora formatada ISO8601
    total_memory: u64,         // Memória total disponível (KB)
    used_memory: u64,          // Memória em uso (KB)
    cpu_usage_percent: f32,    // Porcentagem de uso da CPU (0-100%)
    total_disk: u64,           // Espaço total em disco (bytes)
    used_disk: u64,            // Espaço usado em disco (bytes)
    folder_files: Vec<String>, // Lista de arquivos na pasta monitorada
}

/// Estrutura para registrar erros encontrados durante a coleta
/// Permite diagnóstico e tratamento de falhas
#[derive(Serialize)]
struct LogError {
    agent_id: String, // Identificador único do agente
    hostname: String, // Nome do host onde o erro ocorreu
    datetime: String, // Data/hora do erro
    timestamp: u64,   // Timestamp UNIX do erro
    error: String,    // Mensagem de erro detalhada
}

/// =========================================================================
/// FUNÇÕES DE UTILITÁRIOS E CAMINHOS
/// =========================================================================

/// Retorna o caminho completo do arquivo de log
/// Localizado em ~/.snapshot_agent/snapshot.log
fn get_log_path() -> PathBuf {
    let mut path = home_dir().expect("Não foi possível obter a home do usuário");
    path.push(".snapshot_agent");
    path.push("snapshot.log");
    path
}

/// Retorna o diretório a ser monitorado para listar arquivos
/// No Linux, monitora o diretório home do usuário
fn get_folder_to_monitor() -> PathBuf {
    home_dir().expect("Não foi possível obter a home do usuário")
}

/// Adiciona uma entrada JSON ao arquivo de log
/// Cria diretórios necessários se não existirem
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

/// Obtém o nome do host atual do sistema
/// Utiliza a crate hostname para acessar esta informação
fn get_hostname() -> String {
    hostname::get()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string()
}

/// =========================================================================
/// FUNÇÕES DE LOG E STATUS
/// =========================================================================

/// Registra o status de execução do serviço (iniciado/parado)
/// Essencial para rastreamento do ciclo de vida do agente
fn log_service_status(status: &'static str, agent_id: &str) {
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("Erro ao obter timestamp");

    let status_log = ServiceStatus {
        agent_id: agent_id.to_string(),
        datetime: Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true),
        timestamp: now.as_secs(),
        status,
        hostname: get_hostname(),
    };

    let json = serde_json::to_string(&status_log).expect("Erro ao serializar status");
    append_to_log(&json);
}

/// =========================================================================
/// COLETA PARALELA DE INFORMAÇÕES DO SISTEMA
/// =========================================================================

/// Executa a coleta de dados do sistema através de threads paralelas
/// Cada métrica é coletada em sua própria thread para maximizar desempenho
fn executar_snapshot(agent_id: &str, hostname: &str) {
    // Canal para comunicação entre threads e thread principal
    let (tx, rx) = mpsc::channel();

    // =====================================================================
    // THREAD DE COLETA DE MEMÓRIA
    // =====================================================================
    // Obtém memória total e utilizada do sistema
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
    // Captura porcentagem de utilização da CPU
    let tx_cpu = tx.clone();
    spawn(move || {
        let mut sys = System::new_all();
        sys.refresh_cpu();
        tx_cpu
            .send(("cpu", sys.global_cpu_info().cpu_usage() as u64, 0))
            .unwrap();
    });

    // =====================================================================
    // THREAD DE COLETA DE USO DE DISCO
    // =====================================================================
    // Calcula espaço total e utilizado de todos os discos
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
    // THREAD DE LISTAGEM DE ARQUIVOS
    // =====================================================================
    // Lista arquivos na pasta monitorada e registra erros se necessário
    let tx_files = tx.clone();
    let agent_id_files = agent_id.to_string();
    let hostname_files = hostname.to_string();
    let folder_path = get_folder_to_monitor();
    let datetime_files = Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true);
    let now_files = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("Erro ao obter timestamp");
    let timestamp_files = now_files.as_secs();
    spawn(move || {
        let folder_files = match fs::read_dir(&folder_path) {
            Ok(entries) => entries
                .filter_map(|entry| {
                    entry
                        .ok()
                        .map(|e| e.file_name().to_string_lossy().into_owned())
                })
                .collect::<Vec<String>>(),
            Err(e) => {
                // Registra erro em caso de falha na leitura do diretório
                let log_error = LogError {
                    agent_id: agent_id_files,
                    hostname: hostname_files,
                    datetime: datetime_files.clone(),
                    timestamp: timestamp_files,
                    error: format!("Erro ao ler pasta {:?}: {}", folder_path, e),
                };
                let json = serde_json::to_string(&log_error).expect("Erro ao serializar erro");
                append_to_log(&json);
                vec!["<pasta não encontrada>".to_string()]
            }
        };
        // Sinaliza conclusão e armazena resultados em arquivo temporário
        tx_files.send(("folder_files_vec", 0, 0)).unwrap();
        // Armazena em arquivo temporário para evitar limites do canal
        let _ = fs::write(
            "/tmp/snapshot_folder_files.tmp",
            serde_json::to_string(&folder_files).unwrap(),
        );
    });

    // =====================================================================
    // PROCESSAMENTO DOS RESULTADOS
    // =====================================================================
    // Coleta resultados das threads e monta o snapshot final
    let mut total_memory = 0;
    let mut used_memory = 0;
    let mut cpu_usage_percent = 0.0;
    let mut total_disk = 0;
    let mut used_disk = 0;

    // Aguarda todas as 4 threads concluírem e processa resultados
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
            _ => {} // Sinal da thread de arquivos
        }
    }

    // Recupera a lista de arquivos do arquivo temporário
    let folder_files: Vec<String> = match fs::read("/tmp/snapshot_folder_files.tmp") {
        Ok(data) => serde_json::from_slice(&data)
            .unwrap_or_else(|_| vec!["<erro ao ler arquivos>".to_string()]),
        Err(_) => vec!["<pasta não encontrada>".to_string()],
    };
    // Remove arquivo temporário após leitura
    let _ = fs::remove_file("/tmp/snapshot_folder_files.tmp");

    // =====================================================================
    // GERAÇÃO E ARMAZENAMENTO DO SNAPSHOT
    // =====================================================================
    // Obtém timestamp atual para o snapshot
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("Erro ao obter timestamp");
    let timestamp = now.as_secs();
    let datetime = Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true);

    // Cria o objeto Snapshot com todos os dados coletados
    let snapshot = Snapshot {
        agent_id: agent_id.to_string(),
        hostname: hostname.to_string(),
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

/// =========================================================================
/// FUNÇÃO PRINCIPAL E CICLO DE VIDA DO AGENTE
/// =========================================================================

fn main() {
    // Inicialização: gera identificador único para esta instância
    let agent_id = Uuid::new_v4().to_string();
    let hostname = get_hostname();
    let args: Vec<String> = std::env::args().collect();

    // Processamento de argumentos: suporte ao modo reset
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

    // Registra início do serviço no log
    log_service_status("STARTED", &agent_id);

    // Configura handler para término gracioso com CTRL+C
    let agent_id_handler = agent_id.clone();
    ctrlc::set_handler(move || {
        log_service_status("STOPPED", &agent_id_handler);
        std::process::exit(0);
    })
    .expect("Erro ao configurar handler de término");

    // Loop principal: coleta snapshots periodicamente
    loop {
        executar_snapshot(&agent_id, &hostname);
        sleep(Duration::from_secs(30));
    }
}
