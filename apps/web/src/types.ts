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
  voiceLab: VoiceLabSummary;
  sfxStudio: SfxStudioSummary;
  samplesStudio: SamplesStudioSummary;
  songStudio: SongStudioSummary;
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

export type VoiceLabSummary = {
  schemaVersion: number;
  modeCount: number;
  profileCount: number;
  providerCount: number;
  safetyGateCount: number;
  canSubmitConversion: boolean;
  selectedConversionCandidateId: string;
  savedAssetKind: string;
};

export type SfxStudioSummary = {
  schemaVersion: number;
  variantCount: number;
  savedOutputCount: number;
  providerCount: number;
  scorecardCount: number;
  canSubmit: boolean;
  selectedProviderId: string;
  selectedModelId: string;
  savedAssetKinds: string[];
};

export type SamplesStudioSummary = {
  schemaVersion: number;
  variantCount: number;
  savedOutputCount: number;
  providerCount: number;
  scorecardCount: number;
  canSubmit: boolean;
  selectedProviderId: string;
  selectedModelId: string;
  packCollectionId: string;
  savedAssetKinds: string[];
};

export type SongStudioSummary = {
  schemaVersion: number;
  sectionCount: number;
  variantCount: number;
  savedOutputCount: number;
  providerCount: number;
  scorecardCount: number;
  canSubmit: boolean;
  selectedProviderId: string;
  selectedModelId: string;
  requestedStems: string[];
  savedAssetKinds: string[];
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

export type VoiceLabMode =
  | "zero-shot-clone"
  | "few-shot-fine-tune"
  | "voice-conversion";

export type VoiceLabOverview = {
  schemaVersion: number;
  modes: Array<{
    mode: VoiceLabMode;
    label: string;
    workflow: CapabilityWorkflow;
    inputAssetKinds: string[];
    outputAssetKind: string;
    providerCandidateIds: string[];
    ready: boolean;
  }>;
  voiceProfiles: Array<{
    profile: {
      id: string;
      displayName: string;
      consent: VoiceConsentStatus;
      allowedUses: string[];
    };
    speakerIdentity: string;
    language: string;
    sourceClipIds: string[];
    modeReadiness: Array<{
      mode: VoiceLabMode;
      ready: boolean;
      reason?: string | null;
    }>;
    commercialUseAllowed: boolean;
    safetySummary: string;
  }>;
  referenceClips: Array<{
    id: string;
    assetId: string;
    profileId: string;
    label: string;
    durationMs: number;
    consent: VoiceConsentStatus;
    ownerAttestation: string;
    acceptedForModes: VoiceLabMode[];
  }>;
  conversionSource: {
    assetId: string;
    name: string;
    durationMs: number;
    kind: string;
  };
  providerScorecards: Array<{
    candidateId: string;
    name: string;
    provider: string;
    lanes: string[];
    status: string;
    productEligibility: string;
    readiness: string;
    runtimePath: string;
    commercialUse: string;
    recommended: boolean;
    blockers: string[];
    notes: string;
  }>;
  selectedConversion: {
    canSubmit: boolean;
    job: {
      id: string;
      recipeId: string;
      kind: string;
      status: string;
      outputVersionIds: string[];
      error?: string | null;
    };
    recipe: {
      id: string;
      workflow: string;
      provider: {
        providerId: string;
        modelId: string;
        runtime: string;
      };
      request: {
        kind: string;
        sourceAudioAssetId?: string;
        targetVoiceProfileId?: string;
        preserveTiming?: boolean;
      };
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
    waveformPreviewReady: boolean;
  };
  safetyGates: Array<{
    id: string;
    status: "passed" | "warning" | "blocked";
    summary: string;
  }>;
  qaChecks: Array<{
    id: string;
    label: string;
    status: "ready" | "needs-review";
    target: string;
  }>;
};

export type SfxCategory =
  | "foley-impact"
  | "ambience-bed"
  | "transition"
  | "ui-sound"
  | "creature"
  | "weather";

export type SfxStudioOverview = {
  schemaVersion: number;
  prompt: {
    id: string;
    text: string;
    negativePrompt: string;
    category: SfxCategory;
    tags: string[];
    referenceAudioAssetId?: string | null;
  };
  controls: {
    durationMs: number;
    variationCount: number;
    intensity: number;
    realism: number;
    loopable: boolean;
    trimSilence: boolean;
    normalizeLoudnessLufs: number;
    fadeInMs: number;
    fadeOutMs: number;
    loopCrossfadeMs: number;
    promoteToProjectLibrary: boolean;
  };
  categoryPresets: Array<{
    category: SfxCategory;
    label: string;
    defaultDurationMs: number;
    loopableDefault: boolean;
    outputKind: string;
  }>;
  providerOptions: Array<{
    providerId: string;
    modelId: string;
    modelVersion?: string | null;
    displayName: string;
    workflow: CapabilityWorkflow;
    runtime: string;
    installStatus: string;
    runnable: boolean;
    outputAssetKind: string;
    outputFormat: string;
    sampleRateHz: number;
    channelLayout: string;
    minDurationMs?: number | null;
    maxDurationMs?: number | null;
    supportsReferenceAudio: boolean;
    supportsLooping: boolean;
    commercialUseAllowed: boolean;
    watermark: string;
    supportedControls: string[];
    limitations: string[];
  }>;
  selectedProvider: {
    providerId: string;
    modelId: string;
    modelVersion?: string | null;
    workflow: CapabilityWorkflow;
    runtime: string;
    accepted: boolean;
    blocker?: string | null;
  };
  providerScorecards: Array<{
    candidateId: string;
    name: string;
    provider: string;
    lanes: string[];
    status: string;
    productEligibility: string;
    readiness: string;
    runtimePath: string;
    commercialUse: string;
    recommended: boolean;
    blockers: string[];
    notes: string;
  }>;
  deferredMultimodalCandidateIds: string[];
  variants: Array<{
    id: string;
    label: string;
    workflow: CapabilityWorkflow;
    assetKind: string;
    category: SfxCategory;
    durationMs: number;
    loudnessLufs: number;
    truePeakDbfs: number;
    loopable: boolean;
    loopPoints?: {
      startSample: number;
      endSample: number;
    } | null;
    tags: string[];
    selectedForSave: boolean;
  }>;
  comparison: {
    selectedVariantId: string;
    variantCount: number;
    loopableVariantIds: string[];
    savedVariantIds: string[];
  };
  submission: {
    canSubmit: boolean;
    job: {
      id: string;
      recipeId: string;
      kind: string;
      status: string;
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
  savedOutputs: Array<{
    variantId: string;
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
        loopPoints?: {
          startSample: number;
          endSample: number;
        } | null;
      };
    };
    exported: boolean;
    waveformPreviewReady: boolean;
  }>;
  postProcessingActions: Array<{
    id: string;
    operation: string;
    enabled: boolean;
    summary: string;
  }>;
  validationChecks: Array<{
    id: string;
    status: "passed" | "warning" | "failed";
    summary: string;
  }>;
};

export type InstrumentFamily =
  | "drums"
  | "bass"
  | "synth-bass"
  | "guitar"
  | "keys"
  | "strings"
  | "brass"
  | "texture";

export type SamplesStudioOverview = {
  schemaVersion: number;
  prompt: {
    id: string;
    text: string;
    negativePrompt: string;
    instrumentFamily: InstrumentFamily;
    articulation: string;
    genreTags: string[];
    referenceAudioAssetId?: string | null;
  };
  controls: {
    musicalKey: string;
    scale: string;
    bpm: number;
    bars: number;
    beats: number;
    loopable: boolean;
    dryWetAmbience: number;
    velocityEnergy: number;
    variationCount: number;
    batchSize: number;
    promoteToProjectLibrary: boolean;
  };
  providerOptions: Array<{
    providerId: string;
    modelId: string;
    modelVersion?: string | null;
    displayName: string;
    workflow: CapabilityWorkflow;
    runtime: string;
    installStatus: string;
    runnable: boolean;
    outputAssetKind: string;
    outputFormat: string;
    sampleRateHz: number;
    channelLayout: string;
    minDurationMs?: number | null;
    maxDurationMs?: number | null;
    supportsReferenceAudio: boolean;
    supportsTempo: boolean;
    supportsKey: boolean;
    supportsLoopPoints: boolean;
    commercialUseAllowed: boolean;
    watermark: string;
    supportedControls: string[];
    limitations: string[];
  }>;
  selectedProvider: {
    providerId: string;
    modelId: string;
    modelVersion?: string | null;
    workflow: CapabilityWorkflow;
    runtime: string;
    accepted: boolean;
    blocker?: string | null;
  };
  providerScorecards: Array<{
    candidateId: string;
    name: string;
    provider: string;
    lanes: string[];
    status: string;
    productEligibility: string;
    readiness: string;
    runtimePath: string;
    commercialUse: string;
    recommended: boolean;
    blockers: string[];
    notes: string;
  }>;
  variants: Array<{
    id: string;
    label: string;
    workflow: CapabilityWorkflow;
    assetKind: string;
    instrumentFamily: InstrumentFamily;
    articulation: string;
    durationMs: number;
    bpm?: number | null;
    musicalKey?: string | null;
    timeSignature?: string | null;
    loopPoints?: {
      startSample: number;
      endSample: number;
    } | null;
    transientOneShot: boolean;
    loudnessLufs: number;
    truePeakDbfs: number;
    hasClipping: boolean;
    tags: string[];
    collectionId: string;
    selectedForPack: boolean;
    favorite: boolean;
    duplicateOfVariantId?: string | null;
  }>;
  pack: {
    collectionId: string;
    name: string;
    variantCount: number;
    selectedVariantIds: string[];
    favoriteVariantIds: string[];
    loopVariantIds: string[];
    oneShotVariantIds: string[];
    exportFormats: string[];
  };
  submission: {
    canSubmit: boolean;
    jobs: Array<{
      id: string;
      recipeId: string;
      kind: string;
      status: string;
      outputVersionIds: string[];
      error?: string | null;
    }>;
    recipes: Array<{
      id: string;
      workflow: string;
      outputAssetIds: string[];
    }>;
    blockingReasons: string[];
    warnings: string[];
  };
  savedOutputs: Array<{
    variantId: string;
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
        bpm?: number | null;
        musicalKey?: string | null;
        loopPoints?: {
          startSample: number;
          endSample: number;
        } | null;
      };
    };
    exported: boolean;
    waveformPreviewReady: boolean;
  }>;
  postProcessingActions: Array<{
    id: string;
    operation: string;
    enabled: boolean;
    summary: string;
  }>;
  qaChecks: Array<{
    id: string;
    status: "passed" | "warning" | "failed";
    summary: string;
  }>;
};

export type SongVocalMode = "vocal" | "instrumental" | "both";

export type SongStudioOverview = {
  schemaVersion: number;
  draft: {
    id: string;
    title: string;
    prompt: string;
    lyrics: string;
    styleTags: string[];
    language: string;
    vocalist: SongVocalMode;
    singerHint?: string | null;
    referenceAudioAssetIds: string[];
    sections: Array<{
      id: string;
      label: string;
      bars: number;
      lyrics?: string | null;
      regenerateLocked: boolean;
    }>;
  };
  controls: {
    bpm: number;
    musicalKey: string;
    timeSignature: string;
    targetDurationMs: number;
    sectionLengthBars: number;
    variationCount: number;
    generateStems: boolean;
    requestedStems: string[];
    allowReferenceAudio: boolean;
    promoteToProjectLibrary: boolean;
  };
  providerOptions: Array<{
    providerId: string;
    modelId: string;
    modelVersion?: string | null;
    displayName: string;
    workflow: CapabilityWorkflow;
    runtime: string;
    installStatus: string;
    runnable: boolean;
    outputAssetKinds: string[];
    outputFormat: string;
    sampleRateHz: number;
    channelLayout: string;
    minDurationMs?: number | null;
    maxDurationMs?: number | null;
    supportsLyrics: boolean;
    supportsStyleTags: boolean;
    supportsReferenceAudio: boolean;
    supportsStems: boolean;
    supportedStems: string[];
    commercialUseAllowed: boolean;
    watermark: string;
    supportedControls: string[];
    limitations: string[];
  }>;
  selectedProvider: {
    providerId: string;
    modelId: string;
    modelVersion?: string | null;
    workflow: CapabilityWorkflow;
    runtime: string;
    accepted: boolean;
    blocker?: string | null;
  };
  providerScorecards: Array<{
    candidateId: string;
    name: string;
    provider: string;
    lanes: string[];
    status: string;
    productEligibility: string;
    readiness: string;
    runtimePath: string;
    commercialUse: string;
    recommended: boolean;
    blockers: string[];
    notes: string;
  }>;
  arrangement: {
    sectionCount: number;
    totalBars: number;
    estimatedDurationMs: number;
    sections: Array<{
      id: string;
      label: string;
      startBar: number;
      bars: number;
      hasLyrics: boolean;
      locked: boolean;
    }>;
  };
  variants: Array<{
    id: string;
    label: string;
    assetKind: string;
    durationMs: number;
    bpm: number;
    musicalKey: string;
    vocalMode: SongVocalMode;
    stemKinds: string[];
    loudnessLufs: number;
    truePeakDbfs: number;
    lyricAlignmentScore: number;
    structureMatchScore: number;
    tags: string[];
    selectedForSave: boolean;
  }>;
  submission: {
    canSubmit: boolean;
    job: {
      id: string;
      recipeId: string;
      kind: string;
      status: string;
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
  savedOutputs: Array<{
    variantId: string;
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
        truePeakDbfs?: number | null;
        bpm?: number | null;
        musicalKey?: string | null;
      };
    };
    exportReady: boolean;
    waveformPreviewReady: boolean;
  }>;
  exportTargets: Array<{
    id: string;
    label: string;
    formats: string[];
    includesStems: boolean;
    includesSidecar: boolean;
    summary: string;
  }>;
  qaChecks: Array<{
    id: string;
    status: "passed" | "warning" | "failed";
    summary: string;
  }>;
};
