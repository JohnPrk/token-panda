import type { PetState } from "./types";
import pandaFull from "./skins/panda/full.png";
import pandaHigh from "./skins/panda/high.png";
import pandaGood from "./skins/panda/good.png";
import pandaMid from "./skins/panda/mid.png";
import pandaLow from "./skins/panda/low.png";
import pandaTired from "./skins/panda/tired.png";
import pandaSleepy from "./skins/panda/sleepy.png";
import pandaDead from "./skins/panda/dead.png";

// Placeholder for the `disconnected` state — shown when claude.ai's
// API rejects our request (expired cookie, Cloudflare challenge, etc.).
// Currently reuses apple.png as a visibly different stand-in so the
// user can see "something is off". To finish this state, drop a real
// `src/skins/panda/disconnected.png` (256×256, transparent PNG) into
// the panda folder and switch this import to:
//   import pandaDisconnected from "./skins/panda/disconnected.png";
import pandaDisconnected from "./skins/panda/apple.png";

import pandaBamboo from "./skins/panda/bamboo.png";
import pandaApple from "./skins/panda/apple.png";
import pandaDumbbell from "./skins/panda/dumbbell.png";

// Action names used by the idle micro-action loop in App.tsx.
// A skin can optionally provide a .gif for any of these to express the
// motion via the gif itself instead of relying on CSS transforms.
export type ActionName =
  | "roll"
  | "bamboo"
  | "jump"
  | "spin"
  | "run"
  | "shy"
  | "doze"
  | "scratch"
  | "wave"
  | "lying"
  | "front-roll"
  | "eat-fruit"
  | "exercise";

export type Skin = {
  id: string;
  name: string;
  /** Static PNG (or any image) per pet state. Required. */
  frames: Record<PetState, string>;
  /**
   * Optional motion GIFs per idle action. If a gif is provided for an
   * action, the renderer swaps the static state PNG for the gif while
   * the action plays. If absent, the static PNG remains visible and the
   * existing CSS keyframes provide a fallback motion.
   */
  actions?: Partial<Record<ActionName, string>>;
};

export const SKINS: Skin[] = [
  {
    id: "panda",
    name: "Panda",
    frames: {
      full: pandaFull,
      high: pandaHigh,
      good: pandaGood,
      mid: pandaMid,
      low: pandaLow,
      tired: pandaTired,
      sleepy: pandaSleepy,
      dead: pandaDead,
      disconnected: pandaDisconnected,
    },
    // No motion GIFs yet — drop files into src/skins/panda/<action>.gif and
    // wire them up here (e.g. `roll: pandaRollGif`) to enable per-action
    // gif playback. Until then, CSS keyframes animate the static PNG.
    actions: {},
  },
];

export const ACCESSORIES = {
  bamboo: pandaBamboo,
  apple: pandaApple,
  dumbbell: pandaDumbbell,
};

export const DEFAULT_SKIN_ID = "panda";

export function findSkin(id: string): Skin {
  return SKINS.find((s) => s.id === id) ?? SKINS[0];
}
