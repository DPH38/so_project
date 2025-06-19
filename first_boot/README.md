# first_boot

## Descrição

Este projeto é um utilitário em Rust para gerenciamento simples de usuários, executado localmente em uma máquina Ubuntu (ou qualquer sistema compatível). Ele permite cadastrar e consultar usuários por meio de um menu interativo no terminal. Os dados de cada usuário (nome, idade, matrícula) são armazenados em arquivos de texto individuais dentro da pasta `usuarios` no diretório do projeto.

---

## Funcionalidades

- **Cadastrar novo usuário:**
  - Solicita nome, idade e matrícula.
  - Cria um arquivo de texto para cada usuário em `usuarios/<matricula>.txt`.
- **Consultar usuários:**
  - Lista todos os usuários cadastrados, exibindo os dados de cada um.
- **Sair:**
  - Encerra o programa.

---

## Como usar

1. **Pré-requisitos:**
   - Rust instalado ([instalação oficial](https://www.rust-lang.org/tools/install))

2. **Compilar o projeto:**
   ```bash
   cargo build
   ```

3. **Executar o projeto:**
   ```bash
   cargo run
   ```

4. **Durante a execução:**
   - Siga o menu apresentado no terminal para cadastrar ou consultar usuários.
   - Os dados serão salvos na pasta `usuarios` criada automaticamente no diretório do projeto.

---

## Estrutura dos arquivos de usuário

Cada usuário é salvo em um arquivo de texto com o nome da matrícula, por exemplo: `usuarios/2025001.txt`.

Exemplo de conteúdo:
```
Nome: Maria
Idade: 22
Matrícula: 2025001
```

---

## Observações

- O programa cria a pasta `usuarios` automaticamente, se ela não existir.
- Não é necessário rodar como root.
- Para remover todos os usuários cadastrados, basta apagar a pasta `usuarios`.

---

## Licença

Este projeto é livre para uso acadêmico e pessoal.
