import { Store } from "@tauri-apps/plugin-store";
import type { ApiConfig, PlanConfig } from "./types";

const STORE_FILE = "config.json";
const KEY_PLAN = "plan_config";
const KEY_API = "api_config";

let storePromise: Promise<Store> | null = null;
function getStore() {
  if (!storePromise) storePromise = Store.load(STORE_FILE);
  return storePromise;
}

export async function loadPlanConfig(): Promise<PlanConfig | null> {
  const store = await getStore();
  const v = await store.get<PlanConfig>(KEY_PLAN);
  return v ?? null;
}

export async function savePlanConfig(cfg: PlanConfig): Promise<void> {
  const store = await getStore();
  await store.set(KEY_PLAN, cfg);
  await store.save();
}

export async function loadApiConfig(): Promise<ApiConfig | null> {
  const store = await getStore();
  const v = await store.get<ApiConfig>(KEY_API);
  return v ?? null;
}

export async function saveApiConfig(cfg: ApiConfig | null): Promise<void> {
  const store = await getStore();
  if (cfg) {
    await store.set(KEY_API, cfg);
  } else {
    await store.delete(KEY_API);
  }
  await store.save();
}
