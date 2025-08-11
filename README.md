# Rust Blockchain API 🚀

Uma blockchain simplificada escrita em **Rust**, servida por uma **API REST com Actix Web**.  
Atualmente com suporte a:

- Faucet para criar UTXOs de teste
- Transações com validação real de entradas (modelo UTXO)
- Mempool para transações pendentes
- Logging detalhado para debug
- Estrutura de pastas organizada (`mod.rs` em cada módulo)
- Endpoints padronizados em `/api/v1/.../` (com barra final)

---

## 📂 Estrutura de Pastas

```

src/
├── api/
│   ├── chain.rs        # Endpoints relacionados à blockchain (get, validate, mine, difficulty)
│   ├── health.rs       # Health check
│   ├── mod.rs          # Registro das rotas
│   ├── models.rs       # Modelos de request/response + AppState
│   └── tx.rs           # Faucet, transações e mempool
├── blockchain/
│   ├── block.rs        # Estrutura de bloco + PoW
│   ├── mod.rs          # Módulo principal da blockchain
│   └── ...
├── transaction/
│   ├── model.rs        # Transaction, TxInput, TxOutput
│   ├── utxo.rs         # UTXO set + OutPoint
│   └── mod.rs          # Reexporta submódulos
└── main.rs             # Inicializa servidor e AppState

````

---

## ⚙️ Requisitos

- Rust >= 1.70
- `cargo` instalado

---

## ▶️ Rodando o Servidor

```bash
# Clone e entre no diretório do projeto
git clone <repo-url>
cd <repo>

# Instale dependências e rode
cargo run
````

# Ou
./start_server.sh
````

Com logs de debug:

```bash
RUST_LOG=debug,actix_web=info cargo run
```

Servidor sobe por padrão em:

```
http://127.0.0.1:8080
```

---

## 🌐 Endpoints Disponíveis

### **1. Health Check**

`GET /api/v1/health/`
Verifica se a API está online.

```bash
curl http://127.0.0.1:8080/api/v1/health/
```

---

### **2. Faucet** (DEV)

Cria um UTXO diretamente para testes.

```
POST /api/v1/faucet/
```

**Request:**

```json
{ "address": "alice", "amount": 100 }
```

**Response:**

```json
{
  "txid": "6c1a0c8b7f2f4b8f9a3e6d1c...",
  "outpoints": [{ "txid": "6c1a0c8b7f2f4b8f9a3e6d1c...", "vout": 0 }]
}
```

---

### **3. Nova Transação**

Cria uma transação usando UTXOs existentes.

```
POST /api/v1/tx/
```

**Request:**

```json
{
  "inputs": [
    { "outpoint": { "txid": "6c1a0c8b7f2f4b8f9a3e6d1c...", "vout": 0 } }
  ],
  "outputs": [
    { "address": "bob", "amount": 60 },
    { "address": "change-alice", "amount": 39 }
  ]
}
```

**Response:**

```json
{ "txid": "def123..." }
```

⚠️ Importante:

* Use **txid** e **vout** retornados pelo faucet.
* `vout` deve ser **número**, não string.
* Barra final obrigatória: `/api/v1/tx/`

---

### **4. Mempool**

Lista transações pendentes de mineração.

```
GET /api/v1/mempool/
```

**Response:**

```json
{
  "size": 1,
  "transactions": [
    "def123..."
  ]
}
```

---

## 🔍 Fluxo de Teste Completo

1. Criar UTXO com faucet:

```bash
curl -X POST http://127.0.0.1:8080/api/v1/faucet/ \
  -H "Content-Type: application/json" \
  -d '{ "address": "alice", "amount": 100 }'
```

2. Criar transação usando UTXO retornado:

```bash
curl -X POST http://127.0.0.1:8080/api/v1/tx/ \
  -H "Content-Type: application/json" \
  -d '{
    "inputs": [
      { "outpoint": { "txid": "TXID_DO_FAUCET", "vout": 0 } }
    ],
    "outputs": [
      { "address": "bob", "amount": 60 },
      { "address": "change-alice", "amount": 39 }
    ]
  }'
```

3. Verificar mempool:

```bash
curl http://127.0.0.1:8080/api/v1/mempool/
```

---

## 📌 Próximos Passos

* Implementar mineração real (`/mine/`) a partir da mempool.
* Aplicar blocos minerados ao UTXO set.
* Ajuste automático de dificuldade.
* Suporte para mineradores externos (template/submit).

---

## 📜 Licença

MIT License
