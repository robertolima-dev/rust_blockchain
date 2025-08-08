# 🦀 Rust Blockchain API

A simple blockchain implementation in Rust with an HTTP API built using **Actix Web**.  
This project is structured in a modular way and supports basic Proof-of-Work mining, chain validation, difficulty adjustment, and is ready to be extended with transactions, wallets, and P2P networking.

---

## 📌 Features Implemented
- Modular architecture with context-based folders (`api`, `blockchain`, etc.)
- In-memory blockchain with:
  - Genesis block
  - SHA-256 hashing
  - Proof-of-Work mining
  - Chain validation
  - Adjustable difficulty
- REST API with Actix Web (`/api/v1/` prefix)
- Hot reload support with `cargo-watch`
- Configurable host/port via `.env`
- JSON responses for all API endpoints

---

## 📂 Project Structure
```text
src/
├── api/                 # HTTP API routes (controllers)
│   ├── mod.rs
│   ├── health.rs
│   └── chain.rs
├── blockchain/          # Blockchain core logic
│   ├── mod.rs
│   └── block.rs
├── main.rs              # API entrypoint
└── lib.rs               # Module declarations
````

---

## ⚙️ Requirements

* [Rust](https://www.rust-lang.org/tools/install) (latest stable)
* [cargo-watch](https://crates.io/crates/cargo-watch) for hot reload (optional)

Install `cargo-watch`:

```bash
cargo install cargo-watch
```

---

## 🚀 Running the Project

### 1. Clone and enter the project folder

```bash
git clone <repo-url>
cd rust_blockchain_api
```

### 2. Install dependencies (done automatically by Cargo)

```bash
cargo build
```

### 3. Configure environment variables

Create a `.env` file:

```env
HOST=127.0.0.1
PORT=8080
RUST_LOG=info
```

### 4. Run the API

Normal mode:

```bash
cargo run
```

Hot reload mode:

```bash
cargo watch -q -c -w src -w Cargo.toml -w .env -x 'run'
```

Script mode:

```bash
./start_server.sh
```

---

## 🌐 API Endpoints

All endpoints are prefixed with `/api/v1/` and **must end with a trailing slash `/`**.

| Method | Endpoint              | Description                | Body Example          |
| ------ | --------------------- | -------------------------- | --------------------- |
| GET    | `/api/v1/health/`     | Health check               | —                     |
| GET    | `/api/v1/chain/`      | Get the entire blockchain  | —                     |
| GET    | `/api/v1/validate/`   | Validate the blockchain    | —                     |
| POST   | `/api/v1/mine/`       | Mine a new block with data | `{ "data": "hello" }` |
| GET    | `/api/v1/difficulty/` | Get current PoW difficulty | —                     |
| POST   | `/api/v1/difficulty/` | Set PoW difficulty         | `{ "difficulty": 3 }` |

---

## 📌 Example Requests

### Health Check

```bash
curl http://127.0.0.1:8080/api/v1/health/
```

### Mine a New Block

```bash
curl -X POST http://127.0.0.1:8080/api/v1/mine/ \
  -H "Content-Type: application/json" \
  -d '{ "data": "my first mined block" }'
```

### Validate Blockchain

```bash
curl http://127.0.0.1:8080/api/v1/validate/
```

---

## 🛠 Tech Stack

* **Rust**
* [Actix Web](https://actix.rs/)
* [Serde](https://serde.rs/) for JSON serialization
* [SHA2](https://docs.rs/sha2/latest/sha2/) for hashing
* [Chrono](https://docs.rs/chrono/) for timestamps
* [dotenvy](https://crates.io/crates/dotenvy) for env config

---

## 📈 Next Steps

* Add **Transaction** module with signed transactions
* Implement **Wallets** using ECDSA (`secp256k1`)
* Add **mining rewards** and mempool
* Implement **P2P networking** for node synchronization
* Optional: persistent storage

---

## 📜 License

MIT License
