# MD Corpus Guide

## Purpose

Use a two-layer markdown corpus:

- `normalized-md` for fast, structured retrieval.
- `raw-md` for 학칙 원문 백업(`학칙.raw.md`) fallback only.

## Paths

Normalized first-pass:

- `/Users/joonwoo/Documents/GitHub/Soongsil/knowledge/normalized-md/학칙.md`
- `/Users/joonwoo/Documents/GitHub/Soongsil/knowledge/normalized-md/학점 이수 체계.md`
- `/Users/joonwoo/Documents/GitHub/Soongsil/knowledge/normalized-md/교양 필수.md`
- `/Users/joonwoo/Documents/GitHub/Soongsil/knowledge/normalized-md/교양 선택.md`
- `/Users/joonwoo/Documents/GitHub/Soongsil/knowledge/normalized-md/학사 일정.md`

Raw fallback:

- `/Users/joonwoo/Documents/GitHub/Soongsil/knowledge/raw-md/학칙.raw.md`

## Update Workflow

1. Regenerate raw-md from PDF:

```bash
python3 /Users/joonwoo/Documents/GitHub/Soongsil/mcp/soongsil-mcp/scripts/convert_pdfs_to_md.py
```

2. Refresh normalized-md using:
- `/Users/joonwoo/Documents/GitHub/Soongsil/mcp/soongsil-mcp/references/normalization-rules.md`
- `/Users/joonwoo/Documents/GitHub/Soongsil/mcp/soongsil-mcp/references/normalization-template.md`
- `/Users/joonwoo/Documents/GitHub/Soongsil/mcp/soongsil-mcp/references/normalization-checklist.md`

3. For 학칙 질의, keep `law-topic-index.md`, `law-articles.md`, `law-numeric-rules.md` in sync with normalized changes.

## Evidence Rule

- locate candidate rule in normalized-md first;
- if ambiguous, verify PDF first; for 학칙 세부 문구는 `학칙.raw.md` 대조;
- finalize with PDF page citation: `(문서명.pdf, p.N)`.

## Freshness Rule

When user asks for latest policy:

1. check PDF refresh date first;
2. if uncertain, request latest PDF export;
3. refresh normalized-md (and `학칙.raw.md` only when needed);
4. answer with explicit document date context.
