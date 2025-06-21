// Importa módulos necessários para entrada/saída e manipulação de arquivos
use std::fs::{self, File};
use std::io::{self, Write, Read};

/// Diretório padrão para armazenar os arquivos de usuários no diretório atual do projeto
const USUARIOS_DIR: &str = "usuarios";

/// Função principal do programa
/// Apresenta um menu interativo para consultar ou cadastrar usuários, ou encerrar o sistema.
fn main() {
    println!("Gerenciamento de usuários - Execução local");

    // Garante que o diretório de usuários existe
    fs::create_dir_all(USUARIOS_DIR).expect("Falha ao criar diretório de usuários");

    // Loop principal do menu
    loop {
        println!("\nMenu:");
        println!("1. Consultar usuários");
        println!("2. Gravar novo usuário");
        println!("3. Sair");
        print!("Escolha uma opção: ");
        io::stdout().flush().unwrap();

        let mut opcao = String::new();
        io::stdin().read_line(&mut opcao).unwrap();
        let opcao = opcao.trim();

        match opcao {
            "1" => {
                // Opção 1: Consultar usuários cadastrados
                match fs::read_dir(USUARIOS_DIR) {
                    Ok(entries) => {
                        let mut encontrou = false;
                        for entry in entries.flatten() {
                            if let Ok(mut file) = File::open(entry.path()) {
                                let mut conteudo = String::new();
                                file.read_to_string(&mut conteudo).unwrap_or(0);
                                println!("\n{}", conteudo);
                                encontrou = true;
                            }
                        }
                        if !encontrou {
                            println!("Nenhum usuário cadastrado.");
                        }
                    }
                    Err(_) => println!("Nenhum usuário cadastrado."),
                }
            }
            "2" => {
                // Opção 2: Gravar novo usuário
                let nome = input("Nome: ");
                let idade = input("Idade: ");
                let matricula = input("Matrícula: ");

                let conteudo = format!("Nome: {}\nIdade: {}\nMatrícula: {}\n", nome, idade, matricula);
                let arquivo = format!("{}/{}.txt", USUARIOS_DIR, matricula);

                match File::create(&arquivo) {
                    Ok(mut file) => {
                        if file.write_all(conteudo.as_bytes()).is_ok() {
                            println!("Usuário cadastrado com sucesso!");
                        } else {
                            println!("Erro ao gravar usuário.");
                        }
                    }
                    Err(_) => println!("Erro ao criar arquivo de usuário."),
                }
            }
            "3" => {
                // Opção 3: Sair do sistema
                println!("Encerrando o sistema. Obrigado por usar!");
                break;
            }
            _ => println!("Opção inválida!"), // Opção não reconhecida
        }
    }
}

/// Função auxiliar para ler entrada do usuário via terminal
/// Exibe uma mensagem e retorna a string digitada (sem espaços extras)
fn input(msg: &str) -> String {
    print!("{}", msg);
    io::stdout().flush().unwrap(); // Garante que o prompt seja exibido
    let mut buf = String::new();
    io::stdin().read_line(&mut buf).unwrap();
    buf.trim().to_string()
}