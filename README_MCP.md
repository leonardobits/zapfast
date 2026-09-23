# ZapFast - Servidor MCP HTTP Embutido

Este fork do **ZapFast** inclui um servidor **MCP (Model Context Protocol)** HTTP/SSE nativo rodando no mesmo runtime Tokio assíncrono do backend, permitindo que agentes de Inteligência Artificial (Claude, Cursor, Antigravity, etc.) leiam e enviem mensagens no WhatsApp diretamente pela sessão já autenticada do ZapFast, sem abrir instâncias paralelas nem lock de banco de dados SQLite.

---

## 🚀 Como Executar

1. Compile e inicie o ZapFast normalmente:
```bash
cargo run --release
```

2. Assim que o ZapFast inicializa, o servidor MCP sobe automaticamente na porta local:
```
http://127.0.0.1:8765/mcp
```
*(Também expõe o endpoint SSE padrão em `http://127.0.0.1:8765/sse`)*

---

## 🛠️ Tools MCP Disponíveis

| Tool | Descrição | Parâmetros |
|---|---|---|
| `search_chats` | Lista e filtra conversas com contagem de não lidas e metadados | `query` (opcional), `limit` (padrão: 20) |
| `get_messages` | Obtém o histórico recente de mensagens de um chat | `chat_id` (obrigatório), `limit` (padrão: 20) |
| `search_messages` | Busca mensagens no arquivo local pelo conteúdo de texto | `query` (obrigatório), `limit` (padrão: 20) |
| `send_message` | Envia uma nova mensagem de texto | `chat_id` (obrigatório), `text` (obrigatório) |
| `reply_message` | Responde citando uma mensagem específica | `chat_id` (obrigatório), `message_id` (obrigatório), `text` (obrigatório) |

---

## 🔌 Configuração no seu Agente IA

### Exemplo de Configuração MCP (JSON)

Para clientes que suportam transporte SSE/HTTP (como Claude Desktop ou Antigravity):

```json
{
  "mcpServers": {
    "zapfast": {
      "url": "http://127.0.0.1:8765/sse"
    }
  }
}
```

Ou usando Direct HTTP POST (`http://127.0.0.1:8765/mcp`):

### Teste via cURL / PowerShell

#### 1. Listar Tools
```bash
curl -X POST http://127.0.0.1:8765/mcp \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc": "2.0", "id": 1, "method": "tools/list"}'
```

#### 2. Buscar Conversas
```bash
curl -X POST http://127.0.0.1:8765/mcp \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 2,
    "method": "tools/call",
    "params": {
      "name": "search_chats",
      "arguments": { "limit": 10 }
    }
  }'
```

#### 3. Obter Mensagens de um Chat
```bash
curl -X POST http://127.0.0.1:8765/mcp \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 3,
    "method": "tools/call",
    "params": {
      "name": "get_messages",
      "arguments": {
        "chat_id": "551199999999@s.whatsapp.net",
        "limit": 20
      }
    }
  }'
```

#### 4. Enviar Mensagem
```bash
curl -X POST http://127.0.0.1:8765/mcp \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 4,
    "method": "tools/call",
    "params": {
      "name": "send_message",
      "arguments": {
        "chat_id": "551199999999@s.whatsapp.net",
        "text": "Olá! Mensagem enviada pelo MCP."
      }
    }
  }'
```

---

## 🏛️ Arquitetura Interna

```
    Agente IA (Claude / Cursor / Antigravity)
                       │
                       ▼  HTTP/SSE JSON-RPC
               127.0.0.1:8765/mcp
                       │
       ┌───────────────▼───────────────┐
       │            ZapFast            │
       │                               │
       │   MCP Server (Axum)           │
       │          │ tokio::sync::mpsc  │
       │          ▼                    │
       │   Backend Worker Loop         │
       │     ┌─────────┴─────────┐     │
       │     ▼                   ▼     │
       │  whatsapp-rust      archive.db│
       └───────────────────────────────┘
```
