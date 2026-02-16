#!/usr/bin/env python3
"""
Search a PDF for keywords and print page-cited snippets.

Dependency:
  - pdftotext (Poppler)
"""

from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
from pathlib import Path


def extract_pdf_text(pdf_path: Path) -> str:
    command = ["pdftotext", "-layout", str(pdf_path), "-"]
    result = subprocess.run(command, capture_output=True, text=True)
    if result.returncode != 0:
        raise RuntimeError(result.stderr.strip() or "pdftotext failed")
    return result.stdout.replace("\r\n", "\n")


def split_pages(text: str) -> list[str]:
    pages = text.split("\f")
    if pages and not pages[-1].strip():
        pages = pages[:-1]
    return pages


def build_pattern(query: str, use_regex: bool, case_sensitive: bool) -> re.Pattern[str]:
    flags = 0 if case_sensitive else re.IGNORECASE
    if use_regex:
        return re.compile(query, flags)
    return re.compile(re.escape(query), flags)


def search_pages(
    pages: list[str],
    pattern: re.Pattern[str],
    context_chars: int,
    max_hits: int,
) -> list[dict[str, object]]:
    hits: list[dict[str, object]] = []
    for page_index, page in enumerate(pages, start=1):
        for line_index, line in enumerate(page.splitlines(), start=1):
            match = pattern.search(line)
            if not match:
                continue
            left = max(0, match.start() - context_chars)
            right = min(len(line), match.end() + context_chars)
            snippet = line[left:right].strip()
            hits.append(
                {
                    "page": page_index,
                    "line": line_index,
                    "match": match.group(0),
                    "snippet": snippet,
                }
            )
            if len(hits) >= max_hits:
                return hits
    return hits


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Find keyword matches in a PDF and return page-cited snippets.",
    )
    parser.add_argument("--pdf", required=True, help="Path to PDF")
    parser.add_argument("--query", required=True, help="Keyword or regex pattern")
    parser.add_argument(
        "--regex",
        action="store_true",
        help="Interpret --query as regex",
    )
    parser.add_argument(
        "--case-sensitive",
        action="store_true",
        help="Use case-sensitive search",
    )
    parser.add_argument(
        "--max-hits",
        type=int,
        default=20,
        help="Maximum number of hits to return (default: 20)",
    )
    parser.add_argument(
        "--context",
        type=int,
        default=50,
        help="Characters of context around each match (default: 50)",
    )
    parser.add_argument(
        "--json",
        action="store_true",
        help="Print JSON output",
    )
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    pdf_path = Path(args.pdf).expanduser().resolve()
    if not pdf_path.exists():
        print(f"[ERROR] PDF not found: {pdf_path}", file=sys.stderr)
        return 1
    if args.max_hits <= 0:
        print("[ERROR] --max-hits must be > 0", file=sys.stderr)
        return 1
    if args.context < 0:
        print("[ERROR] --context must be >= 0", file=sys.stderr)
        return 1

    try:
        text = extract_pdf_text(pdf_path)
        pages = split_pages(text)
        pattern = build_pattern(args.query, args.regex, args.case_sensitive)
        hits = search_pages(pages, pattern, args.context, args.max_hits)
    except re.error as error:
        print(f"[ERROR] Invalid regex: {error}", file=sys.stderr)
        return 1
    except RuntimeError as error:
        print(f"[ERROR] {error}", file=sys.stderr)
        return 1

    payload = {
        "pdf": str(pdf_path),
        "query": args.query,
        "total_pages": len(pages),
        "hit_count": len(hits),
        "hits": hits,
    }

    if args.json:
        print(json.dumps(payload, ensure_ascii=False, indent=2))
        return 0

    print(f"PDF: {pdf_path}")
    print(f"Query: {args.query}")
    print(f"Pages: {len(pages)}")
    print(f"Hits: {len(hits)}")
    if not hits:
        print("No matches found.")
        return 0

    for hit in hits:
        page = hit["page"]
        line = hit["line"]
        snippet = hit["snippet"]
        print(f"- p.{page} line {line}: {snippet}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
