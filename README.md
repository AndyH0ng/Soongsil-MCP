# Soongsil MCP

숭실대학교 학칙·학사 문서를 근거로 질의응답, 졸업요건 판정, 장학 기준 확인을 지원하는 MCP 서버입니다.

현재 구조는 **MCP 단일 운영**이며, 기존 Skill 기반 구성은 제거되었습니다.

## Repository Layout

- `docs/`: 원본 근거 PDF
- `knowledge/normalized-md/`: 질의응답용 정규화 코퍼스
- `knowledge/raw-md/`: 원문 fallback (`학칙.raw.md` 유지)
- `mcp/soongsil-mcp/`: MCP 서버 구현 및 참조 규칙

## Quick Start

```bash
cd /Users/joonwoo/Documents/GitHub/Soongsil/mcp/soongsil-mcp
python3 -m venv .venv
source .venv/bin/activate
pip install -e .
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
      "command": "/Users/joonwoo/Documents/GitHub/Soongsil/mcp/soongsil-mcp/.venv/bin/python",
      "args": [
        "/Users/joonwoo/Documents/GitHub/Soongsil/mcp/soongsil-mcp/server.py"
      ]
    }
  }
}
```

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
python3 /Users/joonwoo/Documents/GitHub/Soongsil/mcp/soongsil-mcp/scripts/convert_pdfs_to_md.py
```

추가 운영 문서:

- `/Users/joonwoo/Documents/GitHub/Soongsil/mcp/soongsil-mcp/README.md`
- `/Users/joonwoo/Documents/GitHub/Soongsil/mcp/soongsil-mcp/references/md-corpus.md`
