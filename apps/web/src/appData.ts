// Web-preview fallback fixtures.
//
// This file is a thin typed shim over `appData.generated.json`, which is
// generated from the Rust reference fixtures by the `gen_web_fixtures` bin
// (`npm run gen:fixtures`). The Rust crate is the single source of truth (F-012);
// `crates/soundworks-core/tests/web_fixtures_parity.rs` fails if the committed
// JSON drifts from the fixtures. Do not edit the JSON by hand — regenerate it.
import generated from "./appData.generated.json";
import type {
  AppOverview,
  AssetLibraryOverview,
  CompositionEditorOverview,
  ExportWorkflowOverview,
  ModelManagerOverview,
  MvpValidationOverview,
  ReviewWorkspaceOverview,
  RightsSafetyOverview,
  RuntimeOverview,
  SamplesStudioOverview,
  SfxStudioOverview,
  SongStudioOverview,
  TtsStudioOverview,
  VideoToAudioOverview,
  VoiceLabOverview,
  WorkspaceOverview,
} from "./types";

export const fallbackOverview = generated.fallbackOverview as unknown as AppOverview;
export const fallbackRuntime = generated.fallbackRuntime as unknown as RuntimeOverview;
export const fallbackWorkspace =
  generated.fallbackWorkspace as unknown as WorkspaceOverview;
export const fallbackAssetLibrary =
  generated.fallbackAssetLibrary as unknown as AssetLibraryOverview;
export const fallbackCompositionEditor =
  generated.fallbackCompositionEditor as unknown as CompositionEditorOverview;
export const fallbackExportWorkflow =
  generated.fallbackExportWorkflow as unknown as ExportWorkflowOverview;
export const fallbackMvpValidation =
  generated.fallbackMvpValidation as unknown as MvpValidationOverview;
export const fallbackModelManager =
  generated.fallbackModelManager as unknown as ModelManagerOverview;
export const fallbackRightsSafety =
  generated.fallbackRightsSafety as unknown as RightsSafetyOverview;
export const fallbackReviewWorkspace =
  generated.fallbackReviewWorkspace as unknown as ReviewWorkspaceOverview;
export const fallbackTtsStudio =
  generated.fallbackTtsStudio as unknown as TtsStudioOverview;
export const fallbackVoiceLab =
  generated.fallbackVoiceLab as unknown as VoiceLabOverview;
export const fallbackSfxStudio =
  generated.fallbackSfxStudio as unknown as SfxStudioOverview;
export const fallbackSamplesStudio =
  generated.fallbackSamplesStudio as unknown as SamplesStudioOverview;
export const fallbackSongStudio =
  generated.fallbackSongStudio as unknown as SongStudioOverview;
export const fallbackVideoToAudio =
  generated.fallbackVideoToAudio as unknown as VideoToAudioOverview;
