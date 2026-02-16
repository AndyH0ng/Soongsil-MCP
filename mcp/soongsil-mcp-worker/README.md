# Soongsil MCP Worker (Rust + Cloudflare Workers)

`/mcp` endpoint로 JSON-RPC 기반 MCP 도구를 제공하는 Rust Worker 구성입니다.

## 1) 준비

필수:

- Rust toolchain (`rustup`, `cargo`)
- `wasm32-unknown-unknown` target
- Node.js (Wrangler 실행용)

```bash
cd /Users/joonwoo/Documents/GitHub/Soongsil-MCP/mcp/soongsil-mcp-worker
rustup target add wasm32-unknown-unknown
npm install
```

## 2) 로컬 개발

```bash
npm run dev
```

## 3) Cloudflare 배포

```bash
npx wrangler login
npm run deploy
```

배포 후 endpoint 예시:

- `https://soongsil-mcp-worker.<your-subdomain>.workers.dev/mcp`

## 4) Claude Desktop 연결 (remote MCP)

`claude_desktop_config.json`의 `mcpServers` 예시:

```json
{
  "mcpServers": {
    "soongsil-mcp-remote": {
      "command": "npx",
      "args": [
        "-y",
        "mcp-remote",
        "https://soongsil-mcp-worker.<your-subdomain>.workers.dev/mcp"
      ]
    }
  }
}
```

## 5) GitHub Actions 자동 배포

워크플로우:

- `/Users/joonwoo/Documents/GitHub/Soongsil-MCP/.github/workflows/deploy-worker.yml`

필수 GitHub Repository Secrets:

- `CLOUDFLARE_API_TOKEN`
- `CLOUDFLARE_ACCOUNT_ID`

Cloudflare API Token은 최소 `Workers Scripts:Edit` 권한을 포함해야 합니다.

## 제공 도구

- `ssu_classify_request`
- `ssu_search_evidence`
- `ssu_rule_brief`
- `ssu_evaluate_graduation`
- `ssu_get_calendar_events`
- `ssu_check_scholarship_threshold`
- `ssu_list_sources`
