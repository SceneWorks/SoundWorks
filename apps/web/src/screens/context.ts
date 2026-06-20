// DR-02: the app context that lets per-ActiveView screen components read shared
// state + handlers + derived values without prop-drilling (mirrors SceneWorks'
// useAppContext). App builds the value once and provides it; screens consume the
// slice they need. This is the single contract between the App shell and the
// extracted screens.
import { createContext, useContext } from "react";
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
  ReviewWorkspaceOverview,
  RightsSafetyOverview,
  RuntimeJobRequest,
  RuntimeJobSnapshot,
  RuntimeOverview,
  SamplesStudioOverview,
  VoiceConsentStatus,
  SfxStudioOverview,
  SongStudioOverview,
  TtsStudioOverview,
  VideoToAudioOverview,
  VoiceLabOverview,
  WorkspaceOverview,
} from "../types";
import type { ThemeMode } from "../accents";
import type {
  ActionFeedback,
  ActiveView,
  NavItem,
  RuntimeModelState,
} from "../viewModel";

export interface AppContextValue {
  // Navigation + theme chrome
  activeView: ActiveView;
  setActiveView: (view: ActiveView) => void;
  activeViewMeta: NavItem;
  theme: ThemeMode;
  accent: string;
  changeTheme: (next: ThemeMode) => void;
  changeAccent: (next: string) => void;
  webPreview: boolean;
  dataError: string | null;

  // Overviews (per-view state)
  overview: AppOverview;
  runtime: RuntimeOverview;
  modelManager: ModelManagerOverview;
  modelManagerOperation: ModelManagerOperation | null;
  runtimeOperation: RuntimeJobSnapshot | null;
  workspace: WorkspaceOverview;
  assetLibrary: AssetLibraryOverview;
  exportWorkflow: ExportWorkflowOverview;
  compositionEditor: CompositionEditorOverview;
  mvpValidation: MvpValidationOverview;
  ttsStudio: TtsStudioOverview;
  voiceLab: VoiceLabOverview;
  sfxStudio: SfxStudioOverview;
  samplesStudio: SamplesStudioOverview;
  songStudio: SongStudioOverview;
  reviewWorkspace: ReviewWorkspaceOverview;
  rightsSafety: RightsSafetyOverview;
  videoToAudio: VideoToAudioOverview;
  libraryPlayback: LibraryPlayback | null;

  // Action feedback tri-state (+ the one setter a screen drives directly)
  libraryActionStatus: ActionFeedback;
  setLibraryActionStatus: (status: ActionFeedback) => void;
  reviewActionStatus: ActionFeedback;
  exportActionStatus: ActionFeedback;

  // Derived
  scaffoldedLayerCount: number;
  voiceCandidateFocus: VoiceLabOverview["providerScorecards"];
  sfxCandidateFocus: SfxStudioOverview["providerScorecards"];
  videoCandidateFocus: VideoToAudioOverview["providerScorecards"];
  samplesCandidateFocus: SamplesStudioOverview["providerScorecards"];
  songCandidateFocus: SongStudioOverview["providerScorecards"];
  ttsRuntimeModel: RuntimeModelState | null;
  sfxRuntimeModel: RuntimeModelState | null;
  voiceRuntimeModel: RuntimeModelState | null;
  videoRuntimeModel: RuntimeModelState | null;
  songRuntimeModel: RuntimeModelState | null;
  latestImportableRuntimeJob: RuntimeJobSnapshot | null;

  // Handlers
  runModelManagerAction: (
    candidateId: string,
    action: "install" | "revalidate",
  ) => void;
  createProject: (name?: string) => void;
  openProject: (projectId: string) => void;
  openRecentProject: () => void;
  importLatestRuntimeArtifact: () => void;
  mutateSelectedLibraryItem: (action: LibraryMutationAction) => void;
  previewLibraryItem: (itemId: string) => void;
  saveSelectedReviewEdit: () => void;
  exportSelectedLibraryItem: (override?: {
    presetId: string;
    formats: ExportWorkflowOverview["selectedExport"]["formats"];
  }) => void;
  runRuntimeJob: (
    workflow: RuntimeJobRequest["workflow"],
    prompt: string,
    parameters?: Record<string, unknown>,
  ) => void;
  cancelRuntimeOperation: (jobId: string) => void;
  retryRuntimeOperation: (jobId: string) => void;
  recordVoiceProfileConsent: (
    profileId: string,
    consent: VoiceConsentStatus,
  ) => void;
  renderComposition: (mutedTrackIds?: string[]) => void;
  // UX-14: deep-link from a studio gate to the Models grid, pre-filtered.
  modelFocus: string | null;
  openModelsFor: (focus: string) => void;
  clearModelFocus: () => void;
  // UX-15: durable demo-library toggle.
  demoMode: boolean;
  changeDemoMode: (next: boolean) => void;
}

export const AppContext = createContext<AppContextValue | null>(null);

export function useAppContext(): AppContextValue {
  const value = useContext(AppContext);
  if (!value) {
    throw new Error("useAppContext must be used within an AppContext.Provider");
  }
  return value;
}
