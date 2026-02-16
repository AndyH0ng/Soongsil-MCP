#!/usr/bin/env python3
"""
Convert PDFs into page-indexed Markdown files.

The output preserves page boundaries so downstream rules can cite
original evidence as (document, p.N).
"""

from __future__ import annotations

import argparse
import re
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path


def extract_pdf_text(pdf_path: Path) -> str:
    cmd = ["pdftotext", "-layout", str(pdf_path), "-"]
    result = subprocess.run(cmd, capture_output=True, text=True)
    if result.returncode != 0:
        message = result.stderr.strip() or "pdftotext failed"
        raise RuntimeError(message)
    return result.stdout.replace("\r\n", "\n")


def split_pages(text: str) -> list[str]:
    pages = text.split("\f")
    if pages and not pages[-1].strip():
        pages = pages[:-1]
    return pages


_NOISE_TO_SPACE = str.maketrans(
    {
        "\u00a0": " ",  # non-breaking space
        "!": " ",
        "%": " ",
        "&": " ",
        "*": " ",
    }
)


def normalize_token(token: str) -> str:
    token = token.translate(_NOISE_TO_SPACE)
    token = re.sub(r"(\d)\s*\$\s*(\d)", r"\1-\2", token)
    token = token.replace("#", ", ")
    token = re.sub(r"\s+", " ", token).strip()
    return token


def normalize_readable_text(page_text: str) -> str:
    lines: list[str] = []
    text = page_text.replace("\r\n", "\n")
    for raw_line in text.splitlines():
        line = normalize_token(raw_line)
        if not line:
            if lines and lines[-1] != "":
                lines.append("")
            continue
        lines.append(line)
    while lines and lines[-1] == "":
        lines.pop()
    return "\n".join(lines).strip()


def split_columns(line: str) -> list[str]:
    parts = [part for part in re.split(r"\s{2,}", line.strip()) if part.strip()]
    return [normalize_token(part) for part in parts if normalize_token(part)]


def looks_like_table_row(parts: list[str]) -> bool:
    if len(parts) < 3:
        return False
    numeric_cells = sum(bool(re.search(r"\d", cell)) for cell in parts)
    short_cells = sum(len(cell) <= 4 for cell in parts)
    return numeric_cells >= 1 or short_cells >= 2


def extract_table_blocks(page_text: str) -> list[list[list[str]]]:
    blocks: list[list[list[str]]] = []
    current: list[list[str]] = []

    for raw_line in page_text.replace("\r\n", "\n").splitlines():
        parts = split_columns(raw_line.translate(_NOISE_TO_SPACE))
        if looks_like_table_row(parts):
            current.append(parts)
            continue

        if current:
            if len(current) >= 3:
                blocks.append(current)
            current = []

    if current and len(current) >= 3:
        blocks.append(current)

    filtered: list[list[list[str]]] = []
    for block in blocks:
        max_cols = max(len(row) for row in block)
        if 3 <= max_cols <= 10:
            filtered.append(block)
    return filtered


def escape_md_cell(text: str) -> str:
    return text.replace("|", "\\|").strip()


def render_table_block(block: list[list[str]], index: int) -> list[str]:
    max_cols = max(len(row) for row in block)
    padded = [row + [""] * (max_cols - len(row)) for row in block]

    first_row = padded[0]
    first_row_has_digit = any(re.search(r"\d", cell) for cell in first_row)
    if not first_row_has_digit and len(padded) >= 2:
        header = first_row
        rows = padded[1:]
    else:
        header = ["항목"] + [f"값{i}" for i in range(1, max_cols)]
        rows = padded

    lines: list[str] = []
    lines.append(f"### table-{index}")
    lines.append("")
    lines.append("| " + " | ".join(escape_md_cell(cell) for cell in header) + " |")
    lines.append("| " + " | ".join("---" for _ in header) + " |")
    for row in rows:
        lines.append("| " + " | ".join(escape_md_cell(cell) for cell in row) + " |")
    lines.append("")
    return lines


def render_markdown(pdf_path: Path, pages: list[str]) -> str:
    generated_at = datetime.now(timezone.utc).isoformat()
    lines: list[str] = []
    lines.append(f"# {pdf_path.stem}")
    lines.append("")
    lines.append(f"- source_pdf: `{pdf_path}`")
    lines.append(f"- generated_at_utc: `{generated_at}`")
    lines.append(f"- total_pages: `{len(pages)}`")
    lines.append("")

    for idx, page in enumerate(pages, start=1):
        lines.append(f"## p.{idx}")
        lines.append("")
        page_text = page.strip("\n")
        if not page_text.strip():
            lines.append("```text")
            lines.append("(blank page)")
            lines.append("```")
            lines.append("")
            continue

        table_blocks = extract_table_blocks(page_text)
        if table_blocks:
            lines.append("### tables")
            lines.append("")
            for table_index, block in enumerate(table_blocks, start=1):
                lines.extend(render_table_block(block, table_index))

        readable = normalize_readable_text(page_text)
        lines.append("### text")
        lines.append("")
        lines.append("```text")
        lines.append(readable or "(blank page)")
        lines.append("```")
        lines.append("")

    return "\n".join(lines).rstrip() + "\n"


def convert_one(pdf_path: Path, output_dir: Path) -> Path:
    text = extract_pdf_text(pdf_path)
    pages = split_pages(text)
    md_content = render_markdown(pdf_path, pages)
    output_path = output_dir / f"{pdf_path.stem}.md"
    output_path.write_text(md_content, encoding="utf-8")
    return output_path


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Convert PDF files to page-indexed markdown."
    )
    parser.add_argument(
        "--input-dir",
        default="/Users/joonwoo/Documents/GitHub/Soongsil/docs",
        help="Directory containing PDFs (default: project docs directory)",
    )
    parser.add_argument(
        "--output-dir",
        default="/Users/joonwoo/Documents/GitHub/Soongsil/knowledge/raw-md",
        help="Directory for markdown output (default: knowledge/raw-md)",
    )
    parser.add_argument(
        "--glob",
        default="*.pdf",
        help="Glob for PDF selection inside --input-dir (default: *.pdf)",
    )
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    input_dir = Path(args.input_dir).expanduser().resolve()
    output_dir = Path(args.output_dir).expanduser().resolve()

    if not input_dir.exists() or not input_dir.is_dir():
        print(f"[ERROR] input directory not found: {input_dir}", file=sys.stderr)
        return 1

    output_dir.mkdir(parents=True, exist_ok=True)
    pdfs = sorted(p for p in input_dir.glob(args.glob) if p.is_file())
    if not pdfs:
        print(f"[ERROR] no PDF files matched in {input_dir}", file=sys.stderr)
        return 1

    failures = 0
    for pdf in pdfs:
        try:
            output = convert_one(pdf, output_dir)
            print(f"[OK] {pdf.name} -> {output}")
        except Exception as exc:
            failures += 1
            print(f"[ERROR] {pdf.name}: {exc}", file=sys.stderr)

    if failures:
        print(f"[DONE] completed with {failures} failure(s)", file=sys.stderr)
        return 1
    print("[DONE] all PDFs converted")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
