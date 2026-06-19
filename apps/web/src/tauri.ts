import { invoke } from "@tauri-apps/api/core";
import {
  fallbackOverview,
  fallbackAssetLibrary,
  fallbackCompositionEditor,
  fallbackExportWorkflow,
  fallbackMvpValidation,
  fallbackRightsSafety,
  fallbackReviewWorkspace,
  fallbackRuntime,
  fallbackSamplesStudio,
  fallbackSongStudio,
  fallbackSfxStudio,
  fallbackTtsStudio,
  fallbackVideoToAudio,
  fallbackVoiceLab,
  fallbackWorkspace,
} from "./appData";
import type {
  AppOverview,
  AssetLibraryOverview,
  CompositionEditorOverview,
  ExportWorkflowOverview,
  MvpValidationOverview,
  RightsSafetyOverview,
  ReviewWorkspaceOverview,
  RuntimeOverview,
  SamplesStudioOverview,
  SongStudioOverview,
  SfxStudioOverview,
  TtsStudioOverview,
  VideoToAudioOverview,
  VoiceLabOverview,
  WorkspaceOverview,
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

export async function loadWorkspaceOverview(): Promise<WorkspaceOverview> {
  try {
    return await invoke<WorkspaceOverview>("get_workspace_overview");
  } catch {
    return fallbackWorkspace;
  }
}

export async function loadAssetLibraryOverview(): Promise<AssetLibraryOverview> {
  try {
    return await invoke<AssetLibraryOverview>("get_asset_library_overview");
  } catch {
    return fallbackAssetLibrary;
  }
}

export async function loadExportWorkflowOverview(): Promise<ExportWorkflowOverview> {
  try {
    return await invoke<ExportWorkflowOverview>("get_export_workflow_overview");
  } catch {
    return fallbackExportWorkflow;
  }
}

export async function loadCompositionEditorOverview(): Promise<CompositionEditorOverview> {
  try {
    return await invoke<CompositionEditorOverview>(
      "get_composition_editor_overview",
    );
  } catch {
    return fallbackCompositionEditor;
  }
}

export async function loadMvpValidationOverview(): Promise<MvpValidationOverview> {
  try {
    return await invoke<MvpValidationOverview>("get_mvp_validation_overview");
  } catch {
    return fallbackMvpValidation;
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

export async function loadSamplesStudioOverview(): Promise<SamplesStudioOverview> {
  try {
    return await invoke<SamplesStudioOverview>("get_samples_studio_overview");
  } catch {
    return fallbackSamplesStudio;
  }
}

export async function loadSongStudioOverview(): Promise<SongStudioOverview> {
  try {
    return await invoke<SongStudioOverview>("get_song_studio_overview");
  } catch {
    return fallbackSongStudio;
  }
}

export async function loadReviewWorkspaceOverview(): Promise<ReviewWorkspaceOverview> {
  try {
    return await invoke<ReviewWorkspaceOverview>(
      "get_review_workspace_overview",
    );
  } catch {
    return fallbackReviewWorkspace;
  }
}

export async function loadRightsSafetyOverview(): Promise<RightsSafetyOverview> {
  try {
    return await invoke<RightsSafetyOverview>("get_rights_safety_overview");
  } catch {
    return fallbackRightsSafety;
  }
}

export async function loadVideoToAudioOverview(): Promise<VideoToAudioOverview> {
  try {
    return await invoke<VideoToAudioOverview>("get_video_to_audio_overview");
  } catch {
    return fallbackVideoToAudio;
  }
}
