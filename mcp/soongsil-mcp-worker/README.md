# Soongsil MCP Worker (Cloudflare Workers)

`/mcp` endpoint로 Streamable HTTP MCP를 제공하는 Cloudflare Workers 배포 설정입니다.

## 1) 준비

```bash
cd /Users/joonwoo/Documents/GitHub/Soongsil-MCP/mcp/soongsil-mcp-worker
npm install
```

## 2) 로컬 개발

```bash
npm run dev
```

`npm run dev`는 실행 전에 자동으로 `sync-assets`를 수행해 아래 소스 MD를 `public/`으로 복사합니다.

- `knowledge/normalized-md/*.md`
- `knowledge/raw-md/*.md`
- `mcp/soongsil-mcp/references/*.md`

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

이미 포함된 워크플로우:

- `/Users/joonwoo/Documents/GitHub/Soongsil-MCP/.github/workflows/deploy-worker.yml`

동작:

- `main` 브랜치 push 시 자동 배포
- `workflow_dispatch` 수동 실행 지원

필수 GitHub Repository Secrets:

- `CLOUDFLARE_API_TOKEN`
- `CLOUDFLARE_ACCOUNT_ID`

Cloudflare API Token에는 최소 `Workers Scripts:Edit` 권한을 포함하세요.

## 제공 도구

- `ssu_classify_request`
- `ssu_search_evidence`
- `ssu_rule_brief`
- `ssu_evaluate_graduation`
- `ssu_get_calendar_events`
- `ssu_check_scholarship_threshold`
- `ssu_list_sources`
