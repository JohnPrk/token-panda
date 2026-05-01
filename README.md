# 토큰판다 (Claude Desk Pet)

> 데스크톱 한구석에서 너의 Claude 토큰 잔량을 지켜봐주는 작은 판다.

[![Download .dmg](https://img.shields.io/badge/Download-.dmg%20v0.21.0-6b4cff?style=for-the-badge&logo=apple)](https://github.com/JohnPrk/claude-desk-helper/releases/latest/download/Claude.Desk.Pet_0.21.0_aarch64.dmg)
[![macOS only](https://img.shields.io/badge/platform-macOS%2011%2B-lightgrey?style=for-the-badge&logo=apple)](#한계)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow?style=for-the-badge)](#라이선스)

---

## 왜 만들었나

Claude Pro·Max 구독을 쓰다 보면 늘 한 번씩 멈칫합니다.

- 5시간 한도 얼마나 남았지?
- 주간 한도는?
- 캐시는 아직 살아있나? `.` 한 번이라도 눌러서 살려둬야 하나?
- claude.ai/settings/usage를 또 새 탭으로 열어야 하나?

매번 탭 열어서 사용량 페이지 새로고침하기엔 흐름이 자주 끊깁니다. 그래서 **메뉴바 한 칸 + 데스크톱 모서리에 붙어서 실시간으로 알려주는 작은 판다**를 만들었습니다. 토큰 다 쓰면 판다가 누워버리고, 캐시가 만료될 때쯤 살짝 흔들리며 신호를 줍니다.

---

## 다운로드 / 설치

### 가장 빠른 방법: 빌드된 .dmg

위쪽 **`Download .dmg`** 배지를 눌러 최신 `.dmg`를 받고, 열어서 `Applications` 폴더로 드래그하면 끝입니다.

> 처음 실행할 때 macOS가 "확인되지 않은 개발자" 경고를 띄울 수 있어요. `시스템 설정 → 개인정보 보호 및 보안 → 그래도 열기`로 한 번 허용하면 다음부터는 바로 실행됩니다.

### 직접 빌드해서 쓰기

```bash
git clone https://github.com/JohnPrk/claude-desk-helper.git
cd claude-desk-helper
npm install
npm run tauri:build
# → src-tauri/target/release/bundle/dmg/Claude Desk Pet_<version>_aarch64.dmg
```

빌드 시 필요한 도구:

- macOS 11+
- Node 18+
- Rust toolchain (`curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y`)

---

## 어떻게 사용하나

### 1. 처음 실행

플랜을 고르라는 모달이 뜹니다 — `Pro` / `Max 5×` / `Max 20×` / `Custom`.
선택하면 데스크톱 한쪽에 작은 판다 창이 뜨고, 메뉴바에는 `🔋 87%` 같은 잔량 표시가 생깁니다.

### 2. 메뉴바 / 펫 창

| 위치 | 보이는 것 | 조작 |
| --- | --- | --- |
| 메뉴바 | 5시간 잔량 % + 배터리 이모지 (🔋 / 🪫 / 😴 / 💀) | 좌클릭으로 메뉴 열기. **클릭해도 펫이 사라지지 않습니다.** |
| 메뉴바 메뉴 | `펫 보이기/숨기기`, `지금 새로고침`, `설정...`, `종료` | 펫을 일시적으로 숨기고 싶을 때 |
| 펫 창 | 5h / 주간 잔량 % + 캐시 타이머 + 캐릭터 모션 | 드래그로 위치 이동, 우클릭으로 즉시 새로고침 |

### 3. API 연동 (실시간 사용량)

기본은 `~/.claude/projects/**/*.jsonl` 로컬 로그를 파싱해서 추정합니다. 더 정확한 값을 원하면 **claude.ai의 `/api/.../usage`**를 직접 조회하도록 연동할 수 있습니다.

#### Org ID 구하기

1. [claude.ai/settings/account](https://claude.ai/settings/account) 접속
2. `계정` 탭의 **조직 ID** 값 복사 (예: `63e058d5-142c-4368-bca3-39d64d78b4f5`)

#### 쿠키 구하기

1. [claude.ai/settings/usage](https://claude.ai/settings/usage) 접속 (한 번 새로고침)
2. `개발자 도구 → Network` 탭 열기
3. `usage` 요청 클릭 → `Headers` 탭 → **`Request Headers`의 `cookie:` 줄 전체 복사**
4. 앱의 설정 창 → `API 연동` → `세션 쿠키` 칸에 통째로 붙여넣기

쿠키 줄에서 실제로 사용하는 키는 5개뿐(`sessionKey`, `cf_clearance`, `__cf_bm`, `_cfuvid`, `routingHint`)이고 나머지는 무시됩니다. 이미 들어있는 다른 키는 그냥 둬도 됩니다.

#### 저장 후

설정의 `테스트` 버튼으로 한 번 확인하고 `저장` 누르면 30초마다 자동 폴링이 시작됩니다. 메뉴바 잔량이 즉시 정확한 값으로 바뀝니다.

---

## 동작 원리

### 데이터 소스

```
[로컬 모드]
~/.claude/projects/**/*.jsonl
  → 각 assistant 메시지의 usage 합산
  → 직전 5시간 / 7일 슬라이딩 윈도우

[API 모드, 선택]
GET https://claude.ai/api/organizations/<org-id>/usage
  → Anthropic 공식 utilization% 그대로
  → 30초마다 폴링
```

### 펫 상태 → 캐릭터

| 잔량 | 상태 | 캐릭터 |
| --- | --- | --- |
| 90~100% | full | 가장 활기찬 모습 |
| 77~90% | high | 활발 |
| 63~77% | good | 보통 |
| 49~63% | mid | 약간 처짐 |
| 33~49% | low | 시름시름 |
| 15~33% | tired | 녹초 |
| 0~15% | sleepy | 졸림 |
| 주간 0% | dead | 💀 뻗음 |

### 캐시는 얼마 만에 초기화되나요?

- **프롬프트 캐시 TTL: 5분.**
  마지막 요청 시각 + 5분이 지나면 다음 요청은 캐시 재생성으로 시작합니다. 펫 머리 위 말풍선이 남은 시간을 카운트다운하고, 1분 이하로 떨어지면 살짝 흔들리며 "`.` 이라도 눌러!" 라고 알려줍니다.
- **세션 쿠키(`sessionKey`, `routingHint`)**: 보통 약 30일.
- **Cloudflare 쿠키(`__cf_bm`, `cf_clearance`, `_cfuvid`)**: 짧으면 30분 ~ 길면 수 시간. claude.ai 탭을 자주 열어두면 자동 갱신됩니다.

### 쿠키가 다 떨어지면

폴링이 HTTP 401·403·404를 받으면 앱이 자동으로 감지하고 **설정 창을 다시 열어줍니다.** 그 자리에서 claude.ai에 다시 들어가 새 쿠키 줄을 복사해 붙여넣고 `저장`만 누르면 끝납니다. 메뉴바 잔량이 곧바로 다시 흐르기 시작합니다.

---

## 한계

- **macOS 11+ 만 지원합니다.** Windows / Linux 빌드는 현재 없습니다. (Tauri 자체는 다 지원하지만, 메뉴바·드래그·항상 위 패널 동작이 macOS 전제로 짜여 있습니다.)
- **5h / 주간 한도는 추정치**입니다. Anthropic이 공식 한도를 공개하지 않아, 로컬 모드는 [`src/types.ts`](src/types.ts)의 `PLAN_PRESETS` 값과 비교합니다. 더 정확한 값을 원하면 **API 연동**을 켜세요 — 그러면 Anthropic의 공식 utilization%를 그대로 가져옵니다.
- **5h 윈도우는 슬라이딩 방식**이라 Anthropic의 "첫 호출 시점부터 5시간" 정의와 미세하게 다를 수 있습니다.
- **메뉴바 트레이 아이콘 사이즈**는 macOS 표준에 맞춰 `22×22` (1×) / `44×44` (2×) 템플릿 이미지로 동작합니다. 마스터 아이콘은 `src-tauri/icons/icon.png` (1024×1024) 한 장에서 자동 생성됩니다.

---

## 보안 / 프라이버시

- ✅ **로컬에서만 동작합니다.** Org ID와 쿠키는 이 컴퓨터 안 (Tauri plugin-store 기본 경로)에만 저장되고, 앱이 직접 `claude.ai`로 요청을 보낼 뿐 **외부 서버로 데이터를 넘기지 않습니다.**
- ⚠️ **Org ID와 세션 쿠키는 사실상 본인의 Claude 계정 자격증명**입니다. 가져간 사람이 본인 계정으로 사용량을 늘리거나 조회할 수 있습니다. **공유하지 마세요.** 화면 녹화·캡쳐·디스코드/슬랙에 붙여넣기 등으로 노출되지 않도록 조심하세요.
- 쿠키가 의심스러우면 [claude.ai/settings/account](https://claude.ai/settings/account) → "활성 세션"에서 해당 세션을 로그아웃하면 즉시 무효화됩니다.
- 읽는 파일: `~/.claude/projects/**/*.jsonl` (Claude Code가 만든 본인 로그)만.

---

## 버전

| 버전 | 날짜 | 주요 변경 |
| --- | --- | --- |
| 0.21.0 | 2026-05-01 | README 한국어 전면 개편, 다운로드 배지/보안 가이드 추가 |
| 0.20.0 | 2026-05-01 | 설정 카드 회색 배경, 설정창 폭 확장 (440→600), 패딩 균일화, skin tile 흰색 박스 제거 |
| 0.19.0 | 2026-05-01 | 🔔 도움말 칩 + 쿠키 흐름 다이어그램, 트레이 좌클릭 토글 제거(메뉴 호출만), 말풍선 드래그, doze 모션 시 캐릭터 강제 스왑 버그 수정 |
| 0.18.0 | — | NonactivatingPanel 마스크 제거, 설정 창 강제 포커스 |
| 이전 | — | API 연동, 5단계 에너지 시스템, 13종 idle 모션, 펫 핀 동작 등 |

---

## 라이선스

MIT — 자유롭게 포크/수정/재배포하세요. 자세한 내용은 [`LICENSE`](LICENSE) 참고.

---

## 컨택트

버그를 발견했거나 기능 추가가 필요하면 편하게 연락주세요.

- 📧 [johnprk1993@gmail.com](mailto:johnprk1993@gmail.com)
- 🐛 [Issues](https://github.com/JohnPrk/claude-desk-helper/issues)
- 🤝 PR 환영합니다.
