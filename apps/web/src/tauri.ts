import { invoke } from "@tauri-apps/api/core";
import {
  fallbackOverview,
  fallbackAssetLibrary,
  fallbackCompositionEditor,
  fallbackExportWorkflow,
  fallbackMvpValidation,
  fallbackModelManager,
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
  RuntimeJobArtifact,
  RuntimeJobRequest,
  RuntimeJobSnapshot,
  ModelManagerOperation,
  ModelManagerOverview,
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

export async function enqueueRuntimeJob(
  request: RuntimeJobRequest,
): Promise<RuntimeJobSnapshot> {
  try {
    return await invoke<RuntimeJobSnapshot>("enqueue_runtime_job", { request });
  } catch {
    return fallbackRuntimeJob(request);
  }
}

export async function cancelRuntimeJob(
  jobId: string,
): Promise<RuntimeJobSnapshot | null> {
  try {
    return await invoke<RuntimeJobSnapshot | null>("cancel_runtime_job", {
      jobId,
    });
  } catch {
    const job = fallbackRuntime.jobs.find(
      (candidate) => candidate.id === jobId,
    );
    return job
      ? {
          ...job,
          status: "cancelled",
          cancellation: "completed",
          progress: {
            percent: job.progress?.percent ?? 0,
            message: "Cancellation persisted by runtime fallback.",
          },
        }
      : null;
  }
}

export async function retryRuntimeJob(
  jobId: string,
): Promise<RuntimeJobSnapshot | null> {
  try {
    return await invoke<RuntimeJobSnapshot | null>("retry_runtime_job", {
      jobId,
    });
  } catch {
    const job = fallbackRuntime.jobs.find(
      (candidate) => candidate.id === jobId,
    );
    return job ? { ...job, retryCount: job.retryCount + 1 } : null;
  }
}

export async function loadRuntimeJobArtifacts(
  jobId: string,
): Promise<RuntimeJobArtifact[]> {
  try {
    return await invoke<RuntimeJobArtifact[]>("get_runtime_job_artifacts", {
      jobId,
    });
  } catch {
    return (
      fallbackRuntime.jobs.find((job) => job.id === jobId)?.artifacts ?? []
    );
  }
}

export async function loadModelManagerOverview(): Promise<ModelManagerOverview> {
  try {
    return await invoke<ModelManagerOverview>("get_model_manager_overview");
  } catch {
    return fallbackModelManager;
  }
}

function fallbackRuntimeJob(request: RuntimeJobRequest): RuntimeJobSnapshot {
  const now = Date.now().toString();
  return {
    id: `fallback-${request.workflow}-${now}`,
    kind: request.kind,
    status: "failed",
    providerId: request.providerId,
    modelId: request.modelId,
    workflow: request.workflow,
    adapter: "research-only",
    progress: {
      percent: 100,
      message: "Runtime command is unavailable in web fallback mode.",
    },
    cancellation: "not-cancellable",
    retryCount: 0,
    createdAt: now,
    updatedAt: now,
    recordRoot: "web-fallback",
    recipePath: "web-fallback/recipe.json",
    modelMetadataPath: "web-fallback/model.json",
    eventsPath: "web-fallback/events.jsonl",
    logTail: ["Tauri runtime command unavailable"],
    artifacts: [],
    actionableError: {
      code: "runtime.command_unavailable",
      summary: "Runtime command unavailable",
      recovery: "Run SoundWorks in the Tauri desktop shell to enqueue jobs.",
    },
  };
}

export async function installModelCandidate(
  candidateId: string,
): Promise<ModelManagerOperation> {
  try {
    return await invoke<ModelManagerOperation>("install_model_candidate", {
      candidateId,
    });
  } catch {
    return (
      fallbackModelManager.operations.find(
        (operation) =>
          operation.candidateId === candidateId &&
          operation.action === "install",
      ) ?? {
        id: `install-${candidateId}`,
        candidateId,
        action: "install",
        status: "failed",
        progressPercent: 100,
        summary: "Install command is unavailable in web fallback mode.",
        recovery:
          "Open the Tauri desktop runtime or inspect the SoundWorks model cache manually.",
        logTail: [],
      }
    );
  }
}

export async function revalidateModelCandidate(
  candidateId: string,
): Promise<ModelManagerOperation> {
  try {
    return await invoke<ModelManagerOperation>("revalidate_model_candidate", {
      candidateId,
    });
  } catch {
    const candidate = fallbackModelManager.candidates.find(
      (candidate) => candidate.candidateId === candidateId,
    );
    return {
      id: `revalidate-${candidateId}`,
      candidateId,
      action: "revalidate",
      status: candidate?.cache.verified ? "succeeded" : "failed",
      progressPercent: 100,
      summary: candidate?.cache.verified
        ? `${candidate.name} cache evidence is verified.`
        : `${candidate?.name ?? candidateId} is not installed.`,
      recovery: candidate?.cache.verified
        ? null
        : "Missing required cache files must be downloaded and revalidated.",
      logTail: candidate ? [candidate.cache.evidence] : [],
    };
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
