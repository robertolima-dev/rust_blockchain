# Rust Blockchain API 🚀

Uma blockchain simplificada escrita em **Rust**, servida por uma **API REST com Actix Web**.
Atualmente com suporte a:

* Faucet para criar UTXOs de teste
* Transações assinadas com **ECDSA secp256k1** (modelo UTXO)
* Validação real de entradas com verificação de assinatura
* Geração de carteiras (chave privada, chave pública e endereço)
* Mempool para transações pendentes
* Mineração com coinbase + mempool
* Ajuste automático de dificuldade para alvo de \~60s/bloco
* Endpoint de estatísticas da rede (`/stats/`)
* Logging detalhado para debug
* Estrutura de pastas organizada (`mod.rs` em cada módulo)
* Endpoints padronizados em `/api/v1/.../` (com barra final)

---

## 📂 Estrutura de Pastas

```
src/
├── api/
│   ├── chain.rs        # Endpoints relacionados à blockchain (get, validate, mine, difficulty)
│   ├── health.rs       # Health check
│   ├── mod.rs          # Registro das rotas
│   ├── models.rs       # Modelos de request/response + AppState
│   ├── stats.rs        # Estatísticas da blockchain
│   ├── tx.rs           # Faucet, transações e mempool
│   └── wallet.rs       # Endpoints de geração de carteiras
├── blockchain/
│   ├── block.rs        # Estrutura de bloco + PoW
│   ├── mod.rs          # Módulo principal da blockchain
│   └── ...
├── transaction/
│   ├── model.rs        # Transaction, TxInput, TxOutput
│   ├── utxo.rs         # UTXO set + OutPoint
│   └── mod.rs          # Reexporta submódulos
├── wallet/
│   └── mod.rs          # Lógica de geração/validação de chaves e assinaturas
└── main.rs             # Inicializa servidor e AppState
```

---

## ⚙️ Requisitos

* Rust >= 1.70
* `cargo` instalado

---

## ▶️ Rodando o Servidor

```bash
# Clone e entre no diretório do projeto
git clone <repo-url>
cd <repo>

# Instale dependências e rode
cargo run
```

Ou com script de hot reload:

```bash
./start_server.sh
```

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

### **2. Criar Wallet**

`POST /api/v1/wallet/new/`
Gera chave privada, chave pública e endereço.

**Response:**

```json
{
  "private_key": "hex...",
  "public_key": "hex...",
  "address": "hex..."
}
```

---

### **3. Faucet (DEV)**

`POST /api/v1/faucet/`
Cria um UTXO diretamente para testes.

**Request:**

```json
{ "address": "hex_pubkey", "amount": 100 }
```

**Response:**

```json
{ "txid": "hash..." }
```

---

### **4. Nova Transação Assinada**

`POST /api/v1/tx/`

**Request:**

```json
{
  "inputs": [
    {
      "outpoint": { "txid": "hash...", "vout": 0 },
      "pubkey": "hex_pubkey",
      "signature": "hex_der_signature"
    }
  ],
  "outputs": [
    { "address": "hex_pubkey_dest", "amount": 60 },
    { "address": "hex_pubkey_troco", "amount": 39 }
  ]
}
```

---

### **5. Mempool**

`GET /api/v1/mempool/`
Lista transações pendentes.

---

### **6. Mine**

`POST /api/v1/mine/`
Mina um novo bloco, pagando coinbase + taxas ao minerador.

**Request:**

```json
{ "miner_address": "hex_pubkey" }
```

---

### **7. Balance**

`GET /api/v1/balance/{address}/`
Consulta saldo e número de UTXOs.

---

### **8. Stats**

`GET /api/v1/stats/`
Mostra altura, dificuldade, tempos de bloco, mempool e tamanho do UTXO.

---

## 🔍 Fluxo Completo de Teste

1. Criar wallet (`/wallet/new/`)
2. Faucet para endereço gerado
3. Criar outra wallet (destinatário)
4. Montar payload assinado e enviar no `/tx/`
5. Minerar com `/mine/`
6. Conferir saldos e stats

---

## 📌 Próximos Passos

* Múltiplos mineradores externos
* Propagação de blocos e transações entre nós
* Persistência de dados em disco
* Melhorias no formato de endereço

---

## 📜 Licença

MIT License
