# Soongsil MCP

숭실대학교 학칙·학사 문서를 근거로 질의응답, 졸업요건 판정, 장학 기준 확인을 지원하는 MCP 서버입니다.

## Repository Layout

- `docs/`: 원본 근거 PDF
- `knowledge/normalized-md/`: 질의응답용 정규화 코퍼스
- `knowledge/raw-md/`: 원문 fallback
- `mcp/soongsil-mcp/`: MCP 서버 구현 및 참조 규칙
- `mcp/soongsil-mcp-worker/`: Rust 기반 Cloudflare Workers 원격 MCP 배포 구성

## Quick Start

```bash
# repository root에서 실행
cd /Users/joonwoo/Documents/GitHub/Soongsil-MCP/mcp/soongsil-mcp
python3 -m venv .venv
source .venv/bin/activate
python -m pip install --upgrade pip
python -m pip install -e .
python server.py
```

`mcp.server.fastmcp` import 오류가 나면:

```bash
pip install fastmcp
```

## Claude Desktop 연결

`claude_desktop_config.json`의 `mcpServers`에 아래 추가:

```json
{
  "mcpServers": {
    "soongsil-mcp": {
      "command": "/Users/joonwoo/Documents/GitHub/Soongsil-MCP/mcp/soongsil-mcp/.venv/bin/python",
      "args": [
        "/Users/joonwoo/Documents/GitHub/Soongsil-MCP/mcp/soongsil-mcp/server.py"
      ]
    }
  }
}
```

## Cloudflare Workers (원격 MCP)

Workers 배포를 사용할 경우 아래 문서를 따르세요.

- `/Users/joonwoo/Documents/GitHub/Soongsil-MCP/mcp/soongsil-mcp-worker/README.md`

핵심 명령:

```bash
cd /Users/joonwoo/Documents/GitHub/Soongsil-MCP/mcp/soongsil-mcp-worker
rustup target add wasm32-unknown-unknown
npm install
npm run deploy
```

자동 배포 워크플로우:

- `/Users/joonwoo/Documents/GitHub/Soongsil-MCP/.github/workflows/deploy-worker.yml`

## Exposed MCP Tools

- `ssu_classify_request`
- `ssu_search_evidence`
- `ssu_rule_brief`
- `ssu_evaluate_graduation`
- `ssu_get_calendar_events`
- `ssu_check_scholarship_threshold`
- `ssu_list_sources`

## Evidence Flow

1. `knowledge/normalized-md` 우선 탐색
2. `mcp/soongsil-mcp/references` 규칙/인덱스 대조
3. 필요 시 `knowledge/raw-md/학칙.raw.md` 대조
4. 최종 답변은 PDF 페이지 인용 `(문서명.pdf, p.N)`

## Maintenance

raw/정규화 업데이트 시:

```bash
python3 /Users/joonwoo/Documents/GitHub/Soongsil-MCP/mcp/soongsil-mcp/scripts/convert_pdfs_to_md.py
```

추가 운영 문서:

- `/Users/joonwoo/Documents/GitHub/Soongsil-MCP/mcp/soongsil-mcp/README.md`
- `/Users/joonwoo/Documents/GitHub/Soongsil-MCP/mcp/soongsil-mcp/references/md-corpus.md`

## Troubleshooting

`bad interpreter` 에러(예: `.venv/bin/pip: .../Soongsil/.venv/bin/python...`)가 나면
예전 경로에서 생성된 가상환경이 남아있는 상태입니다.

```bash
deactivate 2>/dev/null || true
cd /Users/joonwoo/Documents/GitHub/Soongsil-MCP/mcp/soongsil-mcp
rm -rf .venv
python3 -m venv .venv
source .venv/bin/activate
python -m pip install -e .
```
