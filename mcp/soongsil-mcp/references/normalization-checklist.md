# MD Normalization Checklist

## Pre-check

1. 대상 PDF와 raw-md 파일이 최신인지 확인한다.
2. 파일명이 source-map과 일치하는지 확인한다.
3. 질문에 자주 쓰는 숫자 항목(학점/학기/평점/횟수)을 우선 표시한다.

## Formatting Check

1. 깨진 공백/구분자(`!`, 중복 공백, 비정상 기호)를 제거했다.
2. 열거형 정보는 가능한 한 표로 변환했다.
3. 표 컬럼명은 의미 기반으로 명명했다(`값1` 금지).
4. `대학`과 `학과/학부`처럼 엔티티를 분리했다.

## Fact Check

1. 숫자 값이 raw-md 및 PDF와 일치한다.
2. 예외/불허/조건부 규정이 누락되지 않았다.
3. 장학/재수강처럼 별도 규정이 필요한 항목은 `불확실성`으로 표시했다.
4. 필요한 경우 `law-topic-index.md`, `law-articles.md`, `law-numeric-rules.md`와 정합성을 확인했다.

## Publish Check

1. normalized 파일 상단 메타데이터를 채웠다.
2. `knowledge/normalized-md/README.md` 규칙을 준수했다.
3. MCP 문서(`README.md`, `md-corpus.md`, `qa-checklists.md`)의 탐색 순서와 충돌이 없는지 확인했다.
