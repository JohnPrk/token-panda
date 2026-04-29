import {
  isPermissionGranted,
  requestPermission,
  sendNotification,
} from "@tauri-apps/plugin-notification";

let permissionChecked = false;
let permissionGranted = false;

async function ensurePermission(): Promise<boolean> {
  if (permissionChecked) return permissionGranted;
  permissionChecked = true;
  try {
    let granted = await isPermissionGranted();
    if (!granted) {
      const result = await requestPermission();
      granted = result === "granted";
    }
    permissionGranted = granted;
  } catch {
    permissionGranted = false;
  }
  return permissionGranted;
}

const fired = new Set<string>();

export async function maybeNotify(opts: {
  key: string;
  title: string;
  body: string;
}) {
  if (fired.has(opts.key)) return;
  const ok = await ensurePermission();
  if (!ok) return;
  try {
    sendNotification({ title: opts.title, body: opts.body });
    fired.add(opts.key);
  } catch {
    /* ignore */
  }
}

export function resetThreshold(prefix: string) {
  for (const k of [...fired]) {
    if (k.startsWith(prefix)) fired.delete(k);
  }
}
