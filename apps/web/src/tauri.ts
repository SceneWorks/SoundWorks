import { invoke } from "@tauri-apps/api/core";
import {
  fallbackOverview,
  fallbackRuntime,
  fallbackSfxStudio,
  fallbackTtsStudio,
  fallbackVoiceLab,
} from "./appData";
import type {
  AppOverview,
  RuntimeOverview,
  SfxStudioOverview,
  TtsStudioOverview,
  VoiceLabOverview,
} from "./types";

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

export async function loadTtsStudioOverview(): Promise<TtsStudioOverview> {
  try {
    return await invoke<TtsStudioOverview>("get_tts_studio_overview");
  } catch {
    return fallbackTtsStudio;
  }
}

export async function loadVoiceLabOverview(): Promise<VoiceLabOverview> {
  try {
    return await invoke<VoiceLabOverview>("get_voice_lab_overview");
  } catch {
    return fallbackVoiceLab;
  }
}

export async function loadSfxStudioOverview(): Promise<SfxStudioOverview> {
  try {
    return await invoke<SfxStudioOverview>("get_sfx_studio_overview");
  } catch {
    return fallbackSfxStudio;
  }
}
