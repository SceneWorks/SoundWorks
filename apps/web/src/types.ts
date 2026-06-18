export type ArchitectureLayer = {
  id: string;
  name: string;
  responsibility: string;
  status: "planned" | "scaffolded";
};

export type StudioSurface = {
  id: string;
  name: string;
  route: string;
  status: "planned" | "scaffolded";
};

export type CommandBoundary = {
  name: string;
  direction: "ui-to-backend" | "backend-to-ui";
  purpose: string;
};

export type AppOverview = {
  productName: string;
  architecture: {
    layers: ArchitectureLayer[];
  };
  studios: StudioSurface[];
  commands: CommandBoundary[];
  providerCatalog: ProviderCatalogOverview;
  modelEvaluation: ModelEvaluationOverview;
  ttsStudio: TtsStudioSummary;
};

export type RuntimeAvailability = "installed" | "available" | "unavailable";

export type RuntimeModelState = {
  providerId: string;
  modelId: string;
  modelName: string;
  availability: RuntimeAvailability;
  installStatus: string;
  health: string;
  workflows: CapabilityWorkflow[];
  reasons: string[];
  cache: {
    status: string;
    expectedSizeMb?: number | null;
    diskUsageMb?: number | null;
    warmup: string;
  };
  compatibility: {
    supported: boolean;
    selectedAccelerator?: string | null;
    minMemoryMb?: number | null;
    availableMemoryMb?: number | null;
    requiresNetwork: boolean;
    reasons: string[];
  };
};

export type RuntimeJobSnapshot = {
  id: string;
  kind: string;
  status: string;
  providerId: string;
  modelId: string;
  progress?: {
    percent: number;
    message?: string | null;
  } | null;
  cancellation: string;
  retryCount: number;
  logTail: string[];
  actionableError?: {
    code: string;
    summary: string;
    recovery: string;
  } | null;
};

export type RuntimeValidationCheck = {
  id: string;
  status: "passed" | "warning" | "failed";
  summary: string;
  recovery?: string | null;
};

export type RuntimeOverview = {
  schemaVersion: number;
  packagingPolicy: {
    name: string;
    productRuntimeAllowsPython: boolean;
    shippedPlatforms: string[];
    workerProcess: string;
    modelCacheRoots: Array<{
      platform: string;
      pathHint: string;
      purpose: string;
    }>;
  };
  devices: Array<{
    accelerator: string;
    name: string;
    memoryMb?: number | null;
    available: boolean;
    driver?: string | null;
  }>;
  statusCounts: {
    installed: number;
    available: number;
    unavailable: number;
  };
  modelStates: RuntimeModelState[];
  jobs: RuntimeJobSnapshot[];
  validationChecks: RuntimeValidationCheck[];
};

export type CapabilityWorkflow =
  | "tts"
  | "voice-clone"
  | "voice-conversion"
  | "sfx"
  | "ambience"
  | "instrument-sample"
  | "loop"
  | "song"
  | "stem-separation"
  | "video-to-audio"
  | "edit"
  | "composition-render";

export type CapabilityWorkflowSummary = {
  workflow: CapabilityWorkflow;
  defaultProviderId: string;
  defaultModelId: string;
};

export type ProviderCatalogOverview = {
  schemaVersion: number;
  providerCount: number;
  modelCount: number;
  capabilityCount: number;
  workflows: CapabilityWorkflowSummary[];
};

export type ModelEvaluationOverview = {
  schemaVersion: number;
  candidateCount: number;
  sourceCount: number;
  fixtureCount: number;
  laneCount: number;
  statusCounts: Record<string, number>;
  productEligibilityCounts: Record<string, number>;
  recommendedCandidateIds: string[];
};

export type TtsStudioSummary = {
  schemaVersion: number;
  segmentCount: number;
  speakerCount: number;
  providerCount: number;
  canSubmit: boolean;
  selectedProviderId: string;
  selectedModelId: string;
  savedAssetKind: string;
};

export type VoiceConsentStatus =
  | "not-voice-material"
  | "explicit-consent-recorded"
  | "provider-stock-voice"
  | "requires-review"
  | "prohibited";

export type TtsScriptSegment = {
  id: string;
  position: number;
  speakerLabel: string;
  text: string;
  sceneLabel?: string | null;
  targetDurationMs?: number | null;
  regeneratePolicy: "regenerate-independently" | "keep-timing-with-neighbors";
};

export type TtsSpeaker = {
  label: string;
  role: string;
  voiceProfileId: string;
  language: string;
  consentRequired: boolean;
  consentStatus: VoiceConsentStatus;
};

export type TtsProviderOption = {
  providerId: string;
  modelId: string;
  modelVersion?: string | null;
  displayName: string;
  runtime: string;
  installStatus: string;
  runnable: boolean;
  outputFormat: string;
  sampleRateHz: number;
  channelLayout: string;
  supportedLanguages: string[];
  maxSpeakers?: number | null;
  maxDurationMs?: number | null;
  commercialUseAllowed: boolean;
  requiresVoiceConsent: boolean;
  watermark: string;
  limitations: string[];
};

export type TtsStudioOverview = {
  schemaVersion: number;
  script: {
    id: string;
    title: string;
    language: string;
    segments: TtsScriptSegment[];
    pronunciationDictionary: Array<{
      term: string;
      pronunciation: string;
      appliesToLanguage: string;
    }>;
  };
  speakers: TtsSpeaker[];
  voiceProfiles: Array<{
    id: string;
    displayName: string;
    consent: VoiceConsentStatus;
    allowedUses: string[];
  }>;
  providerOptions: TtsProviderOption[];
  selectedProvider: {
    providerId: string;
    modelId: string;
    modelVersion?: string | null;
    runtime: string;
    accepted: boolean;
    blocker?: string | null;
  };
  controls: {
    speed: number;
    style: string;
    emotion?: string | null;
    targetLoudnessLufs: number;
    normalizeOutput: boolean;
    preserveSegmentTiming: boolean;
    promoteToProjectLibrary: boolean;
  };
  generationPlan: {
    chunks: Array<{
      id: string;
      segmentIds: string[];
      speakerLabel: string;
      voiceProfileId?: string | null;
      targetDurationMs: number;
      regeneratePolicy: string;
    }>;
    stitching: {
      crossfadeMs: number;
      preserveSegmentTiming: boolean;
      silenceTrim: boolean;
      normalizeLoudnessLufs?: number | null;
    };
    estimatedTotalDurationMs: number;
    preservesSpeakerConsistency: boolean;
  };
  submission: {
    canSubmit: boolean;
    job: {
      id: string;
      recipeId: string;
      kind: string;
      status: string;
      progress?: {
        percent: number;
        message?: string | null;
      } | null;
      outputVersionIds: string[];
      error?: string | null;
    };
    recipe: {
      id: string;
      workflow: string;
      outputAssetIds: string[];
    };
    blockingReasons: string[];
    warnings: string[];
  };
  savedOutput: {
    asset: {
      id: string;
      kind: string;
      name: string;
      tags: string[];
      currentVersionId: string;
    };
    version: {
      id: string;
      file: {
        storagePath: string;
        format: string;
      };
      technical: {
        sampleRateHz: number;
        channels: number;
        durationMs: number;
        loudnessLufs?: number | null;
      };
    };
    promotedToProjectLibrary: boolean;
    waveformPreviewReady: boolean;
  };
  validationChecks: Array<{
    id: string;
    status: "passed" | "warning" | "failed";
    summary: string;
  }>;
};
