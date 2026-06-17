import { invoke } from "@tauri-apps/api/core";
import { fallbackOverview } from "./appData";
import type { AppOverview } from "./types";

export async function loadAppOverview(): Promise<AppOverview> {
  try {
    return await invoke<AppOverview>("get_app_overview");
  } catch {
    return fallbackOverview;
  }
}
