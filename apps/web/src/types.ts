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
  workspace: WorkspaceSummary;
  providerCatalog: ProviderCatalogOverview;
  assetLibrary: AssetLibrarySummary;
  exportWorkflow: ExportWorkflowSummary;
  compositionEditor: CompositionEditorSummary;
  mvpValidation: MvpValidationSummary;
  modelEvaluation: ModelEvaluationOverview;
  ttsStudio: TtsStudioSummary;
  voiceLab: VoiceLabSummary;
  sfxStudio: SfxStudioSummary;
  samplesStudio: SamplesStudioSummary;
  songStudio: SongStudioSummary;
  reviewWorkspace: ReviewWorkspaceSummary;
  rightsSafety: RightsSafetySummary;
  videoToAudio: VideoToAudioSummary;
};

export type WorkspaceSummary = {
  schemaVersion: number;
  projectCount: number;
  projectAssetCount: number;
  globalAssetCount: number;
  linkedGlobalAssetCount: number;
  transferActionCount: number;
  sourcePickerTargetCount: number;
  parityNoteCount: number;
  activeProjectId: string;
  globalLibraryId: string;
  canCreateProject: boolean;
  canOpenProject: boolean;
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
    verified: boolean;
    evidence: string;
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

export type LibraryItemType =
  | "voice-clip"
  | "music-clip"
  | "sfx"
  | "song"
  | "instrument-sample"
  | "loop"
  | "stem"
  | "ambience"
  | "voice-profile"
  | "reference-audio"
  | "composition"
  | "mixdown-export"
  | "prompt-recipe-preset";

export type AssetLibrarySummary = {
  schemaVersion: number;
  itemCount: number;
  previewableItemCount: number;
  collectionCount: number;
  scopeCount: number;
  filterCount: number;
  supportedTypeCount: number;
  favoriteCount: number;
  rejectedCount: number;
  archivedCount: number;
  selectedItemId: string;
  selectedItemType: LibraryItemType;
};

export type ExportSourceKind =
  | "asset"
  | "collection"
  | "sample-pack"
  | "loop-pack"
  | "song"
  | "stem-bundle"
  | "composition";

export type AudioFileFormat = "wav" | "flac" | "mp3" | "ogg" | "aiff";

export type ExportWorkflowSummary = {
  schemaVersion: number;
  presetCount: number;
  targetCount: number;
  sidecarCount: number;
  readyTargetCount: number;
  selectedPresetId: string;
  selectedSourceKind: ExportSourceKind;
  selectedFormatCount: number;
  canExportSelected: boolean;
  writesDawBundle: boolean;
  writesSceneWorksPackage: boolean;
};

export type ExportWorkflowOverview = {
  schemaVersion: number;
  presets: Array<{
    preset: {
      id: string;
      name: string;
      format: AudioFileFormat;
      sampleRateHz: number;
      bitDepth?: number | null;
      includeSidecar: boolean;
      includeStems: boolean;
      target: string;
    };
    description: string;
    sourceKinds: ExportSourceKind[];
    assetKinds: string[];
    formats: AudioFileFormat[];
    packageArtifacts: string[];
    normalizeLoudness: boolean;
    targetLufs?: number | null;
    preserveLoopMetadata: boolean;
    preserveBpmKeyMetadata: boolean;
    writesSidecar: boolean;
  }>;
  targets: Array<{
    target: string;
    label: string;
    ready: boolean;
    presetIds: string[];
    notes: string[];
  }>;
  selectedExport: {
    id: string;
    presetId: string;
    sourceKind: ExportSourceKind;
    sourceId: string;
    assetIds: string[];
    collectionIds: string[];
    formats: AudioFileFormat[];
    canExport: boolean;
    blockingReasons: string[];
    warnings: string[];
    outputPaths: string[];
    sidecarPath: string;
  };
  sidecars: Array<{
    id: string;
    assetId: string;
    assetKind: string;
    target: string;
    path: string;
    includesRecipe: boolean;
    includesModel: boolean;
    includesSourceMedia: boolean;
    includesRights: boolean;
    includesEditChain: boolean;
    disclosureRequired: boolean;
    eventCount: number;
  }>;
  dawHandoff: {
    id: string;
    presetId: string;
    packagePath: string;
    normalizedFilenameTemplate: string;
    includesZipBundle: boolean;
    includesStems: boolean;
    includesCueMarkers: boolean;
    includesLoopMarkers: boolean;
    includesBpmKeyMetadata: boolean;
    includesLyricsText: boolean;
    includesMidi: boolean;
    stemKinds: string[];
  };
  sceneWorksHandoff: {
    id: string;
    presetId: string;
    packagePath: string;
    renderedMixdownPath: string;
    packageManifestPath: string;
    provenanceSidecarPath: string;
    includesOptionalStems: boolean;
    optionalStemPaths: string[];
    importStrategy: string;
    attachmentMode: string;
    intendedProjectId?: string | null;
    intendedVideoAssetId?: string | null;
    sceneWorksProjectPath?: string | null;
    targetVideoSidecarPath?: string | null;
    sceneWorksAssetType: string;
    sceneWorksMimeType: string;
    durationMs: number;
    targetVideoDurationMs: number;
    startOffsetMs: number;
    sampleRateHz: number;
    channels: number;
    loudnessLufs?: number | null;
    truePeakDbfs?: number | null;
    markerCount: number;
    sectionCount: number;
    replaceExistingAudio: boolean;
    roundTripRecipeUrl: string;
    sourceEvidence: Array<{
      sourceRepo: string;
      filePath: string;
      lineHint: string;
      finding: string;
    }>;
    compatibilityChecks: Array<{
      id: string;
      status: string;
      summary: string;
      mitigation: string;
    }>;
    attachmentSteps: Array<{
      id: string;
      label: string;
      required: boolean;
      source: string;
      target: string;
    }>;
  };
  validationChecks: Array<{
    id: string;
    passed: boolean;
    summary: string;
  }>;
};

export type CompositionEditorSummary = {
  schemaVersion: number;
  trackCount: number;
  clipCount: number;
  assetBinCount: number;
  enabledToolCount: number;
  markerCount: number;
  sectionCount: number;
  selectedClipId: string;
  canRenderMixdown: boolean;
  editableAssetKinds: string[];
  recommendedComponentId: string;
  componentCandidateCount: number;
};

export type CompositionEditorOverview = {
  schemaVersion: number;
  projectId: string;
  composition: {
    id: string;
    name: string;
    tempoBpm?: number | null;
    musicalKey?: string | null;
    markers: Array<{ id: string; atMs: number; label: string }>;
    sections: Array<{
      id: string;
      range: { startMs: number; endMs: number };
      label: string;
    }>;
    exportHistory: Array<{
      id: string;
      jobId: string;
      outputAssetId: string;
      presetId: string;
    }>;
  };
  timeline: {
    durationMs: number;
    zoomPercent: number;
    snapGridMs: number;
    selectedTool: string;
    selectedClipId: string;
    playbackCursorMs: number;
    loopEnabled: boolean;
    loopRange: { startMs: number; endMs: number };
    gridLabels: string[];
    markersEditable: boolean;
    sectionsEditable: boolean;
  };
  assetBin: Array<{
    assetId: string;
    versionId: string;
    name: string;
    kind: LibraryItemType;
    scope: LibraryScope;
    durationMs: number;
    tags: string[];
    sourceWorkflow: string;
    auditionReady: boolean;
    draggableToTimeline: boolean;
    provenanceId: string;
  }>;
  sourceFlows: Array<{
    workflow: string;
    label: string;
    assetKind: LibraryItemType;
    status: "ready" | "planned" | "blocked";
    targetTrackRole: string;
  }>;
  tracks: Array<{
    trackId: string;
    name: string;
    role: string;
    clipCount: number;
    gainDb: number;
    pan: number;
    muted: boolean;
    soloed: boolean;
    automationTargets: string[];
    editable: boolean;
    clips: Array<{
      clipId: string;
      assetId: string;
      versionId: string;
      assetName: string;
      assetKind: LibraryItemType;
      sourceScope: LibraryScope;
      timelineStartMs: number;
      sourceRange: { startMs: number; endMs: number };
      fadeInMs: number;
      fadeOutMs: number;
      gainDb: number;
      pan: number;
      lane: number;
      canTrim: boolean;
      canSplit: boolean;
      canDuplicate: boolean;
      canDelete: boolean;
    }>;
  }>;
  mixer: {
    masterGainDb: number;
    targetLufs: number;
    truePeakCeilingDbfs: number;
    renderReady: boolean;
    loudnessCheck: string;
    warnings: string[];
    trackStates: Array<{
      trackId: string;
      label: string;
      gainDb: number;
      pan: number;
      muted: boolean;
      soloed: boolean;
      effectChain: string[];
      sendTargets: string[];
    }>;
  };
  tools: Array<{
    id: string;
    label: string;
    enabled: boolean;
    appliesTo: string[];
  }>;
  exportPlan: {
    canRenderMixdown: boolean;
    presetIds: string[];
    mixdownPath: string;
    stemPaths: string[];
    provenanceSidecarPath: string;
    requiredProvenanceFields: string[];
    sceneWorksReady: boolean;
    sceneWorksWarning: string;
  };
  componentDecisions: Array<{
    id: string;
    name: string;
    sourceUrl: string;
    license: string;
    fit:
      | "strong-prototype-candidate"
      | "renderer-primitive"
      | "timing-primitive"
      | "needs-spike";
    strengths: string[];
    risks: string[];
    prototypeEvidence: string;
    decision: string;
  }>;
  validationChecks: Array<{
    id: string;
    passed: boolean;
    summary: string;
  }>;
};

export type LibraryScope =
  | {
      kind: "globalLibrary";
    }
  | {
      kind: "project";
      projectId: string;
    };

export type AssetLibraryOverview = {
  schemaVersion: number;
  scopes: Array<{
    id: string;
    label: string;
    scope: LibraryScope;
    ownership: string;
    assetCount: number;
    collectionCount: number;
    canPromoteToGlobal: boolean;
  }>;
  filters: {
    facets: Array<{
      id: string;
      label: string;
      options: Array<{
        id: string;
        label: string;
        count: number;
        selected: boolean;
      }>;
    }>;
    supportedItemTypes: LibraryItemType[];
    coversProjectAndGlobalScopes: boolean;
    includesRejectedArchivedToggle: boolean;
  };
  selectedFilter: {
    searchText: string;
    scope: LibraryScope;
    selectedType?: LibraryItemType | null;
    selectedTags: string[];
    includeRejected: boolean;
    includeArchived: boolean;
    favoriteOnly: boolean;
  };
  items: LibraryItemCard[];
  selectedItem: {
    item: LibraryItemCard;
    versionHistory: Array<{
      versionId: string;
      label: string;
      durationMs?: number | null;
      filePath?: string | null;
      createdBy: string;
      waveformReady: boolean;
      recipeId?: string | null;
    }>;
    recipe?: {
      id: string;
      workflow: string;
      providerId: string;
      modelId: string;
      sourceReferenceCount: number;
      outputAssetCount: number;
      replayable: boolean;
    } | null;
    provenanceLinks: Array<{
      id: string;
      label: string;
      sidecarPath: string;
      inspectable: boolean;
    }>;
    collectionIds: string[];
    versionCount: number;
    sourcePickerTargets: string[];
    notes: string[];
  };
  collections: Array<{
    collection: {
      id: string;
      name: string;
      assetIds: string[];
    };
    collectionType: string;
    description: string;
    itemCount: number;
    dragIntoStudios: string[];
  }>;
  lifecycleActions: Array<{
    id: string;
    label: string;
    appliesTo: LibraryItemType[];
    preservesProvenance: boolean;
    destructive: boolean;
  }>;
  dragTargets: Array<{
    id: string;
    label: string;
    acceptedTypes: LibraryItemType[];
    createsLinkedCopy: boolean;
  }>;
  validationChecks: Array<{
    id: string;
    passed: boolean;
    summary: string;
  }>;
};

export type WorkspaceOverview = {
  schemaVersion: number;
  workspace: {
    id: string;
    globalLibraryId: string;
    recentProjectIds: string[];
  };
  activeProject: WorkspaceProjectCard;
  recentProjects: WorkspaceProjectCard[];
  globalLibrary: {
    id: string;
    label: string;
    assetCount: number;
    reusableVoiceCount: number;
    reusablePresetCount: number;
    reusableCollectionCount: number;
    storageRoot: string;
    canBrowse: boolean;
  };
  scopeControls: Array<{
    id: string;
    label: string;
    scope: LibraryScope;
    active: boolean;
    itemCount: number;
    emptyState: string;
  }>;
  projectAssets: WorkspaceAssetReference[];
  globalAssets: WorkspaceAssetReference[];
  sourcePicker: {
    id: string;
    activeProjectId: string;
    defaultScope: LibraryScope;
    allowsGlobalSources: boolean;
    importModes: Array<"link" | "copy" | "promote-project-asset">;
    targetSurfaces: string[];
    provenanceRequirements: string[];
  };
  transferActions: Array<{
    id: string;
    label: string;
    mode: "link" | "copy" | "promote-project-asset";
    sourceItemId: string;
    targetProjectId?: string | null;
    targetScope: LibraryScope;
    preservesProvenance: boolean;
    createsNewAssetId: boolean;
    createsReuseEvent: boolean;
    enabled: boolean;
    summary: string;
  }>;
  compositionLinks: Array<{
    id: string;
    compositionId: string;
    projectId: string;
    assetId: string;
    versionId: string;
    sourceScope: LibraryScope;
    projectUsage: string;
    preservesOriginalAssetId: boolean;
    provenanceSidecarPath: string;
    warning?: string | null;
  }>;
  parityNotes: Array<{
    id: string;
    area: string;
    convention: string;
    soundworksApplication: string;
  }>;
  validationChecks: Array<{
    id: string;
    passed: boolean;
    summary: string;
  }>;
};

export type WorkspaceProjectCard = {
  project: {
    id: string;
    name: string;
    storageRoot: string;
    assetIds: string[];
    compositionIds: string[];
    recipeIds: string[];
    jobIds: string[];
  };
  openedAt: string;
  assetCount: number;
  compositionCount: number;
  localRecipeCount: number;
  linkedGlobalAssetCount: number;
  canOpen: boolean;
  canCreateFromTemplate: boolean;
  status: "active" | "recent" | "template";
};

export type WorkspaceAssetReference = {
  itemId: string;
  name: string;
  itemType: string;
  scope: LibraryScope;
  ownership: string;
  projectId?: string | null;
  sourceWorkflow?: string | null;
  provenanceId: string;
  sourcePickerEligible: boolean;
  timelinePlaceable: boolean;
  compositionUsageCount: number;
};

export type LibraryItemCard = {
  id: string;
  name: string;
  itemType: LibraryItemType;
  itemTypeLabel: string;
  scope: LibraryScope;
  ownership: string;
  projectId?: string | null;
  createdAt: string;
  sourceWorkflow?: string | null;
  tags: string[];
  generatedTags: string[];
  collectionIds: string[];
  durationMs?: number | null;
  bpm?: number | null;
  musicalKey?: string | null;
  language?: string | null;
  voiceProfileId?: string | null;
  providerId?: string | null;
  modelId?: string | null;
  licenseStatus: string;
  commercialUse: string;
  favorite: boolean;
  rejected: boolean;
  archived: boolean;
  waveformThumbnail?: {
    previewPath: string;
    peakCount: number;
    durationMs: number;
    ready: boolean;
  } | null;
  quickAudition: {
    previewable: boolean;
    playableRangeMs?: [number, number] | null;
    shortcut: string;
  };
  timelinePlaceable: boolean;
  sourcePickerEligible: boolean;
  compositionUsageCount: number;
  recipe?: {
    id: string;
    workflow: string;
    providerId: string;
    modelId: string;
    sourceReferenceCount: number;
    outputAssetCount: number;
    replayable: boolean;
  } | null;
  badges: string[];
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

export type ValidationStatus =
  | "passed"
  | "pending"
  | "manual-required"
  | "failed";

export type ValidationCategory =
  | "job-contracts"
  | "recipe-persistence"
  | "metadata-extraction"
  | "provider-manifest"
  | "asset-lifecycle"
  | "export-sidecars"
  | "safety-gates"
  | "audio-quality"
  | "runtime-evidence"
  | "stress"
  | "documentation";

export type MvpValidationSummary = {
  schemaVersion: number;
  readyForMvp: boolean;
  blockingItemCount: number;
  runtimeEvidenceCount: number;
  satisfiedRuntimeEvidenceCount: number;
  fixtureOnlyEvidenceCount: number;
  demoWorkflowCount: number;
  regressionFixtureCount: number;
  automatedCheckCount: number;
  manualScorecardCount: number;
  stressCaseCount: number;
  knownLimitationCount: number;
  requirementCount: number;
  workflowCount: number;
};

export type MvpValidationOverview = {
  schemaVersion: number;
  releaseGate: {
    readyForMvp: boolean;
    requiredWorkflowCount: number;
    coveredWorkflowCount: number;
    requiredRuntimeEvidenceCount: number;
    satisfiedRuntimeEvidenceCount: number;
    fixtureOnlyEvidenceCount: number;
    requiredAutomatedCheckCount: number;
    passedAutomatedCheckCount: number;
    requiredManualScorecardCount: number;
    passedManualScorecardCount: number;
    requiredStressCaseCount: number;
    passedStressCaseCount: number;
    blockingItems: string[];
  };
  runtimeEvidence: Array<{
    id: string;
    workflow: CapabilityWorkflow;
    requiredForMvp: boolean;
    status: ValidationStatus;
    fixtureOnly: boolean;
    requirement: string;
    evidence: string;
    blocker: string;
  }>;
  demoWorkflows: Array<{
    id: string;
    workflow: CapabilityWorkflow;
    title: string;
    goal: string;
    requiredArtifacts: string[];
    acceptance: string[];
  }>;
  regressionFixtures: Array<{
    id: string;
    workflow: CapabilityWorkflow;
    name: string;
    inputContract: string;
    expectedOutputs: string[];
    automatedCheckIds: string[];
  }>;
  automatedChecks: Array<{
    id: string;
    category: ValidationCategory;
    status: ValidationStatus;
    requiredForMvp: boolean;
    summary: string;
    evidence: string;
  }>;
  manualScorecards: Array<{
    id: string;
    workflow: CapabilityWorkflow;
    status: ValidationStatus;
    requiredForMvp: boolean;
    scoringAxes: string[];
    passThreshold: string;
    reviewerNotes: string;
  }>;
  stressCases: Array<{
    id: string;
    title: string;
    workflow: CapabilityWorkflow;
    status: ValidationStatus;
    requiredForMvp: boolean;
    scenario: string;
    expectedBehavior: string;
  }>;
  knownLimitations: Array<{
    id: string;
    area: string;
    summary: string;
    mitigation: string;
    blocksMvp: boolean;
  }>;
  requirementCoverage: Array<{
    requirementId: string;
    epicRequirement: string;
    demoWorkflowIds: string[];
    fixtureIds: string[];
    checkIds: string[];
    status: ValidationStatus;
  }>;
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

export type ReviewWorkspaceSummary = {
  schemaVersion: number;
  assetCount: number;
  previewableAssetCount: number;
  editActionCount: number;
  comparisonCount: number;
  canSaveEdit: boolean;
  activeAssetId: string;
  editedVersionId: string;
  sourceAssetKinds: string[];
};

export type RightsSafetySummary = {
  schemaVersion: number;
  consentCheckCount: number;
  blockedConsentCount: number;
  modelDecisionCount: number;
  blockedModelDecisionCount: number;
  policyGateCount: number;
  blockedGateCount: number;
  sidecarCount: number;
  disclosureCount: number;
  canExport: boolean;
  watermarkPolicy: string;
};

export type VideoToAudioSummary = {
  schemaVersion: number;
  sourceDurationMs: number;
  targetRangeCount: number;
  detectedEventCount: number;
  syncPointCount: number;
  providerCount: number;
  scorecardCount: number;
  canSubmit: boolean;
  selectedProviderId: string;
  selectedModelId: string;
  savedAssetKind: string;
  exportTargetCount: number;
};

export type RightsSafetyOverview = {
  schemaVersion: number;
  policy: {
    name: string;
    voiceConsentRequiredFor: string[];
    exportRequires: string[];
    blockedPromptCategories: string[];
    warningPromptCategories: string[];
    watermarkPolicy: string;
    provenanceSidecarRequired: boolean;
  };
  consentChecks: Array<{
    id: string;
    workflow: string;
    voiceProfileId: string;
    consentStatus: VoiceConsentStatus;
    allowedUse: string;
    decision: "allowed" | "warn" | "blocked";
    summary: string;
    storedMetadata: {
      licenseStatus: string;
      commercialUse: string;
      voiceConsent: VoiceConsentStatus;
      aiDisclosureRequired: boolean;
      watermark: string;
      referenceMediaOwnership?: string | null;
    };
  }>;
  modelUseDecisions: Array<{
    candidateId: string;
    name: string;
    requestedWorkflow: string;
    exportCandidate: boolean;
    license: string;
    commercialUse: string;
    productEligibility: string;
    runtimePath: string;
    requiresPythonRuntime: boolean;
    decision: "allowed" | "warn" | "blocked";
    reasons: string[];
  }>;
  contentPolicyGates: Array<{
    id: string;
    category: string;
    status: "passed" | "warning" | "blocked";
    appliesTo: string[];
    summary: string;
    enforcement: string;
  }>;
  exportSidecars: Array<{
    id: string;
    assetId: string;
    assetKind: string;
    target: string;
    path: string;
    includesRecipe: boolean;
    includesModel: boolean;
    includesSourceMedia: boolean;
    includesRights: boolean;
    includesEditChain: boolean;
    disclosureRequired: boolean;
    watermark: string;
    rights: {
      licenseStatus: string;
      commercialUse: string;
      voiceConsent: VoiceConsentStatus;
      aiDisclosureRequired: boolean;
      watermark: string;
      referenceMediaOwnership?: string | null;
    };
    provenance: {
      id: string;
      subjectId: string;
      events: Array<{
        eventType: string;
        actor: string;
        summary: string;
        metadata: Record<string, unknown>;
      }>;
    };
  }>;
  disclosureChecks: Array<{
    id: string;
    assetId: string;
    required: boolean;
    reason: string;
    exportTargets: string[];
  }>;
  validationChecks: Array<{
    id: string;
    status: "passed" | "warning" | "failed";
    summary: string;
  }>;
};

export type ReviewAsset = {
  id: string;
  scope: unknown;
  kind: string;
  name: string;
  tags: string[];
  collectionIds: string[];
  currentVersionId: string;
  versionIds: string[];
  rights: unknown;
  provenanceIds: string[];
};

export type ReviewAssetVersion = {
  id: string;
  assetId: string;
  versionIndex: number;
  file: {
    storagePath: string;
    format: string;
    codec?: string | null;
    byteSize?: number | null;
    contentHash?: string | null;
  };
  technical: {
    sampleRateHz: number;
    bitDepth?: number | null;
    channels: number;
    durationMs: number;
    loudnessLufs?: number | null;
    truePeakDbfs?: number | null;
    hasClipping: boolean;
    bpm?: number | null;
    musicalKey?: string | null;
    loopPoints?: { startSample: number; endSample: number } | null;
  };
  createdBy: unknown;
  waveformPreviewCache?: string | null;
  spectrogramPreviewCache?: string | null;
};

export type ReviewAssetPreview = {
  asset: ReviewAsset;
  versions: ReviewAssetVersion[];
  sourceWorkflow: string;
  canPreview: boolean;
  previewStatus: "ready" | "pending" | "missing";
};

export type ReviewWorkspaceOverview = {
  schemaVersion: number;
  assets: ReviewAssetPreview[];
  selectedAsset: ReviewAssetPreview;
  transport: {
    playing: boolean;
    positionMs: number;
    durationMs: number;
    zoomPixelsPerSecond: number;
    selection?: { startMs: number; endMs: number } | null;
    loopRegion?: { startMs: number; endMs: number } | null;
    keyboardShortcuts: Array<{
      id: string;
      keys: string;
      action: string;
    }>;
    accessibleLabels: string[];
  };
  waveform: {
    assetVersionId: string;
    channelCount: number;
    sampleRateHz: number;
    durationMs: number;
    cachePath: string;
    status: "ready" | "pending" | "missing";
    peaks: Array<{ min: number; max: number }>;
  };
  spectrogram: {
    assetVersionId: string;
    cachePath: string;
    status: "ready" | "pending" | "missing";
    frequencyBins: number;
    timeSlices: number;
  };
  editActions: Array<{
    id: string;
    kind: string;
    label: string;
    operation?: string | null;
    destructive: boolean;
    nonDestructiveSave: boolean;
    enabled: boolean;
    parameters: Record<string, unknown>;
  }>;
  editSubmission: {
    id: string;
    canSave: boolean;
    recipe: unknown;
    job: {
      id: string;
      recipeId: string;
      kind: string;
      status: string;
      progress?: unknown | null;
      outputVersionIds: string[];
      error?: string | null;
    };
    sourceAsset: ReviewAsset;
    sourceVersion: ReviewAssetVersion;
    savedAsset: ReviewAsset;
    savedVersion: ReviewAssetVersion;
    warnings: string[];
    blockingReasons: string[];
  };
  versionComparison: {
    id: string;
    mode: string;
    left: {
      label: string;
      assetId: string;
      versionId: string;
      recipeId: string;
      durationMs: number;
      loudnessLufs?: number | null;
      truePeakDbfs?: number | null;
    };
    right: {
      label: string;
      assetId: string;
      versionId: string;
      recipeId: string;
      durationMs: number;
      loudnessLufs?: number | null;
      truePeakDbfs?: number | null;
    };
    metrics: {
      durationDeltaMs: number;
      loudnessDeltaLufs?: number | null;
      truePeakDeltaDb?: number | null;
      waveformDifferenceScore: number;
    };
    notes: string[];
  };
  provenance: {
    inspectable: boolean;
    originalRecipe: {
      id: string;
      workflow: string;
      providerId: string;
      modelId: string;
      sourceReferenceCount: number;
      outputAssetCount: number;
      replayable: boolean;
    };
    editRecipe: {
      id: string;
      workflow: string;
      providerId: string;
      modelId: string;
      sourceReferenceCount: number;
      outputAssetCount: number;
      replayable: boolean;
    };
    sourceVersionId: string;
    editedVersionId: string;
    provenanceIds: string[];
    sidecarPath: string;
  };
  validationChecks: Array<{
    id: string;
    status: "passed" | "warning" | "failed";
    summary: string;
  }>;
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

export type VideoToAudioOverview = {
  schemaVersion: number;
  projectId: string;
  source: {
    videoReferenceId: string;
    videoAssetId: string;
    filename: string;
    durationMs: number;
    frameRate: string;
    resolution: string;
    hasSourceAudio: boolean;
    imageReferenceIds: string[];
    referenceAudioAssetIds: string[];
    ownershipAttestation: string;
  };
  direction: {
    prompt: string;
    negativePrompt: string;
    syncMode: string;
    requestedOutputs: string[];
    durationMs: number;
    regeneratePolicy: string;
    exportTarget: string;
  };
  targetRanges: Array<{
    id: string;
    label: string;
    range: { startMs: number; endMs: number };
    objectLabel?: string | null;
    region?: {
      x: number;
      y: number;
      width: number;
      height: number;
    } | null;
    requestedAction: string;
  }>;
  detectedEvents: Array<{
    id: string;
    label: string;
    atMs: number;
    confidence: number;
    objectLabel?: string | null;
    requestedSound: string;
  }>;
  providerOptions: Array<{
    providerId: string;
    modelId: string;
    modelVersion?: string | null;
    displayName: string;
    workflow: CapabilityWorkflow;
    runtime: string;
    runnable: boolean;
    installStatus: string;
    outputAssetKinds: string[];
    outputFormat: string;
    sampleRateHz: number;
    channelLayout: string;
    supportsVideo: boolean;
    supportsText: boolean;
    supportsReferenceAudio: boolean;
    supportsRangeRefinement: boolean;
    supportsObjectRegions: boolean;
    commercialUseAllowed: boolean;
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
    supports: string[];
    blockers: string[];
    notes: string;
  }>;
  syncPreview: {
    id: string;
    durationMs: number;
    sampleRateHz: number;
    channelLayout: string;
    waveformPreviewPath: string;
    syncPoints: Array<{
      id: string;
      atMs: number;
      label: string;
      confidence: number;
    }>;
    segments: Array<{
      id: string;
      targetRangeId: string;
      label: string;
      range: { startMs: number; endMs: number };
      assetKind: string;
      syncConfidence: number;
      editable: boolean;
    }>;
    warnings: string[];
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
    synchronizedToVideo: boolean;
  };
  exportPackage: {
    id: string;
    mixdownPath: string;
    sidecarPath: string;
    includesSyncPoints: boolean;
    includesSourceMediaRefs: boolean;
    includesDetectedEvents: boolean;
    includesRights: boolean;
    destinationTargets: string[];
    requiredFields: string[];
  };
  provenance: {
    recipeId: string;
    sourceReferenceIds: string[];
    sidecarPath: string;
    capturedFields: string[];
    roundTripNotes: string[];
  };
  safetyGates: Array<{
    id: string;
    status: "passed" | "warning" | "blocked";
    summary: string;
    enforcement: string;
  }>;
  validationChecks: Array<{
    id: string;
    status: "passed" | "warning" | "blocked";
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
