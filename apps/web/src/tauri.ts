import { convertFileSrc, invoke } from "@tauri-apps/api/core";
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
  CreateProjectRequest,
  ExportLibraryItemRequest,
  ExportLibraryItemResult,
  ExportWorkflowOverview,
  ImportRuntimeArtifactRequest,
  LibraryMutationRequest,
  LibraryPlayback,
  MvpValidationOverview,
  ProjectLibraryActionResult,
  ReviewEditResult,
  RightsSafetyOverview,
  ReviewWorkspaceOverview,
  RuntimeOverview,
  RuntimeJobArtifact,
  RuntimeJobRequest,
  RuntimeJobSnapshot,
  ModelManagerOperation,
  ModelManagerOverview,
  SamplesStudioOverview,
  SaveReviewEditRequest,
  SongStudioOverview,
  SfxStudioOverview,
  TtsStudioOverview,
  VideoToAudioOverview,
  VoiceLabOverview,
  WorkspaceOverview,
} from "./types";

/**
 * True when running inside the Tauri desktop shell (so `invoke` is available).
 *
 * Detected once via the injected `__TAURI_INTERNALS__` global. In a plain
 * browser (`npm run dev`, static preview) this is false, and the UI is backed
 * by the `appData` fixtures rather than the real local SoundWorks library.
 */
export function isTauri(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

/**
 * Read a backend overview command.
 *
 * In web-preview mode the command is genuinely unavailable, so we return the
 * documented fixture. In desktop mode we let real command errors propagate to
 * the caller instead of masking them with stale fixtures (see F-008).
 */
async function readOverview<T>(command: string, fallback: T): Promise<T> {
  if (!isTauri()) {
    return fallback;
  }

  return await invoke<T>(command);
}

export async function loadAppOverview(): Promise<AppOverview> {
  return readOverview("get_app_overview", fallbackOverview);
}

export async function loadRuntimeOverview(): Promise<RuntimeOverview> {
  return readOverview("get_runtime_overview", fallbackRuntime);
}

export async function enqueueRuntimeJob(
  request: RuntimeJobRequest,
): Promise<RuntimeJobSnapshot> {
  if (!isTauri()) {
    return fallbackRuntimeJob(request);
  }

  return await invoke<RuntimeJobSnapshot>("enqueue_runtime_job", { request });
}

export async function cancelRuntimeJob(
  jobId: string,
): Promise<RuntimeJobSnapshot | null> {
  if (!isTauri()) {
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
            message: "Cancellation simulated in web preview.",
          },
        }
      : null;
  }

  return await invoke<RuntimeJobSnapshot | null>("cancel_runtime_job", {
    jobId,
  });
}

export async function retryRuntimeJob(
  jobId: string,
): Promise<RuntimeJobSnapshot | null> {
  if (!isTauri()) {
    const job = fallbackRuntime.jobs.find(
      (candidate) => candidate.id === jobId,
    );
    return job ? { ...job, retryCount: job.retryCount + 1 } : null;
  }

  return await invoke<RuntimeJobSnapshot | null>("retry_runtime_job", {
    jobId,
  });
}

export async function loadRuntimeJobArtifacts(
  jobId: string,
): Promise<RuntimeJobArtifact[]> {
  if (!isTauri()) {
    return (
      fallbackRuntime.jobs.find((job) => job.id === jobId)?.artifacts ?? []
    );
  }

  return await invoke<RuntimeJobArtifact[]>("get_runtime_job_artifacts", {
    jobId,
  });
}

export async function loadModelManagerOverview(): Promise<ModelManagerOverview> {
  return readOverview("get_model_manager_overview", fallbackModelManager);
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
  if (!isTauri()) {
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

  return await invoke<ModelManagerOperation>("install_model_candidate", {
    candidateId,
  });
}

export async function revalidateModelCandidate(
  candidateId: string,
): Promise<ModelManagerOperation> {
  if (!isTauri()) {
    const candidate = fallbackModelManager.candidates.find(
      (entry) => entry.candidateId === candidateId,
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

  return await invoke<ModelManagerOperation>("revalidate_model_candidate", {
    candidateId,
  });
}

export async function loadWorkspaceOverview(): Promise<WorkspaceOverview> {
  return readOverview("get_workspace_overview", fallbackWorkspace);
}

export async function loadAssetLibraryOverview(): Promise<AssetLibraryOverview> {
  return readOverview("get_asset_library_overview", fallbackAssetLibrary);
}

export async function createSoundWorksProject(
  request: CreateProjectRequest,
): Promise<ProjectLibraryActionResult> {
  if (!isTauri()) {
    return fallbackProjectLibraryResult(
      "Create project requires the Tauri desktop shell; web preview cannot write the local SoundWorks library.",
    );
  }

  return await invoke<ProjectLibraryActionResult>("create_soundworks_project", {
    request,
  });
}

export async function openSoundWorksProject(
  projectId: string,
): Promise<ProjectLibraryActionResult> {
  if (!isTauri()) {
    return fallbackProjectLibraryResult(
      "Open project requires the Tauri desktop shell; web preview cannot read the persisted SoundWorks workspace.",
    );
  }

  return await invoke<ProjectLibraryActionResult>("open_soundworks_project", {
    projectId,
  });
}

export async function importRuntimeArtifactToLibrary(
  request: ImportRuntimeArtifactRequest,
): Promise<ProjectLibraryActionResult> {
  if (!isTauri()) {
    return fallbackProjectLibraryResult(
      "Saving runtime artifacts requires the Tauri desktop shell; web preview cannot copy audio into the library.",
    );
  }

  return await invoke<ProjectLibraryActionResult>(
    "import_runtime_artifact_to_library",
    { request },
  );
}

export async function mutateLibraryItem(
  request: LibraryMutationRequest,
): Promise<ProjectLibraryActionResult> {
  if (!isTauri()) {
    return fallbackProjectLibraryResult(
      "Library mutations require the Tauri desktop shell; web preview cannot persist metadata sidecars.",
    );
  }

  return await invoke<ProjectLibraryActionResult>("mutate_library_item", {
    request,
  });
}

export async function loadLibraryPlayback(
  itemId: string,
): Promise<LibraryPlayback> {
  if (!isTauri()) {
    return {
      itemId,
      playable: false,
      reason:
        "Preview requires the Tauri desktop shell so SoundWorks can authorize local audio file playback.",
    };
  }

  const playback = await invoke<LibraryPlayback>("get_library_playback", {
    itemId,
  });

  return playback.path
    ? { ...playback, path: convertFileSrc(playback.path) }
    : playback;
}

export async function saveReviewEdit(
  request: SaveReviewEditRequest,
): Promise<ReviewEditResult> {
  if (!isTauri()) {
    throw new Error(
      "Saving review edits requires the Tauri desktop shell and a persisted audio file.",
    );
  }

  return await invoke<ReviewEditResult>("save_review_edit", { request });
}

export async function exportLibraryItem(
  request: ExportLibraryItemRequest,
): Promise<ExportLibraryItemResult> {
  if (!isTauri()) {
    throw new Error(
      "Export requires the Tauri desktop shell and a persisted audio file.",
    );
  }

  return await invoke<ExportLibraryItemResult>("export_library_item", {
    request,
  });
}

function fallbackProjectLibraryResult(
  message: string,
): ProjectLibraryActionResult {
  return {
    workspace: fallbackWorkspace,
    assetLibrary: fallbackAssetLibrary,
    selectedItem: fallbackAssetLibrary.selectedItem,
    message,
  };
}

export async function loadExportWorkflowOverview(): Promise<ExportWorkflowOverview> {
  return readOverview("get_export_workflow_overview", fallbackExportWorkflow);
}

export async function loadCompositionEditorOverview(): Promise<CompositionEditorOverview> {
  return readOverview(
    "get_composition_editor_overview",
    fallbackCompositionEditor,
  );
}

export async function loadMvpValidationOverview(): Promise<MvpValidationOverview> {
  return readOverview("get_mvp_validation_overview", fallbackMvpValidation);
}

export async function loadTtsStudioOverview(): Promise<TtsStudioOverview> {
  return readOverview("get_tts_studio_overview", fallbackTtsStudio);
}

export async function loadVoiceLabOverview(): Promise<VoiceLabOverview> {
  return readOverview("get_voice_lab_overview", fallbackVoiceLab);
}

export async function loadSfxStudioOverview(): Promise<SfxStudioOverview> {
  return readOverview("get_sfx_studio_overview", fallbackSfxStudio);
}

export async function loadSamplesStudioOverview(): Promise<SamplesStudioOverview> {
  return readOverview("get_samples_studio_overview", fallbackSamplesStudio);
}

export async function loadSongStudioOverview(): Promise<SongStudioOverview> {
  return readOverview("get_song_studio_overview", fallbackSongStudio);
}

export async function loadReviewWorkspaceOverview(): Promise<ReviewWorkspaceOverview> {
  return readOverview("get_review_workspace_overview", fallbackReviewWorkspace);
}

export async function loadRightsSafetyOverview(): Promise<RightsSafetyOverview> {
  return readOverview("get_rights_safety_overview", fallbackRightsSafety);
}

export async function loadVideoToAudioOverview(): Promise<VideoToAudioOverview> {
  return readOverview("get_video_to_audio_overview", fallbackVideoToAudio);
}
