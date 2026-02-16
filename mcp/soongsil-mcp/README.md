# Soongsil MCP

숭실대학교 학칙·학사 문서를 근거로 질의응답, 졸업요건 판정, 장학 기준 확인을 지원하는 `stdio` MCP 서버입니다.

## 제공 도구

- `ssu_classify_request`: 질문을 워크플로우 카테고리로 분류
- `ssu_search_evidence`: 관련 문서 라인/페이지 힌트 검색
- `ssu_rule_brief`: 분류 + 근거 + 응답 템플릿을 한 번에 반환
- `ssu_evaluate_graduation`: 학점 이수 체계 기반 졸업요건 계산(초안)
- `ssu_get_calendar_events`: 학사일정 조회
- `ssu_check_scholarship_threshold`: 장학 역치 비교(기준 미제공 시 판정 불가)
- `ssu_list_sources`: 사용 중인 소스 파일 목록

## 로컬 실행

1. 의존성 설치

```bash
cd /Users/joonwoo/Documents/GitHub/Soongsil/mcp/soongsil-mcp
python3 -m venv .venv
source .venv/bin/activate
pip install -e .
```

`mcp.server.fastmcp` import 오류가 나는 환경에서는 아래를 추가 설치:

```bash
pip install fastmcp
```

2. 서버 실행

```bash
python /Users/joonwoo/Documents/GitHub/Soongsil/mcp/soongsil-mcp/server.py
```

## Claude Desktop 설정 예시

Claude Desktop `claude_desktop_config.json`의 `mcpServers`에 추가:

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

## 데이터 소스

- PDFs: `/Users/joonwoo/Documents/GitHub/Soongsil/docs`
- 정규화 MD: `/Users/joonwoo/Documents/GitHub/Soongsil/knowledge/normalized-md`
- 원문 fallback: `/Users/joonwoo/Documents/GitHub/Soongsil/knowledge/raw-md/학칙.raw.md`
- MCP 참조 규칙: `/Users/joonwoo/Documents/GitHub/Soongsil/mcp/soongsil-mcp/references`

## 운영 규칙

- 기본 탐색 순서: `normalized-md -> references -> raw -> PDF`
- 최종 답변은 반드시 `(문서명.pdf, p.N)` 형식 인용
- 장학 정량 기준 미제공 시 `판정 불가(근거 문서 없음)` 처리
