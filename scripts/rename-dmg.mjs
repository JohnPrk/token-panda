#!/usr/bin/env node
// Tauri는 productName(`토큰 판다`)을 dmg 파일명 prefix로 그대로 박는다.
// 한글 + 공백이 들어간 파일명을 GitHub 릴리스에 업로드하면 GitHub이
// 자산 이름을 잘라(한글이 통째로 사라져 `_1.0.x_aarch64.dmg`로 보이는
// 케이스가 있음) README 배지 링크가 깨진다. 그래서 빌드 후 dmg를
// ASCII 통일 이름(`token-panda_X.Y.Z_aarch64.dmg`)으로 강제 rename한다.
// README 배지 링크도 이 파일명을 가리키도록 맞추면 GitHub이 어떻게
// 인코딩하든 무관하게 latest/download 링크가 깨지지 않는다.

import { readdirSync, renameSync, existsSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const here = dirname(fileURLToPath(import.meta.url));
const dmgDir = resolve(here, "..", "src-tauri/target/release/bundle/dmg");

if (!existsSync(dmgDir)) {
  console.log(`[rename-dmg] no dmg dir at ${dmgDir} — skipping`);
  process.exit(0);
}

let renamed = 0;
for (const name of readdirSync(dmgDir)) {
  if (!name.endsWith(".dmg")) continue;
  // tauri는 `<productName>_<X.Y.Z>_<arch>.dmg` 형태로 출력. productName이
  // ASCII가 아닐 수 있어 prefix는 버리고 version/arch만 추출.
  const m = name.match(/_(\d+\.\d+\.\d+)_([^_]+)\.dmg$/);
  if (!m) {
    console.log(`[rename-dmg] skip (unexpected name): ${name}`);
    continue;
  }
  const [, version, arch] = m;
  const target = `token-panda_${version}_${arch}.dmg`;
  if (target === name) continue;
  renameSync(join(dmgDir, name), join(dmgDir, target));
  console.log(`[rename-dmg] ${name} → ${target}`);
  renamed += 1;
}

if (renamed === 0) {
  console.log("[rename-dmg] no rename needed");
}
