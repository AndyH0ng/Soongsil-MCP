# Raw Markdown Corpus

This folder stores raw markdown backups converted from PDFs in `/Users/joonwoo/Documents/GitHub/Soongsil/docs`.

Current policy:
- primary working corpus is `/Users/joonwoo/Documents/GitHub/Soongsil/knowledge/normalized-md`;
- this folder keeps only `학칙.raw.md` as full-text fallback backup.
- table-normalized backup is stored at `/Users/joonwoo/Documents/GitHub/Soongsil/knowledge/normalized-md/학칙.table-normalized.md`.

## Regenerate

```bash
python3 /Users/joonwoo/Documents/GitHub/Soongsil/mcp/soongsil-mcp/scripts/convert_pdfs_to_md.py
```

## Format

- Full-text fallback file: `학칙.raw.md`.
- Regeneration may output additional files, but they are not required for daily operation.

Use this folder only when clause-level wording in 학칙 needs additional verification.  
For final answers, always cite the original PDF pages.
