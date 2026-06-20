import {
  Activity,
  Boxes,
  CircleAlert,
  ClipboardCheck,
  Cpu,
  Download,
  FileVideo,
  Gauge,
  Library,
  Mic2,
  Moon,
  Music2,
  PackageCheck,
  Play,
  Radio,
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
  runtimeModelFor,
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
import { LibraryScreen } from "./screens/LibraryScreen";
import { ExportScreen } from "./screens/ExportScreen";
import { MultitrackScreen } from "./screens/MultitrackScreen";
import { ReviewScreen } from "./screens/ReviewScreen";
import { RightsScreen } from "./screens/RightsScreen";
import { ValidationScreen } from "./screens/ValidationScreen";
import { SettingsScreen } from "./screens/SettingsScreen";
import { ModelsScreen } from "./screens/ModelsScreen";
import { JobsScreen } from "./screens/JobsScreen";
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
    setLibraryActionStatus,
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

        {showLibrary ? <LibraryScreen /> : null}

        {showExport ? <ExportScreen /> : null}

        {showMixer ? <MultitrackScreen /> : null}

        {showValidation ? <ValidationScreen /> : null}

        {showTts ? <TtsScreen /> : null}

        {showVoice ? <VoiceLabScreen /> : null}

        {showSfx ? <SfxScreen /> : null}

        {showVideoToAudio ? <VideoToAudioScreen /> : null}

        {showSamples ? <SamplesScreen /> : null}

        {showSong ? <SongScreen /> : null}

        {showReview ? <ReviewScreen /> : null}

        {showRights ? <RightsScreen /> : null}

        {showSettings ? <SettingsScreen /> : null}
        {showModels ? <ModelsScreen /> : null}
        {showJobs ? <JobsScreen /> : null}
      </section>
    </main>
    </AppContext.Provider>
  );
}
