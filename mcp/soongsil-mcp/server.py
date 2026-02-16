#!/usr/bin/env python3
from __future__ import annotations

import re
from functools import lru_cache
from pathlib import Path
from typing import Any

try:
    from mcp.server.fastmcp import FastMCP
except ImportError:  # pragma: no cover
    # Fallback for environments that expose FastMCP as standalone package.
    from fastmcp import FastMCP


SERVER_NAME = "Soongsil MCP"
mcp = FastMCP(SERVER_NAME)

REPO_ROOT = Path(__file__).resolve().parents[2]
DOCS_DIR = REPO_ROOT / "docs"
KNOWLEDGE_DIR = REPO_ROOT / "knowledge"
NORMALIZED_DIR = KNOWLEDGE_DIR / "normalized-md"
RAW_DIR = KNOWLEDGE_DIR / "raw-md"
REFERENCES_DIR = REPO_ROOT / "mcp" / "soongsil-mcp" / "references"


CATEGORY_KEYWORDS: dict[str, list[str]] = {
    "학칙 Q&A": [
        "학칙",
        "휴학",
        "복학",
        "제적",
        "자퇴",
        "전과",
        "다전공",
        "학사경고",
        "징계",
        "조문",
    ],
    "졸업요건 판정": [
        "졸업",
        "졸업요건",
        "이수",
        "학점",
        "전공기초",
        "복수전공",
        "부전공",
    ],
    "재수강 가능/영향 분석": ["재수강", "중복", "성적", "학점인정"],
    "장학 기준 역치 비교": ["장학", "장학금", "성적우수", "역치", "threshold"],
    "수강신청/학사일정 보조": [
        "수강신청",
        "학사일정",
        "정정",
        "취소",
        "신청기간",
        "등록금",
        "마감",
    ],
}

STOP_TERMS = {
    "은",
    "는",
    "이",
    "가",
    "을",
    "를",
    "에",
    "의",
    "좀",
    "해줘",
    "가능",
    "여부",
    "알려줘",
    "확인",
    "하고",
    "또",
}


def _read_text(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def _safe_int(value: str) -> int | None:
    value = value.strip()
    if not value or value in {"-", "불허"}:
        return None
    match = re.search(r"\d+", value)
    if not match:
        return None
    return int(match.group(0))


def _extract_terms(question: str) -> list[str]:
    terms = re.findall(r"[가-힣A-Za-z0-9]+", question)
    lowered = []
    for term in terms:
        t = term.strip().lower()
        if len(t) < 2 or t in STOP_TERMS:
            continue
        lowered.append(t)
    # preserve order and de-duplicate
    seen: set[str] = set()
    unique_terms: list[str] = []
    for t in lowered:
        if t in seen:
            continue
        seen.add(t)
        unique_terms.append(t)
    return unique_terms


def _classify(question: str) -> tuple[str, list[str]]:
    q = question.lower()
    best_category = "학칙 Q&A"
    best_score = -1
    matched_keywords: list[str] = []
    for category, keywords in CATEGORY_KEYWORDS.items():
        hits = [kw for kw in keywords if kw.lower() in q]
        score = len(hits)
        if score > best_score:
            best_score = score
            best_category = category
            matched_keywords = hits
    if best_score <= 0:
        return ("학칙 Q&A", [])
    return (best_category, matched_keywords)


def _category_paths(category: str) -> list[Path]:
    base: dict[str, list[Path]] = {
        "학칙 Q&A": [
            NORMALIZED_DIR / "학칙.md",
            REFERENCES_DIR / "law-topic-index.md",
            REFERENCES_DIR / "law-articles.md",
            REFERENCES_DIR / "law-numeric-rules.md",
            RAW_DIR / "학칙.raw.md",
        ],
        "졸업요건 판정": [
            NORMALIZED_DIR / "학점 이수 체계.md",
            NORMALIZED_DIR / "교양 필수.md",
            NORMALIZED_DIR / "교양 선택.md",
        ],
        "재수강 가능/영향 분석": [
            NORMALIZED_DIR / "학칙.md",
            NORMALIZED_DIR / "교양 필수.md",
            RAW_DIR / "학칙.raw.md",
        ],
        "장학 기준 역치 비교": [
            REFERENCES_DIR / "source-map.md",
            REFERENCES_DIR / "law-numeric-rules.md",
            NORMALIZED_DIR / "학칙.md",
        ],
        "수강신청/학사일정 보조": [
            NORMALIZED_DIR / "학사 일정.md",
            NORMALIZED_DIR / "학칙.md",
        ],
    }
    return [p for p in base.get(category, []) if p.exists()]


def _citation_hint(path: Path, page: str | None) -> str:
    name = path.name
    if name in {"학칙.md", "학칙.raw.md", "law-topic-index.md", "law-articles.md", "law-numeric-rules.md"}:
        if page:
            return f"(학칙.pdf, p.{page})"
        return "(학칙.pdf, 페이지 확인 필요)"
    if name == "학점 이수 체계.md":
        return "(학점 이수 체계.pdf, p.1)"
    if name == "교양 필수.md":
        return "(교양 필수.pdf, p.1~2)"
    if name == "교양 선택.md":
        return "(교양 선택.pdf, p.1~4)"
    if name == "학사 일정.md":
        if page:
            return f"(학사 일정.pdf, p.{page})"
        return "(학사 일정.pdf, 페이지 확인 필요)"
    return "(원문 PDF 페이지 확인 필요)"


def _iter_lines_with_page(path: Path) -> list[tuple[int, str, str | None]]:
    lines = _read_text(path).splitlines()
    current_page: str | None = None
    output: list[tuple[int, str, str | None]] = []
    for idx, line in enumerate(lines, start=1):
        page_match = re.match(r"^## p\.(\d+)", line.strip())
        if page_match:
            current_page = page_match.group(1)
        output.append((idx, line, current_page))
    return output


def _search_in_file(path: Path, terms: list[str], max_hits: int) -> list[dict[str, Any]]:
    hits: list[dict[str, Any]] = []
    if not terms:
        return hits
    for line_no, line, page in _iter_lines_with_page(path):
        candidate = line.strip()
        if not candidate:
            continue
        low = candidate.lower()
        matched = [term for term in terms if term in low]
        if not matched:
            continue
        hits.append(
            {
                "file": str(path),
                "line": line_no,
                "page": page,
                "snippet": candidate[:240],
                "matched_terms": matched,
                "score": len(matched),
                "citation_hint": _citation_hint(path, page),
            }
        )
        if len(hits) >= max_hits:
            break
    return hits


@lru_cache(maxsize=1)
def _load_credit_rows() -> list[dict[str, str]]:
    path = NORMALIZED_DIR / "학점 이수 체계.md"
    text = _read_text(path)
    lines = text.splitlines()
    start = -1
    for i, line in enumerate(lines):
        if line.strip().startswith("| 대학 | 학과/학부 |"):
            start = i
            break
    if start < 0:
        return []

    headers = [cell.strip() for cell in lines[start].strip().strip("|").split("|")]
    rows: list[dict[str, str]] = []
    for line in lines[start + 2 :]:
        s = line.strip()
        if not s.startswith("|"):
            if rows:
                break
            continue
        cells = [cell.strip() for cell in s.strip("|").split("|")]
        if len(cells) != len(headers):
            continue
        row = dict(zip(headers, cells))
        rows.append(row)
    return rows


def _match_credit_row(college: str, department: str) -> dict[str, str] | None:
    rows = _load_credit_rows()
    if not rows:
        return None
    best: tuple[int, dict[str, str] | None] = (-1, None)
    college = college.strip()
    department = department.strip()

    for row in rows:
        row_college = row.get("대학", "").strip()
        row_dept = row.get("학과/학부", "").strip()
        score = 0

        if not row_college:
            continue
        if row_college == college:
            score += 20
        elif college and college in row_college:
            score += 12
        else:
            continue

        if row_dept == department:
            score += 20
        elif department and department in row_dept:
            score += 14
        elif "전체" in row_dept:
            score += 8
        elif "외" in row_dept:
            score += 6
        elif not department:
            score += 4

        if score > best[0]:
            best = (score, row)

    return best[1]


def _parse_calendar_rows() -> list[dict[str, str]]:
    path = NORMALIZED_DIR / "학사 일정.md"
    lines = _read_text(path).splitlines()
    rows: list[dict[str, str]] = []
    page: str | None = None
    in_table = False
    for line in lines:
        stripped = line.strip()
        page_match = re.match(r"^## p\.(\d+)", stripped)
        if page_match:
            page = page_match.group(1)

        if stripped.startswith("| 기간 | 일정 |"):
            in_table = True
            continue
        if in_table and stripped.startswith("| --- | --- |"):
            continue
        if in_table and stripped.startswith("|"):
            cells = [c.strip() for c in stripped.strip("|").split("|")]
            if len(cells) >= 2:
                rows.append({"기간": cells[0], "일정": cells[1], "page": page or ""})
            continue
        if in_table and not stripped:
            in_table = False
    return rows


def _extract_month(period: str) -> int | None:
    match = re.search(r"(\d{2})-\d{2}", period)
    if not match:
        return None
    return int(match.group(1))


@mcp.tool(name="ssu_classify_request")
def ssu_classify_request(question: str) -> dict[str, Any]:
    """학사 질의를 5개 워크플로우 카테고리로 분류한다."""
    category, matched = _classify(question)
    return {
        "question": question,
        "category": category,
        "matched_keywords": matched,
        "recommended_paths": [str(p) for p in _category_paths(category)],
    }


@mcp.tool(name="ssu_search_evidence")
def ssu_search_evidence(
    question: str,
    category: str | None = None,
    max_hits: int = 12,
) -> dict[str, Any]:
    """질문과 관련된 근거 라인을 검색하고 문서/페이지 힌트를 반환한다."""
    selected_category = category or _classify(question)[0]
    terms = _extract_terms(question)
    paths = _category_paths(selected_category)
    all_hits: list[dict[str, Any]] = []
    for path in paths:
        hits = _search_in_file(path, terms, max_hits=max_hits)
        all_hits.extend(hits)

    all_hits.sort(key=lambda item: item["score"], reverse=True)
    all_hits = all_hits[: max_hits if max_hits > 0 else 12]
    return {
        "question": question,
        "category": selected_category,
        "search_terms": terms,
        "hits": all_hits,
        "citation_rule": "최종 답변은 반드시 (문서명.pdf, p.N) 형식으로 표기",
    }


@mcp.tool(name="ssu_rule_brief")
def ssu_rule_brief(question: str, max_hits: int = 10) -> dict[str, Any]:
    """분류 + 근거검색 + 응답 골격을 한 번에 반환한다."""
    category, matched = _classify(question)
    evidence = ssu_search_evidence(question=question, category=category, max_hits=max_hits)
    return {
        "question": question,
        "category": category,
        "matched_keywords": matched,
        "workflow": [
            "1) normalized-md에서 후보 규정 탐색",
            "2) law-topic-index/law-articles/law-numeric-rules 교차확인(학칙 질의 시)",
            "3) 필요 시 raw-md/학칙.raw.md 대조",
            "4) PDF 페이지 인용으로 최종 확정",
        ],
        "response_template": ["결론", "근거", "계산/비교", "불확실성"],
        "evidence": evidence.get("hits", []),
    }


@mcp.tool(name="ssu_evaluate_graduation")
def ssu_evaluate_graduation(
    college: str,
    department: str,
    major_type: str,
    earned_liberal_required: int,
    earned_liberal_elective: int,
    earned_major_basic: int,
    earned_major: int,
    earned_total: int,
) -> dict[str, Any]:
    """학점 이수 체계 기준으로 졸업요건 충족 여부(초안)를 계산한다."""
    row = _match_credit_row(college=college, department=department)
    if row is None:
        return {
            "judgement": "판정 불가",
            "reason": "학점 이수 체계 표에서 일치하는 대학/학과 행을 찾지 못함",
            "citation": "(학점 이수 체계.pdf, p.1)",
        }

    major_type_col = major_type.strip()
    if major_type_col not in {
        "단일전공자",
        "부전공자",
        "복수전공자(주전공)",
        "복수전공자(복수전공)",
    }:
        return {
            "judgement": "판정 불가",
            "reason": "major_type은 단일전공자/부전공자/복수전공자(주전공)/복수전공자(복수전공) 중 하나여야 함",
            "citation": "(학점 이수 체계.pdf, p.1)",
        }

    raw_major_req = row.get(major_type_col, "")
    major_req = _safe_int(raw_major_req)
    if raw_major_req.strip() == "불허":
        return {
            "judgement": "불가",
            "reason": f"{major_type_col} 경로가 해당 학과에서 불허됨",
            "matched_rule": row,
            "citation": "(학점 이수 체계.pdf, p.1)",
        }

    req_liberal_required = _safe_int(row.get("교양필수", "")) or 19
    req_liberal_elective = _safe_int(row.get("교양선택", "")) or 9
    req_major_basic = _safe_int(row.get("전공기초", "")) or 0
    req_total = _safe_int(row.get("졸업학점", "")) or 133

    req_major = major_req or 0

    gaps = {
        "교양필수": max(req_liberal_required - earned_liberal_required, 0),
        "교양선택": max(req_liberal_elective - earned_liberal_elective, 0),
        "전공기초": max(req_major_basic - earned_major_basic, 0),
        major_type_col: max(req_major - earned_major, 0),
        "졸업학점": max(req_total - earned_total, 0),
    }
    total_gap = sum(gaps.values())
    judgement = "가능" if total_gap == 0 else "불가"

    return {
        "judgement": judgement,
        "matched_rule": row,
        "required": {
            "교양필수": req_liberal_required,
            "교양선택": req_liberal_elective,
            "전공기초": req_major_basic,
            major_type_col: req_major,
            "졸업학점": req_total,
        },
        "earned": {
            "교양필수": earned_liberal_required,
            "교양선택": earned_liberal_elective,
            "전공기초": earned_major_basic,
            major_type_col: earned_major,
            "졸업학점": earned_total,
        },
        "gap": gaps,
        "citation": "(학점 이수 체계.pdf, p.1)",
        "notes": [
            "교양필수/교양선택의 세부 과목 충족 여부는 교양 필수/선택 문서로 추가 확인 필요",
            "최종 졸업판정은 학칙 졸업요건 조문과 함께 검증 권장",
        ],
    }


@mcp.tool(name="ssu_get_calendar_events")
def ssu_get_calendar_events(
    keyword: str = "",
    month: int | None = None,
    limit: int = 20,
) -> dict[str, Any]:
    """학사 일정에서 일정 항목을 조회한다."""
    rows = _parse_calendar_rows()
    key = keyword.strip().lower()

    filtered: list[dict[str, Any]] = []
    for row in rows:
        period = row["기간"]
        event = row["일정"]
        if month is not None:
            m = _extract_month(period)
            if m != month:
                continue
        if key and key not in (period + " " + event).lower():
            continue
        filtered.append(
            {
                "기간": period,
                "일정": event,
                "citation": f"(학사 일정.pdf, p.{row['page']})" if row["page"] else "(학사 일정.pdf, 페이지 확인 필요)",
            }
        )
        if len(filtered) >= max(limit, 1):
            break

    return {
        "keyword": keyword,
        "month": month,
        "count": len(filtered),
        "events": filtered,
    }


@mcp.tool(name="ssu_check_scholarship_threshold")
def ssu_check_scholarship_threshold(
    gpa: float | None = None,
    earned_credits: int | None = None,
    min_gpa: float | None = None,
    min_credits: int | None = None,
) -> dict[str, Any]:
    """장학 역치 비교. 기준값 미제공 시 '판정 불가'를 반환한다."""
    if min_gpa is None and min_credits is None:
        return {
            "judgement": "판정 불가",
            "reason": "현재 docs 묶음에는 장학금 정량 선발기준 문서가 없음",
            "required_action": "장학 규정 PDF/URL 제공 필요",
            "citation": "(학칙.pdf, p.14) + source-map known gap",
        }

    gaps: dict[str, float | int] = {}
    if min_gpa is not None and gpa is not None:
        gaps["gpa_gap"] = round(gpa - min_gpa, 3)
    if min_credits is not None and earned_credits is not None:
        gaps["credit_gap"] = earned_credits - min_credits

    meets_gpa = (min_gpa is None) or (gpa is not None and gpa >= min_gpa)
    meets_credits = (min_credits is None) or (earned_credits is not None and earned_credits >= min_credits)

    return {
        "judgement": "가능" if (meets_gpa and meets_credits) else "불가",
        "input": {
            "gpa": gpa,
            "earned_credits": earned_credits,
            "min_gpa": min_gpa,
            "min_credits": min_credits,
        },
        "gap": gaps,
        "notes": [
            "이 결과는 사용자가 제공한 역치 기준값에 대한 비교임",
            "학교 공식 장학 세부기준 문서로 최종 확인 필요",
        ],
    }


@mcp.tool(name="ssu_list_sources")
def ssu_list_sources() -> dict[str, Any]:
    """서버가 참조하는 주요 소스 파일 경로를 반환한다."""
    return {
        "docs": [
            str(DOCS_DIR / "학칙.pdf"),
            str(DOCS_DIR / "학점 이수 체계.pdf"),
            str(DOCS_DIR / "교양 필수.pdf"),
            str(DOCS_DIR / "교양 선택.pdf"),
            str(DOCS_DIR / "학사 일정.pdf"),
        ],
        "normalized_md": [str(p) for p in sorted(NORMALIZED_DIR.glob("*.md"))],
        "raw_md": [str(p) for p in sorted(RAW_DIR.glob("*.md"))],
        "references": [str(p) for p in sorted(REFERENCES_DIR.glob("*.md"))],
    }


def main() -> None:
    mcp.run()


if __name__ == "__main__":
    main()
