import {
  Activity,
  Archive,
  Boxes,
  CircleAlert,
  CircleCheck,
  ClipboardCheck,
  Cpu,
  Disc3,
  Download,
  FileAudio,
  FileVideo,
  Gauge,
  HardDrive,
  Library,
  Mic2,
  Moon,
  Music2,
  PackageCheck,
  Play,
  Radio,
  Save,
  Search,
  ShieldCheck,
  SlidersHorizontal,
  Sun,
  Waves,
} from "lucide-react";
import { useEffect, useMemo, useRef, useState } from "react";
import type { CSSProperties } from "react";
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
import {
  loadAppOverview,
  loadAssetLibraryOverview,
  loadCompositionEditorOverview,
  cancelRuntimeJob,
  createSoundWorksProject,
  enqueueRuntimeJob,
  exportLibraryItem,
  importRuntimeArtifactToLibrary,
  loadLibraryPlayback,
  loadExportWorkflowOverview,
  installModelCandidate,
  mutateLibraryItem,
  openSoundWorksProject,
  retryRuntimeJob,
  saveReviewEdit,
  loadModelManagerOverview,
  loadMvpValidationOverview,
  loadRightsSafetyOverview,
  loadReviewWorkspaceOverview,
  loadRuntimeOverview,
  loadSamplesStudioOverview,
  loadSongStudioOverview,
  loadSfxStudioOverview,
  loadTtsStudioOverview,
  loadVideoToAudioOverview,
  loadVoiceLabOverview,
  loadWorkspaceOverview,
  revalidateModelCandidate,
  isTauri,
  loadUiPreferences,
  saveUiPreferences,
} from "./tauri";
import {
  ACCENTS,
  DEFAULT_ACCENT,
  isAccentId,
  isThemeMode,
} from "./accents";
import type { ThemeMode } from "./accents";
import {
  countFor,
  formatDuration,
  formatMb,
  isHttpUrl,
  runtimeModelFor,
  scopeLabel,
  statusLabel,
  toLibraryMutationAction,
  visibleModelManagerOperation,
  workflowLabel,
} from "./viewModel";
import type { ActiveView, NavItem } from "./viewModel";
import { AppContext } from "./screens/context";
import type { AppContextValue } from "./screens/context";
import { WorkspaceScreen } from "./screens/WorkspaceScreen";
import { TtsScreen } from "./screens/TtsScreen";
import { VoiceLabScreen } from "./screens/VoiceLabScreen";
import { SfxScreen } from "./screens/SfxScreen";
import { VideoToAudioScreen } from "./screens/VideoToAudioScreen";
import { SamplesScreen } from "./screens/SamplesScreen";
import { SongScreen } from "./screens/SongScreen";
import type {
  AppOverview,
  AssetLibraryOverview,
  CompositionEditorOverview,
  ExportWorkflowOverview,
  LibraryMutationAction,
  LibraryPlayback,
  ModelManagerOperation,
  ModelManagerOverview,
  MvpValidationOverview,
  RightsSafetyOverview,
  ReviewWorkspaceOverview,
  RuntimeOverview,
  RuntimeJobRequest,
  RuntimeJobSnapshot,
  SamplesStudioOverview,
  SongStudioOverview,
  SfxStudioOverview,
  TtsStudioOverview,
  VideoToAudioOverview,
  VoiceLabOverview,
  WorkspaceOverview,
} from "./types";

const navSections: Array<{ label: string; items: NavItem[] }> = [
  {
    label: "Workspace",
    items: [
      {
        id: "workspace",
        label: "Project",
        title: "Project workspace",
        blurb:
          "Create projects, inspect global reuse, and jump into the next audio task.",
        icon: Gauge,
      },
      {
        id: "library",
        label: "Library",
        title: "Audio library",
        blurb:
          "Browse project and global assets with tags, versions, recipes, and playback.",
        icon: Library,
      },
      {
        id: "multitrack",
        label: "Multitrack",
        title: "Multitrack editor",
        blurb:
          "Arrange assets on a timeline, inspect mixer state, and prepare mixdowns.",
        icon: SlidersHorizontal,
      },
    ],
  },
  {
    label: "Studios",
    items: [
      {
        id: "tts",
        label: "TTS Studio",
        title: "TTS Studio",
        blurb: "Generate consented speech from scripts and speaker profiles.",
        icon: Mic2,
      },
      {
        id: "voice",
        label: "Voice Lab",
        title: "Voice Lab",
        blurb:
          "Manage voice profiles, conversion lanes, and consent-gated output.",
        icon: Radio,
      },
      {
        id: "sfx",
        label: "SFX + Ambience",
        title: "SFX + Ambience",
        blurb:
          "Generate effects, ambience beds, loopable variants, and Foley candidates.",
        icon: Waves,
      },
      {
        id: "video-audio",
        label: "Video Audio",
        title: "Video to Audio",
        blurb:
          "Map silent video events to synchronized audio with explicit blockers.",
        icon: FileVideo,
      },
      {
        id: "samples",
        label: "Samples",
        title: "Samples + Loops",
        blurb:
          "Create sample packs, musical loops, and metadata-rich production assets.",
        icon: Boxes,
      },
      {
        id: "song",
        label: "Song Studio",
        title: "Song Studio",
        blurb:
          "Draft songs, inspect sections and stems, and keep full-song blockers visible.",
        icon: Music2,
      },
    ],
  },
  {
    label: "Review / Export",
    items: [
      {
        id: "review",
        label: "Review",
        title: "Waveform review",
        blurb:
          "Preview, trim, normalize, compare, and save non-destructive versions.",
        icon: Play,
      },
      {
        id: "export",
        label: "Export",
        title: "Export workflow",
        blurb:
          "Write audio exports, sidecars, and SceneWorks handoff packages.",
        icon: Download,
      },
      {
        id: "rights",
        label: "Rights",
        title: "Rights + safety",
        blurb:
          "Review consent, model-use gates, disclosure, and provenance requirements.",
        icon: ShieldCheck,
      },
    ],
  },
  {
    label: "System",
    items: [
      {
        id: "jobs",
        label: "Jobs",
        title: "Runtime jobs",
        blurb:
          "Inspect verified runtime models, active jobs, artifacts, and recovery actions.",
        icon: Activity,
      },
      {
        id: "models",
        label: "Models",
        title: "Model manager",
        blurb:
          "Verify model caches, installability, lane readiness, and source-backed blockers.",
        icon: PackageCheck,
      },
      {
        id: "validation",
        label: "Validation",
        title: "MVP validation",
        blurb: "Track release evidence against the epic success criteria.",
        icon: ClipboardCheck,
      },
      {
        id: "settings",
        label: "Settings",
        title: "System settings",
        blurb:
          "Review app layers, command boundaries, provider coverage, and architecture.",
        icon: Cpu,
      },
    ],
  },
];

const navItems = navSections.flatMap((section) => section.items);

// DR-01: localStorage is the instant-paint cache for theme/accent; the durable
// copy lives in the Tauri ui-preferences store (seeded on launch). Both reads are
// guarded so private-mode / SSR never throws.
function readStoredTheme(): ThemeMode {
  if (typeof window === "undefined") {
    return "light";
  }
  try {
    const saved = window.localStorage.getItem("soundworks-theme");
    return isThemeMode(saved) ? saved : "light";
  } catch {
    return "light";
  }
}

function readStoredAccent(): string {
  if (typeof window === "undefined") {
    return DEFAULT_ACCENT;
  }
  try {
    const saved = window.localStorage.getItem("soundworks-accent");
    return isAccentId(saved) ? saved : DEFAULT_ACCENT;
  } catch {
    return DEFAULT_ACCENT;
  }
}

export function App() {
  const [activeView, setActiveView] = useState<ActiveView>("workspace");
  const [overview, setOverview] = useState<AppOverview>(fallbackOverview);
  const [runtime, setRuntime] = useState<RuntimeOverview>(fallbackRuntime);
  const [modelManager, setModelManager] =
    useState<ModelManagerOverview>(fallbackModelManager);
  const [modelManagerOperation, setModelManagerOperation] =
    useState<ModelManagerOperation | null>(
      visibleModelManagerOperation(fallbackModelManager.operations),
    );
  const [runtimeOperation, setRuntimeOperation] =
    useState<RuntimeJobSnapshot | null>(fallbackRuntime.jobs[0] ?? null);
  const [workspace, setWorkspace] =
    useState<WorkspaceOverview>(fallbackWorkspace);
  const [assetLibrary, setAssetLibrary] =
    useState<AssetLibraryOverview>(fallbackAssetLibrary);
  const [libraryActionStatus, setLibraryActionStatus] = useState(
    "Project and library actions are ready.",
  );
  const [reviewActionStatus, setReviewActionStatus] = useState(
    "Review actions require a saved runtime audio asset.",
  );
  const [exportActionStatus, setExportActionStatus] = useState(
    "Export writes are ready for persisted runtime audio.",
  );
  const [libraryPlayback, setLibraryPlayback] =
    useState<LibraryPlayback | null>(null);
  const [exportWorkflow, setExportWorkflow] = useState<ExportWorkflowOverview>(
    fallbackExportWorkflow,
  );
  const [compositionEditor, setCompositionEditor] =
    useState<CompositionEditorOverview>(fallbackCompositionEditor);
  const [mvpValidation, setMvpValidation] = useState<MvpValidationOverview>(
    fallbackMvpValidation,
  );
  const [ttsStudio, setTtsStudio] =
    useState<TtsStudioOverview>(fallbackTtsStudio);
  const [voiceLab, setVoiceLab] = useState<VoiceLabOverview>(fallbackVoiceLab);
  const [sfxStudio, setSfxStudio] =
    useState<SfxStudioOverview>(fallbackSfxStudio);
  const [samplesStudio, setSamplesStudio] = useState<SamplesStudioOverview>(
    fallbackSamplesStudio,
  );
  const [songStudio, setSongStudio] =
    useState<SongStudioOverview>(fallbackSongStudio);
  const [reviewWorkspace, setReviewWorkspace] =
    useState<ReviewWorkspaceOverview>(fallbackReviewWorkspace);
  const [rightsSafety, setRightsSafety] =
    useState<RightsSafetyOverview>(fallbackRightsSafety);
  const [videoToAudio, setVideoToAudio] =
    useState<VideoToAudioOverview>(fallbackVideoToAudio);
  const [dataError, setDataError] = useState<string | null>(null);

  // DR-01: theme + accent. Each change updates state instantly and persists the
  // single changed field to the durable store (fire-and-forget; localStorage is
  // the instant cache applied by the effects below).
  const [theme, setTheme] = useState<ThemeMode>(readStoredTheme);
  const changeTheme = (next: ThemeMode) => {
    setTheme(next);
    void saveUiPreferences({ theme: next });
  };
  const [accent, setAccent] = useState<string>(readStoredAccent);
  const changeAccent = (next: string) => {
    setAccent(next);
    void saveUiPreferences({ accent: next });
  };

  const webPreview = !isTauri();
  const mountedRef = useRef(true);
  const loadedViewsRef = useRef<Set<ActiveView>>(new Set());

  // Apply an overview load to component state, surfacing real (desktop) command
  // failures instead of silently keeping stale fixtures (F-008).
  function applyLoad<T>(promise: Promise<T>, apply: (value: T) => void) {
    promise
      .then((value) => {
        if (mountedRef.current) {
          apply(value);
        }
      })
      .catch((error) => {
        if (mountedRef.current) {
          setDataError(String(error));
        }
      });
  }

  // Eager loads: only the data the persistent chrome (header, queue chip,
  // studio grid, generation gating) needs regardless of the active view.
  useEffect(() => {
    mountedRef.current = true;
    applyLoad(loadAppOverview(), setOverview);
    applyLoad(loadRuntimeOverview(), setRuntime);
    applyLoad(loadWorkspaceOverview(), setWorkspace);

    return () => {
      mountedRef.current = false;
    };
  }, []);

  // DR-01: apply theme/accent to <html> and cache to localStorage. Because every
  // token-driven surface reads --bg/--text/--accent, flipping the attribute
  // recolors the app in one paint.
  useEffect(() => {
    if (typeof document === "undefined") {
      return;
    }
    document.documentElement.setAttribute("data-theme", theme);
    try {
      window.localStorage.setItem("soundworks-theme", theme);
    } catch {
      // ignore (private mode etc.)
    }
  }, [theme]);

  useEffect(() => {
    if (typeof document === "undefined") {
      return;
    }
    document.documentElement.setAttribute("data-accent", accent);
    try {
      window.localStorage.setItem("soundworks-accent", accent);
    } catch {
      // ignore (private mode etc.)
    }
  }, [accent]);

  // DR-01: seed from the durable store on launch (the authoritative copy;
  // localStorage is only an instant-paint cache). Each toggle persists itself, so
  // there is no save effect to race with this read. No-op in web preview.
  useEffect(() => {
    let cancelled = false;
    loadUiPreferences()
      .then((prefs) => {
        if (cancelled) {
          return;
        }
        if (isThemeMode(prefs.theme)) {
          setTheme(prefs.theme);
        }
        if (isAccentId(prefs.accent)) {
          setAccent(prefs.accent);
        }
      })
      .catch(() => {});
    return () => {
      cancelled = true;
    };
  }, []);

  // Lazy loads: each view's heavier overview is fetched on first navigation and
  // cached, instead of firing all 16 loaders on mount (F-011).
  useEffect(() => {
    const loaded = loadedViewsRef.current;
    if (loaded.has(activeView)) {
      return;
    }
    loaded.add(activeView);

    const loadAssetLibrary = () =>
      applyLoad(loadAssetLibraryOverview(), setAssetLibrary);

    switch (activeView) {
      case "library":
        loadAssetLibrary();
        break;
      case "multitrack":
        applyLoad(loadCompositionEditorOverview(), setCompositionEditor);
        break;
      case "tts":
        applyLoad(loadTtsStudioOverview(), setTtsStudio);
        break;
      case "voice":
        applyLoad(loadVoiceLabOverview(), setVoiceLab);
        break;
      case "sfx":
        applyLoad(loadSfxStudioOverview(), setSfxStudio);
        break;
      case "video-audio":
        applyLoad(loadVideoToAudioOverview(), setVideoToAudio);
        break;
      case "samples":
        applyLoad(loadSamplesStudioOverview(), setSamplesStudio);
        break;
      case "song":
        applyLoad(loadSongStudioOverview(), setSongStudio);
        break;
      case "review":
        applyLoad(loadReviewWorkspaceOverview(), setReviewWorkspace);
        loadAssetLibrary();
        break;
      case "export":
        applyLoad(loadExportWorkflowOverview(), setExportWorkflow);
        loadAssetLibrary();
        break;
      case "rights":
        applyLoad(loadRightsSafetyOverview(), setRightsSafety);
        break;
      case "models":
        applyLoad(loadModelManagerOverview(), (nextModelManager) => {
          setModelManager(nextModelManager);
          setModelManagerOperation(
            visibleModelManagerOperation(nextModelManager.operations),
          );
        });
        break;
      case "validation":
        applyLoad(loadMvpValidationOverview(), setMvpValidation);
        break;
      default:
        break;
    }
  }, [activeView]);

  function runModelManagerAction(
    candidateId: string,
    action: "install" | "revalidate",
  ) {
    const runner =
      action === "install" ? installModelCandidate : revalidateModelCandidate;
    runner(candidateId).then((operation) => {
      setModelManagerOperation(operation);
    });
  }

  function refreshRuntime() {
    loadRuntimeOverview().then((nextRuntime) => {
      setRuntime(nextRuntime);
      setRuntimeOperation(nextRuntime.jobs[0] ?? null);
    });
  }

  function refreshOverviewSummary() {
    loadAppOverview().then((nextOverview) => setOverview(nextOverview));
  }

  function applyProjectLibraryResult(result: {
    workspace: WorkspaceOverview;
    assetLibrary: AssetLibraryOverview;
    message: string;
  }) {
    setWorkspace(result.workspace);
    setAssetLibrary(result.assetLibrary);
    setLibraryActionStatus(result.message);
    refreshOverviewSummary();
  }

  function createProject() {
    createSoundWorksProject({
      name: `SoundWorks Recovery ${new Date().toLocaleTimeString()}`,
    })
      .then(applyProjectLibraryResult)
      .catch((error) => {
        setLibraryActionStatus(`Create project unavailable: ${String(error)}`);
      });
  }

  function openRecentProject() {
    const project =
      workspace.recentProjects.find(
        (candidate) =>
          candidate.project.id !== workspace.activeProject.project.id,
      ) ?? workspace.activeProject;
    openSoundWorksProject(project.project.id)
      .then(applyProjectLibraryResult)
      .catch((error) => {
        setLibraryActionStatus(`Open project unavailable: ${String(error)}`);
      });
  }

  function importLatestRuntimeArtifact() {
    const job = latestImportableRuntimeJob;
    if (!job) {
      setLibraryActionStatus(
        "No succeeded runtime audio artifact is available to save.",
      );
      return;
    }
    importRuntimeArtifactToLibrary({
      jobId: job.id,
      projectId: workspace.activeProject.project.id,
      name: `${workflowLabel(job.workflow)} saved output`,
      tags: [job.workflow, "saved-output"],
    })
      .then(applyProjectLibraryResult)
      .catch((error) => {
        setLibraryActionStatus(
          `Save runtime artifact unavailable: ${String(error)}`,
        );
      });
  }

  function mutateSelectedLibraryItem(action: LibraryMutationAction) {
    const detail = assetLibrary.selectedItem;
    if (!detail) {
      setLibraryActionStatus("Select a library item first.");
      return;
    }
    mutateLibraryItem({
      itemId: detail.item.id,
      action,
      tag: action === "add-tag" ? "reviewed" : null,
    })
      .then(applyProjectLibraryResult)
      .catch((error) => {
        setLibraryActionStatus(`Library action unavailable: ${String(error)}`);
      });
  }

  function previewLibraryItem(itemId: string) {
    loadLibraryPlayback(itemId)
      .then((playback) => {
        setLibraryPlayback(playback);
        setLibraryActionStatus(
          playback.playable
            ? `Previewing ${itemId} from disk.`
            : (playback.reason ?? "Preview is unavailable."),
        );
      })
      .catch((error) => {
        setLibraryPlayback(null);
        setLibraryActionStatus(`Preview unavailable: ${String(error)}`);
      });
  }

  function saveSelectedReviewEdit() {
    const selection = reviewWorkspace.transport.selection;
    const detail = assetLibrary.selectedItem;
    if (!detail) {
      setReviewActionStatus("Select a library item first.");
      return;
    }
    const itemId = detail.item.id;
    saveReviewEdit({
      itemId,
      startMs: selection?.startMs ?? 0,
      endMs:
        selection?.endMs ??
        detail.item.durationMs ??
        reviewWorkspace.transport.durationMs,
      fadeInMs: 60,
      fadeOutMs: 120,
      normalizeLoudnessLufs: -16,
    })
      .then((result) => {
        applyProjectLibraryResult(result.library);
        setReviewActionStatus(
          `Saved ${result.versionId} from real audio at ${result.editedPath}.`,
        );
        if (result.library.selectedItem) {
          previewLibraryItem(result.library.selectedItem.item.id);
        }
        loadReviewWorkspaceOverview().then(setReviewWorkspace);
      })
      .catch((error) => {
        setReviewActionStatus(`Save version unavailable: ${String(error)}`);
      });
  }

  function exportSelectedLibraryItem() {
    const detail = assetLibrary.selectedItem;
    if (!detail) {
      setExportActionStatus("Select a library item first.");
      return;
    }
    const itemId = detail.item.id;
    exportLibraryItem({
      itemId,
      presetId: exportWorkflow.selectedExport.presetId,
      formats: exportWorkflow.selectedExport.formats,
      sceneWorksProjectId:
        exportWorkflow.sceneWorksHandoff.intendedProjectId ?? null,
      sceneWorksVideoAssetId:
        exportWorkflow.sceneWorksHandoff.intendedVideoAssetId ?? null,
      replaceExistingAudio:
        exportWorkflow.sceneWorksHandoff.replaceExistingAudio,
    })
      .then((result) => {
        const audioCount = result.artifacts.filter(
          (artifact) => artifact.kind === "audio-file",
        ).length;
        const warningText = result.warnings.length
          ? ` ${result.warnings.join(" ")}`
          : "";
        setExportActionStatus(
          `Export wrote ${audioCount} audio file plus sidecars to ${result.outputRoot}.${warningText}`,
        );
        loadExportWorkflowOverview().then(setExportWorkflow);
      })
      .catch((error) => {
        setExportActionStatus(`Export unavailable: ${String(error)}`);
      });
  }

  function runRuntimeJob(
    workflow: RuntimeJobRequest["workflow"],
    prompt: string,
    parameters: Record<string, unknown> = {},
  ) {
    const model = runtimeModelFor(runtime, workflow);
    const request: RuntimeJobRequest = {
      providerId: model?.providerId ?? "missing-provider",
      modelId: model?.modelId ?? "missing-model",
      kind: "generate-audio",
      workflow,
      prompt,
      sourceSurface: workflowLabel(workflow),
      parameters: {
        cachePath: model?.cache.cachePath ?? null,
        modelVersion: null,
        ...parameters,
      },
    };
    enqueueRuntimeJob(request).then((job) => {
      setRuntimeOperation(job);
      if (workflow === "tts" && job.status === "succeeded") {
        importRuntimeArtifactToLibrary({
          jobId: job.id,
          projectId: workspace.activeProject.project.id,
          name: `${workflowLabel(job.workflow)} generated voice clip`,
          tags: ["tts", "voice-clip", "generated-speech"],
        })
          .then(applyProjectLibraryResult)
          .catch((error) => {
            setLibraryActionStatus(
              `TTS generated but save unavailable: ${String(error)}`,
            );
          });
      }
      if (
        (workflow === "sfx" || workflow === "ambience") &&
        job.status === "succeeded"
      ) {
        importRuntimeArtifactToLibrary({
          jobId: job.id,
          projectId: workspace.activeProject.project.id,
          name: `${workflowLabel(job.workflow)} generated ${workflow === "ambience" ? "bed" : "effect"}`,
          tags: [workflow, "generated-audio", ...(sfxStudio.prompt.tags ?? [])],
        })
          .then(applyProjectLibraryResult)
          .catch((error) => {
            setLibraryActionStatus(
              `SFX generated but save unavailable: ${String(error)}`,
            );
          });
      }
      if (
        (workflow === "instrument-sample" || workflow === "loop") &&
        job.status === "succeeded"
      ) {
        importRuntimeArtifactToLibrary({
          jobId: job.id,
          projectId: workspace.activeProject.project.id,
          name: `${workflowLabel(job.workflow)} generated ${workflow === "loop" ? "loop" : "sample"}`,
          tags: [
            workflow,
            "generated-audio",
            samplesStudio.controls.musicalKey,
            `${samplesStudio.controls.bpm}-bpm`,
            ...(samplesStudio.prompt.genreTags ?? []),
          ],
        })
          .then(applyProjectLibraryResult)
          .catch((error) => {
            setLibraryActionStatus(
              `Sample/loop generated but save unavailable: ${String(error)}`,
            );
          });
      }
      if (
        (workflow === "voice-conversion" ||
          workflow === "video-to-audio" ||
          workflow === "song") &&
        job.status === "succeeded"
      ) {
        importRuntimeArtifactToLibrary({
          jobId: job.id,
          projectId: workspace.activeProject.project.id,
          name: `${workflowLabel(job.workflow)} generated output`,
          tags: [workflow, "generated-audio"],
        })
          .then(applyProjectLibraryResult)
          .catch((error) => {
            setLibraryActionStatus(
              `${workflowLabel(workflow)} generated but save unavailable: ${String(error)}`,
            );
          });
      }
      refreshRuntime();
    });
  }

  function cancelRuntimeOperation(jobId: string) {
    cancelRuntimeJob(jobId).then((job) => {
      if (job) {
        setRuntimeOperation(job);
      }
      refreshRuntime();
    });
  }

  function retryRuntimeOperation(jobId: string) {
    retryRuntimeJob(jobId).then((job) => {
      if (job) {
        setRuntimeOperation(job);
      }
      refreshRuntime();
    });
  }

  const scaffoldedLayerCount = useMemo(
    () =>
      overview.architecture.layers.filter(
        (layer) => layer.status === "scaffolded",
      ).length,
    [overview.architecture.layers],
  );
  const voiceCandidateFocus = useMemo(
    () =>
      voiceLab.providerScorecards.filter((scorecard) =>
        ["chatterbox", "rvc", "chatterbox-turbo"].includes(
          scorecard.candidateId,
        ),
      ),
    [voiceLab.providerScorecards],
  );
  const sfxCandidateFocus = useMemo(
    () =>
      sfxStudio.providerScorecards.filter((scorecard) =>
        ["moss-soundeffect", "stable-audio-open-1", "mmaudio"].includes(
          scorecard.candidateId,
        ),
      ),
    [sfxStudio.providerScorecards],
  );
  const videoCandidateFocus = useMemo(
    () =>
      videoToAudio.providerScorecards.filter((scorecard) =>
        ["mmaudio", "audiox", "thinksound", "moss-soundeffect"].includes(
          scorecard.candidateId,
        ),
      ),
    [videoToAudio.providerScorecards],
  );
  const samplesCandidateFocus = useMemo(
    () =>
      samplesStudio.providerScorecards.filter((scorecard) =>
        ["ace-step-1-5", "stable-audio-open-1", "heartmula"].includes(
          scorecard.candidateId,
        ),
      ),
    [samplesStudio.providerScorecards],
  );
  const songCandidateFocus = useMemo(
    () =>
      songStudio.providerScorecards.filter((scorecard) =>
        ["ace-step-1-5", "stable-audio-3", "diffrhythm-2"].includes(
          scorecard.candidateId,
        ),
      ),
    [songStudio.providerScorecards],
  );
  const ttsRuntimeModel = useMemo(
    () => runtimeModelFor(runtime, "tts"),
    [runtime],
  );
  const sfxRuntimeModel = useMemo(
    () => runtimeModelFor(runtime, "sfx"),
    [runtime],
  );
  const voiceRuntimeModel = useMemo(
    () => runtimeModelFor(runtime, "voice-conversion"),
    [runtime],
  );
  const videoRuntimeModel = useMemo(
    () => runtimeModelFor(runtime, "video-to-audio"),
    [runtime],
  );
  const songRuntimeModel = useMemo(
    () => runtimeModelFor(runtime, "song"),
    [runtime],
  );
  const latestImportableRuntimeJob = useMemo(
    () =>
      runtime.jobs.find(
        (job) =>
          job.status === "succeeded" &&
          job.artifacts.some((artifact) => artifact.kind === "audio-preview"),
      ) ?? null,
    [runtime.jobs],
  );

  const activeViewMeta =
    navItems.find((item) => item.id === activeView) ?? navItems[0];
  const showWorkspace = activeView === "workspace";
  const showLibrary = activeView === "library";
  const showMixer = activeView === "multitrack";
  const showTts = activeView === "tts";
  const showVoice = activeView === "voice";
  const showSfx = activeView === "sfx";
  const showVideoToAudio = activeView === "video-audio";
  const showSamples = activeView === "samples";
  const showSong = activeView === "song";
  const showReview = activeView === "review";
  const showExport = activeView === "export";
  const showRights = activeView === "rights";
  const showJobs = activeView === "jobs";
  const showModels = activeView === "models";
  const showValidation = activeView === "validation";
  const showSettings = activeView === "settings";
  const showSystemSurfaces = showJobs || showModels || showSettings;

  // DR-02: the contract the extracted per-ActiveView screens consume via
  // useAppContext() — all shared state, derived values, and handlers in one place.
  const contextValue: AppContextValue = {
    activeView,
    setActiveView,
    activeViewMeta,
    theme,
    accent,
    changeTheme,
    changeAccent,
    webPreview,
    dataError,
    overview,
    runtime,
    modelManager,
    modelManagerOperation,
    runtimeOperation,
    workspace,
    assetLibrary,
    exportWorkflow,
    compositionEditor,
    mvpValidation,
    ttsStudio,
    voiceLab,
    sfxStudio,
    samplesStudio,
    songStudio,
    reviewWorkspace,
    rightsSafety,
    videoToAudio,
    libraryPlayback,
    libraryActionStatus,
    reviewActionStatus,
    exportActionStatus,
    scaffoldedLayerCount,
    voiceCandidateFocus,
    sfxCandidateFocus,
    videoCandidateFocus,
    samplesCandidateFocus,
    songCandidateFocus,
    ttsRuntimeModel,
    sfxRuntimeModel,
    voiceRuntimeModel,
    videoRuntimeModel,
    songRuntimeModel,
    latestImportableRuntimeJob,
    runModelManagerAction,
    createProject,
    openRecentProject,
    importLatestRuntimeArtifact,
    mutateSelectedLibraryItem,
    previewLibraryItem,
    saveSelectedReviewEdit,
    exportSelectedLibraryItem,
    runRuntimeJob,
    cancelRuntimeOperation,
    retryRuntimeOperation,
  };

  return (
    <AppContext.Provider value={contextValue}>
      <main className="app-shell">
      <aside className="sidebar" aria-label="Primary">
        <div className="brand-mark" aria-label={overview.productName}>
          <Waves aria-hidden="true" size={28} />
          <span>{overview.productName}</span>
        </div>

        <nav className="nav-sections" aria-label="SoundWorks views">
          {navSections.map((section) => (
            <section className="sidebar-section" key={section.label}>
              <h2 className="sidebar-section-title">{section.label}</h2>
              <div className="nav-list">
                {section.items.map((item) => {
                  const Icon = item.icon;

                  return (
                    <button
                      aria-current={activeView === item.id ? "page" : undefined}
                      className={
                        activeView === item.id ? "nav-item active" : "nav-item"
                      }
                      key={item.id}
                      onClick={() => setActiveView(item.id)}
                      type="button"
                      title={item.label}
                    >
                      <Icon aria-hidden="true" size={18} />
                      <span className="nav-label">{item.label}</span>
                    </button>
                  );
                })}
              </div>
            </section>
          ))}
        </nav>
      </aside>

      <section className="workspace" aria-label="Workspace">
        {webPreview ? (
          <div className="preview-banner" role="status">
            <CircleAlert aria-hidden="true" size={16} />
            <span>
              Web preview — data is simulated. Launch the SoundWorks desktop
              shell for live local library data and audio generation.
            </span>
          </div>
        ) : null}
        {dataError ? (
          <div className="preview-banner preview-banner-error" role="alert">
            <CircleAlert aria-hidden="true" size={16} />
            <span>Some data could not be loaded: {dataError}</span>
          </div>
        ) : null}
        <header className="topbar">
          <div className="topbar-title">
            <p className="eyebrow">Local workspace</p>
            <h1>{activeViewMeta.title}</h1>
            <p>{activeViewMeta.blurb}</p>
          </div>
          <div className="topbar-actions">
            <span className="status-pill" aria-label="Scaffold status">
              <strong>{scaffoldedLayerCount}</strong>
              <span>active layers</span>
            </span>
            <button
              className="queue-chip"
              onClick={() => setActiveView("jobs")}
              type="button"
            >
              <Activity aria-hidden="true" size={16} />
              <span>{runtime.jobs.length} jobs</span>
            </button>
            <div
              className="accent-picker"
              role="group"
              aria-label="Accent color"
            >
              {ACCENTS.map((option) => (
                <button
                  aria-label={option.name}
                  aria-pressed={accent === option.id}
                  className={
                    accent === option.id
                      ? "accent-swatch active"
                      : "accent-swatch"
                  }
                  key={option.id}
                  onClick={() => changeAccent(option.id)}
                  style={{ "--sw": option.swatch } as CSSProperties}
                  title={option.name}
                  type="button"
                />
              ))}
            </div>
            <button
              className="icon-btn"
              onClick={() => changeTheme(theme === "light" ? "dark" : "light")}
              title={
                theme === "light"
                  ? "Switch to dark mode"
                  : "Switch to light mode"
              }
              aria-label={
                theme === "light"
                  ? "Switch to dark mode"
                  : "Switch to light mode"
              }
              type="button"
            >
              {theme === "light" ? (
                <Moon aria-hidden="true" size={18} />
              ) : (
                <Sun aria-hidden="true" size={18} />
              )}
            </button>
          </div>
        </header>

        {showWorkspace ? <WorkspaceScreen /> : null}

        {showLibrary ? (
          <section className="asset-library-panel" aria-label="Asset Library">
            <div className="library-header">
              <div>
                <p className="eyebrow">Asset Library</p>
                <h2>Project and global audio assets</h2>
              </div>
              <div className="library-search" role="search">
                <Search aria-hidden="true" size={18} />
                <span>{assetLibrary.selectedFilter.searchText}</span>
              </div>
              <button
                className="secondary-action"
                disabled={!latestImportableRuntimeJob}
                onClick={importLatestRuntimeArtifact}
                title="Save latest runtime audio artifact to library"
                type="button"
              >
                Save latest output
              </button>
            </div>
            <p className="action-status">{libraryActionStatus}</p>
            {libraryPlayback?.playable && libraryPlayback.path ? (
              <audio
                className="library-audio-preview"
                controls
                src={libraryPlayback.path}
              />
            ) : null}

            <div className="library-metrics" aria-label="Library status">
              <div>
                <Library aria-hidden="true" size={18} />
                <strong>{overview.assetLibrary.itemCount}</strong>
                <span>items</span>
              </div>
              <div>
                <Play aria-hidden="true" size={18} />
                <strong>{overview.assetLibrary.previewableItemCount}</strong>
                <span>previewable</span>
              </div>
              <div>
                <Boxes aria-hidden="true" size={18} />
                <strong>{overview.assetLibrary.collectionCount}</strong>
                <span>collections</span>
              </div>
              <div>
                <Archive aria-hidden="true" size={18} />
                <strong>{overview.assetLibrary.supportedTypeCount}</strong>
                <span>asset types</span>
              </div>
            </div>

            <div className="library-scope-row" aria-label="Library scopes">
              {assetLibrary.scopes.map((scope) => (
                <article className="library-scope" key={scope.id}>
                  <div>
                    <strong>{scope.label}</strong>
                    <small>{statusLabel(scope.ownership)}</small>
                  </div>
                  <span>{scope.assetCount} assets</span>
                </article>
              ))}
            </div>

            <div className="library-filter-strip" aria-label="Asset filters">
              {assetLibrary.filters.facets.map((facet) => (
                <section className="filter-chip-group" key={facet.id}>
                  <strong>{facet.label}</strong>
                  <div>
                    {facet.options.slice(0, 3).map((option) => (
                      <span
                        className={
                          option.selected
                            ? "filter-chip selected"
                            : "filter-chip"
                        }
                        key={option.id}
                      >
                        {option.label} {option.count}
                      </span>
                    ))}
                  </div>
                </section>
              ))}
            </div>

            <div className="library-layout">
              <div className="library-item-list" aria-label="Library assets">
                {assetLibrary.items.map((item) => (
                  <article
                    className={
                      item.id === assetLibrary.selectedItem?.item.id
                        ? "library-item selected"
                        : "library-item"
                    }
                    key={item.id}
                  >
                    <div className="waveform-thumb" aria-hidden="true">
                      {Array.from({ length: 18 }).map((_, index) => (
                        <span
                          key={`${item.id}-${index}`}
                          style={{ height: `${18 + ((index * 11) % 34)}px` }}
                        />
                      ))}
                    </div>
                    <div className="library-item-main">
                      <div className="library-item-title">
                        <strong>{item.name}</strong>
                        <span>{item.itemTypeLabel}</span>
                      </div>
                      <small>
                        {item.bpm ? `${item.bpm} BPM / ` : ""}
                        {item.musicalKey ? `${item.musicalKey} / ` : ""}
                        {item.durationMs
                          ? formatDuration(item.durationMs)
                          : "metadata"}
                      </small>
                      <div className="asset-tag-row">
                        {[...item.tags, ...item.generatedTags]
                          .slice(0, 5)
                          .map((tag, index) => (
                            <span key={`${item.id}-${index}`}>{tag}</span>
                          ))}
                      </div>
                    </div>
                    <button
                      className="icon-action"
                      disabled={!item.quickAudition.previewable}
                      onClick={() => previewLibraryItem(item.id)}
                      title={`Preview ${item.name}`}
                      type="button"
                    >
                      <Play aria-hidden="true" size={16} />
                    </button>
                  </article>
                ))}
              </div>

              {assetLibrary.selectedItem ? (
              <div className="library-detail" aria-label="Asset detail">
                <section className="tts-subpanel">
                  <div className="subpanel-heading">
                    <h3>{assetLibrary.selectedItem.item.name}</h3>
                    <span>{assetLibrary.selectedItem.item.itemTypeLabel}</span>
                  </div>
                  <p>
                    {assetLibrary.selectedItem.item.ownership} /{" "}
                    {assetLibrary.selectedItem.item.licenseStatus} /{" "}
                    {assetLibrary.selectedItem.item.commercialUse}
                  </p>
                  <div className="asset-tag-row detail-tags">
                    {assetLibrary.selectedItem.item.badges.map((badge) => (
                      <span key={badge}>{badge}</span>
                    ))}
                  </div>
                </section>

                <section className="tts-subpanel">
                  <div className="subpanel-heading">
                    <h3>Version history</h3>
                    <span>{assetLibrary.selectedItem.versionCount}</span>
                  </div>
                  <ol className="version-list">
                    {assetLibrary.selectedItem.versionHistory.map((version) => (
                      <li key={version.versionId}>
                        <CircleCheck aria-hidden="true" size={16} />
                        <div>
                          <strong>{version.label}</strong>
                          <small>{version.versionId}</small>
                        </div>
                      </li>
                    ))}
                  </ol>
                </section>

                <section className="tts-subpanel">
                  <div className="subpanel-heading">
                    <h3>Recipe provenance</h3>
                    <span>
                      {assetLibrary.selectedItem.recipe?.workflow ?? "manual"}
                    </span>
                  </div>
                  <p>
                    {assetLibrary.selectedItem.recipe
                      ? `${assetLibrary.selectedItem.recipe.id} / ${assetLibrary.selectedItem.recipe.modelId}`
                      : "No generation recipe attached."}
                  </p>
                  <ol className="policy-list">
                    {assetLibrary.selectedItem.provenanceLinks.map((link) => (
                      <li key={link.id}>
                        <ShieldCheck aria-hidden="true" size={16} />
                        <span>{link.label}</span>
                      </li>
                    ))}
                  </ol>
                </section>
              </div>
              ) : (
                <div className="library-detail" aria-label="Asset detail">
                  <section className="tts-subpanel">
                    <div className="subpanel-heading">
                      <h3>No asset selected</h3>
                    </div>
                    <p>
                      This library has no saved assets yet. Generate audio in a
                      studio or import a runtime artifact to populate it.
                    </p>
                  </section>
                </div>
              )}
            </div>

            <div className="library-bottom-grid">
              <section className="tts-subpanel" aria-label="Collections">
                <div className="subpanel-heading">
                  <h3>Collections</h3>
                  <span>{assetLibrary.collections.length}</span>
                </div>
                <div className="collection-grid">
                  {assetLibrary.collections.map((collection) => (
                    <article
                      className="collection-card"
                      key={collection.collection.id}
                    >
                      <strong>{collection.collection.name}</strong>
                      <small>
                        {statusLabel(collection.collectionType)} /{" "}
                        {collection.itemCount} items
                      </small>
                      <p>{collection.description}</p>
                    </article>
                  ))}
                </div>
              </section>

              <section className="tts-subpanel" aria-label="Lifecycle actions">
                <div className="subpanel-heading">
                  <h3>Lifecycle</h3>
                  <span>{assetLibrary.lifecycleActions.length}</span>
                </div>
                <div className="lifecycle-actions">
                  {assetLibrary.lifecycleActions.map((action) => (
                    <button
                      className="secondary-action"
                      key={action.id}
                      onClick={() => {
                        const mutation = toLibraryMutationAction(action.id);
                        if (!mutation) {
                          setLibraryActionStatus(
                            `Unknown library action "${action.id}".`,
                          );
                          return;
                        }
                        mutateSelectedLibraryItem(mutation);
                      }}
                      type="button"
                    >
                      {action.label}
                    </button>
                  ))}
                </div>
              </section>

              <section className="tts-subpanel" aria-label="Library validation">
                <div className="subpanel-heading">
                  <h3>Validation</h3>
                  <span>{assetLibrary.validationChecks.length}</span>
                </div>
                <ol className="voice-checks">
                  {assetLibrary.validationChecks.map((check) => (
                    <li
                      className={check.passed ? "passed" : "failed"}
                      key={check.id}
                    >
                      <CircleCheck aria-hidden="true" size={16} />
                      <span>{check.summary}</span>
                    </li>
                  ))}
                </ol>
              </section>
            </div>
          </section>
        ) : null}

        {showExport ? (
          <section
            className="export-workflow-panel"
            aria-label="Export Workflow"
          >
            <div className="export-header">
              <div>
                <p className="eyebrow">Export</p>
                <h2>Presets, stems, and handoff packages</h2>
              </div>
              <button
                className="primary-action export-action"
                disabled={!exportWorkflow.selectedExport.canExport}
                onClick={exportSelectedLibraryItem}
                title="Export selected composition"
                type="button"
              >
                <Download aria-hidden="true" size={18} />
                <span>
                  {exportWorkflow.selectedExport.canExport
                    ? "Export"
                    : "Blocked"}
                </span>
              </button>
            </div>
            <p className="action-status">{exportActionStatus}</p>

            <div className="export-metrics" aria-label="Export status">
              <div>
                <FileAudio aria-hidden="true" size={18} />
                <strong>{overview.exportWorkflow.presetCount}</strong>
                <span>presets</span>
              </div>
              <div>
                <ShieldCheck aria-hidden="true" size={18} />
                <strong>{overview.exportWorkflow.sidecarCount}</strong>
                <span>sidecars</span>
              </div>
              <div>
                <Boxes aria-hidden="true" size={18} />
                <strong>{overview.exportWorkflow.readyTargetCount}</strong>
                <span>targets ready</span>
              </div>
              <div>
                <Save aria-hidden="true" size={18} />
                <strong>{overview.exportWorkflow.selectedFormatCount}</strong>
                <span>formats selected</span>
              </div>
            </div>

            <div className="export-layout">
              <div className="export-preset-grid" aria-label="Export presets">
                {exportWorkflow.presets.map((preset) => (
                  <article
                    className="export-preset-card"
                    key={preset.preset.id}
                  >
                    <div className="export-preset-topline">
                      <strong>{preset.preset.name}</strong>
                      <span>{statusLabel(preset.preset.target)}</span>
                    </div>
                    <p>{preset.description}</p>
                    <div className="asset-tag-row">
                      {preset.formats.map((format, index) => (
                        <span key={`${preset.preset.id}-${index}`}>
                          {format}
                        </span>
                      ))}
                      {preset.preset.includeStems ? <span>stems</span> : null}
                      {preset.writesSidecar ? <span>sidecar</span> : null}
                    </div>
                  </article>
                ))}
              </div>

              <div
                className="export-detail"
                aria-label="Selected export detail"
              >
                <section className="tts-subpanel">
                  <div className="subpanel-heading">
                    <h3>Selected export</h3>
                    <span>
                      {statusLabel(exportWorkflow.selectedExport.sourceKind)}
                    </span>
                  </div>
                  <p>
                    {exportWorkflow.selectedExport.presetId} /{" "}
                    {exportWorkflow.selectedExport.sourceId}
                  </p>
                  <ol className="version-list">
                    {exportWorkflow.selectedExport.outputPaths.map(
                      (path, index) => (
                        <li key={index}>
                          <CircleCheck aria-hidden="true" size={16} />
                          <div>
                            <strong>{path.split("/").pop()}</strong>
                            <small>{path}</small>
                          </div>
                        </li>
                      ),
                    )}
                  </ol>
                </section>

                <section className="tts-subpanel">
                  <div className="subpanel-heading">
                    <h3>DAW bundle</h3>
                    <span>{exportWorkflow.dawHandoff.stemKinds.length}</span>
                  </div>
                  <p>{exportWorkflow.dawHandoff.packagePath}</p>
                  <div className="asset-tag-row detail-tags">
                    <span>zip bundle</span>
                    <span>cue markers</span>
                    <span>loop markers</span>
                    <span>BPM/key</span>
                    <span>lyrics</span>
                  </div>
                </section>

                <section className="tts-subpanel">
                  <div className="subpanel-heading">
                    <h3>SceneWorks handoff</h3>
                    <span>
                      {formatDuration(
                        exportWorkflow.sceneWorksHandoff.durationMs,
                      )}
                    </span>
                  </div>
                  <p>{exportWorkflow.sceneWorksHandoff.packagePath}</p>
                  <small>
                    {exportWorkflow.sceneWorksHandoff.sampleRateHz} Hz /{" "}
                    {exportWorkflow.sceneWorksHandoff.channels} channels /{" "}
                    {exportWorkflow.sceneWorksHandoff.markerCount} marker /{" "}
                    {statusLabel(
                      exportWorkflow.sceneWorksHandoff.importStrategy,
                    )}
                  </small>
                  <div className="asset-tag-row detail-tags">
                    <span>
                      {exportWorkflow.sceneWorksHandoff.sceneWorksAssetType}
                    </span>
                    <span>
                      {exportWorkflow.sceneWorksHandoff.sceneWorksMimeType}
                    </span>
                    <span>
                      {exportWorkflow.sceneWorksHandoff.replaceExistingAudio
                        ? "replace enabled"
                        : "attach only"}
                    </span>
                  </div>
                  <p>{exportWorkflow.sceneWorksHandoff.packageManifestPath}</p>
                  <small>
                    {exportWorkflow.sceneWorksHandoff.intendedProjectId} /{" "}
                    {exportWorkflow.sceneWorksHandoff.intendedVideoAssetId}
                  </small>
                </section>
              </div>
            </div>

            <div className="export-bottom-grid">
              <section className="tts-subpanel" aria-label="Export targets">
                <div className="subpanel-heading">
                  <h3>Targets</h3>
                  <span>{exportWorkflow.targets.length}</span>
                </div>
                <ol className="voice-checks">
                  {exportWorkflow.targets.map((target) => (
                    <li
                      className={target.ready ? "passed" : "failed"}
                      key={target.target}
                    >
                      <CircleCheck aria-hidden="true" size={16} />
                      <span>
                        <strong>{target.label}</strong> {target.notes[0]}
                      </span>
                    </li>
                  ))}
                </ol>
              </section>

              <section className="tts-subpanel" aria-label="Export sidecars">
                <div className="subpanel-heading">
                  <h3>Sidecars</h3>
                  <span>{exportWorkflow.sidecars.length}</span>
                </div>
                <div className="sidecar-list">
                  {exportWorkflow.sidecars.map((sidecar) => (
                    <article key={sidecar.id}>
                      <strong>{sidecar.assetId}</strong>
                      <small>
                        {statusLabel(sidecar.target)} / {sidecar.eventCount}{" "}
                        events
                      </small>
                      <p>{sidecar.path}</p>
                    </article>
                  ))}
                </div>
              </section>

              <section className="tts-subpanel" aria-label="Export validation">
                <div className="subpanel-heading">
                  <h3>Validation</h3>
                  <span>{exportWorkflow.validationChecks.length}</span>
                </div>
                <ol className="voice-checks">
                  {exportWorkflow.validationChecks.map((check) => (
                    <li
                      className={check.passed ? "passed" : "failed"}
                      key={check.id}
                    >
                      <ClipboardCheck aria-hidden="true" size={16} />
                      <span>{check.summary}</span>
                    </li>
                  ))}
                </ol>
              </section>

              <section
                className="tts-subpanel"
                aria-label="SceneWorks compatibility"
              >
                <div className="subpanel-heading">
                  <h3>SceneWorks compatibility</h3>
                  <span>
                    {
                      exportWorkflow.sceneWorksHandoff.compatibilityChecks
                        .length
                    }
                  </span>
                </div>
                <ol className="voice-checks">
                  {exportWorkflow.sceneWorksHandoff.compatibilityChecks.map(
                    (check) => (
                      <li
                        className={
                          check.status === "blocked" ? "failed" : "passed"
                        }
                        key={check.id}
                      >
                        <ClipboardCheck aria-hidden="true" size={16} />
                        <span>
                          <strong>{statusLabel(check.status)}</strong>{" "}
                          {check.summary}
                        </span>
                      </li>
                    ),
                  )}
                </ol>
              </section>

              <section
                className="tts-subpanel"
                aria-label="SceneWorks attachment steps"
              >
                <div className="subpanel-heading">
                  <h3>SceneWorks attachment</h3>
                  <span>
                    {exportWorkflow.sceneWorksHandoff.attachmentSteps.length}
                  </span>
                </div>
                <ol className="version-list">
                  {exportWorkflow.sceneWorksHandoff.attachmentSteps.map(
                    (step) => (
                      <li key={step.id}>
                        <CircleCheck aria-hidden="true" size={16} />
                        <div>
                          <strong>{step.label}</strong>
                          <small>
                            {step.source}
                            {" -> "}
                            {step.target}
                          </small>
                        </div>
                      </li>
                    ),
                  )}
                </ol>
              </section>
            </div>
          </section>
        ) : null}

        {showMixer ? (
          <section
            className="composition-editor-panel"
            aria-label="Multitrack Composition Editor"
          >
            <div className="composition-header">
              <div>
                <p className="eyebrow">Multitrack editor</p>
                <h2>{compositionEditor.composition.name}</h2>
              </div>
              <div
                className={
                  compositionEditor.exportPlan.canRenderMixdown
                    ? "primary-action composition-action is-inert"
                    : "primary-action composition-action is-inert is-blocked"
                }
                title="Composition mixdown render is not available in this build yet"
                aria-disabled="true"
              >
                <Disc3 aria-hidden="true" size={18} />
                <span>
                  {compositionEditor.exportPlan.canRenderMixdown
                    ? "Render (preview)"
                    : "Blocked"}
                </span>
              </div>
            </div>

            <div className="composition-metrics" aria-label="Editor status">
              <div>
                <SlidersHorizontal aria-hidden="true" size={18} />
                <strong>{overview.compositionEditor.trackCount}</strong>
                <span>tracks</span>
              </div>
              <div>
                <FileAudio aria-hidden="true" size={18} />
                <strong>{overview.compositionEditor.clipCount}</strong>
                <span>clips</span>
              </div>
              <div>
                <Library aria-hidden="true" size={18} />
                <strong>{overview.compositionEditor.assetBinCount}</strong>
                <span>assets</span>
              </div>
              <div>
                <Gauge aria-hidden="true" size={18} />
                <strong>{compositionEditor.timeline.zoomPercent}%</strong>
                <span>{compositionEditor.timeline.snapGridMs}ms grid</span>
              </div>
            </div>

            <div className="composition-layout">
              <div className="composition-main">
                <section
                  className="composition-toolbar"
                  aria-label="Editor tools"
                >
                  {compositionEditor.tools.map((tool) => (
                    <div
                      className={
                        tool.id === compositionEditor.timeline.selectedTool
                          ? "tool-button selected is-inert"
                          : "tool-button is-inert"
                      }
                      key={tool.id}
                      title={tool.label}
                      aria-disabled="true"
                    >
                      <span>{tool.label}</span>
                    </div>
                  ))}
                </section>

                <section
                  className="timeline-board"
                  aria-label="Timeline tracks"
                >
                  <div
                    className="timeline-selection"
                    aria-label="Timeline selection"
                  >
                    <span>{compositionEditor.timeline.selectedClipId}</span>
                    <span>
                      cursor{" "}
                      {formatDuration(
                        compositionEditor.timeline.playbackCursorMs,
                      )}
                    </span>
                    <span>
                      loop{" "}
                      {formatDuration(
                        compositionEditor.timeline.loopRange.endMs,
                      )}
                    </span>
                  </div>
                  <div className="timeline-ruler" aria-label="Timeline ruler">
                    {compositionEditor.timeline.gridLabels.map(
                      (label, index) => (
                        <span key={index}>{label}</span>
                      ),
                    )}
                  </div>
                  {compositionEditor.tracks.map((track) => (
                    <article className="timeline-track" key={track.trackId}>
                      <div className="track-strip">
                        <strong>{track.name}</strong>
                        <small>
                          {statusLabel(track.role)} / {track.gainDb} dB / pan{" "}
                          {track.pan}
                        </small>
                        <span>
                          {track.muted ? "Muted" : "Live"} /{" "}
                          {track.soloed ? "Solo" : "Mix"}
                        </span>
                      </div>
                      <div className="clip-lane">
                        {track.clips.map((clip) => (
                          <div
                            className={
                              clip.clipId ===
                              compositionEditor.timeline.selectedClipId
                                ? "timeline-clip selected is-inert"
                                : "timeline-clip is-inert"
                            }
                            key={clip.clipId}
                            style={{
                              marginLeft: `${Math.min(
                                68,
                                clip.timelineStartMs / 420,
                              )}%`,
                              width: `${Math.max(
                                16,
                                Math.min(
                                  38,
                                  (clip.sourceRange.endMs -
                                    clip.sourceRange.startMs) /
                                    520,
                                ),
                              )}%`,
                            }}
                            title={clip.assetName}
                            aria-disabled="true"
                          >
                            <strong>{clip.assetName}</strong>
                            <span>{statusLabel(clip.assetKind)}</span>
                          </div>
                        ))}
                      </div>
                    </article>
                  ))}
                </section>

                <section
                  className="tts-subpanel"
                  aria-label="Editor validation"
                >
                  <div className="subpanel-heading">
                    <h3>Validation</h3>
                    <span>{compositionEditor.validationChecks.length}</span>
                  </div>
                  <ol className="voice-checks">
                    {compositionEditor.validationChecks.map((check) => (
                      <li
                        className={check.passed ? "passed" : "failed"}
                        key={check.id}
                      >
                        <ClipboardCheck aria-hidden="true" size={16} />
                        <span>{check.summary}</span>
                      </li>
                    ))}
                  </ol>
                </section>
              </div>

              <div className="composition-side">
                <section className="tts-subpanel" aria-label="Timeline assets">
                  <div className="subpanel-heading">
                    <h3>Asset bin</h3>
                    <span>{compositionEditor.assetBin.length}</span>
                  </div>
                  <div className="asset-bin-list">
                    {compositionEditor.assetBin.map((asset) => (
                      <article key={asset.assetId}>
                        <strong>{asset.name}</strong>
                        <small>
                          {statusLabel(asset.kind)} / {scopeLabel(asset.scope)}
                        </small>
                        <div className="asset-tag-row">
                          <span>{formatDuration(asset.durationMs)}</span>
                          <span>{statusLabel(asset.sourceWorkflow)}</span>
                          {asset.draggableToTimeline ? (
                            <span>placeable</span>
                          ) : null}
                        </div>
                      </article>
                    ))}
                  </div>
                </section>

                <section className="tts-subpanel" aria-label="Mixer state">
                  <div className="subpanel-heading">
                    <h3>Mixer</h3>
                    <span>{compositionEditor.mixer.targetLufs} LUFS</span>
                  </div>
                  <p>{compositionEditor.mixer.loudnessCheck}</p>
                  <div className="mixer-list">
                    {compositionEditor.mixer.trackStates.map((track) => (
                      <article key={track.trackId}>
                        <strong>{track.label}</strong>
                        <small>
                          {track.gainDb} dB / pan {track.pan}
                        </small>
                        <div className="asset-tag-row">
                          {track.effectChain.map((effect, index) => (
                            <span key={`${track.trackId}-effect-${index}`}>
                              {effect}
                            </span>
                          ))}
                          {track.sendTargets.map((send, index) => (
                            <span key={`${track.trackId}-send-${index}`}>
                              {send}
                            </span>
                          ))}
                        </div>
                      </article>
                    ))}
                  </div>
                </section>
              </div>
            </div>

            <div className="composition-bottom-grid">
              <section
                className="tts-subpanel"
                aria-label="Generated asset flows"
              >
                <div className="subpanel-heading">
                  <h3>Studio flows</h3>
                  <span>{compositionEditor.sourceFlows.length}</span>
                </div>
                <ol className="voice-checks">
                  {compositionEditor.sourceFlows.map((flow) => (
                    <li
                      className={flow.status === "ready" ? "passed" : "failed"}
                      key={`${flow.workflow}-${flow.assetKind}`}
                    >
                      <CircleCheck aria-hidden="true" size={16} />
                      <span>
                        <strong>{flow.label}</strong>{" "}
                        {statusLabel(flow.assetKind)}
                      </span>
                    </li>
                  ))}
                </ol>
              </section>

              <section className="tts-subpanel" aria-label="Render plan">
                <div className="subpanel-heading">
                  <h3>Render plan</h3>
                  <span>
                    {compositionEditor.exportPlan.canRenderMixdown
                      ? "ready"
                      : "blocked"}
                  </span>
                </div>
                <p>{compositionEditor.exportPlan.mixdownPath}</p>
                <div className="asset-tag-row detail-tags">
                  {compositionEditor.exportPlan.presetIds.map(
                    (preset, index) => (
                      <span key={index}>{preset}</span>
                    ),
                  )}
                </div>
                <small>{compositionEditor.exportPlan.sceneWorksWarning}</small>
              </section>

              <section
                className="tts-subpanel"
                aria-label="Editor component decision"
              >
                <div className="subpanel-heading">
                  <h3>Component decision</h3>
                  <span>
                    {overview.compositionEditor.recommendedComponentId}
                  </span>
                </div>
                <div className="component-decision-list">
                  {compositionEditor.componentDecisions.map((decision) => (
                    <article
                      className={
                        decision.fit === "strong-prototype-candidate"
                          ? "recommended"
                          : ""
                      }
                      key={decision.id}
                    >
                      <strong>{decision.name}</strong>
                      <small>
                        {decision.license} / {statusLabel(decision.fit)}
                      </small>
                      <p>{decision.prototypeEvidence}</p>
                      <p>{decision.decision}</p>
                      {isHttpUrl(decision.sourceUrl) ? (
                        <a
                          href={decision.sourceUrl}
                          target="_blank"
                          rel="noopener noreferrer"
                        >
                          {decision.sourceUrl}
                        </a>
                      ) : (
                        <span>{decision.sourceUrl}</span>
                      )}
                    </article>
                  ))}
                </div>
              </section>
            </div>
          </section>
        ) : null}

        {showValidation ? (
          <section className="mvp-validation-panel" aria-label="MVP Validation">
            <div className="mvp-header">
              <div>
                <p className="eyebrow">MVP validation</p>
                <h2>Release gate and demo matrix</h2>
              </div>
              <div
                className={
                  mvpValidation.releaseGate.readyForMvp
                    ? "primary-action validation-action is-inert"
                    : "primary-action validation-action is-inert is-blocked"
                }
                title="MVP release gate status"
                role="status"
              >
                <ClipboardCheck aria-hidden="true" size={18} />
                <span>
                  {mvpValidation.releaseGate.readyForMvp ? "Ready" : "Blocked"}
                </span>
              </div>
            </div>

            <div className="mvp-metrics" aria-label="Validation status">
              <div>
                <CircleCheck aria-hidden="true" size={18} />
                <strong>
                  {mvpValidation.releaseGate.coveredWorkflowCount}/
                  {mvpValidation.releaseGate.requiredWorkflowCount}
                </strong>
                <span>workflows covered</span>
              </div>
              <div>
                <ClipboardCheck aria-hidden="true" size={18} />
                <strong>
                  {mvpValidation.releaseGate.passedAutomatedCheckCount}/
                  {mvpValidation.releaseGate.requiredAutomatedCheckCount}
                </strong>
                <span>automated checks</span>
              </div>
              <div>
                <Gauge aria-hidden="true" size={18} />
                <strong>
                  {mvpValidation.releaseGate.passedManualScorecardCount}/
                  {mvpValidation.releaseGate.requiredManualScorecardCount}
                </strong>
                <span>manual scorecards</span>
              </div>
              <div>
                <HardDrive aria-hidden="true" size={18} />
                <strong>
                  {mvpValidation.releaseGate.satisfiedRuntimeEvidenceCount}/
                  {mvpValidation.releaseGate.requiredRuntimeEvidenceCount}
                </strong>
                <span>runtime evidence</span>
              </div>
              <div>
                <CircleAlert aria-hidden="true" size={18} />
                <strong>{overview.mvpValidation.blockingItemCount}</strong>
                <span>blocking items</span>
              </div>
            </div>

            <div className="mvp-layout">
              <div className="mvp-main">
                <section
                  className="tts-subpanel"
                  aria-label="Golden demo workflows"
                >
                  <div className="subpanel-heading">
                    <h3>Golden demos</h3>
                    <span>{mvpValidation.demoWorkflows.length}</span>
                  </div>
                  <div className="mvp-demo-grid">
                    {mvpValidation.demoWorkflows.map((workflow) => (
                      <article className="mvp-demo-card" key={workflow.id}>
                        <div className="sfx-variant-title">
                          <strong>{workflow.title}</strong>
                          <span>{workflowLabel(workflow.workflow)}</span>
                        </div>
                        <p>{workflow.goal}</p>
                        <div className="asset-tag-row">
                          {workflow.requiredArtifacts
                            .slice(0, 4)
                            .map((artifact, index) => (
                              <span key={`${workflow.id}-${index}`}>
                                {artifact}
                              </span>
                            ))}
                        </div>
                      </article>
                    ))}
                  </div>
                </section>

                <section
                  className="tts-subpanel"
                  aria-label="Requirement coverage"
                >
                  <div className="subpanel-heading">
                    <h3>Epic requirement coverage</h3>
                    <span>{mvpValidation.requirementCoverage.length}</span>
                  </div>
                  <div className="requirement-grid">
                    {mvpValidation.requirementCoverage.map((coverage) => (
                      <article
                        className={`requirement-card ${coverage.status}`}
                        key={coverage.requirementId}
                      >
                        <div className="rights-card-title">
                          <strong>{coverage.requirementId}</strong>
                          <span>{statusLabel(coverage.status)}</span>
                        </div>
                        <p>{coverage.epicRequirement}</p>
                        <small>
                          {coverage.demoWorkflowIds.length} demos /{" "}
                          {coverage.fixtureIds.length} fixtures /{" "}
                          {coverage.checkIds.length} checks
                        </small>
                      </article>
                    ))}
                  </div>
                </section>
              </div>

              <div className="mvp-side">
                <section className="tts-subpanel" aria-label="MVP blockers">
                  <div className="subpanel-heading">
                    <h3>Release blockers</h3>
                    <span>
                      {mvpValidation.releaseGate.blockingItems.length}
                    </span>
                  </div>
                  <ol className="voice-checks">
                    {mvpValidation.releaseGate.blockingItems.map((item) => (
                      <li className="blocked" key={item}>
                        <CircleAlert aria-hidden="true" size={16} />
                        <span>{item}</span>
                      </li>
                    ))}
                  </ol>
                </section>

                <section
                  className="tts-subpanel"
                  aria-label="Automated validation checks"
                >
                  <div className="subpanel-heading">
                    <h3>Automated checks</h3>
                    <span>{mvpValidation.automatedChecks.length}</span>
                  </div>
                  <ol className="voice-checks">
                    {mvpValidation.automatedChecks.map((check) => (
                      <li className={check.status} key={check.id}>
                        <ClipboardCheck aria-hidden="true" size={16} />
                        <span>
                          <strong>{statusLabel(check.category)}</strong>{" "}
                          {check.summary}
                        </span>
                      </li>
                    ))}
                  </ol>
                </section>

                <section className="tts-subpanel" aria-label="Runtime evidence">
                  <div className="subpanel-heading">
                    <h3>Runtime evidence</h3>
                    <span>
                      {mvpValidation.releaseGate.satisfiedRuntimeEvidenceCount}/
                      {mvpValidation.releaseGate.requiredRuntimeEvidenceCount}
                    </span>
                  </div>
                  <ol className="voice-checks">
                    {mvpValidation.runtimeEvidence.map((evidence) => (
                      <li className={evidence.status} key={evidence.id}>
                        <CircleAlert aria-hidden="true" size={16} />
                        <span>
                          <strong>{workflowLabel(evidence.workflow)}</strong>{" "}
                          {evidence.requirement}
                          <em>
                            {evidence.fixtureOnly
                              ? "Fixture-only: "
                              : "Evidence: "}
                            {evidence.evidence}
                          </em>
                          <em>{evidence.blocker}</em>
                        </span>
                      </li>
                    ))}
                  </ol>
                </section>
              </div>
            </div>

            <div className="mvp-bottom-grid">
              <section
                className="tts-subpanel"
                aria-label="Regression fixtures"
              >
                <div className="subpanel-heading">
                  <h3>Regression fixtures</h3>
                  <span>{mvpValidation.regressionFixtures.length}</span>
                </div>
                <div className="fixture-list">
                  {mvpValidation.regressionFixtures.map((fixture) => (
                    <article key={fixture.id}>
                      <strong>{fixture.name}</strong>
                      <small>{workflowLabel(fixture.workflow)}</small>
                      <p>{fixture.inputContract}</p>
                    </article>
                  ))}
                </div>
              </section>

              <section
                className="tts-subpanel"
                aria-label="Manual QA scorecards"
              >
                <div className="subpanel-heading">
                  <h3>Manual QA</h3>
                  <span>{mvpValidation.manualScorecards.length}</span>
                </div>
                <ol className="voice-checks">
                  {mvpValidation.manualScorecards
                    .slice(0, 6)
                    .map((scorecard) => (
                      <li className={scorecard.status} key={scorecard.id}>
                        <Gauge aria-hidden="true" size={16} />
                        <span>
                          <strong>{workflowLabel(scorecard.workflow)}</strong>{" "}
                          {scorecard.passThreshold}
                        </span>
                      </li>
                    ))}
                </ol>
              </section>

              <section
                className="tts-subpanel"
                aria-label="Stress cases and limitations"
              >
                <div className="subpanel-heading">
                  <h3>Stress and limits</h3>
                  <span>{mvpValidation.stressCases.length}</span>
                </div>
                <ol className="voice-checks">
                  {mvpValidation.stressCases.map((stressCase) => (
                    <li className={stressCase.status} key={stressCase.id}>
                      <CircleAlert aria-hidden="true" size={16} />
                      <span>
                        <strong>{stressCase.title}</strong>{" "}
                        {stressCase.expectedBehavior}
                      </span>
                    </li>
                  ))}
                </ol>
                <div className="limitation-list">
                  {mvpValidation.knownLimitations.map((limitation) => (
                    <article
                      className={limitation.blocksMvp ? "blocks" : ""}
                      key={limitation.id}
                    >
                      <strong>{limitation.area}</strong>
                      <p>{limitation.summary}</p>
                    </article>
                  ))}
                </div>
              </section>
            </div>
          </section>
        ) : null}

        {showTts ? <TtsScreen /> : null}

        {showVoice ? <VoiceLabScreen /> : null}

        {showSfx ? <SfxScreen /> : null}

        {showVideoToAudio ? <VideoToAudioScreen /> : null}

        {showSamples ? <SamplesScreen /> : null}

        {showSong ? <SongScreen /> : null}

        {showReview ? (
          <section
            className="review-workspace-panel"
            aria-label="Waveform Review"
          >
            <div className="samples-header">
              <div>
                <p className="eyebrow">Waveform Review</p>
                <h2>{reviewWorkspace.selectedAsset.asset.name}</h2>
              </div>
              <button
                className="primary-action review-action"
                disabled={!reviewWorkspace.editSubmission.canSave}
                onClick={saveSelectedReviewEdit}
                type="button"
                title="Save edited audio version"
              >
                <Save aria-hidden="true" size={18} />
                <span>
                  {reviewWorkspace.editSubmission.canSave
                    ? "Save version"
                    : "Blocked"}
                </span>
              </button>
            </div>
            <p className="action-status">{reviewActionStatus}</p>

            <div
              className="samples-metrics"
              aria-label="Waveform review status"
            >
              <div>
                <Library aria-hidden="true" size={18} />
                <strong>{overview.reviewWorkspace.assetCount}</strong>
                <span>assets</span>
              </div>
              <div>
                <Waves aria-hidden="true" size={18} />
                <strong>
                  {overview.reviewWorkspace.previewableAssetCount}
                </strong>
                <span>previewable</span>
              </div>
              <div>
                <SlidersHorizontal aria-hidden="true" size={18} />
                <strong>{overview.reviewWorkspace.editActionCount}</strong>
                <span>edit actions</span>
              </div>
              <div>
                <ClipboardCheck aria-hidden="true" size={18} />
                <strong>{overview.reviewWorkspace.comparisonCount}</strong>
                <span>comparison</span>
              </div>
            </div>

            <div className="review-layout">
              <div className="review-main">
                <section
                  className="review-transport"
                  aria-label="Waveform transport"
                >
                  <div className="review-transport-topline">
                    <button
                      className="icon-control"
                      disabled={!assetLibrary.selectedItem}
                      onClick={() => {
                        if (assetLibrary.selectedItem) {
                          previewLibraryItem(assetLibrary.selectedItem.item.id);
                        }
                      }}
                      type="button"
                      title="Play or pause preview"
                    >
                      <Play aria-hidden="true" size={18} />
                    </button>
                    <strong>
                      {formatDuration(reviewWorkspace.transport.positionMs)} /{" "}
                      {formatDuration(reviewWorkspace.transport.durationMs)}
                    </strong>
                    <span>
                      {reviewWorkspace.transport.zoomPixelsPerSecond}px/s
                    </span>
                  </div>
                  <div
                    className="waveform-strip"
                    aria-label="Cached waveform preview"
                  >
                    {reviewWorkspace.waveform.peaks.map((peak, index) => (
                      <span
                        aria-hidden="true"
                        className="waveform-bar"
                        key={index}
                        style={{ height: `${Math.max(20, peak.max * 86)}%` }}
                      />
                    ))}
                  </div>
                  <div className="transport-meta">
                    <span>
                      selection{" "}
                      {formatDuration(
                        reviewWorkspace.transport.selection?.startMs ?? 0,
                      )}
                      -
                      {formatDuration(
                        reviewWorkspace.transport.selection?.endMs ?? 0,
                      )}
                    </span>
                    <span>
                      loop{" "}
                      {formatDuration(
                        reviewWorkspace.transport.loopRegion?.startMs ?? 0,
                      )}
                      -
                      {formatDuration(
                        reviewWorkspace.transport.loopRegion?.endMs ?? 0,
                      )}
                    </span>
                    <span>{reviewWorkspace.waveform.cachePath}</span>
                  </div>
                </section>

                <div
                  className="review-asset-grid"
                  aria-label="Reviewable assets"
                >
                  {reviewWorkspace.assets.map((asset) => (
                    <article
                      className={
                        asset.asset.id ===
                        reviewWorkspace.selectedAsset.asset.id
                          ? "review-asset selected"
                          : "review-asset"
                      }
                      key={asset.asset.id}
                    >
                      <div className="sfx-variant-title">
                        <strong>{asset.asset.name}</strong>
                        <span>{statusLabel(asset.asset.kind)}</span>
                      </div>
                      <small>
                        {statusLabel(asset.sourceWorkflow)} /{" "}
                        {asset.versions.length} version
                      </small>
                      <p>
                        {asset.canPreview
                          ? "waveform and spectrogram cached"
                          : "preview pending"}
                      </p>
                    </article>
                  ))}
                </div>

                <div
                  className="edit-action-grid"
                  aria-label="Lightweight edit actions"
                >
                  {reviewWorkspace.editActions.map((action) => (
                    <div
                      className={
                        action.enabled
                          ? "edit-action enabled is-inert"
                          : "edit-action is-inert"
                      }
                      key={action.id}
                      title={action.label}
                      aria-disabled="true"
                    >
                      <SlidersHorizontal aria-hidden="true" size={16} />
                      <span>{action.label}</span>
                    </div>
                  ))}
                </div>
              </div>

              <div className="review-side">
                <section
                  className="tts-subpanel"
                  aria-label="Version comparison"
                >
                  <div className="subpanel-heading">
                    <h3>Version comparison</h3>
                    <span>{reviewWorkspace.versionComparison.mode}</span>
                  </div>
                  <div className="comparison-grid">
                    {[
                      reviewWorkspace.versionComparison.left,
                      reviewWorkspace.versionComparison.right,
                    ].map((side) => (
                      <article key={side.versionId}>
                        <strong>{side.label}</strong>
                        <small>{side.versionId}</small>
                        <p>
                          {formatDuration(side.durationMs)} /{" "}
                          {side.loudnessLufs} LUFS / {side.truePeakDbfs} dBTP
                        </p>
                      </article>
                    ))}
                  </div>
                  <div className="comparison-metrics">
                    <span>
                      {
                        reviewWorkspace.versionComparison.metrics
                          .durationDeltaMs
                      }
                      ms
                    </span>
                    <span>
                      {
                        reviewWorkspace.versionComparison.metrics
                          .loudnessDeltaLufs
                      }{" "}
                      LUFS
                    </span>
                    <span>
                      diff{" "}
                      {
                        reviewWorkspace.versionComparison.metrics
                          .waveformDifferenceScore
                      }
                    </span>
                  </div>
                </section>

                <section className="tts-subpanel" aria-label="Edited version">
                  <div className="subpanel-heading">
                    <h3>Edited version</h3>
                    <span>{reviewWorkspace.editSubmission.job.status}</span>
                  </div>
                  <div className="output-card">
                    <strong>
                      {reviewWorkspace.editSubmission.savedVersion.id}
                    </strong>
                    <small>
                      v
                      {reviewWorkspace.editSubmission.savedVersion.versionIndex}{" "}
                      /{" "}
                      {reviewWorkspace.editSubmission.savedVersion.file.format}
                    </small>
                    <p>
                      {
                        reviewWorkspace.editSubmission.savedVersion.file
                          .storagePath
                      }
                    </p>
                  </div>
                </section>

                <section
                  className="tts-subpanel"
                  aria-label="Recipe provenance"
                >
                  <div className="subpanel-heading">
                    <h3>Provenance</h3>
                    <span>
                      {reviewWorkspace.provenance.inspectable
                        ? "inspectable"
                        : "blocked"}
                    </span>
                  </div>
                  <div className="output-card">
                    <strong>{reviewWorkspace.provenance.editRecipe.id}</strong>
                    <small>
                      {statusLabel(
                        reviewWorkspace.provenance.originalRecipe.workflow,
                      )}{" "}
                      to{" "}
                      {statusLabel(
                        reviewWorkspace.provenance.editRecipe.workflow,
                      )}
                    </small>
                    <p>{reviewWorkspace.provenance.sidecarPath}</p>
                  </div>
                </section>
              </div>
            </div>

            <div className="samples-review-grid">
              <ol
                className="voice-checks"
                aria-label="Review validation checks"
              >
                {reviewWorkspace.validationChecks.map((check) => (
                  <li className={check.status} key={check.id}>
                    <CircleCheck aria-hidden="true" size={16} />
                    <span>{check.summary}</span>
                  </li>
                ))}
              </ol>
              <ol className="voice-checks" aria-label="Review shortcuts">
                {reviewWorkspace.transport.keyboardShortcuts.map((shortcut) => (
                  <li className="ready" key={shortcut.id}>
                    <ClipboardCheck aria-hidden="true" size={16} />
                    <span>
                      <strong>{shortcut.keys}</strong> {shortcut.action}
                    </span>
                  </li>
                ))}
              </ol>
            </div>
          </section>
        ) : null}

        {showRights ? (
          <section
            className="rights-safety-panel"
            aria-label="Rights and Safety"
          >
            <div className="samples-header">
              <div>
                <p className="eyebrow">Rights + Safety</p>
                <h2>{rightsSafety.policy.name}</h2>
              </div>
              <div
                className={
                  overview.rightsSafety.canExport
                    ? "primary-action safety-action is-inert"
                    : "primary-action safety-action is-inert is-blocked"
                }
                title="Rights export gate status — run exports from the Export screen"
                role="status"
              >
                <ShieldCheck aria-hidden="true" size={18} />
                <span>
                  {overview.rightsSafety.canExport ? "Ready" : "Blocked"}
                </span>
              </div>
            </div>

            <div
              className="samples-metrics"
              aria-label="Rights workflow status"
            >
              <div>
                <ShieldCheck aria-hidden="true" size={18} />
                <strong>{overview.rightsSafety.blockedConsentCount}</strong>
                <span>consent blocks</span>
              </div>
              <div>
                <CircleAlert aria-hidden="true" size={18} />
                <strong>
                  {overview.rightsSafety.blockedModelDecisionCount}
                </strong>
                <span>model blocks</span>
              </div>
              <div>
                <Save aria-hidden="true" size={18} />
                <strong>{overview.rightsSafety.sidecarCount}</strong>
                <span>sidecars</span>
              </div>
              <div>
                <ClipboardCheck aria-hidden="true" size={18} />
                <strong>{overview.rightsSafety.disclosureCount}</strong>
                <span>disclosures</span>
              </div>
            </div>

            <div className="rights-layout">
              <div className="rights-main">
                <section className="tts-subpanel" aria-label="Consent checks">
                  <div className="subpanel-heading">
                    <h3>Consent</h3>
                    <span>{rightsSafety.consentChecks.length}</span>
                  </div>
                  <div className="rights-card-grid">
                    {rightsSafety.consentChecks.map((check) => (
                      <article
                        className={`rights-card ${check.decision}`}
                        key={check.id}
                      >
                        <div className="rights-card-title">
                          <strong>{statusLabel(check.workflow)}</strong>
                          <span>{statusLabel(check.decision)}</span>
                        </div>
                        <small>
                          {check.voiceProfileId} /{" "}
                          {statusLabel(check.consentStatus)}
                        </small>
                        <p>{check.summary}</p>
                      </article>
                    ))}
                  </div>
                </section>

                <section
                  className="tts-subpanel"
                  aria-label="Model export decisions"
                >
                  <div className="subpanel-heading">
                    <h3>Model export gates</h3>
                    <span>{rightsSafety.modelUseDecisions.length}</span>
                  </div>
                  <div className="rights-card-grid model-gate-grid">
                    {rightsSafety.modelUseDecisions.map((decision) => (
                      <article
                        className={`rights-card ${decision.decision}`}
                        key={decision.candidateId}
                      >
                        <div className="rights-card-title">
                          <strong>{decision.name}</strong>
                          <span>{statusLabel(decision.decision)}</span>
                        </div>
                        <small>
                          {statusLabel(decision.commercialUse)} /{" "}
                          {statusLabel(decision.productEligibility)}
                        </small>
                        <p>{decision.reasons[0]}</p>
                      </article>
                    ))}
                  </div>
                </section>
              </div>

              <div className="rights-side">
                <section
                  className="tts-subpanel"
                  aria-label="Export provenance sidecars"
                >
                  <div className="subpanel-heading">
                    <h3>Sidecars</h3>
                    <span>
                      {statusLabel(rightsSafety.policy.watermarkPolicy)}
                    </span>
                  </div>
                  {rightsSafety.exportSidecars.map((sidecar) => (
                    <div className="output-card" key={sidecar.id}>
                      <strong>{sidecar.assetId}</strong>
                      <small>
                        {statusLabel(sidecar.target)} /{" "}
                        {statusLabel(sidecar.watermark)}
                      </small>
                      <p>{sidecar.path}</p>
                    </div>
                  ))}
                </section>

                <section
                  className="tts-subpanel"
                  aria-label="Policy requirements"
                >
                  <div className="subpanel-heading">
                    <h3>SoundWorks export</h3>
                    <span>
                      {rightsSafety.policy.provenanceSidecarRequired
                        ? "sidecar"
                        : "manual"}
                    </span>
                  </div>
                  <ol className="policy-list">
                    {rightsSafety.policy.exportRequires.map(
                      (requirement, index) => (
                        <li key={index}>
                          <CircleCheck aria-hidden="true" size={16} />
                          <span>{requirement}</span>
                        </li>
                      ),
                    )}
                  </ol>
                </section>
              </div>
            </div>

            <div className="rights-review-grid">
              <ol className="voice-checks" aria-label="Content policy gates">
                {rightsSafety.contentPolicyGates.map((gate) => (
                  <li className={gate.status} key={gate.id}>
                    <ShieldCheck aria-hidden="true" size={16} />
                    <span>
                      <strong>{statusLabel(gate.category)}</strong>{" "}
                      {gate.summary}
                    </span>
                  </li>
                ))}
              </ol>
              <ol
                className="voice-checks"
                aria-label="Rights validation checks"
              >
                {rightsSafety.validationChecks.map((check) => (
                  <li className={check.status} key={check.id}>
                    <ClipboardCheck aria-hidden="true" size={16} />
                    <span>{check.summary}</span>
                  </li>
                ))}
              </ol>
            </div>
          </section>
        ) : null}

        {showSystemSurfaces ? (
          <section className="system-grid" aria-label="Architecture">
            {showSettings ? (
              <>
                <div className="panel">
                  <div className="panel-heading">
                    <h2>Runtime Layers</h2>
                    <span>{overview.architecture.layers.length}</span>
                  </div>
                  <ol className="layer-list">
                    {overview.architecture.layers.map((layer) => (
                      <li key={layer.id}>
                        <span className={`layer-dot ${layer.status}`} />
                        <div>
                          <strong>{layer.name}</strong>
                          <p>{layer.responsibility}</p>
                        </div>
                      </li>
                    ))}
                  </ol>
                </div>

                <div className="panel">
                  <div className="panel-heading">
                    <h2>Command Boundary</h2>
                    <span>{overview.commands.length}</span>
                  </div>
                  <div className="command-list">
                    {overview.commands.map((command) => (
                      <article className="command-row" key={command.name}>
                        <strong>{command.name}</strong>
                        <span>{command.direction}</span>
                        <p>{command.purpose}</p>
                      </article>
                    ))}
                  </div>
                </div>
              </>
            ) : null}

            {showSettings || showModels ? (
              <>
                <div className="panel provider-panel">
                  <div className="panel-heading">
                    <h2>Provider Coverage</h2>
                    <span>{overview.providerCatalog.capabilityCount}</span>
                  </div>
                  <div
                    className="provider-metrics"
                    aria-label="Provider catalog"
                  >
                    <div>
                      <strong>{overview.providerCatalog.providerCount}</strong>
                      <span>providers</span>
                    </div>
                    <div>
                      <strong>{overview.providerCatalog.modelCount}</strong>
                      <span>models</span>
                    </div>
                  </div>
                  <ol className="workflow-list">
                    {overview.providerCatalog.workflows.map((workflow) => (
                      <li key={workflow.workflow}>
                        <Cpu aria-hidden="true" size={16} />
                        <span>{workflowLabel(workflow.workflow)}</span>
                        <small>{workflow.defaultModelId}</small>
                      </li>
                    ))}
                  </ol>
                </div>

                <div className="panel evaluation-panel">
                  <div className="panel-heading">
                    <h2>Evaluation Scorecard</h2>
                    <span>{overview.modelEvaluation.candidateCount}</span>
                  </div>
                  <div
                    className="evaluation-summary"
                    aria-label="Model evaluation"
                  >
                    <div>
                      <ClipboardCheck aria-hidden="true" size={18} />
                      <strong>{overview.modelEvaluation.sourceCount}</strong>
                      <span>sources</span>
                    </div>
                    <div>
                      <Boxes aria-hidden="true" size={18} />
                      <strong>{overview.modelEvaluation.fixtureCount}</strong>
                      <span>fixtures</span>
                    </div>
                    <div>
                      <CircleCheck aria-hidden="true" size={18} />
                      <strong>
                        {countFor(
                          overview.modelEvaluation.productEligibilityCounts,
                          "product-candidate",
                        )}
                      </strong>
                      <span>product candidates</span>
                    </div>
                    <div>
                      <CircleAlert aria-hidden="true" size={18} />
                      <strong>
                        {countFor(
                          overview.modelEvaluation.statusCounts,
                          "blocked",
                        )}
                      </strong>
                      <span>blocked</span>
                    </div>
                  </div>
                  <ol
                    className="recommendation-list"
                    aria-label="Recommended spikes"
                  >
                    {overview.modelEvaluation.recommendedCandidateIds.map(
                      (candidateId) => (
                        <li key={candidateId}>
                          <span>{candidateId}</span>
                        </li>
                      ),
                    )}
                  </ol>
                </div>
              </>
            ) : null}

            {showModels ? (
              <div className="panel model-manager-panel">
                <div className="panel-heading">
                  <h2>Model Manager</h2>
                  <span>{modelManager.summary.candidateCount}</span>
                </div>
                <div
                  className="runtime-summary"
                  aria-label="Model manager status"
                >
                  <div>
                    <PackageCheck aria-hidden="true" size={18} />
                    <strong>
                      {modelManager.summary.verifiedInstalledCount}
                    </strong>
                    <span>verified</span>
                  </div>
                  <div>
                    <Download aria-hidden="true" size={18} />
                    <strong>{modelManager.summary.installableCount}</strong>
                    <span>installable</span>
                  </div>
                  <div>
                    <HardDrive aria-hidden="true" size={18} />
                    <strong>{modelManager.summary.missingCacheCount}</strong>
                    <span>missing cache</span>
                  </div>
                  <div>
                    <CircleAlert aria-hidden="true" size={18} />
                    <strong>{modelManager.summary.failedOperationCount}</strong>
                    <span>failed ops</span>
                  </div>
                </div>

                <div className="runtime-policy">
                  <strong>{modelManager.cacheRoot}</strong>
                  <span>
                    No model is installed until required files verify on disk.
                  </span>
                  {modelManagerOperation ? (
                    <small>
                      {statusLabel(modelManagerOperation.action)}:{" "}
                      {statusLabel(modelManagerOperation.status)}
                    </small>
                  ) : null}
                </div>

                <div className="model-manager-grid">
                  <div className="runtime-stack">
                    <h3>Lane readiness</h3>
                    <ol className="runtime-list">
                      {modelManager.laneReadiness.map((lane) => (
                        <li key={`${lane.lane}-${lane.recommendedCandidateId}`}>
                          <span className={`runtime-dot ${lane.state}`} />
                          <div>
                            <strong>{workflowLabel(lane.lane)}</strong>
                            <small>
                              {lane.recommendedCandidateId} /{" "}
                              {statusLabel(lane.state)}
                            </small>
                            <em>{lane.summary}</em>
                            {lane.blocker ? <em>{lane.blocker}</em> : null}
                          </div>
                        </li>
                      ))}
                    </ol>
                  </div>

                  <div className="runtime-stack">
                    <h3>Candidate cache</h3>
                    <ol className="runtime-list model-cache-list">
                      {modelManager.candidates.slice(0, 10).map((candidate) => (
                        <li key={candidate.candidateId}>
                          <span
                            className={`runtime-dot ${candidate.installState}`}
                          />
                          <div>
                            <strong>{candidate.name}</strong>
                            <small>
                              {candidate.candidateId} /{" "}
                              {statusLabel(candidate.installState)} /{" "}
                              {candidate.cache.presentFileCount} of{" "}
                              {candidate.cache.expectedFileCount}
                            </small>
                            <em>{candidate.cache.evidence}</em>
                            {candidate.cache.missingRequiredFiles[0] ? (
                              <em>
                                missing{" "}
                                {candidate.cache.missingRequiredFiles.join(
                                  ", ",
                                )}
                              </em>
                            ) : null}
                            <div className="model-manager-actions">
                              <button
                                className="icon-button small"
                                disabled={
                                  !candidate.actions.includes("install")
                                }
                                onClick={() =>
                                  runModelManagerAction(
                                    candidate.candidateId,
                                    "install",
                                  )
                                }
                                title={`Install ${candidate.name}`}
                                type="button"
                              >
                                <Download aria-hidden="true" size={15} />
                              </button>
                              <button
                                className="icon-button small"
                                onClick={() =>
                                  runModelManagerAction(
                                    candidate.candidateId,
                                    "revalidate",
                                  )
                                }
                                title={`Revalidate ${candidate.name}`}
                                type="button"
                              >
                                <CircleCheck aria-hidden="true" size={15} />
                              </button>
                            </div>
                          </div>
                        </li>
                      ))}
                    </ol>
                  </div>
                </div>

                {modelManagerOperation ? (
                  <div
                    className={`operation-banner ${modelManagerOperation.status}`}
                  >
                    <strong>{modelManagerOperation.summary}</strong>
                    {modelManagerOperation.recovery ? (
                      <span>{modelManagerOperation.recovery}</span>
                    ) : null}
                  </div>
                ) : null}

                <ol
                  className="validation-list"
                  aria-label="Model manager checks"
                >
                  {modelManager.validationChecks.map((check) => (
                    <li
                      className={check.passed ? "passed" : "failed"}
                      key={check.id}
                    >
                      <CircleCheck aria-hidden="true" size={16} />
                      <span>
                        {check.summary}
                        {check.recovery ? <em>{check.recovery}</em> : null}
                      </span>
                    </li>
                  ))}
                </ol>
              </div>
            ) : null}

            {showJobs ? (
              <div className="panel runtime-panel">
                <div className="panel-heading">
                  <h2>Worker Runtime</h2>
                  <span>{runtime.schemaVersion}</span>
                </div>
                <div className="runtime-summary" aria-label="Runtime status">
                  <div>
                    <PackageCheck aria-hidden="true" size={18} />
                    <strong>{runtime.statusCounts.installed}</strong>
                    <span>verified installs</span>
                  </div>
                  <div>
                    <HardDrive aria-hidden="true" size={18} />
                    <strong>{runtime.statusCounts.available}</strong>
                    <span>available</span>
                  </div>
                  <div>
                    <CircleAlert aria-hidden="true" size={18} />
                    <strong>{runtime.statusCounts.unavailable}</strong>
                    <span>unavailable</span>
                  </div>
                  <div>
                    <Cpu aria-hidden="true" size={18} />
                    <strong>{runtime.devices.length}</strong>
                    <span>devices</span>
                  </div>
                </div>

                <div className="runtime-policy">
                  <strong>{runtime.packagingPolicy.name}</strong>
                  <span>
                    Python runtime:{" "}
                    {runtime.packagingPolicy.productRuntimeAllowsPython
                      ? "allowed"
                      : "blocked"}
                  </span>
                  <small>
                    {runtime.packagingPolicy.shippedPlatforms
                      .map(statusLabel)
                      .join(" / ")}
                  </small>
                </div>

                <div className="runtime-columns">
                  <div className="runtime-stack">
                    <h3>Models</h3>
                    <ol className="runtime-list">
                      {runtime.modelStates.map((model) => (
                        <li key={`${model.providerId}-${model.modelId}`}>
                          <span
                            className={`runtime-dot ${model.availability}`}
                          />
                          <div>
                            <strong>{model.modelName}</strong>
                            <small>
                              {statusLabel(model.availability)} /{" "}
                              {statusLabel(model.installStatus)} /{" "}
                              {formatMb(model.cache.diskUsageMb)}
                            </small>
                            <em>{model.cache.evidence}</em>
                            {model.reasons[0] ? (
                              <em>{model.reasons[0]}</em>
                            ) : null}
                          </div>
                        </li>
                      ))}
                    </ol>
                  </div>

                  <div className="runtime-stack">
                    <h3>Jobs</h3>
                    <ol className="runtime-list">
                      {runtime.jobs.length === 0 ? (
                        <li>
                          <span className="runtime-dot unavailable" />
                          <div>
                            <strong>No runtime jobs</strong>
                            <small>
                              Fixture/demo actions are blocked until provider
                              execution is wired.
                            </small>
                          </div>
                        </li>
                      ) : null}
                      {runtime.jobs.map((job) => (
                        <li key={job.id}>
                          <span className={`runtime-dot ${job.status}`} />
                          <div>
                            <strong>{statusLabel(job.kind)}</strong>
                            <small>
                              {statusLabel(job.status)} /{" "}
                              {Math.round(job.progress?.percent ?? 0)}% /{" "}
                              {statusLabel(job.cancellation)}
                            </small>
                            <em>{job.recordRoot}</em>
                            {job.artifacts[0] ? (
                              <em>
                                {job.artifacts[0].summary}:{" "}
                                {job.artifacts[0].path}
                              </em>
                            ) : null}
                            {job.actionableError ? (
                              <em>{job.actionableError.recovery}</em>
                            ) : null}
                            <div className="runtime-job-actions">
                              <button
                                className="secondary-action"
                                disabled={job.cancellation !== "cancellable"}
                                onClick={() => cancelRuntimeOperation(job.id)}
                                type="button"
                              >
                                Cancel
                              </button>
                              <button
                                className="secondary-action"
                                disabled={job.status !== "failed"}
                                onClick={() => retryRuntimeOperation(job.id)}
                                type="button"
                              >
                                Retry
                              </button>
                            </div>
                          </div>
                        </li>
                      ))}
                    </ol>
                  </div>
                </div>

                <ol className="validation-list" aria-label="Runtime checks">
                  {runtime.validationChecks.map((check) => (
                    <li className={check.status} key={check.id}>
                      <CircleCheck aria-hidden="true" size={16} />
                      <span>
                        {check.summary}
                        {check.recovery ? <em>{check.recovery}</em> : null}
                      </span>
                    </li>
                  ))}
                </ol>
              </div>
            ) : null}
          </section>
        ) : null}
      </section>
    </main>
    </AppContext.Provider>
  );
}
