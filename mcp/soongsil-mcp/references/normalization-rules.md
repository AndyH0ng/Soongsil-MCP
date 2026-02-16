# MD Normalization Rules

## Purpose

Define a reproducible way to transform `raw-md` into `normalized-md` for fast retrieval while preserving factual accuracy.

## Scope

- input: `/Users/joonwoo/Documents/GitHub/Soongsil/knowledge/raw-md/*.md`
- output: `/Users/joonwoo/Documents/GitHub/Soongsil/knowledge/normalized-md/*.md`
- source of truth: `/Users/joonwoo/Documents/GitHub/Soongsil/docs/*.pdf`

## Canonical Structure

Each normalized file should keep this top metadata:

- `source_pdf`
- `normalized_from_raw_md`
- `normalized_at` (YYYY-MM-DD)
- `last_verified_against_pdf` (YYYY-MM-DD)
- optional `related_references`

## Transformation Rules

1. Keep all numeric values unchanged.
2. Keep legal or policy wording faithful; avoid paraphrasing clauses that alter meaning.
3. Convert list-like values to markdown tables when possible.
4. Normalize noisy separators (`!`, repeated spaces, broken punctuation).
5. Preserve unit labels (`학점`, `학기`, `주`, `등급`) explicitly.
6. For unknown or non-applicable values, use `-` or `불허` exactly as source context indicates.
7. Keep page anchors in section titles such as `## p.N` where page mapping matters.

## Conflict Rule

- If `normalized-md` conflicts with `raw-md`, treat `raw-md` as upstream and verify with PDF.
- After verification, patch `normalized-md` and record the corrected page context.

## Citation Rule

Final user answers must cite PDF pages, not only markdown files:
- `(학칙.pdf, p.7)`
- `(학점 이수 체계.pdf, p.1)`

## Freshness Rule

When PDFs are updated:

1. regenerate raw-md;
2. re-run normalization file-by-file;
3. re-check critical numeric rules (`휴학 한도`, `졸업학점`, `연간 이수학점`, `학사경고`, `장학 관련 조문`).
