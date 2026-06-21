# 🧛 Vamp Investment Portfolio

Aplicação full-stack de carteira de investimentos construída com **Rust**, **Axum**, **PostgreSQL** e **Askama**.

---

## 🛠️ Stack

| Camada       | Tecnologia                        |
|--------------|-----------------------------------|
| Backend      | Rust (edition 2021)               |
| Web Framework| Axum 0.8                          |
| Banco        | PostgreSQL 16 (via SQLx 0.8)      |
| Templates    | Askama 0.15                       |
| Auth         | JWT Simple 0.12 + Password Auth 1 |

---

## ⚙️ Pré-requisitos

- [Rust](https://rustup.rs/) (stable)
- [Docker](https://www.docker.com/) + Docker Compose
- `sqlx-cli` (ver seção abaixo)

---

## 🚀 Configuração do ambiente

### 1. Clonar o repositório

```bash
git clone https://github.com/alissonjcjk/vamp-investment-portfolio.git
cd vamp-investment-portfolio
```

### 2. Configurar variáveis de ambiente

```bash
cp .env.example .env
```

O `.env` padrão já está pronto para uso local:

```env
DATABASE_URL=postgres://postgres:postgres@localhost:5432/postgres
```

### 3. Subir o banco de dados (Docker)

```bash
docker compose up -d
```

Para verificar se o container está rodando:

```bash
docker compose ps
```

Para conectar diretamente ao PostgreSQL:

```bash
docker exec -it vamp-investment-portfolio-db-1 psql -U postgres
```

### 4. Instalar o sqlx-cli

O `sqlx-cli` é necessário para rodar e criar migrations. Instale com:

```bash
cargo install sqlx-cli --no-default-features --features postgres
```

> ⚠️ A instalação pode demorar alguns minutos na primeira vez.

### 5. Rodar as migrations

```bash
sqlx migrate run
```

---

## 🏃 Rodando o servidor

```bash
cargo run
```

---

## 🗄️ Gerenciamento do banco

| Comando                        | Descrição                        |
|-------------------------------|----------------------------------|
| `docker compose up -d`        | Sobe o PostgreSQL em background  |
| `docker compose down`         | Para e remove os containers      |
| `docker compose down -v`      | Para e remove containers + volume|
| `sqlx migrate run`            | Aplica as migrations pendentes   |
| `sqlx migrate revert`         | Reverte a última migration       |
| `sqlx migrate add <nome>`     | Cria uma nova migration          |

---

## 🧪 Testes

```bash
cargo test
```

---

## 📁 Estrutura do projeto

```
vamp-investment-portfolio/
├── compose.yml          # Docker Compose (PostgreSQL)
├── .env.example         # Template de variáveis de ambiente
├── migrations/          # Migrations SQL (sqlx)
├── templates/           # Templates HTML (Askama)
└── src/
    ├── main.rs          # Entry point
    ├── app.rs           # AppState e router
    ├── error.rs         # AppError enum
    ├── models.rs        # Structs de domínio
    ├── repository.rs    # Acesso ao banco (SQLx)
    ├── auth/
    │   ├── mod.rs
    │   ├── user.rs      # Auth de usuários (JWT + cookie)
    │   └── admin.rs     # Auth de admin (header)
    └── routes/
        ├── mod.rs
        ├── api.rs       # Endpoints REST/JSON
        └── frontend.rs  # Rotas HTML (Askama)
```

---

## 📝 Tarefas (roadmap)

- [x] Task 1 — Inicializar projeto Rust e estrutura de pastas
- [x] Task 2 — Configurar Docker Compose e variáveis de ambiente
- [ ] Task 3 — Configurar SQLx e criar migrations
- [ ] Task 4 — Implementar AppState e conexão com o banco
- [ ] Task 5 — Criar módulo de erros (AppError)
- [ ] Task 6 — Implementar Repository
- [ ] Task 7 — Auth: UnauthenticatedUser e User
- [ ] Task 8 — Auth: Admin extractor
- [ ] Task 9 — Rotas API (assets)
- [ ] Task 10 — Rotas frontend (templates Askama)
- [ ] Task 11 — Testes de integração
- [ ] Task 12 — Testes snapshot com Insta
- [ ] Task 13 — Logging e tracing
- [ ] Task 14 — Refinamentos e tratamento de erros
- [ ] Task 15 — Build final e documentação
