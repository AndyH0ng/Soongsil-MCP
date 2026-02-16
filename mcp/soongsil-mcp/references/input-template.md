# Input Template

## Student Profile (minimum)

Use this schema before any graduation or retake analysis.

```json
{
  "student_id": "2024xxxx",
  "admission_year": 2024,
  "college": "IT대학",
  "department": "컴퓨터학부",
  "track": "단일전공",
  "credits": {
    "general_required": 16,
    "general_elective": 9,
    "major": 64,
    "total": 112
  },
  "gpa": {
    "overall": 3.62,
    "latest_term": 3.85
  },
  "latest_term_credits": 15,
  "retake_courses": [
    {
      "course_name": "컴퓨팅적사고",
      "semester": "2025-2",
      "before_grade": "C+",
      "after_grade": "A0"
    }
  ]
}
```

## Answer Schema

```markdown
결론:
- ...

근거:
- (문서명, p.N) ...
- (문서명, p.N) ...

계산/비교:
- ...

불확실성:
- ...
```

## Scholarship Comparison Rule

If scholarship criteria are not in source PDFs:
- Do not infer thresholds from memory.
- Return `판정 불가(근거 문서 없음)`.
- Request scholarship regulation source (PDF or official page export).
