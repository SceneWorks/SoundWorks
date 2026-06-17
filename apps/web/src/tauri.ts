import { invoke } from "@tauri-apps/api/core";
import { fallbackOverview, fallbackRuntime } from "./appData";
import type { AppOverview, RuntimeOverview } from "./types";

export async function loadAppOverview(): Promise<AppOverview> {
  try {
    return await invoke<AppOverview>("get_app_overview");
  } catch {
    return fallbackOverview;
  }
}

export async function loadRuntimeOverview(): Promise<RuntimeOverview> {
  try {
    return await invoke<RuntimeOverview>("get_runtime_overview");
  } catch {
    return fallbackRuntime;
  }
}
