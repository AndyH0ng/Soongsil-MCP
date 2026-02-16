use once_cell::sync::Lazy;
use regex::Regex;
use serde_json::{json, Map, Value};
use worker::*;

const SERVER_NAME: &str = "Soongsil MCP Worker (Rust)";
const SERVER_VERSION: &str = "0.1.0";
const DEFAULT_PROTOCOL_VERSION: &str = "2025-03-26";

const PATH_HAKCHIK: &str = "/knowledge/normalized-md/학칙.md";
const PATH_CREDIT_SYSTEM: &str = "/knowledge/normalized-md/학점 이수 체계.md";
const PATH_LIBERAL_REQUIRED: &str = "/knowledge/normalized-md/교양 필수.md";
const PATH_LIBERAL_ELECTIVE: &str = "/knowledge/normalized-md/교양 선택.md";
const PATH_CALENDAR: &str = "/knowledge/normalized-md/학사 일정.md";
const PATH_HAKCHIK_RAW: &str = "/knowledge/raw-md/학칙.raw.md";
const PATH_LAW_TOPIC: &str = "/references/law-topic-index.md";
const PATH_LAW_ARTICLES: &str = "/references/law-articles.md";
const PATH_LAW_NUMERIC: &str = "/references/law-numeric-rules.md";
const PATH_SOURCE_MAP: &str = "/references/source-map.md";

const FILE_HAKCHIK: &str = include_str!("../../../knowledge/normalized-md/학칙.md");
const FILE_CREDIT_SYSTEM: &str = include_str!("../../../knowledge/normalized-md/학점 이수 체계.md");
const FILE_LIBERAL_REQUIRED: &str = include_str!("../../../knowledge/normalized-md/교양 필수.md");
const FILE_LIBERAL_ELECTIVE: &str = include_str!("../../../knowledge/normalized-md/교양 선택.md");
const FILE_CALENDAR: &str = include_str!("../../../knowledge/normalized-md/학사 일정.md");
const FILE_HAKCHIK_RAW: &str = include_str!("../../../knowledge/raw-md/학칙.raw.md");
const FILE_LAW_TOPIC: &str = include_str!("../../../mcp/soongsil-mcp/references/law-topic-index.md");
const FILE_LAW_ARTICLES: &str = include_str!("../../../mcp/soongsil-mcp/references/law-articles.md");
const FILE_LAW_NUMERIC: &str = include_str!("../../../mcp/soongsil-mcp/references/law-numeric-rules.md");
const FILE_SOURCE_MAP: &str = include_str!("../../../mcp/soongsil-mcp/references/source-map.md");

const NORMALIZED_FILES: &[&str] = &[
    "/knowledge/normalized-md/README.md",
    "/knowledge/normalized-md/교양 선택.md",
    "/knowledge/normalized-md/교양 필수.md",
    "/knowledge/normalized-md/학사 일정.md",
    "/knowledge/normalized-md/학점 이수 체계.md",
    "/knowledge/normalized-md/학칙.md",
    "/knowledge/normalized-md/학칙.table-normalized.md",
];

const RAW_FILES: &[&str] = &["/knowledge/raw-md/README.md", "/knowledge/raw-md/학칙.raw.md"];

const REFERENCE_FILES: &[&str] = &[
    "/references/input-template.md",
    "/references/law-articles.md",
    "/references/law-numeric-rules.md",
    "/references/law-topic-index.md",
    "/references/md-corpus.md",
    "/references/normalization-checklist.md",
    "/references/normalization-rules.md",
    "/references/normalization-template.md",
    "/references/qa-checklists.md",
    "/references/source-map.md",
];

static TERM_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"[가-힣A-Za-z0-9]+").expect("TERM_RE compile failure"));
static MONTH_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(\d{2})-\d{2}").expect("MONTH_RE compile failure"));

#[derive(Clone)]
struct SearchHit {
    file: &'static str,
    line: usize,
    page: Option<String>,
    snippet: String,
    matched_terms: Vec<String>,
    score: usize,
    citation_hint: String,
}

#[derive(Clone)]
struct CreditRow {
    values: Map<String, Value>,
}

#[derive(Clone)]
struct CalendarRow {
    period: String,
    event: String,
    page: String,
}

#[derive(Debug)]
struct RpcFailure {
    code: i64,
    message: String,
}

fn rpc_failure(code: i64, message: impl Into<String>) -> RpcFailure {
    RpcFailure {
        code,
        message: message.into(),
    }
}

fn get_file_text(path: &str) -> Option<&'static str> {
    match path {
        PATH_HAKCHIK => Some(FILE_HAKCHIK),
        PATH_CREDIT_SYSTEM => Some(FILE_CREDIT_SYSTEM),
        PATH_LIBERAL_REQUIRED => Some(FILE_LIBERAL_REQUIRED),
        PATH_LIBERAL_ELECTIVE => Some(FILE_LIBERAL_ELECTIVE),
        PATH_CALENDAR => Some(FILE_CALENDAR),
        PATH_HAKCHIK_RAW => Some(FILE_HAKCHIK_RAW),
        PATH_LAW_TOPIC => Some(FILE_LAW_TOPIC),
        PATH_LAW_ARTICLES => Some(FILE_LAW_ARTICLES),
        PATH_LAW_NUMERIC => Some(FILE_LAW_NUMERIC),
        PATH_SOURCE_MAP => Some(FILE_SOURCE_MAP),
        _ => None,
    }
}

fn category_paths(category: &str) -> Vec<&'static str> {
    match category {
        "학칙 Q&A" => vec![
            PATH_HAKCHIK,
            PATH_LAW_TOPIC,
            PATH_LAW_ARTICLES,
            PATH_LAW_NUMERIC,
            PATH_HAKCHIK_RAW,
        ],
        "졸업요건 판정" => vec![PATH_CREDIT_SYSTEM, PATH_LIBERAL_REQUIRED, PATH_LIBERAL_ELECTIVE],
        "재수강 가능/영향 분석" => vec![PATH_HAKCHIK, PATH_LIBERAL_REQUIRED, PATH_HAKCHIK_RAW],
        "장학 기준 역치 비교" => vec![PATH_SOURCE_MAP, PATH_LAW_NUMERIC, PATH_HAKCHIK],
        "수강신청/학사일정 보조" => vec![PATH_CALENDAR, PATH_HAKCHIK],
        _ => vec![PATH_HAKCHIK],
    }
}

fn is_stop_term(term: &str) -> bool {
    matches!(
        term,
        "은"
            | "는"
            | "이"
            | "가"
            | "을"
            | "를"
            | "에"
            | "의"
            | "좀"
            | "해줘"
            | "가능"
            | "여부"
            | "알려줘"
            | "확인"
            | "하고"
            | "또"
    )
}

fn classify(question: &str) -> (String, Vec<String>) {
    let categories: [(&str, &[&str]); 5] = [
        (
            "학칙 Q&A",
            &[
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
        ),
        (
            "졸업요건 판정",
            &["졸업", "졸업요건", "이수", "학점", "전공기초", "복수전공", "부전공"],
        ),
        ("재수강 가능/영향 분석", &["재수강", "중복", "성적", "학점인정"]),
        ("장학 기준 역치 비교", &["장학", "장학금", "성적우수", "역치", "threshold"]),
        (
            "수강신청/학사일정 보조",
            &["수강신청", "학사일정", "정정", "취소", "신청기간", "등록금", "마감"],
        ),
    ];

    let lowered = question.to_lowercase();
    let mut best_category = "학칙 Q&A".to_string();
    let mut best_score = -1_i32;
    let mut best_hits: Vec<String> = Vec::new();

    for (category, keywords) in categories {
        let hits: Vec<String> = keywords
            .iter()
            .filter(|kw| lowered.contains(&kw.to_lowercase()))
            .map(|kw| (*kw).to_string())
            .collect();
        let score = hits.len() as i32;
        if score > best_score {
            best_score = score;
            best_category = category.to_string();
            best_hits = hits;
        }
    }

    if best_score <= 0 {
        ("학칙 Q&A".to_string(), vec![])
    } else {
        (best_category, best_hits)
    }
}

fn extract_terms(question: &str) -> Vec<String> {
    let mut out = Vec::new();
    for mat in TERM_RE.find_iter(question) {
        let term = mat.as_str().trim().to_lowercase();
        if term.len() < 2 || is_stop_term(&term) || out.iter().any(|s| s == &term) {
            continue;
        }
        out.push(term);
    }
    out
}

fn extract_first_int(value: &str) -> Option<i64> {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed == "-" || trimmed == "불허" {
        return None;
    }
    let mut digits = String::new();
    let mut started = false;
    for c in trimmed.chars() {
        if c.is_ascii_digit() {
            digits.push(c);
            started = true;
        } else if started {
            break;
        }
    }
    if digits.is_empty() {
        None
    } else {
        digits.parse::<i64>().ok()
    }
}

fn parse_page_heading(line: &str) -> Option<String> {
    let trimmed = line.trim();
    let suffix = trimmed.strip_prefix("## p.")?;
    let digits: String = suffix.chars().take_while(|c| c.is_ascii_digit()).collect();
    if digits.is_empty() {
        None
    } else {
        Some(digits)
    }
}

fn citation_hint(path: &str, page: Option<&str>) -> String {
    let name = path.rsplit('/').next().unwrap_or(path);
    match name {
        "학칙.md" | "학칙.raw.md" | "law-topic-index.md" | "law-articles.md" | "law-numeric-rules.md" => {
            if let Some(page) = page {
                format!("(학칙.pdf, p.{page})")
            } else {
                "(학칙.pdf, 페이지 확인 필요)".to_string()
            }
        }
        "학점 이수 체계.md" => "(학점 이수 체계.pdf, p.1)".to_string(),
        "교양 필수.md" => "(교양 필수.pdf, p.1~2)".to_string(),
        "교양 선택.md" => "(교양 선택.pdf, p.1~4)".to_string(),
        "학사 일정.md" => {
            if let Some(page) = page {
                format!("(학사 일정.pdf, p.{page})")
            } else {
                "(학사 일정.pdf, 페이지 확인 필요)".to_string()
            }
        }
        _ => "(원문 PDF 페이지 확인 필요)".to_string(),
    }
}

fn search_in_file(path: &'static str, terms: &[String], max_hits: usize) -> Vec<SearchHit> {
    if max_hits == 0 || terms.is_empty() {
        return vec![];
    }

    let mut hits = Vec::new();
    let mut current_page: Option<String> = None;

    if let Some(text) = get_file_text(path) {
        for (idx, line) in text.lines().enumerate() {
            if let Some(page) = parse_page_heading(line) {
                current_page = Some(page);
            }

            let candidate = line.trim();
            if candidate.is_empty() {
                continue;
            }

            let lowered = candidate.to_lowercase();
            let matched_terms: Vec<String> = terms
                .iter()
                .filter(|term| lowered.contains(term.as_str()))
                .cloned()
                .collect();

            if matched_terms.is_empty() {
                continue;
            }

            hits.push(SearchHit {
                file: path,
                line: idx + 1,
                page: current_page.clone(),
                snippet: candidate.chars().take(240).collect(),
                matched_terms: matched_terms.clone(),
                score: matched_terms.len(),
                citation_hint: citation_hint(path, current_page.as_deref()),
            });

            if hits.len() >= max_hits {
                break;
            }
        }
    }

    hits
}

fn split_pipe_row(line: &str) -> Vec<String> {
    line.trim()
        .trim_start_matches('|')
        .trim_end_matches('|')
        .split('|')
        .map(|cell| cell.trim().to_string())
        .collect()
}

fn load_credit_rows() -> Vec<CreditRow> {
    let lines: Vec<&str> = FILE_CREDIT_SYSTEM.lines().collect();
    let mut start = None;
    for (idx, line) in lines.iter().enumerate() {
        if line.trim().starts_with("| 대학 | 학과/학부 |") {
            start = Some(idx);
            break;
        }
    }
    let Some(start_idx) = start else {
        return vec![];
    };

    let headers = split_pipe_row(lines[start_idx]);
    let mut rows = Vec::new();

    for line in lines.iter().skip(start_idx + 2) {
        let stripped = line.trim();
        if !stripped.starts_with('|') {
            if !rows.is_empty() {
                break;
            }
            continue;
        }

        let cells = split_pipe_row(stripped);
        if cells.len() != headers.len() {
            continue;
        }

        let mut row = Map::new();
        for (header, value) in headers.iter().zip(cells.iter()) {
            row.insert(header.clone(), Value::String(value.clone()));
        }
        rows.push(CreditRow { values: row });
    }

    rows
}

fn row_string(row: &CreditRow, key: &str) -> String {
    row.values
        .get(key)
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string()
}

fn match_credit_row(rows: &[CreditRow], college: &str, department: &str) -> Option<CreditRow> {
    let college = college.trim();
    let department = department.trim();
    let mut best_score = -1_i32;
    let mut best_row: Option<CreditRow> = None;

    for row in rows {
        let row_college = row_string(row, "대학");
        let row_department = row_string(row, "학과/학부");
        let mut score = 0_i32;

        if row_college.is_empty() {
            continue;
        }

        if row_college == college {
            score += 20;
        } else if !college.is_empty() && row_college.contains(college) {
            score += 12;
        } else {
            continue;
        }

        if row_department == department {
            score += 20;
        } else if !department.is_empty() && row_department.contains(department) {
            score += 14;
        } else if row_department.contains("전체") {
            score += 8;
        } else if row_department.contains("외") {
            score += 6;
        } else if department.is_empty() {
            score += 4;
        }

        if score > best_score {
            best_score = score;
            best_row = Some(row.clone());
        }
    }

    best_row
}

fn parse_calendar_rows() -> Vec<CalendarRow> {
    let mut rows = Vec::new();
    let mut page = String::new();
    let mut in_table = false;

    for line in FILE_CALENDAR.lines() {
        let stripped = line.trim();
        if let Some(p) = parse_page_heading(stripped) {
            page = p;
        }

        if stripped.starts_with("| 기간 | 일정 |") {
            in_table = true;
            continue;
        }
        if in_table && stripped.starts_with("| --- | --- |") {
            continue;
        }
        if in_table && stripped.starts_with('|') {
            let cells = split_pipe_row(stripped);
            if cells.len() >= 2 {
                rows.push(CalendarRow {
                    period: cells[0].clone(),
                    event: cells[1].clone(),
                    page: page.clone(),
                });
            }
            continue;
        }
        if in_table && stripped.is_empty() {
            in_table = false;
        }
    }

    rows
}

fn extract_month(period: &str) -> Option<i64> {
    let captures = MONTH_RE.captures(period)?;
    captures
        .get(1)
        .and_then(|v| v.as_str().parse::<i64>().ok())
}

fn search_evidence_impl(question: &str, category: &str, max_hits: usize) -> Value {
    let terms = extract_terms(question);
    let paths = category_paths(category);

    let mut all_hits: Vec<SearchHit> = Vec::new();
    for path in paths {
        all_hits.extend(search_in_file(path, &terms, max_hits));
    }

    all_hits.sort_by(|a, b| b.score.cmp(&a.score));
    let clipped = all_hits
        .into_iter()
        .take(if max_hits > 0 { max_hits } else { 12 })
        .collect::<Vec<_>>();

    let hits_json: Vec<Value> = clipped
        .iter()
        .map(|hit| {
            json!({
                "file": hit.file,
                "line": hit.line,
                "page": hit.page,
                "snippet": hit.snippet,
                "matched_terms": hit.matched_terms,
                "score": hit.score,
                "citation_hint": hit.citation_hint,
            })
        })
        .collect();

    json!({
        "question": question,
        "category": category,
        "search_terms": terms,
        "hits": hits_json,
        "citation_rule": "최종 답변은 반드시 (문서명.pdf, p.N) 형식으로 표기"
    })
}

fn number_to_i64(value: &Value) -> Option<i64> {
    if let Some(v) = value.as_i64() {
        return Some(v);
    }
    if let Some(v) = value.as_u64() {
        return i64::try_from(v).ok();
    }
    if let Some(v) = value.as_f64() {
        if (v.fract() - 0.0).abs() < f64::EPSILON {
            return Some(v as i64);
        }
    }
    None
}

fn number_to_f64(value: &Value) -> Option<f64> {
    value.as_f64()
}

fn required_string(
    args: &Map<String, Value>,
    key: &str,
) -> std::result::Result<String, String> {
    args.get(key)
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| format!("'{key}' is required and must be a string"))
}

fn required_i64(args: &Map<String, Value>, key: &str) -> std::result::Result<i64, String> {
    args.get(key)
        .and_then(number_to_i64)
        .ok_or_else(|| format!("'{key}' is required and must be an integer"))
}

fn optional_string(args: &Map<String, Value>, key: &str) -> Option<String> {
    args.get(key).and_then(Value::as_str).map(str::to_string)
}

fn optional_i64(args: &Map<String, Value>, key: &str) -> Option<i64> {
    args.get(key).and_then(number_to_i64)
}

fn optional_f64(args: &Map<String, Value>, key: &str) -> Option<f64> {
    args.get(key).and_then(number_to_f64)
}

fn tool_content(payload: Value) -> Value {
    let text = serde_json::to_string_pretty(&payload).unwrap_or_else(|_| payload.to_string());
    json!({
        "content": [
            {
                "type": "text",
                "text": text
            }
        ]
    })
}

fn tool_error_content(message: &str) -> Value {
    json!({
        "content": [
            {
                "type": "text",
                "text": message
            }
        ],
        "isError": true
    })
}

fn call_tool(
    name: &str,
    args: &Map<String, Value>,
) -> std::result::Result<Value, String> {
    match name {
        "ssu_classify_request" => {
            let question = required_string(args, "question")?;
            let (category, matched_keywords) = classify(&question);
            let paths = category_paths(&category);
            Ok(json!({
                "question": question,
                "category": category,
                "matched_keywords": matched_keywords,
                "recommended_paths": paths,
            }))
        }
        "ssu_search_evidence" => {
            let question = required_string(args, "question")?;
            let category = optional_string(args, "category")
                .unwrap_or_else(|| classify(&question).0.to_string());
            let max_hits = optional_i64(args, "max_hits").unwrap_or(12).max(1) as usize;
            Ok(search_evidence_impl(&question, &category, max_hits))
        }
        "ssu_rule_brief" => {
            let question = required_string(args, "question")?;
            let max_hits = optional_i64(args, "max_hits").unwrap_or(10).max(1) as usize;
            let (category, matched_keywords) = classify(&question);
            let evidence = search_evidence_impl(&question, &category, max_hits);
            let evidence_hits = evidence
                .get("hits")
                .cloned()
                .unwrap_or_else(|| Value::Array(vec![]));

            Ok(json!({
                "question": question,
                "category": category,
                "matched_keywords": matched_keywords,
                "workflow": [
                    "1) normalized-md에서 후보 규정 탐색",
                    "2) law-topic-index/law-articles/law-numeric-rules 교차확인(학칙 질의 시)",
                    "3) 필요 시 raw-md/학칙.raw.md 대조",
                    "4) PDF 페이지 인용으로 최종 확정"
                ],
                "response_template": ["결론", "근거", "계산/비교", "불확실성"],
                "evidence": evidence_hits
            }))
        }
        "ssu_evaluate_graduation" => {
            let college = required_string(args, "college")?;
            let department = optional_string(args, "department").unwrap_or_default();
            let major_type = required_string(args, "major_type")?;
            let earned_liberal_required = required_i64(args, "earned_liberal_required")?;
            let earned_liberal_elective = required_i64(args, "earned_liberal_elective")?;
            let earned_major_basic = required_i64(args, "earned_major_basic")?;
            let earned_major = required_i64(args, "earned_major")?;
            let earned_total = required_i64(args, "earned_total")?;

            let rows = load_credit_rows();
            let Some(row) = match_credit_row(&rows, &college, &department) else {
                return Ok(json!({
                    "judgement": "판정 불가",
                    "reason": "학점 이수 체계 표에서 일치하는 대학/학과 행을 찾지 못함",
                    "citation": "(학점 이수 체계.pdf, p.1)"
                }));
            };

            let allowed_major_types = [
                "단일전공자",
                "부전공자",
                "복수전공자(주전공)",
                "복수전공자(복수전공)",
            ];
            if !allowed_major_types.contains(&major_type.as_str()) {
                return Ok(json!({
                    "judgement": "판정 불가",
                    "reason": "major_type은 단일전공자/부전공자/복수전공자(주전공)/복수전공자(복수전공) 중 하나여야 함",
                    "citation": "(학점 이수 체계.pdf, p.1)"
                }));
            }

            let raw_major_requirement = row_string(&row, &major_type);
            let major_requirement = extract_first_int(&raw_major_requirement);
            if raw_major_requirement.trim() == "불허" {
                return Ok(json!({
                    "judgement": "불가",
                    "reason": format!("{major_type} 경로가 해당 학과에서 불허됨"),
                    "matched_rule": row.values,
                    "citation": "(학점 이수 체계.pdf, p.1)"
                }));
            }

            let req_liberal_required = extract_first_int(&row_string(&row, "교양필수")).unwrap_or(19);
            let req_liberal_elective = extract_first_int(&row_string(&row, "교양선택")).unwrap_or(9);
            let req_major_basic = extract_first_int(&row_string(&row, "전공기초")).unwrap_or(0);
            let req_total = extract_first_int(&row_string(&row, "졸업학점")).unwrap_or(133);
            let req_major = major_requirement.unwrap_or(0);

            let mut gap = Map::new();
            gap.insert(
                "교양필수".to_string(),
                json!((req_liberal_required - earned_liberal_required).max(0)),
            );
            gap.insert(
                "교양선택".to_string(),
                json!((req_liberal_elective - earned_liberal_elective).max(0)),
            );
            gap.insert(
                "전공기초".to_string(),
                json!((req_major_basic - earned_major_basic).max(0)),
            );
            gap.insert(
                major_type.clone(),
                json!((req_major - earned_major).max(0)),
            );
            gap.insert("졸업학점".to_string(), json!((req_total - earned_total).max(0)));

            let total_gap: i64 = gap.values().filter_map(Value::as_i64).sum();
            let judgement = if total_gap == 0 { "가능" } else { "불가" };

            let mut required = Map::new();
            required.insert("교양필수".to_string(), json!(req_liberal_required));
            required.insert("교양선택".to_string(), json!(req_liberal_elective));
            required.insert("전공기초".to_string(), json!(req_major_basic));
            required.insert(major_type.clone(), json!(req_major));
            required.insert("졸업학점".to_string(), json!(req_total));

            let mut earned = Map::new();
            earned.insert("교양필수".to_string(), json!(earned_liberal_required));
            earned.insert("교양선택".to_string(), json!(earned_liberal_elective));
            earned.insert("전공기초".to_string(), json!(earned_major_basic));
            earned.insert(major_type.clone(), json!(earned_major));
            earned.insert("졸업학점".to_string(), json!(earned_total));

            Ok(json!({
                "judgement": judgement,
                "matched_rule": row.values,
                "required": required,
                "earned": earned,
                "gap": gap,
                "citation": "(학점 이수 체계.pdf, p.1)",
                "notes": [
                    "교양필수/교양선택의 세부 과목 충족 여부는 교양 필수/선택 문서로 추가 확인 필요",
                    "최종 졸업판정은 학칙 졸업요건 조문과 함께 검증 권장"
                ]
            }))
        }
        "ssu_get_calendar_events" => {
            let keyword = optional_string(args, "keyword").unwrap_or_default();
            let month = optional_i64(args, "month");
            let limit = optional_i64(args, "limit").unwrap_or(20).max(1) as usize;

            let rows = parse_calendar_rows();
            let lowered_keyword = keyword.to_lowercase();
            let mut events = Vec::new();

            for row in rows {
                if let Some(target_month) = month {
                    let parsed_month = extract_month(&row.period);
                    if parsed_month != Some(target_month) {
                        continue;
                    }
                }

                let searchable = format!("{} {}", row.period, row.event).to_lowercase();
                if !lowered_keyword.is_empty() && !searchable.contains(&lowered_keyword) {
                    continue;
                }

                let citation = if row.page.is_empty() {
                    "(학사 일정.pdf, 페이지 확인 필요)".to_string()
                } else {
                    format!("(학사 일정.pdf, p.{})", row.page)
                };

                events.push(json!({
                    "기간": row.period,
                    "일정": row.event,
                    "citation": citation,
                }));

                if events.len() >= limit {
                    break;
                }
            }

            Ok(json!({
                "keyword": keyword,
                "month": month,
                "count": events.len(),
                "events": events,
            }))
        }
        "ssu_check_scholarship_threshold" => {
            let gpa = optional_f64(args, "gpa");
            let earned_credits = optional_i64(args, "earned_credits");
            let min_gpa = optional_f64(args, "min_gpa");
            let min_credits = optional_i64(args, "min_credits");

            if min_gpa.is_none() && min_credits.is_none() {
                return Ok(json!({
                    "judgement": "판정 불가",
                    "reason": "현재 docs 묶음에는 장학금 정량 선발기준 문서가 없음",
                    "required_action": "장학 규정 PDF/URL 제공 필요",
                    "citation": "(학칙.pdf, p.14) + source-map known gap"
                }));
            }

            let mut gap = Map::new();
            if let (Some(g), Some(min)) = (gpa, min_gpa) {
                let rounded = ((g - min) * 1000.0).round() / 1000.0;
                gap.insert("gpa_gap".to_string(), json!(rounded));
            }
            if let (Some(earned), Some(min)) = (earned_credits, min_credits) {
                gap.insert("credit_gap".to_string(), json!(earned - min));
            }

            let meets_gpa = min_gpa.map(|min| gpa.map(|v| v >= min).unwrap_or(false)).unwrap_or(true);
            let meets_credits = min_credits
                .map(|min| earned_credits.map(|v| v >= min).unwrap_or(false))
                .unwrap_or(true);

            Ok(json!({
                "judgement": if meets_gpa && meets_credits { "가능" } else { "불가" },
                "input": {
                    "gpa": gpa,
                    "earned_credits": earned_credits,
                    "min_gpa": min_gpa,
                    "min_credits": min_credits,
                },
                "gap": gap,
                "notes": [
                    "이 결과는 사용자가 제공한 역치 기준값에 대한 비교임",
                    "학교 공식 장학 세부기준 문서로 최종 확인 필요"
                ]
            }))
        }
        "ssu_list_sources" => Ok(json!({
            "docs": [
                "학칙.pdf",
                "학점 이수 체계.pdf",
                "교양 필수.pdf",
                "교양 선택.pdf",
                "학사 일정.pdf"
            ],
            "normalized_md": NORMALIZED_FILES,
            "raw_md": RAW_FILES,
            "references": REFERENCE_FILES,
        })),
        _ => Err(format!("Unknown tool: {name}")),
    }
}

fn tool_definitions() -> Vec<Value> {
    vec![
        json!({
            "name": "ssu_classify_request",
            "description": "학사 질의를 5개 워크플로우 카테고리로 분류한다.",
            "inputSchema": {
                "type": "object",
                "properties": { "question": { "type": "string" } },
                "required": ["question"],
                "additionalProperties": false
            }
        }),
        json!({
            "name": "ssu_search_evidence",
            "description": "질문과 관련된 근거 라인을 검색하고 문서/페이지 힌트를 반환한다.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "question": { "type": "string" },
                    "category": { "type": "string" },
                    "max_hits": { "type": "integer" }
                },
                "required": ["question"],
                "additionalProperties": false
            }
        }),
        json!({
            "name": "ssu_rule_brief",
            "description": "분류 + 근거검색 + 응답 골격을 한 번에 반환한다.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "question": { "type": "string" },
                    "max_hits": { "type": "integer" }
                },
                "required": ["question"],
                "additionalProperties": false
            }
        }),
        json!({
            "name": "ssu_evaluate_graduation",
            "description": "학점 이수 체계 기준으로 졸업요건 충족 여부(초안)를 계산한다.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "college": { "type": "string" },
                    "department": { "type": "string" },
                    "major_type": { "type": "string" },
                    "earned_liberal_required": { "type": "integer" },
                    "earned_liberal_elective": { "type": "integer" },
                    "earned_major_basic": { "type": "integer" },
                    "earned_major": { "type": "integer" },
                    "earned_total": { "type": "integer" }
                },
                "required": [
                    "college",
                    "major_type",
                    "earned_liberal_required",
                    "earned_liberal_elective",
                    "earned_major_basic",
                    "earned_major",
                    "earned_total"
                ],
                "additionalProperties": false
            }
        }),
        json!({
            "name": "ssu_get_calendar_events",
            "description": "학사 일정에서 일정 항목을 조회한다.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "keyword": { "type": "string" },
                    "month": { "type": "integer" },
                    "limit": { "type": "integer" }
                },
                "additionalProperties": false
            }
        }),
        json!({
            "name": "ssu_check_scholarship_threshold",
            "description": "장학 역치 비교. 기준값 미제공 시 '판정 불가'를 반환한다.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "gpa": { "type": "number" },
                    "earned_credits": { "type": "integer" },
                    "min_gpa": { "type": "number" },
                    "min_credits": { "type": "integer" }
                },
                "additionalProperties": false
            }
        }),
        json!({
            "name": "ssu_list_sources",
            "description": "서버가 참조하는 주요 소스 파일 경로를 반환한다.",
            "inputSchema": {
                "type": "object",
                "properties": {},
                "additionalProperties": false
            }
        }),
    ]
}

fn rpc_success(id: Value, result: Value) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": result,
    })
}

fn rpc_error(id: Value, code: i64, message: &str) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": {
            "code": code,
            "message": message,
        }
    })
}

fn process_single_rpc(request: &Value) -> Option<Value> {
    let Some(obj) = request.as_object() else {
        return Some(rpc_error(Value::Null, -32600, "Invalid Request"));
    };

    let id = obj.get("id").cloned();
    let method = match obj.get("method").and_then(Value::as_str) {
        Some(method) => method,
        None => {
            return Some(rpc_error(
                id.unwrap_or(Value::Null),
                -32600,
                "Invalid Request: missing method",
            ))
        }
    };

    if method == "notifications/initialized" {
        return None;
    }

    let params = obj.get("params").and_then(Value::as_object);

    let result = match method {
        "initialize" => {
            let protocol_version = params
                .and_then(|p| p.get("protocolVersion"))
                .and_then(Value::as_str)
                .unwrap_or(DEFAULT_PROTOCOL_VERSION);
            Ok(json!({
                "protocolVersion": protocol_version,
                "capabilities": {
                    "tools": {
                        "listChanged": false
                    }
                },
                "serverInfo": {
                    "name": SERVER_NAME,
                    "version": SERVER_VERSION,
                }
            }))
        }
        "ping" => Ok(json!({})),
        "tools/list" => Ok(json!({
            "tools": tool_definitions()
        })),
        "tools/call" => {
            match params {
                Some(params) => match params.get("name").and_then(Value::as_str) {
                    Some(name) => {
                        let arguments = params
                            .get("arguments")
                            .and_then(Value::as_object)
                            .cloned()
                            .unwrap_or_default();

                        let call_result = call_tool(name, &arguments);
                        let rpc_payload = match call_result {
                            Ok(payload) => tool_content(payload),
                            Err(error_message) => tool_error_content(&error_message),
                        };
                        Ok(rpc_payload)
                    }
                    None => Err(rpc_failure(-32602, "Invalid params: missing tool name")),
                },
                None => Err(rpc_failure(-32602, "Invalid params")),
            }
        }
        _ => Err(rpc_failure(-32601, format!("Method not found: {method}"))),
    };

    if id.is_none() {
        return None;
    }

    let id = id.unwrap_or(Value::Null);
    Some(match result {
        Ok(result) => rpc_success(id, result),
        Err(err) => rpc_error(id, err.code, &err.message),
    })
}

fn json_response(value: &Value) -> Result<Response> {
    Response::from_json(value)
}

async fn handle_mcp_request(mut req: Request) -> Result<Response> {
    let body_text = req.text().await?;
    let parsed: Value = match serde_json::from_str(&body_text) {
        Ok(value) => value,
        Err(_) => {
            let err = rpc_error(Value::Null, -32700, "Parse error");
            return json_response(&err);
        }
    };

    if let Some(batch) = parsed.as_array() {
        let mut responses = Vec::new();
        for item in batch {
            if let Some(response) = process_single_rpc(item) {
                responses.push(response);
            }
        }
        if responses.is_empty() {
            let response = Response::empty()?;
            return Ok(response.with_status(202));
        }
        return json_response(&Value::Array(responses));
    }

    if let Some(response) = process_single_rpc(&parsed) {
        return json_response(&response);
    }

    let response = Response::empty()?;
    Ok(response.with_status(202))
}

#[event(fetch)]
pub async fn fetch(req: Request, _env: Env, _ctx: Context) -> Result<Response> {
    let path = req.url()?.path().to_string();
    let method = req.method().clone();

    match (method, path.as_str()) {
        (Method::Get, "/") => Response::from_json(&json!({
            "service": SERVER_NAME,
            "version": SERVER_VERSION,
            "endpoint": "/mcp",
            "transport": "JSON-RPC over HTTP"
        })),
        (Method::Post, "/mcp") => handle_mcp_request(req).await,
        (Method::Get, "/mcp") => Response::from_json(&json!({
            "service": SERVER_NAME,
            "message": "POST /mcp with JSON-RPC request body"
        })),
        _ => Response::error("Not Found", 404),
    }
}
