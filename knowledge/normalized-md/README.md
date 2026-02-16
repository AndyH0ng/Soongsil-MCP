# Normalized Markdown Corpus

This folder stores cleaned and structured markdown normalized from `/Users/joonwoo/Documents/GitHub/Soongsil/knowledge/raw-md`.

## Purpose

- provide stable tables and section layout for faster Q&A;
- reduce OCR and spacing noise before rule extraction;
- keep source traceability to raw-md and PDF pages.

## File Set

- `/Users/joonwoo/Documents/GitHub/Soongsil/knowledge/normalized-md/학칙.md`
- `/Users/joonwoo/Documents/GitHub/Soongsil/knowledge/normalized-md/학칙.table-normalized.md` (raw 표 정규화 백업본)
- `/Users/joonwoo/Documents/GitHub/Soongsil/knowledge/normalized-md/학점 이수 체계.md`
- `/Users/joonwoo/Documents/GitHub/Soongsil/knowledge/normalized-md/교양 필수.md`
- `/Users/joonwoo/Documents/GitHub/Soongsil/knowledge/normalized-md/교양 선택.md`
- `/Users/joonwoo/Documents/GitHub/Soongsil/knowledge/normalized-md/학사 일정.md`

## Processing Order

1. Convert PDF to raw-md.
2. Normalize raw-md into this folder using:
   - `/Users/joonwoo/Documents/GitHub/Soongsil/mcp/soongsil-mcp/references/normalization-rules.md`
   - `/Users/joonwoo/Documents/GitHub/Soongsil/mcp/soongsil-mcp/references/normalization-template.md`
   - `/Users/joonwoo/Documents/GitHub/Soongsil/mcp/soongsil-mcp/references/normalization-checklist.md`
3. Use normalized-md first for search and calculation.
4. Re-verify final claims in PDF and cite `(문서명.pdf, p.N)`.

## Guardrail

If normalized-md and raw-md conflict, follow `raw-md + PDF` and fix normalized-md.
