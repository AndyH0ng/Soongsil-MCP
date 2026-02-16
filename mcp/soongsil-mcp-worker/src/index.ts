import { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { WebStandardStreamableHTTPServerTransport } from "@modelcontextprotocol/sdk/server/webStandardStreamableHttp.js";
import { z } from "zod";

interface AssetsBinding {
  fetch(input: RequestInfo | URL, init?: RequestInit): Promise<Response>;
}

interface Env {
  ASSETS: AssetsBinding;
}

interface SearchHit {
  file: string;
  line: number;
  page: string | null;
  snippet: string;
  matched_terms: string[];
  score: number;
  citation_hint: string;
}

interface CreditRow {
  [key: string]: string;
}

interface CalendarRow {
  "기간": string;
  "일정": string;
  page: string;
}

const SERVER_NAME = "Soongsil MCP Worker";
const SERVER_VERSION = "0.1.0";
const ASSETS_BASE_URL = "https://assets.local";

const CATEGORY_KEYWORDS: Record<string, string[]> = {
  "학칙 Q&A": ["학칙", "휴학", "복학", "제적", "자퇴", "전과", "다전공", "학사경고", "징계", "조문"],
  "졸업요건 판정": ["졸업", "졸업요건", "이수", "학점", "전공기초", "복수전공", "부전공"],
  "재수강 가능/영향 분석": ["재수강", "중복", "성적", "학점인정"],
  "장학 기준 역치 비교": ["장학", "장학금", "성적우수", "역치", "threshold"],
  "수강신청/학사일정 보조": ["수강신청", "학사일정", "정정", "취소", "신청기간", "등록금", "마감"],
};

const STOP_TERMS = new Set([
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
]);

const NORMALIZED_FILES = [
  "/knowledge/normalized-md/README.md",
  "/knowledge/normalized-md/교양 선택.md",
  "/knowledge/normalized-md/교양 필수.md",
  "/knowledge/normalized-md/학사 일정.md",
  "/knowledge/normalized-md/학점 이수 체계.md",
  "/knowledge/normalized-md/학칙.md",
  "/knowledge/normalized-md/학칙.table-normalized.md",
];

const RAW_FILES = ["/knowledge/raw-md/README.md", "/knowledge/raw-md/학칙.raw.md"];

const REFERENCE_FILES = [
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

const CATEGORY_PATHS: Record<string, string[]> = {
  "학칙 Q&A": [
    "/knowledge/normalized-md/학칙.md",
    "/references/law-topic-index.md",
    "/references/law-articles.md",
    "/references/law-numeric-rules.md",
    "/knowledge/raw-md/학칙.raw.md",
  ],
  "졸업요건 판정": [
    "/knowledge/normalized-md/학점 이수 체계.md",
    "/knowledge/normalized-md/교양 필수.md",
    "/knowledge/normalized-md/교양 선택.md",
  ],
  "재수강 가능/영향 분석": [
    "/knowledge/normalized-md/학칙.md",
    "/knowledge/normalized-md/교양 필수.md",
    "/knowledge/raw-md/학칙.raw.md",
  ],
  "장학 기준 역치 비교": [
    "/references/source-map.md",
    "/references/law-numeric-rules.md",
    "/knowledge/normalized-md/학칙.md",
  ],
  "수강신청/학사일정 보조": [
    "/knowledge/normalized-md/학사 일정.md",
    "/knowledge/normalized-md/학칙.md",
  ],
};

const ASSET_TEXT_CACHE = new Map<string, Promise<string>>();
let CREDIT_ROWS_CACHE: Promise<CreditRow[]> | null = null;
let CALENDAR_ROWS_CACHE: Promise<CalendarRow[]> | null = null;

function buildToolResponse(payload: unknown) {
  return {
    content: [
      {
        type: "text" as const,
        text: JSON.stringify(payload, null, 2),
      },
    ],
  };
}

function getFileName(pathname: string) {
  const parts = pathname.split("/");
  return parts[parts.length - 1] ?? pathname;
}

function safeInt(value: string): number | null {
  const trimmed = value.trim();
  if (!trimmed || trimmed === "-" || trimmed === "불허") {
    return null;
  }
  const match = trimmed.match(/\d+/);
  if (!match) {
    return null;
  }
  return Number.parseInt(match[0], 10);
}

function extractTerms(question: string): string[] {
  const terms = question.match(/[가-힣A-Za-z0-9]+/g) ?? [];
  const unique: string[] = [];
  const seen = new Set<string>();

  for (const term of terms) {
    const normalized = term.trim().toLowerCase();
    if (normalized.length < 2 || STOP_TERMS.has(normalized)) {
      continue;
    }
    if (seen.has(normalized)) {
      continue;
    }
    seen.add(normalized);
    unique.push(normalized);
  }

  return unique;
}

function classify(question: string): { category: string; matched_keywords: string[] } {
  const lowered = question.toLowerCase();
  let bestCategory = "학칙 Q&A";
  let bestScore = -1;
  let matchedKeywords: string[] = [];

  for (const [category, keywords] of Object.entries(CATEGORY_KEYWORDS)) {
    const hits = keywords.filter((keyword) => lowered.includes(keyword.toLowerCase()));
    const score = hits.length;
    if (score > bestScore) {
      bestScore = score;
      bestCategory = category;
      matchedKeywords = hits;
    }
  }

  if (bestScore <= 0) {
    return { category: "학칙 Q&A", matched_keywords: [] };
  }

  return { category: bestCategory, matched_keywords: matchedKeywords };
}

function citationHint(pathname: string, page: string | null): string {
  const name = getFileName(pathname);
  if (
    name === "학칙.md" ||
    name === "학칙.raw.md" ||
    name === "law-topic-index.md" ||
    name === "law-articles.md" ||
    name === "law-numeric-rules.md"
  ) {
    if (page) {
      return `(학칙.pdf, p.${page})`;
    }
    return "(학칙.pdf, 페이지 확인 필요)";
  }
  if (name === "학점 이수 체계.md") {
    return "(학점 이수 체계.pdf, p.1)";
  }
  if (name === "교양 필수.md") {
    return "(교양 필수.pdf, p.1~2)";
  }
  if (name === "교양 선택.md") {
    return "(교양 선택.pdf, p.1~4)";
  }
  if (name === "학사 일정.md") {
    if (page) {
      return `(학사 일정.pdf, p.${page})`;
    }
    return "(학사 일정.pdf, 페이지 확인 필요)";
  }
  return "(원문 PDF 페이지 확인 필요)";
}

function normalizePipes(line: string): string[] {
  return line
    .trim()
    .replace(/^\|/, "")
    .replace(/\|$/, "")
    .split("|")
    .map((cell) => cell.trim());
}

async function readAssetText(env: Env, pathname: string): Promise<string> {
  if (!ASSET_TEXT_CACHE.has(pathname)) {
    ASSET_TEXT_CACHE.set(
      pathname,
      (async () => {
        const url = new URL(pathname, ASSETS_BASE_URL);
        const response = await env.ASSETS.fetch(url);
        if (!response.ok) {
          throw new Error(`Failed to load asset ${pathname}: ${response.status}`);
        }
        return response.text();
      })(),
    );
  }

  return ASSET_TEXT_CACHE.get(pathname)!;
}

async function iterLinesWithPage(
  env: Env,
  pathname: string,
): Promise<Array<{ lineNo: number; line: string; page: string | null }>> {
  const text = await readAssetText(env, pathname);
  const lines = text.split(/\r?\n/);
  const output: Array<{ lineNo: number; line: string; page: string | null }> = [];
  let currentPage: string | null = null;

  lines.forEach((line, index) => {
    const match = line.trim().match(/^## p\.(\d+)/);
    if (match) {
      currentPage = match[1] ?? null;
    }
    output.push({ lineNo: index + 1, line, page: currentPage });
  });

  return output;
}

async function searchInFile(env: Env, pathname: string, terms: string[], maxHits: number): Promise<SearchHit[]> {
  if (terms.length === 0 || maxHits <= 0) {
    return [];
  }

  const hits: SearchHit[] = [];
  const lines = await iterLinesWithPage(env, pathname);

  for (const row of lines) {
    const candidate = row.line.trim();
    if (!candidate) {
      continue;
    }

    const lowered = candidate.toLowerCase();
    const matched = terms.filter((term) => lowered.includes(term));
    if (matched.length === 0) {
      continue;
    }

    hits.push({
      file: pathname,
      line: row.lineNo,
      page: row.page,
      snippet: candidate.slice(0, 240),
      matched_terms: matched,
      score: matched.length,
      citation_hint: citationHint(pathname, row.page),
    });

    if (hits.length >= maxHits) {
      break;
    }
  }

  return hits;
}

async function loadCreditRows(env: Env): Promise<CreditRow[]> {
  const text = await readAssetText(env, "/knowledge/normalized-md/학점 이수 체계.md");
  const lines = text.split(/\r?\n/);

  let start = -1;
  for (let i = 0; i < lines.length; i += 1) {
    if (lines[i]?.trim().startsWith("| 대학 | 학과/학부 |")) {
      start = i;
      break;
    }
  }

  if (start < 0) {
    return [];
  }

  const headerLine = lines[start];
  if (!headerLine) {
    return [];
  }

  const headers = normalizePipes(headerLine);
  const rows: CreditRow[] = [];

  for (let i = start + 2; i < lines.length; i += 1) {
    const line = lines[i]?.trim() ?? "";
    if (!line.startsWith("|")) {
      if (rows.length > 0) {
        break;
      }
      continue;
    }

    const cells = normalizePipes(line);
    if (cells.length !== headers.length) {
      continue;
    }

    const row: CreditRow = {};
    headers.forEach((header, index) => {
      row[header] = cells[index] ?? "";
    });
    rows.push(row);
  }

  return rows;
}

async function getCreditRows(env: Env): Promise<CreditRow[]> {
  if (!CREDIT_ROWS_CACHE) {
    CREDIT_ROWS_CACHE = loadCreditRows(env);
  }
  return CREDIT_ROWS_CACHE;
}

function matchCreditRow(rows: CreditRow[], college: string, department: string): CreditRow | null {
  let bestScore = -1;
  let bestRow: CreditRow | null = null;

  const normalizedCollege = college.trim();
  const normalizedDepartment = department.trim();

  for (const row of rows) {
    const rowCollege = (row["대학"] ?? "").trim();
    const rowDepartment = (row["학과/학부"] ?? "").trim();
    let score = 0;

    if (!rowCollege) {
      continue;
    }

    if (rowCollege === normalizedCollege) {
      score += 20;
    } else if (normalizedCollege && rowCollege.includes(normalizedCollege)) {
      score += 12;
    } else {
      continue;
    }

    if (rowDepartment === normalizedDepartment) {
      score += 20;
    } else if (normalizedDepartment && rowDepartment.includes(normalizedDepartment)) {
      score += 14;
    } else if (rowDepartment.includes("전체")) {
      score += 8;
    } else if (rowDepartment.includes("외")) {
      score += 6;
    } else if (!normalizedDepartment) {
      score += 4;
    }

    if (score > bestScore) {
      bestScore = score;
      bestRow = row;
    }
  }

  return bestRow;
}

async function parseCalendarRows(env: Env): Promise<CalendarRow[]> {
  if (!CALENDAR_ROWS_CACHE) {
    CALENDAR_ROWS_CACHE = (async () => {
      const text = await readAssetText(env, "/knowledge/normalized-md/학사 일정.md");
      const lines = text.split(/\r?\n/);

      const rows: CalendarRow[] = [];
      let page = "";
      let inTable = false;

      for (const line of lines) {
        const stripped = line.trim();
        const pageMatch = stripped.match(/^## p\.(\d+)/);
        if (pageMatch) {
          page = pageMatch[1] ?? "";
        }

        if (stripped.startsWith("| 기간 | 일정 |")) {
          inTable = true;
          continue;
        }
        if (inTable && stripped.startsWith("| --- | --- |")) {
          continue;
        }
        if (inTable && stripped.startsWith("|")) {
          const cells = normalizePipes(stripped);
          if (cells.length >= 2) {
            rows.push({
              "기간": cells[0] ?? "",
              "일정": cells[1] ?? "",
              page,
            });
          }
          continue;
        }
        if (inTable && !stripped) {
          inTable = false;
        }
      }

      return rows;
    })();
  }

  return CALENDAR_ROWS_CACHE;
}

function extractMonth(period: string): number | null {
  const match = period.match(/(\d{2})-\d{2}/);
  if (!match) {
    return null;
  }
  return Number.parseInt(match[1] ?? "", 10);
}

async function searchEvidence(question: string, category: string, maxHits: number, env: Env) {
  const terms = extractTerms(question);
  const paths = CATEGORY_PATHS[category] ?? [];
  const allHits: SearchHit[] = [];

  for (const path of paths) {
    const hits = await searchInFile(env, path, terms, maxHits);
    allHits.push(...hits);
  }

  allHits.sort((a, b) => b.score - a.score);
  const limitedHits = allHits.slice(0, maxHits > 0 ? maxHits : 12);

  return {
    question,
    category,
    search_terms: terms,
    hits: limitedHits,
    citation_rule: "최종 답변은 반드시 (문서명.pdf, p.N) 형식으로 표기",
  };
}

function createServer(env: Env): McpServer {
  const server = new McpServer({
    name: SERVER_NAME,
    version: SERVER_VERSION,
  });

  server.tool(
    "ssu_classify_request",
    "학사 질의를 5개 워크플로우 카테고리로 분류한다.",
    {
      question: z.string().min(1),
    },
    async ({ question }) => {
      const classified = classify(question);
      return buildToolResponse({
        question,
        category: classified.category,
        matched_keywords: classified.matched_keywords,
        recommended_paths: CATEGORY_PATHS[classified.category] ?? [],
      });
    },
  );

  server.tool(
    "ssu_search_evidence",
    "질문과 관련된 근거 라인을 검색하고 문서/페이지 힌트를 반환한다.",
    {
      question: z.string().min(1),
      category: z.string().optional(),
      max_hits: z.number().int().min(1).max(50).optional(),
    },
    async ({ question, category, max_hits }) => {
      const selectedCategory = category ?? classify(question).category;
      const result = await searchEvidence(question, selectedCategory, max_hits ?? 12, env);
      return buildToolResponse(result);
    },
  );

  server.tool(
    "ssu_rule_brief",
    "분류 + 근거검색 + 응답 골격을 한 번에 반환한다.",
    {
      question: z.string().min(1),
      max_hits: z.number().int().min(1).max(50).optional(),
    },
    async ({ question, max_hits }) => {
      const classified = classify(question);
      const evidence = await searchEvidence(question, classified.category, max_hits ?? 10, env);
      return buildToolResponse({
        question,
        category: classified.category,
        matched_keywords: classified.matched_keywords,
        workflow: [
          "1) normalized-md에서 후보 규정 탐색",
          "2) law-topic-index/law-articles/law-numeric-rules 교차확인(학칙 질의 시)",
          "3) 필요 시 raw-md/학칙.raw.md 대조",
          "4) PDF 페이지 인용으로 최종 확정",
        ],
        response_template: ["결론", "근거", "계산/비교", "불확실성"],
        evidence: evidence.hits,
      });
    },
  );

  server.tool(
    "ssu_evaluate_graduation",
    "학점 이수 체계 기준으로 졸업요건 충족 여부(초안)를 계산한다.",
    {
      college: z.string().min(1),
      department: z.string().default(""),
      major_type: z.string().min(1),
      earned_liberal_required: z.number().int().min(0),
      earned_liberal_elective: z.number().int().min(0),
      earned_major_basic: z.number().int().min(0),
      earned_major: z.number().int().min(0),
      earned_total: z.number().int().min(0),
    },
    async ({
      college,
      department,
      major_type,
      earned_liberal_required,
      earned_liberal_elective,
      earned_major_basic,
      earned_major,
      earned_total,
    }) => {
      const rows = await getCreditRows(env);
      const row = matchCreditRow(rows, college, department);
      if (!row) {
        return buildToolResponse({
          judgement: "판정 불가",
          reason: "학점 이수 체계 표에서 일치하는 대학/학과 행을 찾지 못함",
          citation: "(학점 이수 체계.pdf, p.1)",
        });
      }

      const majorType = major_type.trim();
      const allowedTypes = ["단일전공자", "부전공자", "복수전공자(주전공)", "복수전공자(복수전공)"];
      if (!allowedTypes.includes(majorType)) {
        return buildToolResponse({
          judgement: "판정 불가",
          reason:
            "major_type은 단일전공자/부전공자/복수전공자(주전공)/복수전공자(복수전공) 중 하나여야 함",
          citation: "(학점 이수 체계.pdf, p.1)",
        });
      }

      const rawMajorRequirement = row[majorType] ?? "";
      const majorRequirement = safeInt(rawMajorRequirement);
      if (rawMajorRequirement.trim() === "불허") {
        return buildToolResponse({
          judgement: "불가",
          reason: `${majorType} 경로가 해당 학과에서 불허됨`,
          matched_rule: row,
          citation: "(학점 이수 체계.pdf, p.1)",
        });
      }

      const reqLiberalRequired = safeInt(row["교양필수"] ?? "") ?? 19;
      const reqLiberalElective = safeInt(row["교양선택"] ?? "") ?? 9;
      const reqMajorBasic = safeInt(row["전공기초"] ?? "") ?? 0;
      const reqTotal = safeInt(row["졸업학점"] ?? "") ?? 133;
      const reqMajor = majorRequirement ?? 0;

      const gaps: Record<string, number> = {
        교양필수: Math.max(reqLiberalRequired - earned_liberal_required, 0),
        교양선택: Math.max(reqLiberalElective - earned_liberal_elective, 0),
        전공기초: Math.max(reqMajorBasic - earned_major_basic, 0),
        [majorType]: Math.max(reqMajor - earned_major, 0),
        졸업학점: Math.max(reqTotal - earned_total, 0),
      };

      const totalGap = Object.values(gaps).reduce((sum, value) => sum + value, 0);
      const judgement = totalGap === 0 ? "가능" : "불가";

      return buildToolResponse({
        judgement,
        matched_rule: row,
        required: {
          교양필수: reqLiberalRequired,
          교양선택: reqLiberalElective,
          전공기초: reqMajorBasic,
          [majorType]: reqMajor,
          졸업학점: reqTotal,
        },
        earned: {
          교양필수: earned_liberal_required,
          교양선택: earned_liberal_elective,
          전공기초: earned_major_basic,
          [majorType]: earned_major,
          졸업학점: earned_total,
        },
        gap: gaps,
        citation: "(학점 이수 체계.pdf, p.1)",
        notes: [
          "교양필수/교양선택의 세부 과목 충족 여부는 교양 필수/선택 문서로 추가 확인 필요",
          "최종 졸업판정은 학칙 졸업요건 조문과 함께 검증 권장",
        ],
      });
    },
  );

  server.tool(
    "ssu_get_calendar_events",
    "학사 일정에서 일정 항목을 조회한다.",
    {
      keyword: z.string().optional(),
      month: z.number().int().min(1).max(12).optional(),
      limit: z.number().int().min(1).max(100).optional(),
    },
    async ({ keyword, month, limit }) => {
      const rows = await parseCalendarRows(env);
      const normalizedKeyword = (keyword ?? "").trim().toLowerCase();
      const targetLimit = limit ?? 20;
      const filtered: Array<{ "기간": string; "일정": string; citation: string }> = [];

      for (const row of rows) {
        const period = row["기간"];
        const event = row["일정"];

        if (typeof month === "number") {
          const extracted = extractMonth(period);
          if (extracted !== month) {
            continue;
          }
        }

        const searchable = `${period} ${event}`.toLowerCase();
        if (normalizedKeyword && !searchable.includes(normalizedKeyword)) {
          continue;
        }

        filtered.push({
          "기간": period,
          "일정": event,
          citation: row.page ? `(학사 일정.pdf, p.${row.page})` : "(학사 일정.pdf, 페이지 확인 필요)",
        });

        if (filtered.length >= targetLimit) {
          break;
        }
      }

      return buildToolResponse({
        keyword: keyword ?? "",
        month: month ?? null,
        count: filtered.length,
        events: filtered,
      });
    },
  );

  server.tool(
    "ssu_check_scholarship_threshold",
    "장학 역치 비교. 기준값 미제공 시 '판정 불가'를 반환한다.",
    {
      gpa: z.number().optional(),
      earned_credits: z.number().int().optional(),
      min_gpa: z.number().optional(),
      min_credits: z.number().int().optional(),
    },
    async ({ gpa, earned_credits, min_gpa, min_credits }) => {
      if (typeof min_gpa === "undefined" && typeof min_credits === "undefined") {
        return buildToolResponse({
          judgement: "판정 불가",
          reason: "현재 docs 묶음에는 장학금 정량 선발기준 문서가 없음",
          required_action: "장학 규정 PDF/URL 제공 필요",
          citation: "(학칙.pdf, p.14) + source-map known gap",
        });
      }

      const gaps: Record<string, number> = {};
      if (typeof min_gpa === "number" && typeof gpa === "number") {
        gaps.gpa_gap = Math.round((gpa - min_gpa) * 1000) / 1000;
      }
      if (typeof min_credits === "number" && typeof earned_credits === "number") {
        gaps.credit_gap = earned_credits - min_credits;
      }

      const meetsGpa = typeof min_gpa === "undefined" || (typeof gpa === "number" && gpa >= min_gpa);
      const meetsCredits =
        typeof min_credits === "undefined" ||
        (typeof earned_credits === "number" && earned_credits >= min_credits);

      return buildToolResponse({
        judgement: meetsGpa && meetsCredits ? "가능" : "불가",
        input: {
          gpa: gpa ?? null,
          earned_credits: earned_credits ?? null,
          min_gpa: min_gpa ?? null,
          min_credits: min_credits ?? null,
        },
        gap: gaps,
        notes: [
          "이 결과는 사용자가 제공한 역치 기준값에 대한 비교임",
          "학교 공식 장학 세부기준 문서로 최종 확인 필요",
        ],
      });
    },
  );

  server.tool("ssu_list_sources", "서버가 참조하는 주요 소스 파일 경로를 반환한다.", {}, async () => {
    return buildToolResponse({
      docs: [
        "학칙.pdf",
        "학점 이수 체계.pdf",
        "교양 필수.pdf",
        "교양 선택.pdf",
        "학사 일정.pdf",
      ],
      normalized_md: NORMALIZED_FILES,
      raw_md: RAW_FILES,
      references: REFERENCE_FILES,
    });
  });

  return server;
}

export default {
  async fetch(request: Request, env: Env): Promise<Response> {
    const url = new URL(request.url);

    if (url.pathname === "/") {
      return Response.json({
        service: SERVER_NAME,
        endpoint: "/mcp",
        transport: "Streamable HTTP",
      });
    }

    if (url.pathname !== "/mcp") {
      return new Response("Not Found", { status: 404 });
    }

    const transport = new WebStandardStreamableHTTPServerTransport();
    const server = createServer(env);

    try {
      await server.connect(transport);
      return await transport.handleRequest(request);
    } catch (error) {
      const message = error instanceof Error ? error.message : "Unknown error";
      return new Response(`MCP handler error: ${message}`, { status: 500 });
    }
  },
};
