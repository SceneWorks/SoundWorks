import type {
  AppOverview,
  AssetLibraryOverview,
  CompositionEditorOverview,
  ExportWorkflowOverview,
  ModelManagerOverview,
  MvpValidationOverview,
  RightsSafetyOverview,
  ReviewWorkspaceOverview,
  RuntimeOverview,
  SamplesStudioOverview,
  SfxStudioOverview,
  SongStudioOverview,
  TtsStudioOverview,
  VideoToAudioOverview,
  VoiceLabOverview,
  WorkspaceOverview,
} from "./types";

const fallbackModelManagerSummary = {
  candidateCount: 28,
  verifiedInstalledCount: 1,
  installableCount: 6,
  blockedCount: 19,
  missingCacheCount: 1,
  failedOperationCount: 1,
};

const modelManagerCandidates = [
  ["stable-audio-3", "Stable Audio 3", "Stability AI", "song"],
  ["ace-step-1-5", "ACE-Step 1.5", "ACE-Step", "song"],
  ["levo-2", "LeVo 2 / SongGeneration 2", "Tencent AI Lab", "song"],
  ["yue", "YuE", "M-A-P", "song"],
  ["diffrhythm-2", "DiffRhythm 2", "ASLP Lab / Xiaomi Research", "song"],
  ["khala", "Khala", "Khala Music AI", "song"],
  ["heartmula", "HeartMuLa", "HeartMuLa", "loop"],
  ["muse-song", "Muse", "Muse authors", "song"],
  ["kokoro-82m", "Kokoro 82M", "hexgrad", "tts"],
  ["vibevoice", "VibeVoice", "Microsoft", "tts"],
  ["xtts-v2", "XTTS-v2", "Coqui", "voice-clone"],
  ["chattts", "ChatTTS", "2Noise", "tts"],
  ["fish-speech", "Fish Speech", "Fish Audio", "voice-clone"],
  ["chatterbox", "Chatterbox", "Resemble AI", "voice-clone"],
  ["chatterbox-turbo", "Chatterbox Turbo", "Resemble AI", "tts"],
  ["gpt-sovits", "GPT-SoVITS", "RVC-Boss", "voice-clone"],
  ["f5-tts", "F5-TTS", "SWivid", "voice-clone"],
  ["cosyvoice-2", "CosyVoice 2", "FunAudioLLM / Alibaba", "tts"],
  ["openvoice-v2", "OpenVoice V2", "MyShell AI", "voice-conversion"],
  ["rvc", "RVC", "RVC Project", "voice-conversion"],
  ["stable-audio-open-1", "Stable Audio Open 1.0", "Stability AI", "sfx"],
  ["audiocraft-audiogen", "AudioCraft / AudioGen", "Meta", "sfx"],
  ["audioldm", "AudioLDM", "AudioLDM authors", "sfx"],
  ["audioldm-2", "AudioLDM 2", "AudioLDM authors", "sfx"],
  ["audiox", "AudioX", "AudioX authors", "video-to-audio"],
  ["mmaudio", "MMAudio", "MMAudio authors", "video-to-audio"],
  ["thinksound", "ThinkSound", "FunAudioLLM", "video-to-audio"],
  ["moss-soundeffect", "MOSS-SoundEffect", "OpenMOSS", "sfx"],
] as const;

export const fallbackModelManager: ModelManagerOverview = {
  schemaVersion: 1,
  cacheRoot: "~/Library/Application Support/SoundWorks/models",
  summary: fallbackModelManagerSummary,
  laneReadiness: [
    {
      lane: "tts",
      recommendedCandidateId: "kokoro-82m",
      state: "verified",
      summary: "Kokoro 82M has verified cache evidence.",
      blocker: null,
    },
    {
      lane: "voice-clone",
      recommendedCandidateId: "chatterbox",
      state: "missing-cache",
      summary:
        "Chatterbox needs a product-safe provider package before voice clone enablement.",
      blocker:
        "Need no-Python product path and watermark/provenance validation",
    },
    {
      lane: "voice-conversion",
      recommendedCandidateId: "rvc",
      state: "missing-cache",
      summary: "RVC remains a consent-gated external-executable candidate.",
      blocker: "README.md",
    },
    {
      lane: "sfx",
      recommendedCandidateId: "moss-soundeffect",
      state: "missing-cache",
      summary:
        "MOSS-SoundEffect is selected for SFX, but cache verification is incomplete.",
      blocker: "model.safetensors",
    },
    {
      lane: "song",
      recommendedCandidateId: "ace-step-1-5",
      state: "missing-cache",
      summary:
        "ACE-Step 1.5 needs packaged runtime proof before song generation enablement.",
      blocker: "checkpoints",
    },
    {
      lane: "video-to-audio",
      recommendedCandidateId: "mmaudio",
      state: "blocked",
      summary:
        "MMAudio remains blocked for product use until license and runtime path are resolved.",
      blocker: "Candidate currently requires a Python runtime.",
    },
  ],
  candidates: modelManagerCandidates.map(
    ([candidateId, name, provider, lane]) => {
      const installState =
        candidateId === "kokoro-82m"
          ? "installed"
          : candidateId === "moss-soundeffect"
            ? "missing-cache"
            : candidateId === "xtts-v2" ||
                candidateId === "chattts" ||
                candidateId === "f5-tts" ||
                candidateId === "audioldm"
              ? "blocked"
              : candidateId === "mmaudio" ||
                  candidateId === "audiox" ||
                  candidateId === "thinksound" ||
                  candidateId === "levo-2" ||
                  candidateId === "yue" ||
                  candidateId === "diffrhythm-2" ||
                  candidateId === "khala" ||
                  candidateId === "heartmula" ||
                  candidateId === "muse-song"
                ? "research-only"
                : "needs-runtime-port";
      const sourceUrl =
        candidateId === "kokoro-82m"
          ? "https://huggingface.co/onnx-community/Kokoro-82M-v1.0-ONNX"
          : candidateId === "moss-soundeffect"
            ? "https://huggingface.co/mlx-community/MOSS-SoundEffect-v2.0-4bit"
            : "https://huggingface.co";
      const repositoryId =
        candidateId === "kokoro-82m"
          ? "onnx-community/Kokoro-82M-v1.0-ONNX"
          : candidateId === "moss-soundeffect"
            ? "mlx-community/MOSS-SoundEffect-v2.0-4bit"
            : null;
      const verifiedKokoro = candidateId === "kokoro-82m";

      return {
        candidateId,
        name,
        provider,
        lanes: [lane],
        sourceLabel: "Primary source",
        sourceUrl,
        licenseLabel:
          installState === "blocked"
            ? "License/runtime blocker"
            : "Source-backed license review",
        evaluationStatus:
          installState === "blocked" ? "blocked" : "promising-spike",
        productEligibility:
          installState === "missing-cache"
            ? "product-candidate"
            : installState === "installed"
              ? "product-candidate"
              : installState === "research-only"
                ? "research-only"
                : installState === "blocked"
                  ? "blocked"
                  : "needs-runtime-port",
        evidenceLevel:
          installState === "missing-cache" || installState === "installed"
            ? "install-documented"
            : "source-metadata",
        runtimePath:
          installState === "research-only"
            ? "python-poc-only"
            : "external-executable",
        requiresPythonRuntime: installState === "research-only",
        installState,
        blockers:
          installState === "missing-cache" || installState === "installed"
            ? []
            : ["Product-safe runtime and cache evidence are not verified."],
        downloadPlan: {
          mechanism:
            installState === "blocked"
              ? "blocked"
              : installState === "research-only"
                ? "research-poc"
                : "hugging-face-snapshot",
          sourceUrl,
          repositoryId,
          cacheSubdir: candidateId,
          expectedFiles:
            candidateId === "kokoro-82m"
              ? [
                  { path: "config.json", required: true },
                  { path: "onnx/model.onnx", required: true },
                  { path: "voices/af_heart.bin", required: true },
                ]
              : candidateId === "moss-soundeffect"
                ? [
                    { path: "config.json", required: true },
                    { path: "model.safetensors", required: true },
                    { path: "tokenizer.json", required: true },
                  ]
                : [{ path: "README.md", required: true }],
          expectedSizeMb:
            candidateId === "moss-soundeffect"
              ? 8000
              : candidateId === "kokoro-82m"
                ? 400
                : null,
          requiresLicenseAcceptance:
            installState !== "missing-cache" && installState !== "installed",
          supportsAutomatedDownload:
            installState === "missing-cache" || installState === "installed",
          commandHint: `Download provider files into ${candidateId}, then revalidate.`,
          notes: [
            "No candidate is installed until expected files exist on disk.",
          ],
        },
        cache: {
          cachePath: `~/Library/Application Support/SoundWorks/models/${candidateId}`,
          verified: verifiedKokoro,
          expectedFileCount:
            candidateId === "kokoro-82m" || candidateId === "moss-soundeffect"
              ? 3
              : 1,
          presentFileCount: verifiedKokoro ? 3 : 0,
          missingRequiredFiles: verifiedKokoro
            ? []
            : candidateId === "moss-soundeffect"
              ? ["config.json", "model.safetensors", "tokenizer.json"]
              : ["README.md"],
          diskUsageMb: verifiedKokoro ? 400 : null,
          evidence: verifiedKokoro
            ? "verified 3 expected file(s) under Hugging Face snapshot cache"
            : `missing cache directory ~/Library/Application Support/SoundWorks/models/${candidateId}`,
        },
        actions:
          installState === "missing-cache"
            ? ["revalidate", "open-source", "install", "repair-cache"]
            : ["revalidate", "open-source"],
      };
    },
  ),
  operations: [
    {
      id: "revalidate-kokoro-82m",
      candidateId: "kokoro-82m",
      action: "revalidate",
      status: "succeeded",
      progressPercent: 100,
      summary: "Kokoro 82M cache evidence is verified.",
      recovery: null,
      logTail: [
        "verified 3 expected file(s) under Hugging Face snapshot cache",
      ],
    },
    {
      id: "install-moss-soundeffect",
      candidateId: "moss-soundeffect",
      action: "install",
      status: "failed",
      progressPercent: 100,
      summary: "MOSS-SoundEffect install failed cache verification.",
      recovery:
        "The downloader did not leave the required MLX model files in a verifiable cache path; retry download or keep the SFX lane blocked.",
      logTail: [
        "Download provider files into moss-soundeffect, then revalidate.",
        "missing cache directory ~/Library/Application Support/SoundWorks/models/moss-soundeffect",
      ],
    },
  ],
  validationChecks: [
    {
      id: "model-manager.candidate-coverage",
      passed: true,
      summary: "Model manager covers 28 epic candidate(s).",
    },
    {
      id: "model-manager.no-metadata-installs",
      passed: true,
      summary:
        "Installed state is derived from verified cache files, not model metadata alone.",
    },
    {
      id: "model-manager.missing-cache-visible",
      passed: true,
      summary: "Missing-cache state is visible for install/revalidate QA.",
    },
    {
      id: "model-manager.failed-download-visible",
      passed: true,
      summary: "Failed download recovery is visible.",
    },
  ],
};

export const fallbackOverview: AppOverview = {
  productName: "SoundWorks",
  architecture: {
    layers: [
      {
        id: "react-ui",
        name: "React UI",
        responsibility:
          "Workflow surfaces, library navigation, waveform review, and composition controls.",
        status: "scaffolded",
      },
      {
        id: "tauri-commands",
        name: "Tauri Commands",
        responsibility:
          "Narrow command bridge between the UI and local Rust services.",
        status: "scaffolded",
      },
      {
        id: "soundworks-core",
        name: "Rust Core",
        responsibility:
          "Shared domain contracts for assets, recipes, jobs, providers, and exports.",
        status: "scaffolded",
      },
      {
        id: "worker-runtime",
        name: "Worker Runtime",
        responsibility:
          "Model execution, installation, device capabilities, progress, and cancellation.",
        status: "scaffolded",
      },
    ],
  },
  studios: [
    {
      id: "tts",
      name: "TTS Studio",
      route: "/studios/tts",
      status: "scaffolded",
    },
    {
      id: "voice-lab",
      name: "Voice Lab",
      route: "/studios/voice-lab",
      status: "scaffolded",
    },
    {
      id: "sfx",
      name: "SFX + Ambience",
      route: "/studios/sfx",
      status: "scaffolded",
    },
    {
      id: "loops",
      name: "Samples + Loops",
      route: "/studios/loops",
      status: "scaffolded",
    },
    {
      id: "songs",
      name: "Song Studio",
      route: "/studios/songs",
      status: "scaffolded",
    },
    {
      id: "review",
      name: "Waveform Review",
      route: "/review",
      status: "scaffolded",
    },
    {
      id: "rights-safety",
      name: "Rights + Safety",
      route: "/rights",
      status: "scaffolded",
    },
    {
      id: "composition-editor",
      name: "Multitrack Editor",
      route: "/composition",
      status: "scaffolded",
    },
    {
      id: "video-to-audio",
      name: "Video to Audio",
      route: "/studios/video-to-audio",
      status: "scaffolded",
    },
  ],
  commands: [
    {
      name: "get_app_overview",
      direction: "ui-to-backend",
      purpose:
        "Load scaffolded architecture and workflow metadata from the Rust backend.",
    },
    {
      name: "get_provider_catalog",
      direction: "ui-to-backend",
      purpose:
        "Load provider/model manifests, capability defaults, and matching inputs.",
    },
    {
      name: "get_workspace_overview",
      direction: "ui-to-backend",
      purpose:
        "Load active project workspace, global library, source picker, reuse actions, and SceneWorks-style scope conventions.",
    },
    {
      name: "get_asset_library_overview",
      direction: "ui-to-backend",
      purpose:
        "Load searchable asset library facets, project/global scope, lifecycle state, collections, previews, and provenance detail.",
    },
    {
      name: "get_export_workflow_overview",
      direction: "ui-to-backend",
      purpose:
        "Load export presets, formats, stem bundles, DAW handoff, SceneWorks handoff, and metadata sidecar readiness.",
    },
    {
      name: "get_composition_editor_overview",
      direction: "ui-to-backend",
      purpose:
        "Load multitrack timeline state, asset placement readiness, clip edit tools, mixer state, render plan, and editor component decision evidence.",
    },
    {
      name: "get_runtime_overview",
      direction: "ui-to-backend",
      purpose:
        "Report worker runtime policy, verified model state, persisted jobs, progress, cancellation, and artifact records.",
    },
    {
      name: "enqueue_runtime_job",
      direction: "ui-to-backend",
      purpose:
        "Create a durable runtime job with recipe, model metadata, event log, output manifest, and adapter result.",
    },
    {
      name: "cancel_runtime_job",
      direction: "ui-to-backend",
      purpose:
        "Persist cancellation against a queued or running local runtime job.",
    },
    {
      name: "retry_runtime_job",
      direction: "ui-to-backend",
      purpose: "Create a new auditable runtime job from a failed job recipe.",
    },
    {
      name: "get_runtime_job_artifacts",
      direction: "ui-to-backend",
      purpose:
        "Return output manifests, smoke audio artifacts, logs, and actionable error reports for a runtime job.",
    },
    {
      name: "get_model_evaluation_catalog",
      direction: "ui-to-backend",
      purpose:
        "Load source-backed model scorecards, fixtures, recommendation status, and product eligibility gates.",
    },
    {
      name: "get_mvp_validation_overview",
      direction: "ui-to-backend",
      purpose:
        "Load MVP validation matrix, demo workflows, fixtures, scorecards, stress cases, and release gate.",
    },
    {
      name: "get_tts_studio_overview",
      direction: "ui-to-backend",
      purpose:
        "Load TTS script segmentation, voice consent gates, provider limits, submission preview, and saved voice-clip output.",
    },
    {
      name: "get_voice_lab_overview",
      direction: "ui-to-backend",
      purpose:
        "Load voice profile consent state, clone/fine-tune/conversion modes, provider scorecards, safety gates, and saved conversion output.",
    },
    {
      name: "get_sfx_studio_overview",
      direction: "ui-to-backend",
      purpose:
        "Load SFX and ambience prompts, capability-driven controls, variant previews, provider scorecards, loop checks, post-processing, and saved outputs.",
    },
    {
      name: "get_samples_studio_overview",
      direction: "ui-to-backend",
      purpose:
        "Load instrument sample and loop controls, provider scorecards, sample-pack variants, QA checks, recipes, and saved outputs.",
    },
    {
      name: "get_song_studio_overview",
      direction: "ui-to-backend",
      purpose:
        "Load complete-song lyrics, structure, style controls, provider scorecards, variants, recipes, stems, export targets, and saved outputs.",
    },
    {
      name: "get_review_workspace_overview",
      direction: "ui-to-backend",
      purpose:
        "Load waveform review transport, preview caches, lightweight edit actions, non-destructive edited versions, comparison state, and recipe provenance.",
    },
    {
      name: "get_rights_safety_overview",
      direction: "ui-to-backend",
      purpose:
        "Load rights, consent, model-license, disclosure, watermark, and export provenance policy gates.",
    },
    {
      name: "get_video_to_audio_overview",
      direction: "ui-to-backend",
      purpose:
        "Load multimodal video/image/audio-conditioned Foley workflow state, provider readiness, sync preview, provenance, safety gates, and export package metadata.",
    },
  ],
  providerCatalog: {
    schemaVersion: 1,
    providerCount: 1,
    modelCount: 3,
    capabilityCount: 12,
    workflows: [
      {
        workflow: "tts",
        defaultProviderId: "soundworks-reference",
        defaultModelId: "reference-speech-suite",
      },
      {
        workflow: "voice-clone",
        defaultProviderId: "soundworks-reference",
        defaultModelId: "reference-speech-suite",
      },
      {
        workflow: "voice-conversion",
        defaultProviderId: "soundworks-reference",
        defaultModelId: "reference-speech-suite",
      },
      {
        workflow: "sfx",
        defaultProviderId: "soundworks-reference",
        defaultModelId: "reference-generation-suite",
      },
      {
        workflow: "ambience",
        defaultProviderId: "soundworks-reference",
        defaultModelId: "reference-generation-suite",
      },
      {
        workflow: "instrument-sample",
        defaultProviderId: "soundworks-reference",
        defaultModelId: "reference-generation-suite",
      },
      {
        workflow: "loop",
        defaultProviderId: "soundworks-reference",
        defaultModelId: "reference-generation-suite",
      },
      {
        workflow: "song",
        defaultProviderId: "soundworks-reference",
        defaultModelId: "reference-generation-suite",
      },
      {
        workflow: "stem-separation",
        defaultProviderId: "soundworks-reference",
        defaultModelId: "reference-utility-suite",
      },
      {
        workflow: "video-to-audio",
        defaultProviderId: "soundworks-reference",
        defaultModelId: "reference-generation-suite",
      },
      {
        workflow: "edit",
        defaultProviderId: "soundworks-reference",
        defaultModelId: "reference-utility-suite",
      },
      {
        workflow: "composition-render",
        defaultProviderId: "soundworks-reference",
        defaultModelId: "reference-utility-suite",
      },
    ],
  },
  workspace: {
    schemaVersion: 1,
    projectCount: 2,
    projectAssetCount: 10,
    globalAssetCount: 3,
    linkedGlobalAssetCount: 1,
    transferActionCount: 3,
    sourcePickerTargetCount: 5,
    parityNoteCount: 3,
    activeProjectId: "project-demo",
    globalLibraryId: "global-library",
    canCreateProject: true,
    canOpenProject: true,
  },
  assetLibrary: {
    schemaVersion: 1,
    itemCount: 13,
    previewableItemCount: 10,
    collectionCount: 3,
    scopeCount: 2,
    filterCount: 15,
    supportedTypeCount: 13,
    favoriteCount: 2,
    rejectedCount: 0,
    archivedCount: 0,
    selectedItemId: "asset-loop-001",
    selectedItemType: "loop",
  },
  exportWorkflow: {
    schemaVersion: 1,
    presetCount: 7,
    targetCount: 4,
    sidecarCount: 2,
    readyTargetCount: 4,
    selectedPresetId: "preset-sceneworks-video-track",
    selectedSourceKind: "composition",
    selectedFormatCount: 2,
    canExportSelected: true,
    writesDawBundle: true,
    writesSceneWorksPackage: true,
  },
  compositionEditor: {
    schemaVersion: 1,
    trackCount: 4,
    clipCount: 7,
    assetBinCount: 5,
    enabledToolCount: 9,
    markerCount: 1,
    sectionCount: 1,
    selectedClipId: "clip-voice-intro",
    canRenderMixdown: true,
    editableAssetKinds: [
      "ambience",
      "loop",
      "music-clip",
      "sfx",
      "song",
      "stem",
      "voice-clip",
    ],
    recommendedComponentId: "waveform-playlist",
    componentCandidateCount: 4,
  },
  mvpValidation: {
    schemaVersion: 1,
    readyForMvp: false,
    blockingItemCount: 6,
    runtimeEvidenceCount: 5,
    satisfiedRuntimeEvidenceCount: 0,
    fixtureOnlyEvidenceCount: 5,
    demoWorkflowCount: 12,
    regressionFixtureCount: 12,
    automatedCheckCount: 9,
    manualScorecardCount: 9,
    stressCaseCount: 8,
    knownLimitationCount: 4,
    requirementCount: 8,
    workflowCount: 12,
  },
  modelEvaluation: {
    schemaVersion: 1,
    candidateCount: 28,
    sourceCount: 67,
    fixtureCount: 9,
    laneCount: 10,
    statusCounts: {
      "promising-spike": 24,
      blocked: 4,
    },
    productEligibilityCounts: {
      "product-candidate": 2,
      "needs-runtime-port": 7,
      "research-only": 15,
      blocked: 4,
    },
    recommendedCandidateIds: [
      "kokoro-82m",
      "chatterbox",
      "rvc",
      "moss-soundeffect",
      "ace-step-1-5",
      "mmaudio",
    ],
  },
  modelManager: fallbackModelManagerSummary,
  ttsStudio: {
    schemaVersion: 1,
    segmentCount: 3,
    speakerCount: 2,
    providerCount: 1,
    canSubmit: false,
    selectedProviderId: "soundworks-reference",
    selectedModelId: "reference-speech-suite",
    savedAssetKind: "voice-clip",
  },
  voiceLab: {
    schemaVersion: 1,
    modeCount: 3,
    profileCount: 2,
    providerCount: 8,
    safetyGateCount: 4,
    canSubmitConversion: true,
    selectedConversionCandidateId: "rvc",
    savedAssetKind: "voice-clip",
  },
  sfxStudio: {
    schemaVersion: 1,
    variantCount: 3,
    savedOutputCount: 2,
    providerCount: 2,
    scorecardCount: 9,
    canSubmit: false,
    selectedProviderId: "soundworks-reference",
    selectedModelId: "reference-generation-suite",
    savedAssetKinds: ["sfx", "ambience"],
  },
  samplesStudio: {
    schemaVersion: 1,
    variantCount: 4,
    savedOutputCount: 3,
    providerCount: 2,
    scorecardCount: 5,
    canSubmit: false,
    selectedProviderId: "soundworks-reference",
    selectedModelId: "reference-generation-suite",
    packCollectionId: "collection-neon-bass-pack",
    savedAssetKinds: ["instrument-sample", "loop"],
  },
  songStudio: {
    schemaVersion: 1,
    sectionCount: 4,
    variantCount: 2,
    savedOutputCount: 2,
    providerCount: 1,
    scorecardCount: 8,
    canSubmit: false,
    selectedProviderId: "soundworks-reference",
    selectedModelId: "reference-generation-suite",
    requestedStems: ["full-mix", "vocals", "drums", "bass", "instruments"],
    savedAssetKinds: ["music-clip", "song"],
  },
  reviewWorkspace: {
    schemaVersion: 1,
    assetCount: 5,
    previewableAssetCount: 5,
    editActionCount: 8,
    comparisonCount: 1,
    canSaveEdit: true,
    activeAssetId: "asset-loop-001",
    editedVersionId: "version-loop-001-b-review-edit",
    sourceAssetKinds: [
      "instrument-sample",
      "loop",
      "sfx",
      "song",
      "voice-clip",
    ],
  },
  rightsSafety: {
    schemaVersion: 1,
    consentCheckCount: 3,
    blockedConsentCount: 2,
    modelDecisionCount: 4,
    blockedModelDecisionCount: 2,
    policyGateCount: 5,
    blockedGateCount: 2,
    sidecarCount: 2,
    disclosureCount: 2,
    canExport: false,
    watermarkPolicy: "advisory-until-provider-support",
  },
  videoToAudio: {
    schemaVersion: 1,
    sourceDurationMs: 14400,
    targetRangeCount: 3,
    detectedEventCount: 5,
    syncPointCount: 5,
    providerCount: 1,
    scorecardCount: 4,
    canSubmit: true,
    selectedProviderId: "soundworks-reference",
    selectedModelId: "reference-generation-suite",
    savedAssetKind: "sfx",
    exportTargetCount: 2,
  },
};

export const fallbackWorkspace: WorkspaceOverview = {
  schemaVersion: 1,
  workspace: {
    id: "workspace-local",
    globalLibraryId: "global-library",
    recentProjectIds: [
      "project-demo",
      "project-podcast-open",
      "project-game-ui-pack",
    ],
  },
  activeProject: {
    project: {
      id: "project-demo",
      name: "Demo SoundWorks Project",
      storageRoot: "soundworks-library/projects/project-demo",
      assetIds: ["asset-voice-001", "asset-loop-001", "asset-song-001"],
      compositionIds: ["composition-demo"],
      recipeIds: ["recipe-tts-001", "recipe-loop-001"],
      jobIds: ["job-asset-voice-001"],
    },
    openedAt: "2026-06-19T01:04:08Z",
    assetCount: 10,
    compositionCount: 1,
    localRecipeCount: 2,
    linkedGlobalAssetCount: 1,
    canOpen: true,
    canCreateFromTemplate: true,
    status: "active",
  },
  recentProjects: [
    {
      project: {
        id: "project-demo",
        name: "Demo SoundWorks Project",
        storageRoot: "soundworks-library/projects/project-demo",
        assetIds: ["asset-voice-001", "asset-loop-001", "asset-song-001"],
        compositionIds: ["composition-demo"],
        recipeIds: ["recipe-tts-001", "recipe-loop-001"],
        jobIds: ["job-asset-voice-001"],
      },
      openedAt: "2026-06-19T01:04:08Z",
      assetCount: 10,
      compositionCount: 1,
      localRecipeCount: 2,
      linkedGlobalAssetCount: 1,
      canOpen: true,
      canCreateFromTemplate: true,
      status: "active",
    },
    {
      project: {
        id: "project-podcast-open",
        name: "Podcast Open Package",
        storageRoot: "soundworks-library/projects/project-podcast-open",
        assetIds: ["asset-voice-host", "asset-sfx-sting"],
        compositionIds: ["composition-podcast-open"],
        recipeIds: ["recipe-voice-host"],
        jobIds: ["job-podcast-open-render"],
      },
      openedAt: "2026-06-18T19:30:00Z",
      assetCount: 2,
      compositionCount: 1,
      localRecipeCount: 1,
      linkedGlobalAssetCount: 1,
      canOpen: true,
      canCreateFromTemplate: false,
      status: "recent",
    },
  ],
  globalLibrary: {
    id: "global-library",
    label: "Global audio library",
    assetCount: 3,
    reusableVoiceCount: 1,
    reusablePresetCount: 1,
    reusableCollectionCount: 1,
    storageRoot: "soundworks-library/global",
    canBrowse: true,
  },
  scopeControls: [
    {
      id: "scope-project-library",
      label: "Project library",
      scope: { kind: "project", projectId: "project-demo" },
      active: true,
      itemCount: 10,
      emptyState: "Create or import audio into this project.",
    },
    {
      id: "scope-global-library",
      label: "Global library",
      scope: { kind: "globalLibrary" },
      active: false,
      itemCount: 3,
      emptyState:
        "Promote reusable voices, loops, references, and presets here.",
    },
  ],
  projectAssets: [
    {
      itemId: "asset-loop-001",
      name: "Dusty trip-hop drums",
      itemType: "loop",
      scope: { kind: "project", projectId: "project-demo" },
      ownership: "project-local",
      projectId: "project-demo",
      sourceWorkflow: "loop",
      provenanceId: "provenance-asset-loop-001",
      sourcePickerEligible: true,
      timelinePlaceable: true,
      compositionUsageCount: 1,
    },
    {
      itemId: "asset-stem-drums-001",
      name: "City Lights drum stem",
      itemType: "stem",
      scope: { kind: "project", projectId: "project-demo" },
      ownership: "copied-from-global",
      projectId: "project-demo",
      sourceWorkflow: null,
      provenanceId: "provenance-asset-stem-drums-001",
      sourcePickerEligible: true,
      timelinePlaceable: true,
      compositionUsageCount: 1,
    },
  ],
  globalAssets: [
    {
      itemId: "asset-reference-neon-bass",
      name: "Neon bass reference",
      itemType: "reference-audio",
      scope: { kind: "globalLibrary" },
      ownership: "global",
      projectId: null,
      sourceWorkflow: null,
      provenanceId: "provenance-asset-reference-neon-bass",
      sourcePickerEligible: true,
      timelinePlaceable: true,
      compositionUsageCount: 2,
    },
    {
      itemId: "voice-profile-narrator",
      name: "Narrator profile",
      itemType: "voice-profile",
      scope: { kind: "globalLibrary" },
      ownership: "linked-global",
      projectId: null,
      sourceWorkflow: "tts",
      provenanceId: "provenance-voice-profile-narrator",
      sourcePickerEligible: true,
      timelinePlaceable: false,
      compositionUsageCount: 0,
    },
    {
      itemId: "preset-noir-narration",
      name: "Noir narration recipe",
      itemType: "prompt-recipe-preset",
      scope: { kind: "globalLibrary" },
      ownership: "global",
      projectId: null,
      sourceWorkflow: "tts",
      provenanceId: "provenance-preset-noir-narration",
      sourcePickerEligible: true,
      timelinePlaceable: false,
      compositionUsageCount: 0,
    },
  ],
  sourcePicker: {
    id: "source-picker-project-plus-global",
    activeProjectId: "project-demo",
    defaultScope: { kind: "project", projectId: "project-demo" },
    allowsGlobalSources: true,
    importModes: ["link", "copy", "promote-project-asset"],
    targetSurfaces: [
      "TTS Studio",
      "Voice Lab",
      "Samples + Loops",
      "Multitrack Editor",
      "Waveform Review",
    ],
    provenanceRequirements: [
      "source scope and original asset ID",
      "source version ID",
      "recipe or import sidecar",
      "link/copy/promote event ID",
    ],
  },
  transferActions: [
    {
      id: "promote-loop-to-global",
      label: "Promote loop to global",
      mode: "promote-project-asset",
      sourceItemId: "asset-loop-001",
      targetProjectId: null,
      targetScope: { kind: "globalLibrary" },
      preservesProvenance: true,
      createsNewAssetId: false,
      createsReuseEvent: true,
      enabled: true,
      summary:
        "Project loop becomes reusable globally while retaining recipe, version, and source project provenance.",
    },
    {
      id: "link-global-reference-into-project",
      label: "Link global reference",
      mode: "link",
      sourceItemId: "asset-reference-neon-bass",
      targetProjectId: "project-demo",
      targetScope: { kind: "project", projectId: "project-demo" },
      preservesProvenance: true,
      createsNewAssetId: false,
      createsReuseEvent: true,
      enabled: true,
      summary:
        "Global reference stays globally owned and is linked into the active project composition.",
    },
    {
      id: "copy-global-voice-preset",
      label: "Copy global preset",
      mode: "copy",
      sourceItemId: "preset-noir-narration",
      targetProjectId: "project-demo",
      targetScope: { kind: "project", projectId: "project-demo" },
      preservesProvenance: true,
      createsNewAssetId: true,
      createsReuseEvent: true,
      enabled: true,
      summary:
        "Reusable prompt preset can be copied into a project for local edits without mutating the global original.",
    },
  ],
  compositionLinks: [
    {
      id: "composition-link-global-bass-reference",
      compositionId: "composition-demo",
      projectId: "project-demo",
      assetId: "asset-reference-neon-bass",
      versionId: "version-reference-neon-bass-a",
      sourceScope: { kind: "globalLibrary" },
      projectUsage: "linked-into-composition",
      preservesOriginalAssetId: true,
      provenanceSidecarPath:
        "soundworks-library/projects/project-demo/compositions/composition-demo/provenance/global-asset-links.json",
      warning: null,
    },
  ],
  parityNotes: [
    {
      id: "project-first-entry",
      area: "Workspace entry",
      convention: "Open into a project workspace with recent projects visible.",
      soundworksApplication:
        "SoundWorks starts with an active audio project and exposes create/open actions.",
    },
    {
      id: "library-scope-language",
      area: "Library scope",
      convention:
        "Keep project assets and reusable global assets visibly separate.",
      soundworksApplication:
        "Project Library and Global Library filters are first-class controls.",
    },
    {
      id: "provenance-detail",
      area: "Asset detail",
      convention:
        "Asset cards preserve recipe, version, source, and export provenance.",
      soundworksApplication:
        "Links, copies, and promotions create reuse events and keep provenance sidecars inspectable.",
    },
  ],
  validationChecks: [
    {
      id: "project-entry-actions",
      passed: true,
      summary:
        "Workspace exposes create and open project affordances from the app boundary.",
    },
    {
      id: "scope-browsing",
      passed: true,
      summary:
        "Project-scoped assets and global reusable assets are browsable as separate scopes.",
    },
    {
      id: "global-use-preserves-provenance",
      passed: true,
      summary:
        "Global assets can be linked or copied into a project while preserving original identity and provenance.",
    },
    {
      id: "promotion-preserves-provenance",
      passed: true,
      summary:
        "Project assets can be promoted to the global library with recipe, version, and source project provenance intact.",
    },
    {
      id: "sceneworks-parity-documented",
      passed: true,
      summary:
        "SceneWorks-style project, library, source picker, and asset-detail conventions are documented without hidden coupling.",
    },
  ],
};

export const fallbackMvpValidation: MvpValidationOverview = {
  schemaVersion: 1,
  releaseGate: {
    readyForMvp: false,
    requiredWorkflowCount: 12,
    coveredWorkflowCount: 12,
    requiredRuntimeEvidenceCount: 5,
    satisfiedRuntimeEvidenceCount: 0,
    fixtureOnlyEvidenceCount: 5,
    requiredAutomatedCheckCount: 10,
    passedAutomatedCheckCount: 4,
    requiredManualScorecardCount: 9,
    passedManualScorecardCount: 0,
    requiredStressCaseCount: 8,
    passedStressCaseCount: 3,
    blockingItems: [
      "Required automated validation checks have not all passed.",
      "Required manual audio-quality scorecards are not all passed.",
      "Required stress cases are not all passed on release hardware.",
      "Runtime evidence is missing; fixture/demo data cannot satisfy generated audio, playback, edit, or export criteria.",
      "Fixture-only evidence is still present in MVP-critical runtime criteria.",
      "Known MVP-blocking limitations remain documented.",
    ],
  },
  runtimeEvidence: [
    {
      id: "evidence-model-cache",
      workflow: "tts",
      requiredForMvp: true,
      status: "pending",
      fixtureOnly: true,
      requirement:
        "Installed model counts must come from verified cache/package files, not static provider manifests.",
      evidence:
        "Reference manifests currently describe packaged models, but no cache/package verification is attached.",
      blocker:
        "SC-6467 must implement model download/cache verification before any model can be counted as installed.",
    },
    {
      id: "evidence-generation-jobs",
      workflow: "tts",
      requiredForMvp: true,
      status: "pending",
      fixtureOnly: true,
      requirement:
        "Generation controls must enqueue persisted runtime jobs with progress, errors, logs, and artifacts.",
      evidence: "Current UI and Rust snapshots are contract/demo data only.",
      blocker: "SC-6468 must replace snapshots with runtime job execution.",
    },
    {
      id: "evidence-generated-audio",
      workflow: "sfx",
      requiredForMvp: true,
      status: "pending",
      fixtureOnly: true,
      requirement:
        "TTS, SFX, samples, loops, and song criteria require generated audio files from selected providers.",
      evidence:
        "Fixture media paths are representative and do not prove generated bytes exist.",
      blocker:
        "SC-6470, SC-6471, and SC-6472 must attach generated audio artifacts or source-backed blockers.",
    },
    {
      id: "evidence-playback-edit",
      workflow: "edit",
      requiredForMvp: true,
      status: "pending",
      fixtureOnly: true,
      requirement:
        "Playback, trim, fade, normalize, loop inspection, and version comparison must run against real audio files.",
      evidence:
        "Review/edit fixtures describe the workflow but do not prove audible playback or file edits.",
      blocker:
        "SC-6473 must validate real playback and non-destructive edited versions.",
    },
    {
      id: "evidence-export-files",
      workflow: "composition-render",
      requiredForMvp: true,
      status: "pending",
      fixtureOnly: true,
      requirement:
        "Export criteria require actual WAV/FLAC/MP3/OGG files plus provenance sidecars on disk.",
      evidence:
        "Export contract data exists, but no runtime file-writing evidence is attached.",
      blocker:
        "SC-6473 must write and validate real export files before this gate can pass.",
    },
  ],
  demoWorkflows: [
    {
      id: "demo-short-narration",
      workflow: "tts",
      title: "Narrate short script",
      goal: "Generate a clean 30-60 second voice clip from a script with pronunciation metadata.",
      requiredArtifacts: [
        "voice clip WAV",
        "recipe JSON",
        "voice consent record",
        "provenance sidecar",
      ],
      acceptance: [
        "speech is intelligible",
        "script segments are preserved",
        "saved output is reusable from the library",
      ],
    },
    {
      id: "demo-podcast-dialogue",
      workflow: "voice-clone",
      title: "Multi-speaker podcast segment",
      goal: "Generate a short two-speaker segment with explicit voice-profile consent gates.",
      requiredArtifacts: [
        "multi-speaker voice clips",
        "speaker map",
        "consent audit",
        "export sidecar",
      ],
      acceptance: [
        "each speaker maps to a consented profile",
        "blocked voices cannot submit",
        "dialogue can export with provenance",
      ],
    },
    {
      id: "demo-voice-conversion",
      workflow: "voice-conversion",
      title: "Consented voice conversion",
      goal: "Convert a source read into an approved target voice profile without treating it as TTS.",
      requiredArtifacts: [
        "source clip",
        "target voice profile",
        "converted voice clip",
        "conversion recipe",
      ],
      acceptance: [
        "source and target IDs are preserved",
        "voice conversion scorecard applies",
        "output remains a voice clip asset",
      ],
    },
    {
      id: "demo-game-ui-sfx",
      workflow: "sfx",
      title: "Generate game UI SFX",
      goal: "Create short selectable impacts, buttons, and confirmations with tags and one-shot metadata.",
      requiredArtifacts: [
        "SFX variants",
        "tags",
        "loudness metrics",
        "saved output assets",
      ],
      acceptance: [
        "variants are auditionable",
        "selected outputs save to library",
        "game export preset accepts the assets",
      ],
    },
    {
      id: "demo-loopable-ambience",
      workflow: "ambience",
      title: "Create loopable ambience",
      goal: "Generate a seamless ambience bed with loop markers and crossfade QA.",
      requiredArtifacts: [
        "ambience asset",
        "loop points",
        "waveform preview",
        "loopability score",
      ],
      acceptance: [
        "loop points are inspectable",
        "loudness stays within target",
        "loop export preserves metadata",
      ],
    },
    {
      id: "demo-instrument-sample-pack",
      workflow: "instrument-sample",
      title: "Generate instrument sample pack",
      goal: "Create one-shots with family, articulation, tags, and pack membership.",
      requiredArtifacts: [
        "sample variants",
        "pack collection",
        "BPM/key metadata",
        "sample-pack export",
      ],
      acceptance: [
        "pack contains selected variants",
        "metadata survives export",
        "duplicates can be identified",
      ],
    },
    {
      id: "demo-loop-pack",
      workflow: "loop",
      title: "Generate musical loop pack",
      goal: "Create tempo-aligned loops with key, bar count, and loop marker metadata.",
      requiredArtifacts: [
        "loop variants",
        "loop points",
        "pack collection",
        "DAW handoff",
      ],
      acceptance: [
        "BPM/key fields are populated",
        "loop points are valid",
        "DAW export preserves loop metadata",
      ],
    },
    {
      id: "demo-complete-song",
      workflow: "song",
      title: "Generate complete song from lyrics and structure",
      goal: "Generate a song draft from lyrics, sections, style tags, and requested stems.",
      requiredArtifacts: [
        "song variants",
        "section map",
        "lyrics sidecar",
        "stem request metadata",
      ],
      acceptance: [
        "structure matches requested sections",
        "lyrics alignment is scored",
        "song master export includes disclosure",
      ],
    },
    {
      id: "demo-stem-separation",
      workflow: "stem-separation",
      title: "Prepare stem bundle",
      goal: "Validate that song or composition outputs can carry stem metadata for DAW handoff.",
      requiredArtifacts: [
        "stem asset records",
        "stem kinds",
        "bundle manifest",
        "sidecar",
      ],
      acceptance: [
        "stem kinds are explicit",
        "bundle references source asset",
        "sidecar links model and recipe",
      ],
    },
    {
      id: "demo-video-foley",
      workflow: "video-to-audio",
      title: "Prototype silent video Foley",
      goal: "Track the video-to-audio demo path so MVP cannot forget multimodal SFX coverage.",
      requiredArtifacts: [
        "source video ID",
        "time range map",
        "generated sync points",
        "provenance sidecar",
      ],
      acceptance: [
        "source media rights are checked",
        "time ranges survive export",
        "provider capability is flagged until sc-6183 ships",
      ],
    },
    {
      id: "demo-edit-trim-normalize",
      workflow: "edit",
      title: "Edit, trim, and normalize",
      goal: "Round-trip a generated asset through non-destructive trim, fade, loop crossfade, and normalize actions.",
      requiredArtifacts: [
        "edited asset version",
        "edit recipe",
        "version comparison",
        "preview cache",
      ],
      acceptance: [
        "original version remains intact",
        "edit chain is inspectable",
        "saved version can be exported",
      ],
    },
    {
      id: "demo-composition-export",
      workflow: "composition-render",
      title: "Export composition with provenance",
      goal: "Render a composition mixdown with optional stems, DAW handoff, and SceneWorks package metadata.",
      requiredArtifacts: [
        "mixdown",
        "optional stems",
        "DAW bundle",
        "SceneWorks handoff",
        "provenance manifest",
      ],
      acceptance: [
        "export preset is selectable",
        "sidecar contains recipe/model/source/rights",
        "SceneWorks constraints are carried as warnings",
      ],
    },
  ],
  regressionFixtures: [
    {
      id: "fixture-short-narration",
      workflow: "tts",
      name: "Short narration script",
      inputContract: "three-segment script with two consented speakers",
      expectedOutputs: ["voice clip asset", "generation job", "recipe summary"],
      automatedCheckIds: [
        "check-job-contracts",
        "check-recipe-persistence",
        "check-safety-gates",
      ],
    },
    {
      id: "fixture-podcast-dialogue",
      workflow: "voice-clone",
      name: "Podcast voice clone gate",
      inputContract:
        "voice-clone request with approved and rejected profile variants",
      expectedOutputs: [
        "allowed submission",
        "blocked submission",
        "consent audit",
      ],
      automatedCheckIds: ["check-job-contracts", "check-safety-gates"],
    },
    {
      id: "fixture-voice-conversion",
      workflow: "voice-conversion",
      name: "Voice conversion source-target pair",
      inputContract: "source audio plus consented target voice profile",
      expectedOutputs: [
        "conversion job",
        "voice clip output",
        "source-target provenance",
      ],
      automatedCheckIds: ["check-job-contracts", "check-recipe-persistence"],
    },
    {
      id: "fixture-game-sfx",
      workflow: "sfx",
      name: "Game UI SFX batch",
      inputContract:
        "short SFX prompt with duration, category, negative prompt, and tags",
      expectedOutputs: ["SFX variants", "loudness metrics", "saved output"],
      automatedCheckIds: [
        "check-job-contracts",
        "check-metadata-extraction",
        "check-asset-lifecycle",
      ],
    },
    {
      id: "fixture-ambience-loop",
      workflow: "ambience",
      name: "Loopable ambience bed",
      inputContract:
        "ambience prompt with loopable control and crossfade target",
      expectedOutputs: ["ambience asset", "loop points", "export warning set"],
      automatedCheckIds: ["check-metadata-extraction", "check-export-sidecars"],
    },
    {
      id: "fixture-sample-pack",
      workflow: "instrument-sample",
      name: "Instrument sample pack",
      inputContract:
        "sample generation request with instrument family and articulation",
      expectedOutputs: [
        "sample variants",
        "pack collection",
        "sample export metadata",
      ],
      automatedCheckIds: ["check-metadata-extraction", "check-asset-lifecycle"],
    },
    {
      id: "fixture-loop-pack",
      workflow: "loop",
      name: "Musical loop pack",
      inputContract: "four-bar loop request with BPM, key, and loop points",
      expectedOutputs: ["loop variants", "BPM/key metadata", "DAW handoff"],
      automatedCheckIds: ["check-metadata-extraction", "check-export-sidecars"],
    },
    {
      id: "fixture-song-structure",
      workflow: "song",
      name: "Complete song structure",
      inputContract: "lyrics, section map, style tags, stems requested",
      expectedOutputs: ["song variants", "section scores", "export targets"],
      automatedCheckIds: [
        "check-job-contracts",
        "check-metadata-extraction",
        "check-export-sidecars",
      ],
    },
    {
      id: "fixture-stem-bundle",
      workflow: "stem-separation",
      name: "Stem bundle handoff",
      inputContract: "song output with requested stem kinds",
      expectedOutputs: ["stem records", "stem bundle sidecar", "DAW package"],
      automatedCheckIds: ["check-export-sidecars", "check-provider-manifests"],
    },
    {
      id: "fixture-video-foley",
      workflow: "video-to-audio",
      name: "Silent video Foley map",
      inputContract:
        "video source ID, target ranges, object notes, and text direction",
      expectedOutputs: [
        "sync points",
        "source media provenance",
        "blocked provider gate",
      ],
      automatedCheckIds: ["check-provider-manifests", "check-safety-gates"],
    },
    {
      id: "fixture-edit-normalize",
      workflow: "edit",
      name: "Non-destructive edit chain",
      inputContract:
        "trim, fade, normalize, and loop crossfade operations on a generated loop",
      expectedOutputs: [
        "edited version",
        "comparison metrics",
        "edit provenance",
      ],
      automatedCheckIds: ["check-recipe-persistence", "check-asset-lifecycle"],
    },
    {
      id: "fixture-composition-export",
      workflow: "composition-render",
      name: "Composition render and handoff",
      inputContract: "two-track composition with voice clip and loop assets",
      expectedOutputs: [
        "mixdown",
        "optional stems",
        "SceneWorks package metadata",
      ],
      automatedCheckIds: ["check-export-sidecars", "check-recipe-persistence"],
    },
  ],
  automatedChecks: [
    {
      id: "check-job-contracts",
      category: "job-contracts",
      status: "pending",
      requiredForMvp: true,
      summary:
        "Generation job contracts serialize status, progress, outputs, cancellation, and actionable errors, but product-runtime execution is not proven.",
      evidence:
        "Current tests cover fixture snapshots only. SC-6468 must attach real queued job records before this can pass.",
    },
    {
      id: "check-recipe-persistence",
      category: "recipe-persistence",
      status: "passed",
      requiredForMvp: true,
      summary:
        "Recipes preserve provider/model, seed, references, post-processing, outputs, and replayability.",
      evidence:
        "Fixture and review tests assert serializable inspectable recipes and edit chains.",
    },
    {
      id: "check-metadata-extraction",
      category: "metadata-extraction",
      status: "pending",
      requiredForMvp: true,
      summary:
        "Audio metadata contracts include duration, sample rate, channels, loudness, true peak, BPM, key, and loop points, but generated-file extraction is not proven.",
      evidence:
        "SFX, samples, songs, review, library, and export reference data expose fields. Real generated audio must replace fixture values.",
    },
    {
      id: "check-provider-manifests",
      category: "provider-manifest",
      status: "passed",
      requiredForMvp: true,
      summary:
        "Provider manifests distinguish workflows, inputs, outputs, limits, hardware, license, and runnable defaults.",
      evidence:
        "Provider catalog covers all capability workflows with capability-driven matching.",
    },
    {
      id: "check-asset-lifecycle",
      category: "asset-lifecycle",
      status: "pending",
      requiredForMvp: true,
      summary:
        "Asset lifecycle contracts cover project/global library, tags, collections, saved outputs, version history, and reuse targets, but persisted runtime assets are not proven.",
      evidence:
        "Asset library fixtures cover scopes, tags, collections, lifecycle actions, and provenance links. SC-6469 must prove persisted assets.",
    },
    {
      id: "check-export-sidecars",
      category: "export-sidecars",
      status: "pending",
      requiredForMvp: true,
      summary:
        "Export sidecar contracts include preset, target, formats, DAW bundle, SceneWorks handoff, and metadata, but file-writing evidence is not attached.",
      evidence:
        "Export workflow sidecars include recipe, model, source media, rights, disclosure, and edit-chain fields. SC-6473 must prove real files.",
    },
    {
      id: "check-safety-gates",
      category: "safety-gates",
      status: "passed",
      requiredForMvp: true,
      summary:
        "Voice consent, commercial model eligibility, content policy, watermark, and disclosure gates are first-class.",
      evidence:
        "Rights and Voice Lab overviews include blocked consent, model-use, and commercial export decisions.",
    },
    {
      id: "check-runtime-evidence",
      category: "runtime-evidence",
      status: "pending",
      requiredForMvp: true,
      summary:
        "Runtime installed counts, queued jobs, generated audio, playback, edits, and export claims require real artifact evidence.",
      evidence:
        "SC-6466 blocks fixture-only completion. Follow-on recovery stories must attach cache, job, media, playback, edit, and export artifacts.",
    },
    {
      id: "check-release-docs",
      category: "documentation",
      status: "passed",
      requiredForMvp: true,
      summary:
        "Validation matrix maps back to epic requirements and states what remains unverified.",
      evidence: "docs/mvp-validation.md is the human-readable release matrix.",
    },
    {
      id: "check-release-run-artifacts",
      category: "stress",
      status: "pending",
      requiredForMvp: true,
      summary:
        "Release run artifacts must capture current Mac and Windows validation evidence before MVP signoff.",
      evidence:
        "This story defines the evidence contract; release artifacts remain pending until real provider runs exist.",
    },
  ],
  manualScorecards: [
    {
      id: "score-tts-quality",
      workflow: "tts",
      status: "manual-required",
      requiredForMvp: true,
      scoringAxes: [
        "intelligibility",
        "pronunciation",
        "prosody",
        "noise floor",
      ],
      passThreshold: "mean 4/5 with no blocker on intelligibility",
      reviewerNotes:
        "Needs real generated audio from the selected first TTS provider.",
    },
    {
      id: "score-dialogue-quality",
      workflow: "voice-clone",
      status: "manual-required",
      requiredForMvp: true,
      scoringAxes: [
        "speaker consistency",
        "consent fit",
        "turn-taking",
        "artifact rate",
      ],
      passThreshold:
        "all consent gates pass and mean 4/5 on speaker consistency",
      reviewerNotes:
        "Requires approved reference voices and generated dialogue artifacts.",
    },
    {
      id: "score-voice-conversion-quality",
      workflow: "voice-conversion",
      status: "manual-required",
      requiredForMvp: true,
      scoringAxes: [
        "source preservation",
        "target timbre",
        "intelligibility",
        "artifact rate",
      ],
      passThreshold: "mean 4/5 with explicit source-target provenance",
      reviewerNotes: "Requires RVC-style conversion smoke evidence.",
    },
    {
      id: "score-sfx-quality",
      workflow: "sfx",
      status: "manual-required",
      requiredForMvp: true,
      scoringAxes: [
        "prompt adherence",
        "transient quality",
        "loudness",
        "game usability",
      ],
      passThreshold: "at least two variants accepted by a reviewer",
      reviewerNotes: "Requires generated game UI SFX artifacts.",
    },
    {
      id: "score-ambience-loop-quality",
      workflow: "ambience",
      status: "manual-required",
      requiredForMvp: true,
      scoringAxes: ["loop seam", "tonal stability", "noise", "loudness drift"],
      passThreshold: "seam is not distracting across three loop passes",
      reviewerNotes: "Requires loop audition evidence.",
    },
    {
      id: "score-sample-pack-quality",
      workflow: "instrument-sample",
      status: "manual-required",
      requiredForMvp: true,
      scoringAxes: [
        "transient cleanliness",
        "pitch usefulness",
        "tag accuracy",
        "pack consistency",
      ],
      passThreshold: "sample pack receives reviewer acceptance for reuse",
      reviewerNotes: "Requires sample pack preview artifacts.",
    },
    {
      id: "score-loop-pack-quality",
      workflow: "loop",
      status: "manual-required",
      requiredForMvp: true,
      scoringAxes: ["BPM fit", "key fit", "loop seam", "musical usefulness"],
      passThreshold: "loop aligns to grid and repeats cleanly",
      reviewerNotes: "Requires DAW or timeline audition.",
    },
    {
      id: "score-song-quality",
      workflow: "song",
      status: "manual-required",
      requiredForMvp: true,
      scoringAxes: [
        "lyric alignment",
        "section structure",
        "mix balance",
        "originality disclosure",
      ],
      passThreshold: "song passes structure and disclosure review",
      reviewerNotes: "Requires complete song artifacts.",
    },
    {
      id: "score-video-foley-quality",
      workflow: "video-to-audio",
      status: "manual-required",
      requiredForMvp: true,
      scoringAxes: [
        "sync accuracy",
        "event coverage",
        "ambience fit",
        "rights clarity",
      ],
      passThreshold: "generated audio syncs to target ranges",
      reviewerNotes: "Blocked on sc-6183 implementation evidence.",
    },
  ],
  stressCases: [
    {
      id: "stress-long-script",
      title: "Long script chunking",
      workflow: "tts",
      status: "pending",
      requiredForMvp: true,
      scenario:
        "Generate a long-form narration with many segments and speakers.",
      expectedBehavior:
        "chunks resume cleanly, output order is stable, partial failures are recoverable",
    },
    {
      id: "stress-long-song",
      title: "Long song generation",
      workflow: "song",
      status: "pending",
      requiredForMvp: true,
      scenario:
        "Generate a multi-minute lyrics-to-song draft with stems requested.",
      expectedBehavior:
        "duration stays within provider limits and section metadata survives",
    },
    {
      id: "stress-cancellation",
      title: "Cancellation during generation",
      workflow: "composition-render",
      status: "passed",
      requiredForMvp: true,
      scenario: "Cancel a running provider job and preserve actionable state.",
      expectedBehavior:
        "job enters canceling/canceled state without orphaned outputs",
    },
    {
      id: "stress-failed-download",
      title: "Failed model download",
      workflow: "tts",
      status: "pending",
      requiredForMvp: true,
      scenario: "Simulate a provider package download failure.",
      expectedBehavior:
        "runtime exposes recovery guidance and generation remains blocked",
    },
    {
      id: "stress-missing-gpu",
      title: "Missing GPU or accelerator",
      workflow: "song",
      status: "pending",
      requiredForMvp: true,
      scenario: "Run provider preflight on unsupported hardware.",
      expectedBehavior:
        "manifest compatibility reports unavailable rather than queueing",
    },
    {
      id: "stress-unsupported-language",
      title: "Unsupported language",
      workflow: "tts",
      status: "pending",
      requiredForMvp: true,
      scenario: "Submit a script language outside provider support.",
      expectedBehavior: "provider matcher blocks or warns before generation",
    },
    {
      id: "stress-rejected-voice-consent",
      title: "Rejected voice consent",
      workflow: "voice-clone",
      status: "passed",
      requiredForMvp: true,
      scenario: "Attempt cloning or conversion with rejected consent.",
      expectedBehavior:
        "submission remains blocked and audit reason is visible",
    },
    {
      id: "stress-noncommercial-commercial-project",
      title: "Noncommercial model in commercial project",
      workflow: "song",
      status: "passed",
      requiredForMvp: true,
      scenario:
        "Request commercial export from a noncommercial or unknown model.",
      expectedBehavior: "commercial export is blocked with license reasons",
    },
  ],
  knownLimitations: [
    {
      id: "limit-no-real-provider-audio",
      area: "Provider evidence",
      summary:
        "Reference fixtures define contracts but do not yet prove generated audio quality from real selected providers.",
      mitigation:
        "Run first-provider smoke tests and attach artifacts to this matrix.",
      blocksMvp: true,
    },
    {
      id: "limit-video-to-audio-prototype",
      area: "Multimodal SFX",
      summary:
        "Video-to-audio has a fixture and gate, but the product workflow remains tracked by sc-6183.",
      mitigation:
        "Keep the workflow blocked until sc-6183 supplies prototype evidence.",
      blocksMvp: true,
    },
    {
      id: "limit-sceneworks-import",
      area: "SceneWorks handoff",
      summary:
        "SoundWorks export package metadata, target video identity, compatibility checks, and provenance manifest are defined; direct runtime attachment still needs a SceneWorks-side importer.",
      mitigation:
        "Do not claim in-app SceneWorks attachment until the target importer endpoint is implemented and tested in SceneWorks.",
      blocksMvp: false,
    },
    {
      id: "limit-release-hardware",
      area: "Runtime validation",
      summary:
        "Mac and Windows release hardware runs are not captured by this static reference matrix.",
      mitigation: "Attach release-run artifacts before MVP signoff.",
      blocksMvp: true,
    },
  ],
  requirementCoverage: [
    {
      requirementId: "epic-req-1",
      epicRequirement:
        "Text-to-speech with many voices and consent-aware voice profiles.",
      demoWorkflowIds: ["demo-short-narration", "demo-podcast-dialogue"],
      fixtureIds: ["fixture-short-narration", "fixture-podcast-dialogue"],
      checkIds: ["check-job-contracts", "check-safety-gates"],
      status: "manual-required",
    },
    {
      requirementId: "epic-req-2",
      epicRequirement:
        "Generated sound effects, Foley, ambience, and loopable background beds.",
      demoWorkflowIds: [
        "demo-game-ui-sfx",
        "demo-loopable-ambience",
        "demo-video-foley",
      ],
      fixtureIds: [
        "fixture-game-sfx",
        "fixture-ambience-loop",
        "fixture-video-foley",
      ],
      checkIds: ["check-metadata-extraction", "check-provider-manifests"],
      status: "manual-required",
    },
    {
      requirementId: "epic-req-3",
      epicRequirement:
        "Instrument samples and loops with BPM, key, loop points, tags, and provenance.",
      demoWorkflowIds: ["demo-instrument-sample-pack", "demo-loop-pack"],
      fixtureIds: ["fixture-sample-pack", "fixture-loop-pack"],
      checkIds: ["check-metadata-extraction", "check-asset-lifecycle"],
      status: "manual-required",
    },
    {
      requirementId: "epic-req-4",
      epicRequirement:
        "Complete song generation with lyrics, structure, stems, and exportable masters.",
      demoWorkflowIds: ["demo-complete-song", "demo-stem-separation"],
      fixtureIds: ["fixture-song-structure", "fixture-stem-bundle"],
      checkIds: ["check-job-contracts", "check-export-sidecars"],
      status: "manual-required",
    },
    {
      requirementId: "epic-req-5",
      epicRequirement:
        "Recipe, model, seed, reference, license, provenance, and post-processing persistence.",
      demoWorkflowIds: ["demo-edit-trim-normalize", "demo-composition-export"],
      fixtureIds: ["fixture-edit-normalize", "fixture-composition-export"],
      checkIds: ["check-recipe-persistence", "check-export-sidecars"],
      status: "passed",
    },
    {
      requirementId: "epic-req-6",
      epicRequirement:
        "Voice cloning, style imitation, copyrighted music similarity, and disclosure safety gates.",
      demoWorkflowIds: [
        "demo-podcast-dialogue",
        "demo-voice-conversion",
        "demo-complete-song",
      ],
      fixtureIds: [
        "fixture-podcast-dialogue",
        "fixture-voice-conversion",
        "fixture-song-structure",
      ],
      checkIds: ["check-safety-gates"],
      status: "passed",
    },
    {
      requirementId: "epic-req-7",
      epicRequirement:
        "Capability-based provider manifests rather than one-off model assumptions.",
      demoWorkflowIds: ["demo-video-foley", "demo-stem-separation"],
      fixtureIds: ["fixture-video-foley", "fixture-stem-bundle"],
      checkIds: ["check-provider-manifests"],
      status: "passed",
    },
    {
      requirementId: "epic-req-8",
      epicRequirement:
        "Audio-native review tools, version comparison, edits, and production exports.",
      demoWorkflowIds: ["demo-edit-trim-normalize", "demo-composition-export"],
      fixtureIds: ["fixture-edit-normalize", "fixture-composition-export"],
      checkIds: ["check-asset-lifecycle", "check-export-sidecars"],
      status: "passed",
    },
  ],
};

export const fallbackExportWorkflow: ExportWorkflowOverview = {
  schemaVersion: 1,
  presets: [
    {
      preset: {
        id: "preset-podcast-dialogue",
        name: "Podcast/dialogue",
        format: "wav",
        sampleRateHz: 48000,
        bitDepth: 24,
        includeSidecar: true,
        includeStems: false,
        target: "audio-file",
      },
      description:
        "Dialogue export writes WAV/MP3 with loudness normalization and voice consent provenance.",
      sourceKinds: ["asset", "collection"],
      assetKinds: ["voice-clip"],
      formats: ["wav", "mp3"],
      packageArtifacts: ["audio-file", "metadata-sidecar"],
      normalizeLoudness: true,
      targetLufs: -18,
      preserveLoopMetadata: false,
      preserveBpmKeyMetadata: false,
      writesSidecar: true,
    },
    {
      preset: {
        id: "preset-game-sfx",
        name: "Game SFX",
        format: "ogg",
        sampleRateHz: 48000,
        bitDepth: 24,
        includeSidecar: true,
        includeStems: false,
        target: "audio-file",
      },
      description:
        "Game SFX export keeps short filenames, OGG/WAV formats, and one-shot metadata.",
      sourceKinds: ["asset", "collection"],
      assetKinds: ["sfx", "ambience"],
      formats: ["ogg", "wav"],
      packageArtifacts: ["audio-file", "metadata-sidecar"],
      normalizeLoudness: true,
      targetLufs: -16,
      preserveLoopMetadata: false,
      preserveBpmKeyMetadata: false,
      writesSidecar: true,
    },
    {
      preset: {
        id: "preset-sample-pack",
        name: "Sample pack",
        format: "wav",
        sampleRateHz: 48000,
        bitDepth: 24,
        includeSidecar: true,
        includeStems: false,
        target: "daw-handoff",
      },
      description:
        "Sample pack export bundles WAV/FLAC files with BPM, key, loop points, and normalized filenames.",
      sourceKinds: ["sample-pack", "collection"],
      assetKinds: ["instrument-sample", "loop", "reference-audio"],
      formats: ["wav", "flac"],
      packageArtifacts: [
        "zip-bundle",
        "audio-file",
        "loop-markers",
        "metadata-sidecar",
      ],
      normalizeLoudness: false,
      targetLufs: null,
      preserveLoopMetadata: true,
      preserveBpmKeyMetadata: true,
      writesSidecar: true,
    },
    {
      preset: {
        id: "preset-loop-pack",
        name: "Loop pack",
        format: "flac",
        sampleRateHz: 48000,
        bitDepth: 24,
        includeSidecar: true,
        includeStems: false,
        target: "daw-handoff",
      },
      description:
        "Loop pack export preserves tempo, key, and loop marker metadata for DAW import.",
      sourceKinds: ["loop-pack", "collection"],
      assetKinds: ["loop", "music-clip", "ambience"],
      formats: ["wav", "flac", "ogg"],
      packageArtifacts: [
        "zip-bundle",
        "loop-markers",
        "cue-markers",
        "metadata-sidecar",
      ],
      normalizeLoudness: false,
      targetLufs: null,
      preserveLoopMetadata: true,
      preserveBpmKeyMetadata: true,
      writesSidecar: true,
    },
    {
      preset: {
        id: "preset-song-master",
        name: "Song master",
        format: "wav",
        sampleRateHz: 48000,
        bitDepth: 24,
        includeSidecar: true,
        includeStems: false,
        target: "audio-file",
      },
      description:
        "Song master export writes WAV/FLAC/MP3 masters plus lyrics and disclosure sidecar.",
      sourceKinds: ["song"],
      assetKinds: ["song", "music-clip"],
      formats: ["wav", "flac", "mp3"],
      packageArtifacts: ["audio-file", "lyrics-text", "metadata-sidecar"],
      normalizeLoudness: true,
      targetLufs: -14,
      preserveLoopMetadata: false,
      preserveBpmKeyMetadata: true,
      writesSidecar: true,
    },
    {
      preset: {
        id: "preset-daw-stem-bundle",
        name: "DAW stem bundle",
        format: "wav",
        sampleRateHz: 48000,
        bitDepth: 24,
        includeSidecar: true,
        includeStems: true,
        target: "stem-folder",
      },
      description:
        "Stem export writes vocal/accompaniment/instrument folders with provider-native artifacts when available.",
      sourceKinds: ["stem-bundle", "composition"],
      assetKinds: ["stem", "song", "mixdown-export"],
      formats: ["wav", "flac"],
      packageArtifacts: [
        "stem-folder",
        "zip-bundle",
        "provider-native",
        "metadata-sidecar",
      ],
      normalizeLoudness: false,
      targetLufs: null,
      preserveLoopMetadata: false,
      preserveBpmKeyMetadata: true,
      writesSidecar: true,
    },
    {
      preset: {
        id: "preset-sceneworks-video-track",
        name: "SceneWorks video track",
        format: "wav",
        sampleRateHz: 48000,
        bitDepth: 24,
        includeSidecar: true,
        includeStems: true,
        target: "sceneworks-video-track",
      },
      description:
        "SceneWorks handoff writes a rendered mixdown, optional stems, alignment metadata, and provenance sidecar.",
      sourceKinds: ["composition"],
      assetKinds: ["composition", "mixdown-export", "stem"],
      formats: ["wav", "flac"],
      packageArtifacts: [
        "sceneworks-package",
        "audio-file",
        "stem-folder",
        "metadata-sidecar",
      ],
      normalizeLoudness: true,
      targetLufs: -16,
      preserveLoopMetadata: false,
      preserveBpmKeyMetadata: true,
      writesSidecar: true,
    },
  ],
  targets: [
    {
      target: "audio-file",
      label: "Audio files",
      ready: true,
      presetIds: [
        "preset-podcast-dialogue",
        "preset-game-sfx",
        "preset-song-master",
      ],
      notes: ["WAV, FLAC, MP3, and OGG exports keep recipe sidecars."],
    },
    {
      target: "stem-folder",
      label: "Stem folders",
      ready: true,
      presetIds: ["preset-daw-stem-bundle"],
      notes: ["Song and composition stems retain source asset IDs."],
    },
    {
      target: "daw-handoff",
      label: "DAW handoff",
      ready: true,
      presetIds: [
        "preset-sample-pack",
        "preset-loop-pack",
        "preset-daw-stem-bundle",
      ],
      notes: [
        "ZIP bundles use normalized filenames and cue/loop marker metadata.",
      ],
    },
    {
      target: "sceneworks-video-track",
      label: "SceneWorks video track",
      ready: true,
      presetIds: ["preset-sceneworks-video-track"],
      notes: [
        "Handoff package, target video metadata, compatibility checks, and provenance manifest are ready; current SceneWorks source still needs an audio-track importer for direct runtime attachment.",
      ],
    },
  ],
  selectedExport: {
    id: "export-demo-composition-sceneworks",
    presetId: "preset-sceneworks-video-track",
    sourceKind: "composition",
    sourceId: "composition-demo",
    assetIds: ["asset-voice-001", "asset-loop-001"],
    collectionIds: ["collection-neon-bass-pack", "collection-demo-song-folder"],
    formats: ["wav", "flac"],
    canExport: true,
    blockingReasons: [],
    warnings: [
      "SceneWorks direct audio import is not present in current source; export writes a package and manifest for the SceneWorks-side importer.",
    ],
    outputPaths: [
      "soundworks-exports/project-demo/demo-timeline/mixdown.wav",
      "soundworks-exports/project-demo/demo-timeline/stems/track-voice.wav",
      "soundworks-exports/project-demo/demo-timeline/stems/track-loop.wav",
      "soundworks-exports/project-demo/demo-timeline/soundworks-export.json",
    ],
    sidecarPath:
      "soundworks-exports/project-demo/demo-timeline/soundworks-export.json",
  },
  sidecars: [
    {
      id: "sidecar-voice-commercial-export",
      assetId: "asset-voice-lab-conversion-reference",
      assetKind: "voice-clip",
      target: "audio-file",
      path: "soundworks-library/projects/project-demo/voice-clips/asset-voice-lab-conversion-reference/version-voice-lab-conversion-reference-a/metadata/recipe-provenance.json",
      includesRecipe: true,
      includesModel: true,
      includesSourceMedia: true,
      includesRights: true,
      includesEditChain: false,
      disclosureRequired: true,
      eventCount: 3,
    },
    {
      id: "sidecar-song-stem-export",
      assetId: "asset-song-city-lights-full",
      assetKind: "song",
      target: "stem-folder",
      path: "soundworks-library/projects/project-demo/songs/asset-song-city-lights-full/version-song-city-lights-full-a/metadata/recipe-provenance.json",
      includesRecipe: true,
      includesModel: true,
      includesSourceMedia: true,
      includesRights: true,
      includesEditChain: true,
      disclosureRequired: true,
      eventCount: 4,
    },
  ],
  dawHandoff: {
    id: "daw-demo-timeline-bundle",
    presetId: "preset-daw-stem-bundle",
    packagePath:
      "soundworks-exports/project-demo/demo-timeline/demo-timeline-daw.zip",
    normalizedFilenameTemplate:
      "{project}_{asset}_{bpm}bpm_{key}_{version}.{ext}",
    includesZipBundle: true,
    includesStems: true,
    includesCueMarkers: true,
    includesLoopMarkers: true,
    includesBpmKeyMetadata: true,
    includesLyricsText: true,
    includesMidi: false,
    stemKinds: ["vocals", "drums", "bass"],
  },
  sceneWorksHandoff: {
    id: "sceneworks-demo-video-track",
    presetId: "preset-sceneworks-video-track",
    packagePath:
      "soundworks-exports/project-demo/demo-timeline/sceneworks-audio-track.zip",
    renderedMixdownPath:
      "soundworks-exports/project-demo/demo-timeline/mixdown.wav",
    packageManifestPath:
      "soundworks-exports/project-demo/demo-timeline/sceneworks-handoff.json",
    provenanceSidecarPath:
      "soundworks-exports/project-demo/demo-timeline/soundworks-export.json",
    includesOptionalStems: true,
    optionalStemPaths: [
      "soundworks-exports/project-demo/demo-timeline/stems/track-voice.wav",
      "soundworks-exports/project-demo/demo-timeline/stems/track-loop.wav",
    ],
    importStrategy: "file-package",
    attachmentMode: "attach-or-replace",
    intendedProjectId: "sceneworks-project-demo",
    intendedVideoAssetId: "asset_scene_video_airlock",
    sceneWorksProjectPath: "SceneWorks/projects/sceneworks-project-demo",
    targetVideoSidecarPath:
      "assets/videos/asset_scene_video_airlock.sceneworks.json",
    sceneWorksAssetType: "video",
    sceneWorksMimeType: "video/mp4",
    durationMs: 11163,
    targetVideoDurationMs: 12000,
    startOffsetMs: 0,
    sampleRateHz: 48000,
    channels: 2,
    loudnessLufs: -16,
    truePeakDbfs: -1,
    markerCount: 1,
    sectionCount: 1,
    replaceExistingAudio: true,
    roundTripRecipeUrl:
      "soundworks://project/project-demo/compositions/composition-demo/exports/export-demo-composition-sceneworks",
    sourceEvidence: [
      {
        sourceRepo: "SceneWorks",
        filePath: "crates/sceneworks-core/src/project_store.rs",
        lineHint: "import_asset",
        finding:
          "Manual imports currently accept image/* and video/* content, write a .sceneworks.json sidecar, and store free-form provenance under the asset extra field.",
      },
      {
        sourceRepo: "SceneWorks",
        filePath: "crates/sceneworks-worker/src/video_jobs.rs",
        lineHint: "AudioTrack",
        finding:
          "Generated video jobs can carry synchronized interleaved PCM audio internally as sample rate, channel count, and f32 samples.",
      },
      {
        sourceRepo: "SceneWorks",
        filePath: "crates/sceneworks-core/src/contracts.rs",
        lineHint: "AssetType",
        finding:
          "The persisted asset contract currently has image, video, upload, frame, render, document, and pose types; there is no standalone audio asset type.",
      },
    ],
    compatibilityChecks: [
      {
        id: "target.video.sidecar",
        status: "passed",
        summary:
          "Target SceneWorks video asset id and sidecar path are carried in the handoff manifest.",
        mitigation:
          "Use the sidecar path to attach or replace audio metadata without guessing the video asset.",
      },
      {
        id: "duration.fits",
        status: "passed",
        summary:
          "SoundWorks mixdown duration 11.163s fits inside the 12.000s target video window at offset 0.",
        mitigation:
          "If the mixdown is longer, require trim, loop, or explicit overflow approval before export.",
      },
      {
        id: "sample_rate.channels",
        status: "passed",
        summary:
          "Package uses 48kHz stereo WAV/FLAC, matching the video worker's explicit sample-rate/channel metadata shape.",
        mitigation:
          "Transcode to 48kHz stereo before handoff when source material differs.",
      },
      {
        id: "loudness.true_peak",
        status: "passed",
        summary:
          "Mixdown is normalized to -16 LUFS with -1 dBTP ceiling for video-safe playback.",
        mitigation:
          "Block export when clipping is detected or loudness analysis is missing.",
      },
      {
        id: "direct.audio.import",
        status: "warning",
        summary:
          "Current SceneWorks project imports do not accept standalone audio files, so the first integration is a handoff package rather than direct upload.",
        mitigation:
          "Add a SceneWorks-side audio-track attachment endpoint or package importer before claiming runtime attachment.",
      },
      {
        id: "round_trip.recipe",
        status: "passed",
        summary:
          "The manifest includes a soundworks:// round-trip URL back to the source composition export.",
        mitigation:
          "SceneWorks can show this link as provenance until native round-trip editing is implemented.",
      },
    ],
    attachmentSteps: [
      {
        id: "select-target-video",
        label: "Choose SceneWorks project and video asset",
        required: true,
        source: "SoundWorks handoff target picker",
        target: "SceneWorks project id plus video asset sidecar path",
      },
      {
        id: "render-package",
        label: "Render mixdown and optional stems",
        required: true,
        source: "SoundWorks composition renderer",
        target: "mixdown.wav, stems folder, and sceneworks-handoff.json",
      },
      {
        id: "attach-or-replace",
        label: "Attach or replace the video's audio track",
        required: true,
        source: "SceneWorks package importer",
        target: "video asset audio metadata and media reference",
      },
      {
        id: "show-provenance",
        label: "Expose SoundWorks provenance and round-trip link",
        required: true,
        source: "SoundWorks export sidecar",
        target: "SceneWorks asset detail provenance panel",
      },
    ],
  },
  validationChecks: [
    {
      id: "formats.covered",
      passed: true,
      summary: "Export presets cover WAV, FLAC, MP3, and OGG.",
    },
    {
      id: "sidecars.required",
      passed: true,
      summary:
        "Every preset writes recipe, provenance, license, and technical sidecar metadata.",
    },
    {
      id: "loops.preserve_metadata",
      passed: true,
      summary:
        "Loop and sample pack exports preserve BPM, key, and loop marker metadata.",
    },
    {
      id: "songs.handle_stems",
      passed: true,
      summary:
        "Song and DAW exports handle master files plus stems when available.",
    },
    {
      id: "sceneworks.source_documented",
      passed: true,
      summary:
        "SceneWorks source requirements are documented: current imports are image/video assets with provenance in sidecar extra fields, while audio is internal video-job PCM.",
    },
    {
      id: "sceneworks.compatibility",
      passed: true,
      summary:
        "SceneWorks handoff validates duration, sample rate, channels, loudness, target video identity, stale exports, and direct-import limitations.",
    },
  ],
};

const libraryLoopItem = {
  id: "asset-loop-001",
  name: "Dusty trip-hop drums",
  itemType: "loop",
  itemTypeLabel: "Loop",
  scope: { kind: "project", projectId: "project-demo" },
  ownership: "project-local",
  projectId: "project-demo",
  createdAt: "2026-06-17T11:40:00Z",
  sourceWorkflow: "loop",
  tags: ["loop", "drums", "trip-hop"],
  generatedTags: ["loopable", "timeline-placeable"],
  collectionIds: ["collection-neon-bass-pack"],
  durationMs: 11163,
  bpm: 86,
  musicalKey: null,
  language: null,
  voiceProfileId: null,
  providerId: "soundworks-reference",
  modelId: "reference-generation-suite",
  licenseStatus: "provider-licensed",
  commercialUse: "allowed",
  favorite: true,
  rejected: false,
  archived: false,
  waveformThumbnail: {
    previewPath:
      "soundworks-library/projects/project-demo/loops/asset-loop-001/version-loop-001-a/previews/waveform.json",
    peakCount: 48,
    durationMs: 11163,
    ready: true,
  },
  quickAudition: {
    previewable: true,
    playableRangeMs: [0, 11163] as [number, number],
    shortcut: "Space",
  },
  timelinePlaceable: true,
  sourcePickerEligible: true,
  compositionUsageCount: 1,
  recipe: {
    id: "recipe-loop-001",
    workflow: "loop",
    providerId: "soundworks-reference",
    modelId: "reference-generation-suite",
    sourceReferenceCount: 0,
    outputAssetCount: 1,
    replayable: true,
  },
  badges: ["Loop", "ProviderLicensed", "waveform", "tempo"],
} satisfies AssetLibraryOverview["items"][number];

export const fallbackAssetLibrary: AssetLibraryOverview = {
  schemaVersion: 1,
  scopes: [
    {
      id: "project-demo",
      label: "Demo SoundWorks Project",
      scope: { kind: "project", projectId: "project-demo" },
      ownership: "project-local",
      assetCount: 9,
      collectionCount: 2,
      canPromoteToGlobal: true,
    },
    {
      id: "global-library",
      label: "Global audio library",
      scope: { kind: "globalLibrary" },
      ownership: "global",
      assetCount: 4,
      collectionCount: 1,
      canPromoteToGlobal: false,
    },
  ],
  filters: {
    supportedItemTypes: [
      "voice-clip",
      "music-clip",
      "sfx",
      "song",
      "instrument-sample",
      "loop",
      "stem",
      "ambience",
      "voice-profile",
      "reference-audio",
      "composition",
      "mixdown-export",
      "prompt-recipe-preset",
    ],
    coversProjectAndGlobalScopes: true,
    includesRejectedArchivedToggle: true,
    facets: [
      {
        id: "type",
        label: "Type",
        options: [
          { id: "loop", label: "Loop", count: 1, selected: true },
          { id: "voice-clip", label: "Voice clip", count: 1, selected: false },
          { id: "song", label: "Song", count: 1, selected: false },
        ],
      },
      {
        id: "tags",
        label: "Tags",
        options: [
          { id: "drums", label: "drums", count: 1, selected: true },
          { id: "loop", label: "loop", count: 1, selected: true },
          { id: "commercial", label: "commercial", count: 8, selected: false },
        ],
      },
      {
        id: "duration",
        label: "Duration",
        options: [
          { id: "short", label: "Short", count: 4, selected: false },
          { id: "medium", label: "Medium", count: 5, selected: true },
          { id: "long", label: "Long", count: 2, selected: false },
        ],
      },
      {
        id: "bpm",
        label: "BPM",
        options: [
          { id: "under-90", label: "Under 90 BPM", count: 2, selected: true },
          { id: "90-124", label: "90-124 BPM", count: 4, selected: false },
        ],
      },
      {
        id: "key",
        label: "Key",
        options: [
          { id: "a-minor", label: "A minor", count: 2, selected: false },
          { id: "c-minor", label: "C minor", count: 2, selected: false },
          { id: "no-key", label: "No key", count: 4, selected: true },
        ],
      },
      {
        id: "language",
        label: "Language",
        options: [
          { id: "en-us", label: "en-US", count: 4, selected: false },
          {
            id: "instrumental",
            label: "Instrumental",
            count: 9,
            selected: true,
          },
        ],
      },
      {
        id: "voice",
        label: "Voice",
        options: [
          {
            id: "voice-profile-narrator",
            label: "voice-profile-narrator",
            count: 3,
            selected: false,
          },
          { id: "no-voice", label: "No voice", count: 10, selected: true },
        ],
      },
      {
        id: "model",
        label: "Model",
        options: [
          {
            id: "reference-generation-suite",
            label: "reference-generation-suite",
            count: 5,
            selected: false,
          },
          {
            id: "manual-imported",
            label: "Manual/imported",
            count: 5,
            selected: false,
          },
        ],
      },
      {
        id: "license",
        label: "License",
        options: [
          { id: "user-owned", label: "UserOwned", count: 6, selected: false },
          {
            id: "provider-licensed",
            label: "ProviderLicensed",
            count: 7,
            selected: true,
          },
        ],
      },
      {
        id: "project",
        label: "Project",
        options: [
          {
            id: "project-demo",
            label: "project-demo",
            count: 9,
            selected: true,
          },
          {
            id: "global-library",
            label: "Global library",
            count: 4,
            selected: false,
          },
        ],
      },
      {
        id: "createdDate",
        label: "Created date",
        options: [
          { id: "2026-06-17", label: "2026-06-17", count: 13, selected: false },
        ],
      },
      {
        id: "collection",
        label: "Collection",
        options: [
          {
            id: "collection-neon-bass-pack",
            label: "collection-neon-bass-pack",
            count: 3,
            selected: true,
          },
          {
            id: "collection-demo-song-folder",
            label: "collection-demo-song-folder",
            count: 4,
            selected: false,
          },
        ],
      },
      {
        id: "lifecycle",
        label: "Lifecycle",
        options: [
          { id: "active", label: "Active", count: 11, selected: true },
          { id: "favorite", label: "Favorite", count: 2, selected: false },
          { id: "rejected", label: "Rejected", count: 0, selected: false },
          { id: "archived", label: "Archived", count: 0, selected: false },
        ],
      },
      {
        id: "sourceWorkflow",
        label: "Source workflow",
        options: [
          { id: "loop", label: "Loop", count: 1, selected: true },
          { id: "tts", label: "Tts", count: 2, selected: false },
          { id: "song", label: "Song", count: 1, selected: false },
        ],
      },
      {
        id: "compositionUsage",
        label: "Composition usage",
        options: [
          {
            id: "used",
            label: "Used in composition",
            count: 5,
            selected: false,
          },
          { id: "unused", label: "Unused", count: 8, selected: false },
        ],
      },
    ],
  },
  selectedFilter: {
    searchText: "loop commercial local",
    scope: { kind: "project", projectId: "project-demo" },
    selectedType: "loop",
    selectedTags: ["drums", "loop"],
    includeRejected: false,
    includeArchived: false,
    favoriteOnly: false,
  },
  items: [
    libraryLoopItem,
    {
      ...libraryLoopItem,
      id: "asset-reference-neon-bass",
      name: "Neon bass reference",
      itemType: "reference-audio",
      itemTypeLabel: "Reference audio",
      ownership: "global",
      projectId: null,
      tags: ["reference", "bass", "global"],
      durationMs: 26400,
      bpm: 120,
      musicalKey: "A minor",
      favorite: false,
      compositionUsageCount: 2,
      badges: ["Reference audio", "UserOwned", "waveform", "tempo", "key"],
    },
    {
      ...libraryLoopItem,
      id: "voice-profile-narrator",
      name: "Narrator profile",
      itemType: "voice-profile",
      itemTypeLabel: "Voice profile",
      ownership: "linked-global",
      projectId: null,
      tags: ["voice", "narrator", "consented"],
      durationMs: null,
      bpm: null,
      language: "en-US",
      voiceProfileId: "voice-profile-narrator",
      timelinePlaceable: false,
      badges: ["voice profile", "consent stored"],
    },
    {
      ...libraryLoopItem,
      id: "preset-noir-narration",
      name: "Noir narration recipe",
      itemType: "prompt-recipe-preset",
      itemTypeLabel: "Prompt/Recipe preset",
      ownership: "global",
      projectId: null,
      tags: ["preset", "tts", "reusable"],
      durationMs: null,
      bpm: null,
      language: "en-US",
      timelinePlaceable: false,
      badges: ["recipe preset", "global"],
    },
  ],
  selectedItem: {
    item: libraryLoopItem,
    versionHistory: [
      {
        versionId: "version-loop-001-a",
        label: "Original",
        durationMs: 11163,
        filePath:
          "soundworks-library/projects/project-demo/loops/asset-loop-001/version-loop-001-a/media.wav",
        createdBy: "Generated/imported",
        waveformReady: true,
        recipeId: "recipe-loop-001",
      },
      {
        versionId: "version-loop-001-a-review-edit",
        label: "Review edit",
        durationMs: 10480,
        filePath:
          "soundworks-library/projects/project-demo/loops/asset-loop-001/version-loop-001-a/review-edit/media.wav",
        createdBy: "Waveform Review",
        waveformReady: true,
        recipeId: "recipe-review-edit-loop-001",
      },
    ],
    recipe: libraryLoopItem.recipe,
    provenanceLinks: [
      {
        id: "provenance-loop-generated",
        label: "Generated from loop recipe",
        sidecarPath:
          "soundworks-library/projects/project-demo/loops/asset-loop-001/version-loop-001-a/metadata/recipe-provenance.json",
        inspectable: true,
      },
      {
        id: "provenance-loop-review-edit",
        label: "Available to review/edit chain",
        sidecarPath:
          "soundworks-library/projects/project-demo/loops/asset-loop-001/version-loop-001-b-review-edit/metadata/recipe-provenance.json",
        inspectable: true,
      },
    ],
    collectionIds: ["collection-neon-bass-pack"],
    versionCount: 2,
    sourcePickerTargets: [
      "Samples + Loops",
      "Waveform Review",
      "Mixer timeline",
    ],
    notes: [
      "Project-local loop can be promoted to global without losing recipe provenance.",
      "Rejected and archived items remain findable only when lifecycle filters include them.",
    ],
  },
  collections: [
    {
      collection: {
        id: "collection-neon-bass-pack",
        name: "Neon bass starter pack",
        assetIds: [
          "asset-reference-neon-bass",
          "asset-sample-001",
          "asset-loop-001",
        ],
      },
      collectionType: "sample-pack",
      description: "Reference, one-shot, and loop assets grouped for reuse.",
      itemCount: 3,
      dragIntoStudios: ["Samples + Loops", "Mixer"],
    },
    {
      collection: {
        id: "collection-demo-song-folder",
        name: "City Lights song folder",
        assetIds: [
          "asset-song-001",
          "asset-stem-drums-001",
          "asset-mixdown-001",
        ],
      },
      collectionType: "song-folder",
      description: "Song, stem, timeline, and export outputs stay together.",
      itemCount: 4,
      dragIntoStudios: ["Song Studio", "Mixer"],
    },
  ],
  lifecycleActions: [
    {
      id: "favorite",
      label: "Favorite",
      appliesTo: ["loop", "voice-clip", "song"],
      preservesProvenance: true,
      destructive: false,
    },
    {
      id: "reject",
      label: "Reject",
      appliesTo: ["loop", "voice-clip", "song"],
      preservesProvenance: true,
      destructive: false,
    },
    {
      id: "archive",
      label: "Archive",
      appliesTo: ["loop", "voice-clip", "song"],
      preservesProvenance: true,
      destructive: false,
    },
    {
      id: "promote-to-global",
      label: "Promote to global",
      appliesTo: ["loop", "voice-clip", "song", "prompt-recipe-preset"],
      preservesProvenance: true,
      destructive: false,
    },
  ],
  dragTargets: [
    {
      id: "sample-pack",
      label: "Samples + Loops pack",
      acceptedTypes: ["instrument-sample", "loop", "reference-audio"],
      createsLinkedCopy: false,
    },
    {
      id: "mixer-timeline",
      label: "Mixer timeline",
      acceptedTypes: [
        "voice-clip",
        "music-clip",
        "sfx",
        "song",
        "instrument-sample",
        "loop",
        "stem",
        "ambience",
        "reference-audio",
        "mixdown-export",
      ],
      createsLinkedCopy: true,
    },
  ],
  validationChecks: [
    {
      id: "all-major-types",
      passed: true,
      summary:
        "Library contract covers voice clips, music, SFX, songs, samples, loops, stems, ambience, voice profiles, references, compositions, exports, and presets.",
    },
    {
      id: "filters-complete",
      passed: true,
      summary:
        "Filter model includes type, tags, duration, BPM, key, language, voice, model, license, project, collection, lifecycle, date, source workflow, and composition usage.",
    },
    {
      id: "provenance-reachable",
      passed: true,
      summary:
        "Selected asset detail links version history, recipe summary, and provenance sidecars.",
    },
  ],
};

const reviewScope = { kind: "project", projectId: "project-demo" };
const reviewRights = {
  licenseStatus: "user-owned",
  commercialUse: "allowed",
  voiceConsent: "not-voice-material",
  aiDisclosureRequired: true,
  watermark: "not-present",
  referenceMediaOwnership: "user-owned",
};

function reviewAsset(
  id: string,
  kind: string,
  name: string,
  versionId: string,
) {
  return {
    id,
    scope: reviewScope,
    kind,
    name,
    tags: [kind],
    collectionIds: [],
    currentVersionId: versionId,
    versionIds: [versionId],
    rights: reviewRights,
    provenanceIds: [`provenance-${id}`],
  };
}

function reviewVersion(
  assetId: string,
  versionId: string,
  kind: string,
  durationMs: number,
  index = 1,
) {
  return {
    id: versionId,
    assetId,
    versionIndex: index,
    file: {
      storagePath: `soundworks-library/projects/project-demo/${kind}/${assetId}/${versionId}/media.wav`,
      format: "wav",
      codec: "pcm_s16le",
      byteSize: null,
      contentHash: index === 1 ? null : "sha256:review-edit-loop-001",
    },
    technical: {
      sampleRateHz: 48000,
      bitDepth: 24,
      channels: kind === "voice-clips" ? 1 : 2,
      durationMs,
      loudnessLufs: index === 1 ? -14 : -16,
      truePeakDbfs: index === 1 ? -1.5 : -1.8,
      hasClipping: false,
      bpm: kind === "loops" ? 86 : null,
      musicalKey: kind === "songs" ? "A major" : null,
      loopPoints:
        kind === "loops" ? { startSample: 0, endSample: 492288 } : null,
    },
    createdBy:
      index === 1
        ? {
            kind: "generated",
            recipeId: `recipe-${assetId.replace("asset-", "")}`,
            jobId: `job-${assetId}`,
          }
        : {
            kind: "edited",
            sourceVersionId: "version-loop-001-a",
            editRecipeId: "recipe-review-edit-loop-001",
          },
    waveformPreviewCache: `soundworks-library/projects/project-demo/${kind}/${assetId}/${versionId}/previews/waveform.json`,
    spectrogramPreviewCache: `soundworks-library/projects/project-demo/${kind}/${assetId}/${versionId}/previews/spectrogram.bin`,
  };
}

const reviewVoiceAsset = reviewAsset(
  "asset-voice-001",
  "voice-clip",
  "Narration scratch",
  "version-voice-001-a",
);
const reviewSfxAsset = reviewAsset(
  "asset-sfx-001",
  "sfx",
  "Metal hatch impact",
  "version-sfx-001-a",
);
const reviewSampleAsset = reviewAsset(
  "asset-sample-001",
  "instrument-sample",
  "Analog pluck C3",
  "version-sample-001-a",
);
const reviewLoopAsset = {
  ...reviewAsset(
    "asset-loop-001",
    "loop",
    "Dusty trip-hop drums",
    "version-loop-001-b-review-edit",
  ),
  versionIds: ["version-loop-001-a", "version-loop-001-b-review-edit"],
  provenanceIds: [
    "provenance-asset-loop-001",
    "provenance-review-edit-loop-001",
  ],
};
const reviewSongAsset = reviewAsset(
  "asset-song-001",
  "song",
  "Signal reveal cue",
  "version-song-001-a",
);
const reviewOriginalLoopVersion = reviewVersion(
  "asset-loop-001",
  "version-loop-001-a",
  "loops",
  11163,
);
const reviewEditedLoopVersion = reviewVersion(
  "asset-loop-001",
  "version-loop-001-b-review-edit",
  "loops",
  10480,
  2,
);

export const fallbackReviewWorkspace: ReviewWorkspaceOverview = {
  schemaVersion: 1,
  assets: [
    {
      asset: reviewVoiceAsset,
      versions: [
        reviewVersion(
          "asset-voice-001",
          "version-voice-001-a",
          "voice-clips",
          4200,
        ),
      ],
      sourceWorkflow: "tts",
      canPreview: true,
      previewStatus: "ready",
    },
    {
      asset: reviewSfxAsset,
      versions: [
        reviewVersion("asset-sfx-001", "version-sfx-001-a", "sfx", 1800),
      ],
      sourceWorkflow: "sfx",
      canPreview: true,
      previewStatus: "ready",
    },
    {
      asset: reviewSampleAsset,
      versions: [
        reviewVersion(
          "asset-sample-001",
          "version-sample-001-a",
          "instrument-samples",
          2600,
        ),
      ],
      sourceWorkflow: "instrument-sample",
      canPreview: true,
      previewStatus: "ready",
    },
    {
      asset: reviewLoopAsset,
      versions: [reviewOriginalLoopVersion, reviewEditedLoopVersion],
      sourceWorkflow: "loop",
      canPreview: true,
      previewStatus: "ready",
    },
    {
      asset: reviewSongAsset,
      versions: [
        reviewVersion("asset-song-001", "version-song-001-a", "songs", 132000),
      ],
      sourceWorkflow: "song",
      canPreview: true,
      previewStatus: "ready",
    },
  ],
  selectedAsset: {
    asset: reviewLoopAsset,
    versions: [reviewOriginalLoopVersion, reviewEditedLoopVersion],
    sourceWorkflow: "loop",
    canPreview: true,
    previewStatus: "ready",
  },
  transport: {
    playing: false,
    positionMs: 3200,
    durationMs: 11163,
    zoomPixelsPerSecond: 92,
    selection: { startMs: 640, endMs: 10480 },
    loopRegion: { startMs: 0, endMs: 11163 },
    keyboardShortcuts: [
      {
        id: "transport.play_pause",
        keys: "Space",
        action: "Play or pause preview",
      },
      {
        id: "transport.seek_backward",
        keys: "ArrowLeft",
        action: "Seek backward",
      },
      {
        id: "transport.scrub",
        keys: "Shift+Drag",
        action: "Scrub waveform selection",
      },
      {
        id: "transport.zoom",
        keys: "Command+Plus / Command+Minus",
        action: "Zoom waveform",
      },
    ],
    accessibleLabels: [
      "Play or pause waveform preview",
      "Seek through selected audio asset",
      "Adjust loop region start and end",
      "Zoom waveform timeline",
    ],
  },
  waveform: {
    assetVersionId: "version-loop-001-a",
    channelCount: 2,
    sampleRateHz: 48000,
    durationMs: 11163,
    cachePath:
      "soundworks-library/projects/project-demo/loops/asset-loop-001/version-loop-001-a/previews/waveform.json",
    status: "ready",
    peaks: [
      { min: -0.08, max: 0.32 },
      { min: -0.14, max: 0.56 },
      { min: -0.2, max: 0.64 },
      { min: -0.11, max: 0.48 },
      { min: -0.22, max: 0.78 },
      { min: -0.17, max: 0.72 },
      { min: -0.28, max: 0.86 },
      { min: -0.19, max: 0.62 },
      { min: -0.12, max: 0.44 },
      { min: -0.24, max: 0.8 },
      { min: -0.18, max: 0.7 },
      { min: -0.3, max: 0.88 },
      { min: -0.2, max: 0.68 },
      { min: -0.1, max: 0.4 },
      { min: -0.16, max: 0.58 },
      { min: -0.25, max: 0.82 },
    ],
  },
  spectrogram: {
    assetVersionId: "version-loop-001-a",
    cachePath:
      "soundworks-library/projects/project-demo/loops/asset-loop-001/version-loop-001-a/previews/spectrogram.bin",
    status: "ready",
    frequencyBins: 256,
    timeSlices: 128,
  },
  editActions: [
    "Trim selection",
    "Fade in",
    "Fade out",
    "Normalize loudness",
    "Remove silence",
    "Loop crossfade",
    "Convert format",
    "Edit metadata",
  ].map((label) => ({
    id: label.toLowerCase().replaceAll(" ", "-"),
    kind: label.toLowerCase().replaceAll(" ", "-"),
    label,
    operation: label.toLowerCase().replaceAll(" ", "-"),
    destructive: true,
    nonDestructiveSave: true,
    enabled: true,
    parameters: {},
  })),
  editSubmission: {
    id: "review-edit-loop-001",
    canSave: true,
    recipe: { id: "recipe-review-edit-loop-001", workflow: "edit" },
    job: {
      id: "job-review-edit-loop-001",
      recipeId: "recipe-review-edit-loop-001",
      kind: "generate-audio",
      status: "succeeded",
      progress: null,
      outputVersionIds: ["version-loop-001-b-review-edit"],
      error: null,
    },
    sourceAsset: reviewLoopAsset,
    sourceVersion: reviewOriginalLoopVersion,
    savedAsset: reviewLoopAsset,
    savedVersion: reviewEditedLoopVersion,
    warnings: [
      "Normalize target is stored as recipe metadata before media mutation.",
    ],
    blockingReasons: [],
  },
  versionComparison: {
    id: "compare-loop-001-a-b",
    mode: "version-ab",
    left: {
      label: "Original loop",
      assetId: "asset-loop-001",
      versionId: "version-loop-001-a",
      recipeId: "recipe-loop-001",
      durationMs: 11163,
      loudnessLufs: -14,
      truePeakDbfs: -1.5,
    },
    right: {
      label: "Edited loop",
      assetId: "asset-loop-001",
      versionId: "version-loop-001-b-review-edit",
      recipeId: "recipe-review-edit-loop-001",
      durationMs: 10480,
      loudnessLufs: -16,
      truePeakDbfs: -1.8,
    },
    metrics: {
      durationDeltaMs: -683,
      loudnessDeltaLufs: -2,
      truePeakDeltaDb: -0.3,
      waveformDifferenceScore: 18,
    },
    notes: [
      "A/B compare can target two versions of one asset.",
      "The same contract accepts generated variants from different assets.",
    ],
  },
  provenance: {
    inspectable: true,
    originalRecipe: {
      id: "recipe-loop-001",
      workflow: "loop",
      providerId: "fixture-provider",
      modelId: "fixture-audio-model",
      sourceReferenceCount: 0,
      outputAssetCount: 1,
      replayable: true,
    },
    editRecipe: {
      id: "recipe-review-edit-loop-001",
      workflow: "edit",
      providerId: "soundworks-reference",
      modelId: "reference-editor",
      sourceReferenceCount: 1,
      outputAssetCount: 1,
      replayable: true,
    },
    sourceVersionId: "version-loop-001-a",
    editedVersionId: "version-loop-001-b-review-edit",
    provenanceIds: [
      "provenance-asset-loop-001",
      "provenance-review-edit-loop-001",
    ],
    sidecarPath:
      "soundworks-library/projects/project-demo/loops/asset-loop-001/version-loop-001-b-review-edit/metadata/recipe-provenance.json",
  },
  validationChecks: [
    {
      id: "review.preview_all_generated_assets",
      status: "passed",
      summary:
        "Every generated fixture asset kind exposes waveform and spectrogram preview cache paths.",
    },
    {
      id: "review.transport_accessibility",
      status: "passed",
      summary:
        "Transport includes play, pause, seek, scrub, zoom, loop region, time display, keyboard shortcuts, and accessible labels.",
    },
    {
      id: "review.non_destructive_edit",
      status: "passed",
      summary:
        "Save creates a new edited version and preserves the original generated version.",
    },
    {
      id: "review.provenance_recipe",
      status: "passed",
      summary:
        "Edit recipe, source version, generated source recipe, and provenance sidecar remain inspectable.",
    },
  ],
};

const consentRights = {
  licenseStatus: "user-owned",
  commercialUse: "allowed",
  voiceConsent: "explicit-consent-recorded",
  aiDisclosureRequired: true,
  watermark: "sidecar-only",
  referenceMediaOwnership: "speaker-signed profile release",
} as const;

const blockedVoiceRights = {
  licenseStatus: "restricted",
  commercialUse: "disallowed",
  voiceConsent: "prohibited",
  aiDisclosureRequired: true,
  watermark: "unsupported",
  referenceMediaOwnership: "unauthorized identity reference",
} as const;

export const fallbackRightsSafety: RightsSafetyOverview = {
  schemaVersion: 1,
  policy: {
    name: "SoundWorks launch rights policy",
    voiceConsentRequiredFor: [
      "voice-clone",
      "voice-conversion",
      "few-shot-fine-tune",
    ],
    exportRequires: [
      "explicit voice consent when voice material is used",
      "SoundWorks non-commercial use compatibility or provider-terms-reviewed model license",
      "provenance sidecar with model, prompt, source media, recipe, and edit chain",
      "AI disclosure flag when generated or AI-edited audio leaves SoundWorks",
    ],
    blockedPromptCategories: [
      "public-figure-voice-clone",
      "unauthorized-voice-reference",
      "incompatible-model-license",
    ],
    warningPromptCategories: [
      "artist-style-imitation",
      "copyrighted-lyrics",
      "watermark-unavailable",
    ],
    watermarkPolicy: "advisory-until-provider-support",
    provenanceSidecarRequired: true,
  },
  consentChecks: [
    {
      id: "consent.voice-clone.narrator",
      workflow: "voice-clone",
      voiceProfileId: "voice-profile-narrator",
      consentStatus: "explicit-consent-recorded",
      allowedUse: "approved voice clone and conversion",
      decision: "allowed",
      summary:
        "Narrator profile can queue clone, fine-tune, and conversion workflows because explicit consent metadata is stored.",
      storedMetadata: consentRights,
    },
    {
      id: "consent.voice-conversion.guest",
      workflow: "voice-conversion",
      voiceProfileId: "voice-profile-guest-review",
      consentStatus: "requires-review",
      allowedUse: "review-only voice conversion",
      decision: "blocked",
      summary:
        "Guest voice conversion is blocked until the speaker consent record is completed.",
      storedMetadata: {
        ...blockedVoiceRights,
        licenseStatus: "unknown",
        commercialUse: "requires-review",
        voiceConsent: "requires-review",
        referenceMediaOwnership: "pending speaker attestation",
      },
    },
    {
      id: "consent.public-figure.clone",
      workflow: "voice-clone",
      voiceProfileId: "voice-profile-public-figure-blocked",
      consentStatus: "prohibited",
      allowedUse: "none",
      decision: "blocked",
      summary:
        "Public-figure or celebrity voice cloning is blocked rather than queued for review.",
      storedMetadata: blockedVoiceRights,
    },
  ],
  modelUseDecisions: [
    {
      candidateId: "kokoro-82m",
      name: "Kokoro 82M",
      requestedWorkflow: "Tts",
      exportCandidate: true,
      license: "Apache-licensed weights",
      commercialUse: "allowed",
      productEligibility: "product-candidate",
      runtimePath: "rust-native",
      requiresPythonRuntime: false,
      decision: "allowed",
      reasons: ["License evidence supports SoundWorks export consideration."],
    },
    {
      candidateId: "chattts",
      name: "ChatTTS",
      requestedWorkflow: "Tts",
      exportCandidate: true,
      license: "AGPLv3+ code / CC BY-NC 4.0 model",
      commercialUse: "non-commercial",
      productEligibility: "research-only",
      runtimePath: "python-poc-only",
      requiresPythonRuntime: true,
      decision: "blocked",
      reasons: [
        "Noncommercial model terms fit SoundWorks' non-commercial posture when other export gates pass.",
        "Research-only or blocked candidates cannot be SoundWorks export choices.",
        "Python runtime dependency is not allowed in shipped SoundWorks export paths.",
      ],
    },
    {
      candidateId: "diffrhythm-2",
      name: "DiffRhythm 2",
      requestedWorkflow: "Song",
      exportCandidate: true,
      license: "Source-backed license review required",
      commercialUse: "unknown",
      productEligibility: "research-only",
      runtimePath: "python-poc-only",
      requiresPythonRuntime: true,
      decision: "blocked",
      reasons: [
        "Unknown model-use terms block SoundWorks export until reviewed.",
        "Research-only or blocked candidates cannot be SoundWorks export choices.",
        "Python runtime dependency is not allowed in shipped SoundWorks export paths.",
      ],
    },
    {
      candidateId: "stable-audio-3",
      name: "Stable Audio 3",
      requestedWorkflow: "Song",
      exportCandidate: true,
      license: "Stability AI Community License / Enterprise terms",
      commercialUse: "provider-terms",
      productEligibility: "needs-runtime-port",
      runtimePath: "managed-api",
      requiresPythonRuntime: false,
      decision: "warn",
      reasons: [
        "Provider terms must be reviewed and attached before SoundWorks export.",
      ],
    },
  ],
  contentPolicyGates: [
    {
      id: "gate.voice.public-figure",
      category: "public-figure-voice-clone",
      status: "blocked",
      appliesTo: ["voice-clone", "voice-conversion"],
      summary:
        "Public-figure or celebrity voice imitation cannot be submitted.",
      enforcement:
        "Disable generation and require a new, consented voice profile.",
    },
    {
      id: "gate.voice.reference-rights",
      category: "unauthorized-voice-reference",
      status: "blocked",
      appliesTo: ["source-voice", "reference-audio"],
      summary:
        "Voice references without owner attestation are blocked before queueing.",
      enforcement:
        "Require consent metadata on the voice profile and generated output.",
    },
    {
      id: "gate.music.style-imitation",
      category: "artist-style-imitation",
      status: "warning",
      appliesTo: ["song", "loop", "instrument-sample"],
      summary:
        "Artist/style imitation prompts require visible review and provenance notes.",
      enforcement:
        "Warn before generation and include the reviewed prompt in the sidecar.",
    },
    {
      id: "gate.music.copyrighted-lyrics",
      category: "copyrighted-lyrics",
      status: "warning",
      appliesTo: ["song"],
      summary:
        "Copyrighted or third-party lyrics require rights review before export.",
      enforcement: "Allow draft generation only; block export until cleared.",
    },
    {
      id: "gate.disclosure.ai-audio",
      category: "ai-disclosure",
      status: "passed",
      appliesTo: ["export", "sidecar"],
      summary:
        "Generated and edited audio carries an AI disclosure flag in export metadata.",
      enforcement:
        "Write disclosureRequired=true into every generated export sidecar.",
    },
  ],
  exportSidecars: [
    {
      id: "sidecar-voice-export",
      assetId: "asset-voice-lab-conversion-reference",
      assetKind: "voice-clip",
      target: "audio-file",
      path: "soundworks-library/projects/project-demo/voice-clips/asset-voice-lab-conversion-reference/version-voice-lab-conversion-reference-a/metadata/recipe-provenance.json",
      includesRecipe: true,
      includesModel: true,
      includesSourceMedia: true,
      includesRights: true,
      includesEditChain: false,
      disclosureRequired: true,
      watermark: "sidecar-only",
      rights: consentRights,
      provenance: {
        id: "provenance-voice-export",
        subjectId: "asset-voice-lab-conversion-reference",
        events: [
          {
            eventType: "rights-reviewed",
            actor: "system",
            summary: "Explicit voice consent and model-use rights checked.",
            metadata: { author: "soundworks-policy" },
          },
          {
            eventType: "generated",
            actor: "system",
            summary:
              "RVC-style conversion recipe and source audio IDs attached.",
            metadata: { author: "soundworks-policy" },
          },
          {
            eventType: "exported",
            actor: "system",
            summary:
              "WAV export wrote recipe, model, source media, rights, and disclosure metadata.",
            metadata: { author: "soundworks-policy" },
          },
        ],
      },
    },
    {
      id: "sidecar-song-stem-export",
      assetId: "asset-song-city-lights-full",
      assetKind: "song",
      target: "stem-folder",
      path: "soundworks-library/projects/project-demo/songs/asset-song-city-lights-full/version-song-city-lights-full-a/metadata/recipe-provenance.json",
      includesRecipe: true,
      includesModel: true,
      includesSourceMedia: true,
      includesRights: true,
      includesEditChain: true,
      disclosureRequired: true,
      watermark: "sidecar-only",
      rights: {
        licenseStatus: "provider-licensed",
        commercialUse: "requires-review",
        voiceConsent: "not-voice-material",
        aiDisclosureRequired: true,
        watermark: "sidecar-only",
        referenceMediaOwnership:
          "original prompt and lyrics drafted inside SoundWorks",
      },
      provenance: {
        id: "provenance-song-stem-export",
        subjectId: "asset-song-city-lights-full",
        events: [
          {
            eventType: "rights-reviewed",
            actor: "system",
            summary:
              "Provider terms and originality disclosure reviewed before export.",
            metadata: { author: "soundworks-policy" },
          },
          {
            eventType: "generated",
            actor: "system",
            summary:
              "Song recipe, sections, lyrics, stems, and model ID attached.",
            metadata: { author: "soundworks-policy" },
          },
          {
            eventType: "edited",
            actor: "system",
            summary: "Review workspace normalization and trim chain attached.",
            metadata: { author: "soundworks-policy" },
          },
          {
            eventType: "exported",
            actor: "system",
            summary: "Stem folder export wrote rights and provenance sidecar.",
            metadata: { author: "soundworks-policy" },
          },
        ],
      },
    },
  ],
  disclosureChecks: [
    {
      id: "disclosure.voice.generated",
      assetId: "asset-voice-lab-conversion-reference",
      required: true,
      reason:
        "Voice conversion output is generated from a source clip and consented target profile.",
      exportTargets: ["audio-file", "sceneworks-video-track"],
    },
    {
      id: "disclosure.song.generated",
      assetId: "asset-song-city-lights-full",
      required: true,
      reason:
        "Full-song and stem exports need AI-generation disclosure and model provenance.",
      exportTargets: ["stem-folder", "daw-handoff"],
    },
  ],
  validationChecks: [
    {
      id: "validation.voice-consent",
      status: "passed",
      summary:
        "Voice clone and conversion requests have allow/block decisions derived from consent metadata.",
    },
    {
      id: "validation.model-license",
      status: "passed",
      summary:
        "SoundWorks export decisions include model license, product eligibility, and runtime dependency blockers.",
    },
    {
      id: "validation.provenance-sidecar",
      status: "passed",
      summary:
        "Export sidecars include recipe, model, source media, rights, disclosure, and edit-chain fields.",
    },
    {
      id: "validation.watermark-policy",
      status: "warning",
      summary:
        "Watermark embedding remains advisory until provider support is selected; sidecar disclosure is mandatory now.",
    },
  ],
};

export const fallbackCompositionEditor: CompositionEditorOverview = {
  schemaVersion: 1,
  projectId: "project-demo",
  composition: {
    id: "composition-demo",
    name: "Demo timeline",
    tempoBpm: 86,
    musicalKey: "C minor",
    markers: [{ id: "marker-intro", atMs: 0, label: "Intro" }],
    sections: [
      {
        id: "section-intro",
        range: { startMs: 0, endMs: 11163 },
        label: "Intro bed",
      },
    ],
    exportHistory: [
      {
        id: "export-demo-mix",
        jobId: "job-export-demo-mix",
        outputAssetId: "asset-mixdown-001",
        presetId: "preset-sceneworks-video-track",
      },
    ],
  },
  timeline: {
    durationMs: 34000,
    zoomPercent: 125,
    snapGridMs: 250,
    selectedTool: "trim",
    selectedClipId: "clip-voice-intro",
    playbackCursorMs: 5250,
    loopEnabled: true,
    loopRange: { startMs: 0, endMs: 11163 },
    gridLabels: ["0:00", "0:08", "0:16", "0:24", "0:32"],
    markersEditable: true,
    sectionsEditable: true,
  },
  assetBin: [
    {
      assetId: "asset-voice-001",
      versionId: "version-voice-001-a",
      name: "Narration scratch",
      kind: "voice-clip",
      scope: { kind: "project", projectId: "project-demo" },
      durationMs: 4200,
      tags: ["dialogue", "narration"],
      sourceWorkflow: "tts",
      auditionReady: true,
      draggableToTimeline: true,
      provenanceId: "provenance-asset-voice-001",
    },
    {
      assetId: "asset-sfx-001",
      versionId: "version-sfx-001-a",
      name: "Metal hatch impact",
      kind: "sfx",
      scope: { kind: "globalLibrary" },
      durationMs: 1800,
      tags: ["impact", "metal"],
      sourceWorkflow: "sfx",
      auditionReady: true,
      draggableToTimeline: true,
      provenanceId: "provenance-asset-sfx-001",
    },
    {
      assetId: "asset-loop-001",
      versionId: "version-loop-001-a",
      name: "Dusty trip-hop drums",
      kind: "loop",
      scope: { kind: "project", projectId: "project-demo" },
      durationMs: 11163,
      tags: ["loop", "drums", "86bpm"],
      sourceWorkflow: "loop",
      auditionReady: true,
      draggableToTimeline: true,
      provenanceId: "provenance-asset-loop-001",
    },
    {
      assetId: "asset-song-001",
      versionId: "version-song-001-a",
      name: "City Lights full mix",
      kind: "song",
      scope: { kind: "project", projectId: "project-demo" },
      durationMs: 180000,
      tags: ["song", "stems"],
      sourceWorkflow: "song",
      auditionReady: true,
      draggableToTimeline: true,
      provenanceId: "provenance-asset-song-001",
    },
    {
      assetId: "asset-sample-001",
      versionId: "version-sample-001-a",
      name: "Analog pluck C3",
      kind: "instrument-sample",
      scope: { kind: "project", projectId: "project-demo" },
      durationMs: 2600,
      tags: ["sample", "pluck", "C3"],
      sourceWorkflow: "instrument-sample",
      auditionReady: true,
      draggableToTimeline: true,
      provenanceId: "provenance-asset-sample-001",
    },
  ],
  sourceFlows: [
    {
      workflow: "tts",
      label: "TTS Studio voice clips",
      assetKind: "voice-clip",
      status: "ready",
      targetTrackRole: "voice",
    },
    {
      workflow: "voice-conversion",
      label: "Voice Lab conversions",
      assetKind: "voice-clip",
      status: "ready",
      targetTrackRole: "voice",
    },
    {
      workflow: "sfx",
      label: "SFX and ambience variants",
      assetKind: "sfx",
      status: "ready",
      targetTrackRole: "sfx",
    },
    {
      workflow: "loop",
      label: "Samples and loops",
      assetKind: "loop",
      status: "ready",
      targetTrackRole: "music",
    },
    {
      workflow: "song",
      label: "Song masters and stems",
      assetKind: "song",
      status: "ready",
      targetTrackRole: "stem",
    },
    {
      workflow: "video-to-audio",
      label: "Video-to-audio Foley",
      assetKind: "ambience",
      status: "planned",
      targetTrackRole: "sfx",
    },
  ],
  tracks: [
    {
      trackId: "track-voice",
      name: "Narration",
      role: "voice",
      clipCount: 1,
      gainDb: 0,
      pan: 0,
      muted: false,
      soloed: false,
      automationTargets: ["gain"],
      editable: true,
      clips: [
        {
          clipId: "clip-voice-intro",
          assetId: "asset-voice-001",
          versionId: "version-voice-001-a",
          assetName: "Narration scratch",
          assetKind: "voice-clip",
          sourceScope: { kind: "project", projectId: "project-demo" },
          timelineStartMs: 0,
          sourceRange: { startMs: 250, endMs: 3900 },
          fadeInMs: 25,
          fadeOutMs: 80,
          gainDb: -1.5,
          pan: 0,
          lane: 0,
          canTrim: true,
          canSplit: true,
          canDuplicate: true,
          canDelete: true,
        },
      ],
    },
    {
      trackId: "track-loop",
      name: "Loop bed",
      role: "music",
      clipCount: 1,
      gainDb: -2,
      pan: 0,
      muted: false,
      soloed: false,
      automationTargets: [],
      editable: true,
      clips: [
        {
          clipId: "clip-loop-bed",
          assetId: "asset-loop-001",
          versionId: "version-loop-001-a",
          assetName: "Dusty trip-hop drums",
          assetKind: "loop",
          sourceScope: { kind: "project", projectId: "project-demo" },
          timelineStartMs: 0,
          sourceRange: { startMs: 0, endMs: 11163 },
          fadeInMs: 0,
          fadeOutMs: 250,
          gainDb: -6,
          pan: 0,
          lane: 0,
          canTrim: true,
          canSplit: true,
          canDuplicate: true,
          canDelete: true,
        },
      ],
    },
    {
      trackId: "track-sfx",
      name: "Foley hits",
      role: "sfx",
      clipCount: 2,
      gainDb: -3,
      pan: -0.18,
      muted: false,
      soloed: false,
      automationTargets: ["gain", "pan"],
      editable: true,
      clips: [
        {
          clipId: "clip-hatch-hit",
          assetId: "asset-sfx-001",
          versionId: "version-sfx-001-a",
          assetName: "Metal hatch impact",
          assetKind: "sfx",
          sourceScope: { kind: "globalLibrary" },
          timelineStartMs: 7000,
          sourceRange: { startMs: 0, endMs: 1800 },
          fadeInMs: 60,
          fadeOutMs: 160,
          gainDb: -2,
          pan: 0,
          lane: 0,
          canTrim: true,
          canSplit: true,
          canDuplicate: true,
          canDelete: true,
        },
        {
          clipId: "clip-ambience-room",
          assetId: "asset-ambience-001",
          versionId: "version-ambience-001-a",
          assetName: "Engine room bed",
          assetKind: "ambience",
          sourceScope: { kind: "project", projectId: "project-demo" },
          timelineStartMs: 11200,
          sourceRange: { startMs: 0, endMs: 12000 },
          fadeInMs: 60,
          fadeOutMs: 160,
          gainDb: -2,
          pan: 0,
          lane: 1,
          canTrim: true,
          canSplit: true,
          canDuplicate: true,
          canDelete: true,
        },
      ],
    },
    {
      trackId: "track-stems",
      name: "Song stems",
      role: "stem",
      clipCount: 3,
      gainDb: -5,
      pan: 0.12,
      muted: false,
      soloed: false,
      automationTargets: ["gain"],
      editable: true,
      clips: [
        {
          clipId: "clip-song-hook",
          assetId: "asset-song-001",
          versionId: "version-song-001-a",
          assetName: "City Lights full mix",
          assetKind: "song",
          sourceScope: { kind: "project", projectId: "project-demo" },
          timelineStartMs: 16000,
          sourceRange: { startMs: 0, endMs: 18000 },
          fadeInMs: 60,
          fadeOutMs: 160,
          gainDb: -2,
          pan: 0,
          lane: 0,
          canTrim: true,
          canSplit: true,
          canDuplicate: true,
          canDelete: true,
        },
        {
          clipId: "clip-vocal-stem",
          assetId: "asset-stem-vocal-001",
          versionId: "version-stem-vocal-001-a",
          assetName: "City Lights vocal stem",
          assetKind: "stem",
          sourceScope: { kind: "project", projectId: "project-demo" },
          timelineStartMs: 16000,
          sourceRange: { startMs: 0, endMs: 18000 },
          fadeInMs: 60,
          fadeOutMs: 160,
          gainDb: -2,
          pan: 0,
          lane: 1,
          canTrim: true,
          canSplit: true,
          canDuplicate: true,
          canDelete: true,
        },
        {
          clipId: "clip-music-reference",
          assetId: "asset-music-001",
          versionId: "version-music-001-a",
          assetName: "Reference cue bed",
          assetKind: "music-clip",
          sourceScope: { kind: "project", projectId: "project-demo" },
          timelineStartMs: 24000,
          sourceRange: { startMs: 0, endMs: 9500 },
          fadeInMs: 60,
          fadeOutMs: 160,
          gainDb: -2,
          pan: 0,
          lane: 2,
          canTrim: true,
          canSplit: true,
          canDuplicate: true,
          canDelete: true,
        },
      ],
    },
  ],
  mixer: {
    masterGainDb: -1,
    targetLufs: -16,
    truePeakCeilingDbfs: -1,
    renderReady: true,
    loudnessCheck: "composition sits at -16.2 LUFS with -1.1 dBTP peak",
    warnings: [
      "SceneWorks package export is defined; direct runtime attachment needs a SceneWorks importer.",
      "Offline render must be revalidated once a production Web Audio editor is adopted.",
    ],
    trackStates: [
      {
        trackId: "track-voice",
        label: "Narration",
        gainDb: 0,
        pan: 0,
        muted: false,
        soloed: false,
        effectChain: ["high-pass filter", "dialogue compressor"],
        sendTargets: ["room-reverb"],
      },
      {
        trackId: "track-loop",
        label: "Loop bed",
        gainDb: -2,
        pan: 0,
        muted: false,
        soloed: false,
        effectChain: ["low-shelf trim"],
        sendTargets: [],
      },
      {
        trackId: "track-sfx",
        label: "Foley hits",
        gainDb: -3,
        pan: -0.18,
        muted: false,
        soloed: false,
        effectChain: ["short room"],
        sendTargets: ["impact-bus"],
      },
      {
        trackId: "track-stems",
        label: "Song stems",
        gainDb: -5,
        pan: 0.12,
        muted: false,
        soloed: false,
        effectChain: ["bus limiter"],
        sendTargets: [],
      },
    ],
  },
  tools: [
    {
      id: "select",
      label: "Select",
      enabled: true,
      appliesTo: ["clip", "track"],
    },
    { id: "trim", label: "Trim", enabled: true, appliesTo: ["clip"] },
    { id: "split", label: "Split", enabled: true, appliesTo: ["clip"] },
    { id: "fade", label: "Fade", enabled: true, appliesTo: ["clip"] },
    { id: "duplicate", label: "Duplicate", enabled: true, appliesTo: ["clip"] },
    {
      id: "snap-grid",
      label: "Snap grid",
      enabled: true,
      appliesTo: ["timeline"],
    },
    { id: "zoom", label: "Zoom", enabled: true, appliesTo: ["timeline"] },
    {
      id: "mute-solo",
      label: "Mute/Solo",
      enabled: true,
      appliesTo: ["track", "mixer"],
    },
    { id: "render", label: "Render", enabled: true, appliesTo: ["export"] },
  ],
  exportPlan: {
    canRenderMixdown: true,
    presetIds: [
      "preset-composition-mixdown",
      "preset-daw-stem-bundle",
      "preset-sceneworks-video-track",
    ],
    mixdownPath: "soundworks-exports/project-demo/demo-timeline/mixdown.wav",
    stemPaths: [
      "soundworks-exports/project-demo/demo-timeline/stems/track-voice.wav",
      "soundworks-exports/project-demo/demo-timeline/stems/track-loop.wav",
      "soundworks-exports/project-demo/demo-timeline/stems/track-sfx.wav",
    ],
    provenanceSidecarPath:
      "soundworks-exports/project-demo/demo-timeline/soundworks-export.json",
    requiredProvenanceFields: [
      "compositionId",
      "projectId",
      "sourceAssetIds",
      "clipEditChain",
      "modelProviderIds",
      "rightsSummary",
      "exportPresetId",
    ],
    sceneWorksReady: true,
    sceneWorksWarning:
      "SoundWorks can render a SceneWorks handoff package; direct attachment waits for a SceneWorks-side importer.",
  },
  componentDecisions: [
    {
      id: "waveform-playlist",
      name: "waveform-playlist",
      sourceUrl: "https://github.com/naomiaro/waveform-playlist",
      license: "MIT",
      fit: "strong-prototype-candidate",
      strengths: [
        "React, Tone.js, and Web Audio are aligned with the target UI runtime.",
        "Official materials describe multitrack editing, canvas waveforms, drag/drop clip editing, and effects.",
      ],
      risks: [
        "Needs packaged Tauri smoke testing before becoming SoundWorks' production editor core.",
        "SoundWorks must keep timeline persistence independent from library internals.",
      ],
      prototypeEvidence:
        "Best first prototype candidate for clip editing completeness and React fit.",
      decision:
        "Spike first; do not hard-depend in product code until runtime/export behavior is proven.",
    },
    {
      id: "wavesurfer-js-custom",
      name: "wavesurfer.js plus custom timeline",
      sourceUrl: "https://wavesurfer.xyz/",
      license: "BSD-3-Clause",
      fit: "renderer-primitive",
      strengths: [
        "Strong TypeScript waveform renderer with plugin ecosystem.",
        "Good fallback when SoundWorks needs full ownership of timeline state and controls.",
      ],
      risks: [
        "Requires more custom timeline and mixer code than waveform-playlist.",
        "Offline composition rendering still needs a separate engine path.",
      ],
      prototypeEvidence:
        "Useful as renderer foundation when persistence and product controls must dominate.",
      decision:
        "Keep as fallback renderer primitive, especially for asset previews and timeline waveforms.",
    },
    {
      id: "wavesurfer-multitrack",
      name: "wavesurfer-multitrack",
      sourceUrl: "https://github.com/katspaugh/wavesurfer-multitrack",
      license: "BSD-3-Clause",
      fit: "needs-spike",
      strengths: [
        "Official repo positions it as a multitrack plugin for wavesurfer.js.",
        "Permissive license posture is compatible with a commercial desktop app.",
      ],
      risks: [
        "Maintainer notes commercial support limits, so support posture needs review.",
        "Needs verification against current wavesurfer versions and SoundWorks editing requirements.",
      ],
      prototypeEvidence:
        "Viable candidate only after compatibility and support spike.",
      decision:
        "Evaluate after waveform-playlist and custom wavesurfer prototype evidence.",
    },
    {
      id: "tone-transport",
      name: "Tone.js Transport",
      sourceUrl: "https://github.com/tonejs/tone.js/wiki/Transport",
      license: "MIT",
      fit: "timing-primitive",
      strengths: [
        "Transport provides a shared timeline for synchronized sources, signals, and events.",
        "Good fit for snap/grid playback, loop ranges, and future sample-accurate scheduling.",
      ],
      risks: [
        "Not a UI editor by itself; needs waveform and persistence layers.",
        "Desktop audio device behavior still needs Tauri packaging validation.",
      ],
      prototypeEvidence:
        "Use as scheduling primitive beneath whichever editor surface wins.",
      decision:
        "Adopt conceptually for timing, but validate with the selected editor prototype.",
    },
  ],
  validationChecks: [
    {
      id: "timeline-state",
      passed: true,
      summary:
        "Composition timeline persists tracks, clips, trim ranges, fades, markers, sections, tempo, key, and export history.",
    },
    {
      id: "clip-editing",
      passed: true,
      summary:
        "Selected clips expose trim, split, duplicate, delete, fade, gain, and pan capabilities.",
    },
    {
      id: "asset-scope",
      passed: true,
      summary:
        "Timeline clips preserve project/global source identity and version IDs for reopen safety.",
    },
    {
      id: "asset-flow",
      passed: true,
      summary:
        "Generated assets from TTS, Voice Lab, SFX, samples, songs, and future video-to-audio can target editor tracks.",
    },
    {
      id: "mixer-render",
      passed: true,
      summary:
        "Track mute/solo, gain, pan, effects, sends, master loudness, mixdown, stems, and sidecar paths are represented.",
    },
    {
      id: "component-decision",
      passed: true,
      summary:
        "Editor component candidates include source links, tradeoffs, prototype notes, and adoption decision.",
    },
    {
      id: "sceneworks-export",
      passed: true,
      summary:
        "SceneWorks handoff package metadata, target video identity, and compatibility checks are represented for importer validation.",
    },
  ],
};

export const fallbackRuntime: RuntimeOverview = {
  schemaVersion: 1,
  packagingPolicy: {
    name: "SoundWorks shipped desktop runtime",
    productRuntimeAllowsPython: false,
    shippedPlatforms: ["mac-os", "windows"],
    workerProcess: "app-managed-sidecar",
    modelCacheRoots: [
      {
        platform: "mac-os",
        pathHint: "~/Library/Application Support/SoundWorks/models",
        purpose: "macOS packaged and user-installed model cache",
      },
      {
        platform: "windows",
        pathHint: "%APPDATA%\\SoundWorks\\models",
        purpose: "Windows packaged and user-installed model cache",
      },
    ],
  },
  devices: [
    {
      accelerator: "cpu",
      name: "Apple Silicon CPU",
      memoryMb: 32768,
      available: true,
    },
    {
      accelerator: "mps",
      name: "Apple Metal Performance Shaders",
      memoryMb: 32768,
      available: true,
      driver: "Metal",
    },
  ],
  statusCounts: {
    installed: 0,
    available: 0,
    unavailable: 3,
  },
  modelStates: [
    {
      providerId: "soundworks-reference",
      modelId: "reference-speech-suite",
      modelName: "Reference Speech Suite",
      availability: "unavailable",
      installStatus: "packaged",
      health: "blocked",
      workflows: ["tts", "voice-clone", "voice-conversion"],
      reasons: [
        "Manifest declares a packaged or installed model, but no verified cache/package evidence is attached.",
      ],
      cache: {
        status: "missing",
        expectedSizeMb: 2048,
        diskUsageMb: null,
        verified: false,
        evidence: "manifest-only; on-disk cache/package has not been verified",
        warmup: "not-available",
      },
      compatibility: {
        supported: true,
        selectedAccelerator: "cpu",
        minMemoryMb: 4096,
        availableMemoryMb: 32768,
        requiresNetwork: false,
        reasons: [],
      },
    },
    {
      providerId: "soundworks-reference",
      modelId: "reference-generation-suite",
      modelName: "Reference Audio Generation Suite",
      availability: "unavailable",
      installStatus: "packaged",
      health: "blocked",
      workflows: [
        "sfx",
        "ambience",
        "instrument-sample",
        "loop",
        "song",
        "video-to-audio",
      ],
      reasons: [
        "Manifest declares a packaged or installed model, but no verified cache/package evidence is attached.",
      ],
      cache: {
        status: "missing",
        expectedSizeMb: 8192,
        diskUsageMb: null,
        verified: false,
        evidence: "manifest-only; on-disk cache/package has not been verified",
        warmup: "not-available",
      },
      compatibility: {
        supported: true,
        selectedAccelerator: "cpu",
        minMemoryMb: 12288,
        availableMemoryMb: 32768,
        requiresNetwork: false,
        reasons: [],
      },
    },
    {
      providerId: "soundworks-reference",
      modelId: "reference-utility-suite",
      modelName: "Reference Utility Suite",
      availability: "unavailable",
      installStatus: "packaged",
      health: "blocked",
      workflows: ["stem-separation", "edit", "composition-render"],
      reasons: [
        "Manifest declares a packaged or installed model, but no verified cache/package evidence is attached.",
      ],
      cache: {
        status: "missing",
        expectedSizeMb: 1024,
        diskUsageMb: null,
        verified: false,
        evidence: "manifest-only; on-disk cache/package has not been verified",
        warmup: "not-available",
      },
      compatibility: {
        supported: true,
        selectedAccelerator: "cpu",
        minMemoryMb: 2048,
        availableMemoryMb: 32768,
        requiresNetwork: false,
        reasons: [],
      },
    },
  ],
  jobs: [],
  validationChecks: [
    {
      id: "runtime.platforms",
      status: "passed",
      summary: "Packaging policy targets macOS and Windows desktop builds.",
    },
    {
      id: "runtime.no_python",
      status: "passed",
      summary: "Product-enabled runtime manifests do not require Python.",
    },
    {
      id: "runtime.cache_evidence",
      status: "failed",
      summary:
        "Manifest-only packaged/install states cannot count as verified runtime installs: soundworks-reference:reference-speech-suite, soundworks-reference:reference-generation-suite, soundworks-reference:reference-utility-suite.",
      recovery:
        "Inspect the on-disk model cache/package and attach file evidence before marking models installed.",
    },
    {
      id: "runtime.devices",
      status: "passed",
      summary: "Runtime device inventory can report available accelerators.",
    },
  ],
};

export const fallbackTtsStudio: TtsStudioOverview = {
  schemaVersion: 1,
  script: {
    id: "script-launch-read",
    title: "Launch read",
    language: "en-US",
    segments: [
      {
        id: "seg-001",
        position: 1,
        speakerLabel: "Narrator",
        text: "SoundWorks keeps the voice pass close to the edit.",
        sceneLabel: "Intro",
        targetDurationMs: 3100,
        regeneratePolicy: "keep-timing-with-neighbors",
      },
      {
        id: "seg-002",
        position: 2,
        speakerLabel: "Producer",
        text: "Try the warmer take, but keep the last phrase locked to picture.",
        sceneLabel: "Direction",
        targetDurationMs: 3900,
        regeneratePolicy: "regenerate-independently",
      },
      {
        id: "seg-003",
        position: 3,
        speakerLabel: "Narrator",
        text: "When it lands, save it as a project voice clip with the recipe attached.",
        sceneLabel: "Outro",
        targetDurationMs: 4300,
        regeneratePolicy: "keep-timing-with-neighbors",
      },
    ],
    pronunciationDictionary: [
      {
        term: "SoundWorks",
        pronunciation: "sound works",
        appliesToLanguage: "en-US",
      },
    ],
  },
  speakers: [
    {
      label: "Narrator",
      role: "Primary narration",
      voiceProfileId: "voice-profile-narrator",
      language: "en-US",
      consentRequired: true,
      consentStatus: "explicit-consent-recorded",
    },
    {
      label: "Producer",
      role: "Direction callout",
      voiceProfileId: "voice-profile-producer",
      language: "en-US",
      consentRequired: true,
      consentStatus: "explicit-consent-recorded",
    },
  ],
  voiceProfiles: [
    {
      id: "voice-profile-narrator",
      displayName: "Narrator consented profile",
      consent: "explicit-consent-recorded",
      allowedUses: ["tts", "project-only", "commercial"],
    },
    {
      id: "voice-profile-producer",
      displayName: "Producer consented profile",
      consent: "explicit-consent-recorded",
      allowedUses: ["tts", "project-only", "commercial"],
    },
  ],
  providerOptions: [
    {
      providerId: "soundworks-reference",
      modelId: "reference-speech-suite",
      modelVersion: "0.1.0",
      displayName:
        "SoundWorks Reference Capability Registry / Reference Speech Suite",
      runtime: "local",
      installStatus: "packaged",
      runnable: true,
      outputFormat: "wav",
      sampleRateHz: 48000,
      channelLayout: "mono",
      supportedLanguages: [],
      maxSpeakers: null,
      maxDurationMs: null,
      commercialUseAllowed: true,
      requiresVoiceConsent: true,
      watermark: "sidecar-only",
      limitations: ["Voice profile consent is required before generation."],
    },
  ],
  selectedProvider: {
    providerId: "soundworks-reference",
    modelId: "reference-speech-suite",
    modelVersion: "0.1.0",
    runtime: "local",
    accepted: true,
    blocker: "Voice profile consent is required before generation.",
  },
  controls: {
    speed: 1,
    style: "clear narration",
    emotion: "warm",
    targetLoudnessLufs: -18,
    normalizeOutput: true,
    preserveSegmentTiming: true,
    promoteToProjectLibrary: true,
  },
  generationPlan: {
    chunks: [
      {
        id: "chunk-seg-001",
        segmentIds: ["seg-001"],
        speakerLabel: "Narrator",
        voiceProfileId: "voice-profile-narrator",
        targetDurationMs: 3100,
        regeneratePolicy: "keep-timing-with-neighbors",
      },
      {
        id: "chunk-seg-002",
        segmentIds: ["seg-002"],
        speakerLabel: "Producer",
        voiceProfileId: "voice-profile-producer",
        targetDurationMs: 3900,
        regeneratePolicy: "regenerate-independently",
      },
      {
        id: "chunk-seg-003",
        segmentIds: ["seg-003"],
        speakerLabel: "Narrator",
        voiceProfileId: "voice-profile-narrator",
        targetDurationMs: 4300,
        regeneratePolicy: "keep-timing-with-neighbors",
      },
    ],
    stitching: {
      crossfadeMs: 35,
      preserveSegmentTiming: true,
      silenceTrim: true,
      normalizeLoudnessLufs: -18,
    },
    estimatedTotalDurationMs: 11300,
    preservesSpeakerConsistency: true,
  },
  submission: {
    canSubmit: false,
    job: {
      id: "job-tts-studio-reference",
      recipeId: "recipe-tts-studio-reference",
      kind: "generate-audio",
      status: "failed",
      progress: {
        percent: 100,
        message: "Blocked until selected model cache is verified.",
      },
      outputVersionIds: [],
      error: "No runnable TTS provider is registered.",
    },
    recipe: {
      id: "recipe-tts-studio-reference",
      workflow: "tts",
      outputAssetIds: ["asset-tts-studio-reference"],
    },
    blockingReasons: ["No runnable TTS provider is registered."],
    warnings: [
      "Post-processing will normalize dialogue loudness after stitching.",
    ],
  },
  savedOutput: {
    asset: {
      id: "asset-tts-studio-reference",
      kind: "voice-clip",
      name: "TTS Studio narration draft",
      tags: ["voice-clips", "tts", "multi-speaker"],
      currentVersionId: "version-tts-studio-reference-a",
    },
    version: {
      id: "version-tts-studio-reference-a",
      file: {
        storagePath:
          "soundworks-library/projects/project-demo/voice-clips/asset-tts-studio-reference/version-tts-studio-reference-a/media.wav",
        format: "wav",
      },
      technical: {
        sampleRateHz: 48000,
        channels: 1,
        durationMs: 11300,
        loudnessLufs: -18,
      },
    },
    promotedToProjectLibrary: true,
    waveformPreviewReady: true,
  },
  validationChecks: [
    {
      id: "tts.script_segments",
      status: "passed",
      summary:
        "Script is segmented by speaker and scene for per-segment regeneration.",
    },
    {
      id: "tts.consent_gate",
      status: "passed",
      summary:
        "Voice-clone capable generation requires explicit consent before submission.",
    },
    {
      id: "tts.asset_promotion",
      status: "passed",
      summary:
        "Successful output is represented as a project Voice clip with recipe provenance.",
    },
  ],
};

export const fallbackVoiceLab: VoiceLabOverview = {
  schemaVersion: 1,
  modes: [
    {
      mode: "zero-shot-clone",
      label: "Zero-shot clone",
      workflow: "voice-clone",
      inputAssetKinds: ["reference-audio"],
      outputAssetKind: "voice-clip",
      providerCandidateIds: [
        "chatterbox",
        "gpt-sovits",
        "f5-tts",
        "cosyvoice-2",
        "openvoice-v2",
        "xtts-v2",
      ],
      ready: true,
    },
    {
      mode: "few-shot-fine-tune",
      label: "Few-shot fine-tune",
      workflow: "voice-clone",
      inputAssetKinds: ["reference-audio"],
      outputAssetKind: "voice-clip",
      providerCandidateIds: ["gpt-sovits", "f5-tts", "cosyvoice-2"],
      ready: true,
    },
    {
      mode: "voice-conversion",
      label: "Voice conversion",
      workflow: "voice-conversion",
      inputAssetKinds: ["reference-audio", "voice-clip"],
      outputAssetKind: "voice-clip",
      providerCandidateIds: ["chatterbox", "openvoice-v2", "rvc"],
      ready: true,
    },
  ],
  voiceProfiles: [
    {
      profile: {
        id: "voice-profile-narrator",
        displayName: "Narrator consented profile",
        consent: "explicit-consent-recorded",
        allowedUses: [
          "tts",
          "voice-conversion",
          "fine-tuning",
          "project-only",
          "commercial",
        ],
      },
      speakerIdentity: "Narrator",
      language: "en-US",
      sourceClipIds: [
        "voice-ref-narrator-close",
        "voice-ref-narrator-energetic",
      ],
      modeReadiness: [
        { mode: "zero-shot-clone", ready: true, reason: null },
        { mode: "few-shot-fine-tune", ready: true, reason: null },
        { mode: "voice-conversion", ready: true, reason: null },
      ],
      commercialUseAllowed: true,
      safetySummary:
        "Explicit consent covers TTS, fine-tuning, and conversion for this project.",
    },
    {
      profile: {
        id: "voice-profile-archival",
        displayName: "Archival interview review",
        consent: "requires-review",
        allowedUses: ["project-only"],
      },
      speakerIdentity: "Interview guest",
      language: "en-US",
      sourceClipIds: ["voice-ref-archival-interview"],
      modeReadiness: [
        {
          mode: "zero-shot-clone",
          ready: false,
          reason: "Explicit voice consent has not been recorded.",
        },
        {
          mode: "few-shot-fine-tune",
          ready: false,
          reason: "Fine-tuning requires explicit consent and ownership notes.",
        },
        {
          mode: "voice-conversion",
          ready: false,
          reason: "Conversion is disabled until consent review passes.",
        },
      ],
      commercialUseAllowed: false,
      safetySummary:
        "Kept visible for review, but all cloning and conversion actions are gated.",
    },
  ],
  referenceClips: [
    {
      id: "voice-ref-narrator-close",
      assetId: "asset-narrator-close-ref",
      profileId: "voice-profile-narrator",
      label: "Close mic neutral read",
      durationMs: 18200,
      consent: "explicit-consent-recorded",
      ownerAttestation: "speaker-signed",
      acceptedForModes: [
        "zero-shot-clone",
        "few-shot-fine-tune",
        "voice-conversion",
      ],
    },
    {
      id: "voice-ref-narrator-energetic",
      assetId: "asset-narrator-energy-ref",
      profileId: "voice-profile-narrator",
      label: "Energetic promo read",
      durationMs: 24600,
      consent: "explicit-consent-recorded",
      ownerAttestation: "speaker-signed",
      acceptedForModes: ["few-shot-fine-tune"],
    },
    {
      id: "voice-ref-archival-interview",
      assetId: "asset-archival-interview-ref",
      profileId: "voice-profile-archival",
      label: "Interview excerpt",
      durationMs: 31400,
      consent: "requires-review",
      ownerAttestation: "review-required",
      acceptedForModes: [],
    },
  ],
  conversionSource: {
    assetId: "asset-voice-lab-source-read",
    name: "Producer dry read",
    durationMs: 7800,
    kind: "reference-audio",
  },
  providerScorecards: [
    {
      candidateId: "chatterbox",
      name: "Chatterbox",
      provider: "Resemble AI",
      lanes: ["tts", "voice-clone", "voice-conversion"],
      status: "promising-spike",
      productEligibility: "needs-runtime-port",
      readiness: "needs-runtime-port",
      runtimePath: "external-executable",
      commercialUse: "allowed",
      recommended: true,
      blockers: [
        "Need no-Python product path and watermark/provenance validation",
      ],
      notes: "Strong voice candidate for a packaged provider spike.",
    },
    {
      candidateId: "chatterbox-turbo",
      name: "Chatterbox Turbo",
      provider: "Resemble AI",
      lanes: ["tts"],
      status: "promising-spike",
      productEligibility: "needs-runtime-port",
      readiness: "unsuitable",
      runtimePath: "external-executable",
      commercialUse: "allowed",
      recommended: false,
      blockers: [
        "Need SoundWorks-owned realtime latency measurements",
        "Candidate is tracked for TTS only and is not a Voice Lab provider.",
      ],
      notes:
        "Track separately from base Chatterbox because latency is the product question.",
    },
    {
      candidateId: "gpt-sovits",
      name: "GPT-SoVITS",
      provider: "RVC-Boss",
      lanes: ["tts", "voice-clone"],
      status: "promising-spike",
      productEligibility: "research-only",
      readiness: "research-only",
      runtimePath: "python-poc-only",
      commercialUse: "unknown",
      recommended: false,
      blockers: ["No product-safe no-Python runtime path yet"],
      notes:
        "Useful zero/few-shot voice baseline; research-only until isolated runtime strategy exists.",
    },
    {
      candidateId: "f5-tts",
      name: "F5-TTS",
      provider: "SWivid",
      lanes: ["tts", "voice-clone"],
      status: "blocked",
      productEligibility: "blocked",
      readiness: "blocked",
      runtimePath: "python-poc-only",
      commercialUse: "non-commercial",
      recommended: false,
      blockers: [
        "Pretrained model license requires SoundWorks compatibility review",
      ],
      notes:
        "Use only as a research comparison unless packaging and license compatibility are resolved.",
    },
    {
      candidateId: "cosyvoice-2",
      name: "CosyVoice 2",
      provider: "FunAudioLLM / Alibaba",
      lanes: ["tts", "voice-clone"],
      status: "promising-spike",
      productEligibility: "research-only",
      readiness: "research-only",
      runtimePath: "python-poc-only",
      commercialUse: "unknown",
      recommended: false,
      blockers: ["License and packaged runtime path unresolved"],
      notes:
        "Evaluate for quality and streaming behavior, not as first product provider.",
    },
    {
      candidateId: "openvoice-v2",
      name: "OpenVoice V2",
      provider: "MyShell AI",
      lanes: ["voice-clone", "voice-conversion", "tts"],
      status: "promising-spike",
      productEligibility: "needs-runtime-port",
      readiness: "needs-runtime-port",
      runtimePath: "external-executable",
      commercialUse: "allowed",
      recommended: false,
      blockers: ["Need consent UX and executable packaging proof"],
      notes:
        "Good voice-clone/conversion candidate once consent and packaging are in place.",
    },
    {
      candidateId: "rvc",
      name: "RVC",
      provider: "RVC Project",
      lanes: ["voice-conversion"],
      status: "promising-spike",
      productEligibility: "needs-runtime-port",
      readiness: "needs-runtime-port",
      runtimePath: "external-executable",
      commercialUse: "allowed",
      recommended: true,
      blockers: [
        "Must be routed only as voice conversion and gated by consent",
      ],
      notes: "Score as speech-to-speech voice conversion, not text-to-speech.",
    },
    {
      candidateId: "xtts-v2",
      name: "XTTS-v2",
      provider: "Coqui",
      lanes: ["tts", "voice-clone"],
      status: "blocked",
      productEligibility: "blocked",
      readiness: "blocked",
      runtimePath: "python-poc-only",
      commercialUse: "non-commercial",
      recommended: false,
      blockers: [
        "Non-commercial model license requires SoundWorks compatibility review",
        "Reference clip requirement must be normalized as 6 seconds from the model card unless docs are reconciled",
      ],
      notes: "Keep as a comparison/reference only unless licensing changes.",
    },
  ],
  selectedConversion: {
    canSubmit: true,
    job: {
      id: "job-voice-lab-conversion-reference",
      recipeId: "recipe-voice-lab-conversion-reference",
      kind: "generate-audio",
      status: "queued",
      outputVersionIds: ["version-voice-lab-conversion-reference-a"],
      error: null,
    },
    recipe: {
      id: "recipe-voice-lab-conversion-reference",
      workflow: "voice-conversion",
      provider: {
        providerId: "rvc",
        modelId: "rvc",
        runtime: "local",
      },
      request: {
        kind: "voiceConversion",
        sourceAudioAssetId: "asset-voice-lab-source-read",
        targetVoiceProfileId: "voice-profile-narrator",
        preserveTiming: true,
      },
      outputAssetIds: ["asset-voice-lab-conversion-reference"],
    },
    blockingReasons: [],
    warnings: [
      "RVC is represented as a gated provider spike until a packaged runtime port exists.",
      "Converted output keeps the source timing and is saved as a Voice clip with recipe provenance.",
    ],
  },
  savedOutput: {
    asset: {
      id: "asset-voice-lab-conversion-reference",
      kind: "voice-clip",
      name: "Narrator converted read",
      tags: ["voice-clips", "voice-conversion", "speech-to-speech"],
      currentVersionId: "version-voice-lab-conversion-reference-a",
    },
    version: {
      id: "version-voice-lab-conversion-reference-a",
      file: {
        storagePath:
          "soundworks-library/projects/project-demo/voice-clips/asset-voice-lab-conversion-reference/version-voice-lab-conversion-reference-a/media.wav",
        format: "wav",
      },
      technical: {
        sampleRateHz: 48000,
        channels: 1,
        durationMs: 7800,
        loudnessLufs: -18,
      },
    },
    waveformPreviewReady: true,
  },
  safetyGates: [
    {
      id: "voice.consent.explicit",
      status: "passed",
      summary:
        "Clone, fine-tune, and conversion modes require explicit voice consent.",
    },
    {
      id: "voice.unauthorized_clone.blocked",
      status: "passed",
      summary:
        "Profiles marked Requires review cannot queue cloning or conversion jobs.",
    },
    {
      id: "voice.conversion.source_audio",
      status: "passed",
      summary:
        "RVC-style conversion requires source audio and a target voice profile.",
    },
    {
      id: "voice.commercial_use.review",
      status: "warning",
      summary:
        "Unknown provider licenses stay visible as blocked scorecards; noncommercial licenses require SoundWorks compatibility review.",
    },
  ],
  qaChecks: [
    {
      id: "qa.similarity",
      label: "Speaker similarity",
      status: "ready",
      target: "Compare converted output against the consented target profile.",
    },
    {
      id: "qa.intelligibility",
      label: "Intelligibility",
      status: "ready",
      target: "Confirm speech remains clear after conversion.",
    },
    {
      id: "qa.artifacts",
      label: "Artifacts",
      status: "needs-review",
      target:
        "Review pitch tracking, breaths, and metallic artifacts before export.",
    },
  ],
};

export const fallbackSfxStudio: SfxStudioOverview = {
  schemaVersion: 1,
  prompt: {
    id: "prompt-sfx-hatch-ambience",
    text: "Close metallic hatch impact with a short pressurized tail, followed by a low engine-room ambience bed.",
    negativePrompt: "music, dialogue, melody, crowd",
    category: "foley-impact",
    tags: ["foley", "metal", "engine-room"],
    referenceAudioAssetId: "asset-reference-metal-room-tone",
  },
  controls: {
    durationMs: 8000,
    variationCount: 3,
    intensity: 72,
    realism: 64,
    loopable: true,
    trimSilence: true,
    normalizeLoudnessLufs: -20,
    fadeInMs: 20,
    fadeOutMs: 140,
    loopCrossfadeMs: 180,
    promoteToProjectLibrary: true,
  },
  categoryPresets: [
    {
      category: "foley-impact",
      label: "Foley impact",
      defaultDurationMs: 1800,
      loopableDefault: false,
      outputKind: "sfx",
    },
    {
      category: "ambience-bed",
      label: "Ambience bed",
      defaultDurationMs: 12000,
      loopableDefault: true,
      outputKind: "ambience",
    },
    {
      category: "transition",
      label: "Transition",
      defaultDurationMs: 2400,
      loopableDefault: false,
      outputKind: "sfx",
    },
    {
      category: "ui-sound",
      label: "UI sound",
      defaultDurationMs: 900,
      loopableDefault: false,
      outputKind: "sfx",
    },
  ],
  providerOptions: [
    {
      providerId: "soundworks-reference",
      modelId: "reference-generation-suite",
      modelVersion: "0.1.0",
      displayName:
        "SoundWorks Reference Capability Registry / Reference Audio Generation Suite",
      workflow: "sfx",
      runtime: "local",
      installStatus: "packaged",
      runnable: true,
      outputAssetKind: "sfx",
      outputFormat: "wav",
      sampleRateHz: 48000,
      channelLayout: "stereo",
      minDurationMs: null,
      maxDurationMs: null,
      supportsReferenceAudio: false,
      supportsLooping: false,
      commercialUseAllowed: true,
      watermark: "sidecar-only",
      supportedControls: [
        "prompt",
        "category",
        "variation-count",
        "intensity",
        "realism",
        "batch-generation",
        "duration",
      ],
      limitations: [],
    },
    {
      providerId: "soundworks-reference",
      modelId: "reference-generation-suite",
      modelVersion: "0.1.0",
      displayName:
        "SoundWorks Reference Capability Registry / Reference Audio Generation Suite",
      workflow: "ambience",
      runtime: "local",
      installStatus: "packaged",
      runnable: true,
      outputAssetKind: "ambience",
      outputFormat: "wav",
      sampleRateHz: 48000,
      channelLayout: "stereo",
      minDurationMs: null,
      maxDurationMs: null,
      supportsReferenceAudio: false,
      supportsLooping: true,
      commercialUseAllowed: true,
      watermark: "sidecar-only",
      supportedControls: [
        "prompt",
        "category",
        "variation-count",
        "intensity",
        "realism",
        "batch-generation",
        "duration",
        "loopable",
      ],
      limitations: [],
    },
  ],
  selectedProvider: {
    providerId: "soundworks-reference",
    modelId: "reference-generation-suite",
    modelVersion: "0.1.0",
    workflow: "sfx",
    runtime: "local",
    accepted: true,
    blocker: null,
  },
  providerScorecards: [
    {
      candidateId: "moss-soundeffect",
      name: "MOSS-SoundEffect",
      provider: "OpenMOSS",
      lanes: ["sfx", "ambience"],
      status: "promising-spike",
      productEligibility: "product-candidate",
      readiness: "ready",
      runtimePath: "native-library-binding",
      commercialUse: "allowed",
      recommended: true,
      blockers: [
        "Need SoundWorks-owned prompt adherence smoke test and Windows packaging answer",
      ],
      notes:
        "Best first SFX spike because an Apache-licensed MLX path exists for local Mac validation.",
    },
    {
      candidateId: "stable-audio-3",
      name: "Stable Audio 3",
      provider: "Stability AI",
      lanes: ["song", "sfx", "ambience"],
      status: "promising-spike",
      productEligibility: "needs-runtime-port",
      readiness: "needs-runtime-port",
      runtimePath: "external-executable",
      commercialUse: "provider-terms",
      recommended: false,
      blockers: [
        "Community license and enterprise threshold must be enforced before product enablement",
      ],
      notes:
        "Strong broad audio candidate, but SoundWorks needs runnable Mac/Windows packaging proof and license gating.",
    },
    {
      candidateId: "stable-audio-open-1",
      name: "Stable Audio Open 1.0",
      provider: "Stability AI",
      lanes: ["sfx", "ambience", "loop"],
      status: "promising-spike",
      productEligibility: "needs-runtime-port",
      readiness: "needs-runtime-port",
      runtimePath: "external-executable",
      commercialUse: "provider-terms",
      recommended: false,
      blockers: ["Community license terms must be enforced"],
      notes:
        "SFX/music-bed candidate, but license terms make it less clean than Apache/MIT options.",
    },
    {
      candidateId: "audiocraft-audiogen",
      name: "AudioCraft / AudioGen",
      provider: "Meta",
      lanes: ["sfx"],
      status: "promising-spike",
      productEligibility: "research-only",
      readiness: "research-only",
      runtimePath: "python-poc-only",
      commercialUse: "unknown",
      recommended: false,
      blockers: [
        "Model card requires downstream risk investigation before SoundWorks use",
      ],
      notes: "Baseline SFX comparator; not first product provider.",
    },
    {
      candidateId: "audioldm",
      name: "AudioLDM",
      provider: "AudioLDM authors",
      lanes: ["sfx", "ambience"],
      status: "blocked",
      productEligibility: "blocked",
      readiness: "blocked",
      runtimePath: "python-poc-only",
      commercialUse: "non-commercial",
      recommended: false,
      blockers: [
        "Non-commercial checkpoint license requires SoundWorks compatibility review",
      ],
      notes: "Research comparison only.",
    },
    {
      candidateId: "audioldm-2",
      name: "AudioLDM 2",
      provider: "AudioLDM authors",
      lanes: ["sfx", "ambience"],
      status: "promising-spike",
      productEligibility: "research-only",
      readiness: "research-only",
      runtimePath: "python-poc-only",
      commercialUse: "unknown",
      recommended: false,
      blockers: ["License and no-Python runtime path unresolved"],
      notes: "Evaluate as SFX baseline only.",
    },
    {
      candidateId: "audiox",
      name: "AudioX",
      provider: "AudioX authors",
      lanes: ["sfx", "video-to-audio", "ambience"],
      status: "promising-spike",
      productEligibility: "research-only",
      readiness: "deferred-to-video-audio",
      runtimePath: "python-poc-only",
      commercialUse: "unknown",
      recommended: false,
      blockers: [
        "License, weights, and runtime packaging need validation",
        "Multimodal/video-to-audio workflow is tracked in sc-6183.",
      ],
      notes: "Good multimodal benchmark; not product-ready.",
    },
    {
      candidateId: "mmaudio",
      name: "MMAudio",
      provider: "MMAudio authors",
      lanes: ["video-to-audio", "sfx"],
      status: "promising-spike",
      productEligibility: "research-only",
      readiness: "deferred-to-video-audio",
      runtimePath: "python-poc-only",
      commercialUse: "unknown",
      recommended: true,
      blockers: [
        "License terms and no-Python product path unresolved",
        "Multimodal/video-to-audio workflow is tracked in sc-6183.",
      ],
      notes: "Primary video-to-audio benchmark candidate.",
    },
    {
      candidateId: "thinksound",
      name: "ThinkSound",
      provider: "FunAudioLLM",
      lanes: ["video-to-audio", "sfx", "ambience"],
      status: "promising-spike",
      productEligibility: "research-only",
      readiness: "deferred-to-video-audio",
      runtimePath: "python-poc-only",
      commercialUse: "unknown",
      recommended: false,
      blockers: [
        "License and dependency footprint unresolved",
        "Multimodal/video-to-audio workflow is tracked in sc-6183.",
      ],
      notes: "Useful for reasoning-heavy video-to-audio comparisons.",
    },
  ],
  deferredMultimodalCandidateIds: ["audiox", "mmaudio", "thinksound"],
  variants: [
    {
      id: "sfx-variant-impact-tight",
      label: "Tight hatch impact",
      workflow: "sfx",
      assetKind: "sfx",
      category: "foley-impact",
      durationMs: 1800,
      loudnessLufs: -17.5,
      truePeakDbfs: -1,
      loopable: false,
      loopPoints: null,
      tags: ["engine-room", "foley", "impact", "metal", "tight"],
      selectedForSave: true,
    },
    {
      id: "sfx-variant-impact-heavy",
      label: "Heavy pressure hit",
      workflow: "sfx",
      assetKind: "sfx",
      category: "foley-impact",
      durationMs: 2400,
      loudnessLufs: -16,
      truePeakDbfs: -0.8,
      loopable: false,
      loopPoints: null,
      tags: ["engine-room", "foley", "heavy", "impact", "metal"],
      selectedForSave: false,
    },
    {
      id: "sfx-variant-engine-room-bed",
      label: "Engine room bed",
      workflow: "ambience",
      assetKind: "ambience",
      category: "ambience-bed",
      durationMs: 8000,
      loudnessLufs: -20,
      truePeakDbfs: -3,
      loopable: true,
      loopPoints: {
        startSample: 2400,
        endSample: 381600,
      },
      tags: ["ambience", "engine-room", "foley", "loopable", "metal"],
      selectedForSave: true,
    },
  ],
  comparison: {
    selectedVariantId: "sfx-variant-impact-tight",
    variantCount: 3,
    loopableVariantIds: ["sfx-variant-engine-room-bed"],
    savedVariantIds: [
      "sfx-variant-impact-tight",
      "sfx-variant-engine-room-bed",
    ],
  },
  submission: {
    canSubmit: false,
    job: {
      id: "job-sfx-studio-reference",
      recipeId: "recipe-sfx-studio-reference",
      kind: "generate-audio",
      status: "failed",
      outputVersionIds: [],
      error: "No runnable SFX or ambience provider is registered.",
    },
    recipe: {
      id: "recipe-sfx-studio-reference",
      workflow: "sfx",
      outputAssetIds: [
        "asset-sfx-variant-impact-tight",
        "asset-sfx-variant-engine-room-bed",
      ],
    },
    blockingReasons: ["No runnable SFX or ambience provider is registered."],
    warnings: [
      "Reference audio is stored with the recipe but the selected provider treats it as context only.",
      "Loopability will be inspected with post-processing because the provider has no native loop control.",
      "Post-processing will trim leading and trailing silence.",
    ],
  },
  savedOutputs: [
    {
      variantId: "sfx-variant-impact-tight",
      asset: {
        id: "asset-sfx-variant-impact-tight",
        kind: "sfx",
        name: "Tight hatch impact",
        tags: ["engine-room", "foley", "impact", "metal", "tight"],
        currentVersionId: "version-sfx-variant-impact-tight-a",
      },
      version: {
        id: "version-sfx-variant-impact-tight-a",
        file: {
          storagePath:
            "soundworks-library/projects/project-demo/sfx/asset-sfx-variant-impact-tight/version-sfx-variant-impact-tight-a/media.wav",
          format: "wav",
        },
        technical: {
          sampleRateHz: 48000,
          channels: 1,
          durationMs: 1800,
          loudnessLufs: -17.5,
          loopPoints: null,
        },
      },
      exported: true,
      waveformPreviewReady: true,
    },
    {
      variantId: "sfx-variant-engine-room-bed",
      asset: {
        id: "asset-sfx-variant-engine-room-bed",
        kind: "ambience",
        name: "Engine room bed",
        tags: ["ambience", "engine-room", "foley", "loopable", "metal"],
        currentVersionId: "version-sfx-variant-engine-room-bed-a",
      },
      version: {
        id: "version-sfx-variant-engine-room-bed-a",
        file: {
          storagePath:
            "soundworks-library/projects/project-demo/ambience/asset-sfx-variant-engine-room-bed/version-sfx-variant-engine-room-bed-a/media.wav",
          format: "wav",
        },
        technical: {
          sampleRateHz: 48000,
          channels: 2,
          durationMs: 8000,
          loudnessLufs: -20,
          loopPoints: {
            startSample: 2400,
            endSample: 381600,
          },
        },
      },
      exported: true,
      waveformPreviewReady: true,
    },
  ],
  postProcessingActions: [
    {
      id: "trim-silence",
      operation: "trim",
      enabled: true,
      summary: "Trim leading and trailing silence before saving variants.",
    },
    {
      id: "normalize",
      operation: "normalize",
      enabled: true,
      summary: "Normalize selected variants for preview and export.",
    },
    {
      id: "fade-loop",
      operation: "fade",
      enabled: true,
      summary: "Apply fades and loop crossfade where loop points are present.",
    },
    {
      id: "convert-export",
      operation: "convert-format",
      enabled: true,
      summary:
        "Export WAV assets with metadata sidecars and recipe provenance.",
    },
  ],
  validationChecks: [
    {
      id: "sfx.provider_capabilities",
      status: "passed",
      summary:
        "Available controls are derived from SFX and ambience provider capabilities.",
    },
    {
      id: "sfx.variant_comparison",
      status: "passed",
      summary:
        "Multiple variants can be previewed, compared, selected, tagged, and saved.",
    },
    {
      id: "sfx.loop_points",
      status: "passed",
      summary: "Loopable ambience output includes inspectable loop points.",
    },
    {
      id: "sfx.multimodal_boundary",
      status: "passed",
      summary:
        "AudioX, MMAudio, and ThinkSound remain deferred to the video-to-audio story.",
    },
  ],
};

export const fallbackVideoToAudio: VideoToAudioOverview = {
  schemaVersion: 1,
  projectId: "project-demo",
  source: {
    videoReferenceId: "ref-video-airlock-approach",
    videoAssetId: "asset-reference-video-airlock",
    filename: "airlock-approach-silent.mp4",
    durationMs: 14400,
    frameRate: "24 fps",
    resolution: "1920x1080",
    hasSourceAudio: false,
    imageReferenceIds: ["ref-keyframe-door-panel"],
    referenceAudioAssetIds: ["asset-reference-metal-room-tone"],
    ownershipAttestation: "User-owned previs clip cleared for generated Foley.",
  },
  direction: {
    prompt:
      "Generate synchronized sci-fi airlock Foley: servo whirr, boot steps, metal hatch hit, pressure seal, and low room tone.",
    negativePrompt: "music, dialogue, crowd, melody, alarm loop",
    syncMode: "frame-synchronized",
    requestedOutputs: ["sfx", "ambience"],
    durationMs: 14400,
    regeneratePolicy: "selected-ranges",
    exportTarget: "SceneWorks video audio track package",
  },
  targetRanges: [
    {
      id: "range-door-servo",
      label: "Door servo",
      range: { startMs: 1000, endMs: 3300 },
      objectLabel: "airlock hatch",
      region: { x: 0.58, y: 0.18, width: 0.28, height: 0.55 },
      requestedAction: "mechanical motor ramp with subtle metal resonance",
    },
    {
      id: "range-footsteps",
      label: "Boot steps",
      range: { startMs: 3650, endMs: 6900 },
      objectLabel: "crew boots",
      region: { x: 0.28, y: 0.58, width: 0.22, height: 0.3 },
      requestedAction: "three dampened footsteps aligned to contact frames",
    },
    {
      id: "range-pressure-seal",
      label: "Pressure seal",
      range: { startMs: 8200, endMs: 11600 },
      objectLabel: "door gasket",
      region: null,
      requestedAction: "hatch impact, air hiss, and pressure release tail",
    },
  ],
  detectedEvents: [
    {
      id: "event-servo-start",
      label: "Servo start",
      atMs: 1120,
      confidence: 0.86,
      objectLabel: "airlock hatch",
      requestedSound: "servo ramp",
    },
    {
      id: "event-step-1",
      label: "Footstep contact",
      atMs: 3920,
      confidence: 0.81,
      objectLabel: "crew boots",
      requestedSound: "boot step",
    },
    {
      id: "event-step-2",
      label: "Footstep contact",
      atMs: 5180,
      confidence: 0.79,
      objectLabel: "crew boots",
      requestedSound: "boot step",
    },
    {
      id: "event-hatch-hit",
      label: "Hatch lock",
      atMs: 8640,
      confidence: 0.88,
      objectLabel: "door gasket",
      requestedSound: "metal impact",
    },
    {
      id: "event-pressure-tail",
      label: "Pressure tail",
      atMs: 10900,
      confidence: 0.74,
      objectLabel: null,
      requestedSound: "air hiss",
    },
  ],
  providerOptions: [
    {
      providerId: "soundworks-reference",
      modelId: "reference-generation-suite",
      modelVersion: "0.1.0",
      displayName:
        "SoundWorks Reference Capability Registry / Reference Audio Generation Suite",
      workflow: "video-to-audio",
      runtime: "local",
      runnable: true,
      installStatus: "packaged",
      outputAssetKinds: ["sfx", "ambience"],
      outputFormat: "wav",
      sampleRateHz: 48000,
      channelLayout: "stereo",
      supportsVideo: true,
      supportsText: true,
      supportsReferenceAudio: false,
      supportsRangeRefinement: true,
      supportsObjectRegions: true,
      commercialUseAllowed: true,
      limitations: [
        "Reference provider proves workflow contract only; real video-to-audio adapters remain model-integration work.",
      ],
    },
  ],
  selectedProvider: {
    providerId: "soundworks-reference",
    modelId: "reference-generation-suite",
    modelVersion: "0.1.0",
    workflow: "video-to-audio",
    runtime: "local",
    accepted: true,
    blocker: null,
  },
  providerScorecards: [
    {
      candidateId: "mmaudio",
      name: "MMAudio",
      provider: "MMAudio authors",
      lanes: ["video-to-audio", "sfx"],
      status: "promising-spike",
      productEligibility: "research-only",
      readiness: "research-only",
      runtimePath: "python-poc-only",
      commercialUse: "unknown",
      recommended: true,
      supports: ["video-conditioning", "frame-sync", "text-to-sfx"],
      blockers: ["License terms and no-Python product path unresolved"],
      notes: "Primary video-to-audio benchmark candidate.",
    },
    {
      candidateId: "audiox",
      name: "AudioX",
      provider: "AudioX authors",
      lanes: ["sfx", "video-to-audio", "ambience"],
      status: "promising-spike",
      productEligibility: "research-only",
      readiness: "research-only",
      runtimePath: "python-poc-only",
      commercialUse: "unknown",
      recommended: false,
      supports: [
        "text-to-sfx",
        "video-conditioning",
        "image-conditioning",
        "reference-audio-conditioning",
      ],
      blockers: ["License, weights, and runtime packaging need validation"],
      notes: "Good multimodal benchmark; not product-ready.",
    },
    {
      candidateId: "thinksound",
      name: "ThinkSound",
      provider: "FunAudioLLM",
      lanes: ["video-to-audio", "sfx", "ambience"],
      status: "promising-spike",
      productEligibility: "research-only",
      readiness: "research-only",
      runtimePath: "python-poc-only",
      commercialUse: "unknown",
      recommended: false,
      supports: [
        "video-conditioning",
        "object-region-refinement",
        "natural-language-editing",
        "frame-sync",
      ],
      blockers: ["License and dependency footprint unresolved"],
      notes: "Useful for reasoning-heavy video-to-audio comparisons.",
    },
    {
      candidateId: "moss-soundeffect",
      name: "MOSS-SoundEffect",
      provider: "OpenMOSS",
      lanes: ["sfx", "ambience"],
      status: "promising-spike",
      productEligibility: "product-candidate",
      readiness: "text-only-sfx",
      runtimePath: "native-library-binding",
      commercialUse: "allowed",
      recommended: false,
      supports: ["text-to-sfx"],
      blockers: [
        "Text-to-SFX candidate only; use for Foley bed comparison, not video-conditioned sync.",
      ],
      notes:
        "Best first SFX spike because an Apache-licensed MLX path exists for local Mac validation.",
    },
  ],
  syncPreview: {
    id: "preview-airlock-foley-sync",
    durationMs: 14400,
    sampleRateHz: 48000,
    channelLayout: "stereo",
    waveformPreviewPath:
      "soundworks-library/projects/project-demo/sfx/asset-video-airlock-foley/version-video-airlock-foley-a/previews/waveform.json",
    syncPoints: [
      {
        id: "sync-event-servo-start",
        atMs: 1120,
        label: "Servo start",
        confidence: 0.86,
      },
      {
        id: "sync-event-step-1",
        atMs: 3920,
        label: "Footstep contact",
        confidence: 0.81,
      },
      {
        id: "sync-event-step-2",
        atMs: 5180,
        label: "Footstep contact",
        confidence: 0.79,
      },
      {
        id: "sync-event-hatch-hit",
        atMs: 8640,
        label: "Hatch lock",
        confidence: 0.88,
      },
      {
        id: "sync-event-pressure-tail",
        atMs: 10900,
        label: "Pressure tail",
        confidence: 0.74,
      },
    ],
    segments: [
      {
        id: "segment-servo",
        targetRangeId: "range-door-servo",
        label: "Servo ramp",
        range: { startMs: 1000, endMs: 3300 },
        assetKind: "sfx",
        syncConfidence: 0.84,
        editable: true,
      },
      {
        id: "segment-steps",
        targetRangeId: "range-footsteps",
        label: "Boot steps",
        range: { startMs: 3650, endMs: 6900 },
        assetKind: "sfx",
        syncConfidence: 0.78,
        editable: true,
      },
      {
        id: "segment-pressure",
        targetRangeId: "range-pressure-seal",
        label: "Pressure seal",
        range: { startMs: 8200, endMs: 11600 },
        assetKind: "sfx",
        syncConfidence: 0.86,
        editable: true,
      },
      {
        id: "segment-room-tone",
        targetRangeId: "range-full-bed",
        label: "Room tone bed",
        range: { startMs: 0, endMs: 14400 },
        assetKind: "ambience",
        syncConfidence: 0.72,
        editable: true,
      },
    ],
    warnings: [
      "Real model sync confidence must be replaced with generated output analysis before release.",
      "Reference audio is captured as provenance; current provider option does not condition on it.",
    ],
  },
  submission: {
    canSubmit: true,
    job: {
      id: "job-video-airlock-foley",
      recipeId: "recipe-video-airlock-foley",
      kind: "generate-audio",
      status: "queued",
      outputVersionIds: ["version-video-airlock-foley-a"],
      error: null,
    },
    recipe: {
      id: "recipe-video-airlock-foley",
      workflow: "video-to-audio",
      outputAssetIds: ["asset-video-airlock-foley"],
    },
    blockingReasons: [],
    warnings: [
      "Real model sync confidence must be replaced with generated output analysis before release.",
      "Reference audio is captured as provenance; current provider option does not condition on it.",
      "Reference contract is queueable, but real generated audio quality is not proven.",
    ],
  },
  savedOutput: {
    asset: {
      id: "asset-video-airlock-foley",
      kind: "sfx",
      name: "Airlock synchronized Foley",
      tags: ["video-to-audio", "foley", "airlock", "sync"],
      currentVersionId: "version-video-airlock-foley-a",
    },
    version: {
      id: "version-video-airlock-foley-a",
      file: {
        storagePath:
          "soundworks-library/projects/project-demo/sfx/asset-video-airlock-foley/version-video-airlock-foley-a/media.wav",
        format: "wav",
      },
      technical: {
        sampleRateHz: 48000,
        channels: 2,
        durationMs: 14400,
        loudnessLufs: -18,
      },
    },
    waveformPreviewReady: true,
    synchronizedToVideo: true,
  },
  exportPackage: {
    id: "export-video-airlock-foley",
    mixdownPath:
      "soundworks-exports/project-demo/airlock-approach/foley-mixdown.wav",
    sidecarPath:
      "soundworks-exports/project-demo/airlock-approach/video-to-audio-provenance.json",
    includesSyncPoints: true,
    includesSourceMediaRefs: true,
    includesDetectedEvents: true,
    includesRights: true,
    destinationTargets: [
      "SoundWorks composition timeline",
      "SceneWorks video audio-track package",
    ],
    requiredFields: [
      "sourceVideoReferenceId",
      "sourceProjectId",
      "timeRanges",
      "syncPoints",
      "modelProvider",
      "sourceMediaRights",
      "aiDisclosureRequired",
    ],
  },
  provenance: {
    recipeId: "recipe-video-airlock-foley",
    sourceReferenceIds: [
      "ref-video-airlock-approach",
      "ref-keyframe-door-panel",
      "ref-asset-reference-metal-room-tone",
    ],
    sidecarPath:
      "soundworks-library/projects/project-demo/sfx/asset-video-airlock-foley/version-video-airlock-foley-a/metadata/recipe-provenance.json",
    capturedFields: [
      "source video asset and ownership attestation",
      "image keyframe references",
      "reference audio asset IDs",
      "time ranges and object labels",
      "3 targeted Foley ranges",
      "sync points and confidence scores",
      "provider/model/runtime and license gate state",
    ],
    roundTripNotes: [
      "Saved output can be dragged into the multitrack editor as synchronized SFX.",
      "SceneWorks handoff package can reuse the SC-6202 manifest shape once target import code is implemented in SceneWorks.",
    ],
  },
  safetyGates: [
    {
      id: "source-media-rights",
      status: "passed",
      summary: "Source video is user-owned and cleared for generated Foley.",
      enforcement:
        "Allow generation and preserve ownership note in the sidecar.",
    },
    {
      id: "protected-media-imitation",
      status: "passed",
      summary:
        "Prompt avoids requests to imitate protected film, game, or library sounds.",
      enforcement:
        "Block export if protected-media imitation language is introduced.",
    },
    {
      id: "real-provider-audio",
      status: "warning",
      summary:
        "Reference contract is queueable, but real generated audio quality is not proven.",
      enforcement:
        "Keep provider scorecards research-only until runnable smoke output is attached.",
    },
  ],
  validationChecks: [
    {
      id: "video_audio.source_inputs",
      status: "passed",
      summary:
        "Workflow captures video, image keyframe, reference audio, and natural-language direction inputs.",
    },
    {
      id: "video_audio.range_refinement",
      status: "passed",
      summary:
        "Target ranges preserve time spans, object labels, optional regions, and requested sounds.",
    },
    {
      id: "video_audio.capability_boundary",
      status: "passed",
      summary:
        "Provider scorecards distinguish video-conditioned candidates from text-only SFX candidates.",
    },
    {
      id: "video_audio.export_sidecar",
      status: "passed",
      summary:
        "Export package includes source media, sync points, detected events, rights, and disclosure fields.",
    },
    {
      id: "video_audio.real_provider_evidence",
      status: "warning",
      summary:
        "Real provider adapters and generated audio bytes still require later runnable model integration.",
    },
  ],
};

export const fallbackSamplesStudio: SamplesStudioOverview = {
  schemaVersion: 1,
  prompt: {
    id: "prompt-sample-pack-synth-bass",
    text: "Tight analog synth bass one-shots and a four-bar driving loop for a neon chase cue.",
    negativePrompt: "full song, vocal, crowd, reverb wash",
    instrumentFamily: "synth-bass",
    articulation: "pluck and short sustain",
    genreTags: ["synthwave", "game-score", "bass"],
    referenceAudioAssetId: "asset-reference-neon-bass",
  },
  controls: {
    musicalKey: "A minor",
    scale: "natural minor",
    bpm: 120,
    bars: 4,
    beats: 4,
    loopable: true,
    dryWetAmbience: 18,
    velocityEnergy: 76,
    variationCount: 4,
    batchSize: 6,
    promoteToProjectLibrary: true,
  },
  providerOptions: [
    {
      providerId: "soundworks-reference",
      modelId: "reference-generation-suite",
      modelVersion: "0.1.0",
      displayName:
        "SoundWorks Reference Capability Registry / Reference Audio Generation Suite",
      workflow: "instrument-sample",
      runtime: "local",
      installStatus: "packaged",
      runnable: true,
      outputAssetKind: "instrument-sample",
      outputFormat: "wav",
      sampleRateHz: 48000,
      channelLayout: "stereo",
      minDurationMs: 250,
      maxDurationMs: 300000,
      supportsReferenceAudio: false,
      supportsTempo: false,
      supportsKey: true,
      supportsLoopPoints: false,
      commercialUseAllowed: true,
      watermark: "sidecar-only",
      supportedControls: [
        "prompt",
        "instrument-family",
        "articulation",
        "musical-key",
        "scale",
        "dry-wet-ambience",
        "velocity-energy",
        "batch-generation",
      ],
      limitations: [],
    },
    {
      providerId: "soundworks-reference",
      modelId: "reference-generation-suite",
      modelVersion: "0.1.0",
      displayName:
        "SoundWorks Reference Capability Registry / Reference Audio Generation Suite",
      workflow: "loop",
      runtime: "local",
      installStatus: "packaged",
      runnable: true,
      outputAssetKind: "loop",
      outputFormat: "wav",
      sampleRateHz: 48000,
      channelLayout: "stereo",
      minDurationMs: 250,
      maxDurationMs: 300000,
      supportsReferenceAudio: false,
      supportsTempo: true,
      supportsKey: true,
      supportsLoopPoints: true,
      commercialUseAllowed: true,
      watermark: "sidecar-only",
      supportedControls: [
        "prompt",
        "instrument-family",
        "articulation",
        "musical-key",
        "scale",
        "tempo",
        "bars-beats",
        "loopable",
        "dry-wet-ambience",
        "velocity-energy",
        "batch-generation",
      ],
      limitations: [],
    },
  ],
  selectedProvider: {
    providerId: "soundworks-reference",
    modelId: "reference-generation-suite",
    modelVersion: "0.1.0",
    workflow: "loop",
    runtime: "local",
    accepted: true,
    blocker: null,
  },
  providerScorecards: [
    {
      candidateId: "stable-audio-3",
      name: "Stable Audio 3",
      provider: "Stability AI",
      lanes: ["song", "sfx", "ambience"],
      status: "promising-spike",
      productEligibility: "needs-runtime-port",
      readiness: "not-sample-focused",
      runtimePath: "needs-native-port",
      commercialUse: "allowed",
      recommended: false,
      blockers: [
        "Local product runtime path needs validation",
        "Candidate is tracked for adjacent audio lanes, not as a primary sample/loop provider.",
      ],
      notes:
        "Useful comparison baseline, but not enough isolated-sample evidence yet.",
    },
    {
      candidateId: "ace-step-1-5",
      name: "ACE-Step 1.5",
      provider: "ACE-Step",
      lanes: ["song", "loop"],
      status: "promising-spike",
      productEligibility: "needs-runtime-port",
      readiness: "needs-runtime-port",
      runtimePath: "needs-native-port",
      commercialUse: "allowed",
      recommended: true,
      blockers: ["No shipped no-Python runtime path yet"],
      notes: "Primary loop spike candidate for local music generation.",
    },
    {
      candidateId: "heartmula",
      name: "HeartMuLa",
      provider: "HeartMuLa",
      lanes: ["song", "loop"],
      status: "promising-spike",
      productEligibility: "research-only",
      readiness: "research-only",
      runtimePath: "python-poc-only",
      commercialUse: "unknown",
      recommended: false,
      blockers: ["Runtime and license terms need validation"],
      notes: "Model-family candidate for song and loop comparisons.",
    },
    {
      candidateId: "muse-song",
      name: "Muse",
      provider: "Muse authors",
      lanes: ["song"],
      status: "promising-spike",
      productEligibility: "research-only",
      readiness: "not-sample-focused",
      runtimePath: "python-poc-only",
      commercialUse: "unknown",
      recommended: false,
      blockers: ["No isolated sample evidence yet"],
      notes:
        "Long-form song control is useful context, not the first sample-pack path.",
    },
    {
      candidateId: "stable-audio-open-1",
      name: "Stable Audio Open 1.0",
      provider: "Stability AI",
      lanes: ["sfx", "ambience", "loop"],
      status: "promising-spike",
      productEligibility: "needs-runtime-port",
      readiness: "needs-runtime-port",
      runtimePath: "needs-native-port",
      commercialUse: "allowed",
      recommended: false,
      blockers: ["Runtime/package path needs validation"],
      notes: "Good loopability benchmark with SFX overlap.",
    },
  ],
  variants: [
    {
      id: "sample-variant-bass-pluck-a",
      label: "Bass pluck A1",
      workflow: "instrument-sample",
      assetKind: "instrument-sample",
      instrumentFamily: "synth-bass",
      articulation: "pluck and short sustain",
      durationMs: 900,
      bpm: null,
      musicalKey: "A1",
      timeSignature: null,
      loopPoints: null,
      transientOneShot: true,
      loudnessLufs: -18,
      truePeakDbfs: -1.4,
      hasClipping: false,
      tags: ["bass", "game-score", "one-shot", "pluck", "synthwave"],
      collectionId: "collection-neon-bass-pack",
      selectedForPack: true,
      favorite: true,
      duplicateOfVariantId: null,
    },
    {
      id: "sample-variant-bass-stab-c2",
      label: "Bass stab C2",
      workflow: "instrument-sample",
      assetKind: "instrument-sample",
      instrumentFamily: "synth-bass",
      articulation: "short stab",
      durationMs: 650,
      bpm: null,
      musicalKey: "C2",
      timeSignature: null,
      loopPoints: null,
      transientOneShot: true,
      loudnessLufs: -17.2,
      truePeakDbfs: -1.1,
      hasClipping: false,
      tags: ["bass", "game-score", "one-shot", "stab", "synthwave"],
      collectionId: "collection-neon-bass-pack",
      selectedForPack: true,
      favorite: false,
      duplicateOfVariantId: null,
    },
    {
      id: "loop-variant-bassline-120a",
      label: "Four-bar chase bassline",
      workflow: "loop",
      assetKind: "loop",
      instrumentFamily: "synth-bass",
      articulation: "pulsed sequence",
      durationMs: 8000,
      bpm: 120,
      musicalKey: "A minor",
      timeSignature: "4/4",
      loopPoints: { startSample: 0, endSample: 352800 },
      transientOneShot: false,
      loudnessLufs: -19,
      truePeakDbfs: -2,
      hasClipping: false,
      tags: ["bass", "four-bar", "game-score", "loop", "synthwave"],
      collectionId: "collection-neon-bass-pack",
      selectedForPack: true,
      favorite: true,
      duplicateOfVariantId: null,
    },
    {
      id: "loop-variant-bassline-drier",
      label: "Dry alternate bassline",
      workflow: "loop",
      assetKind: "loop",
      instrumentFamily: "synth-bass",
      articulation: "dry pulsed sequence",
      durationMs: 8000,
      bpm: 120,
      musicalKey: "A minor",
      timeSignature: "4/4",
      loopPoints: { startSample: 0, endSample: 352800 },
      transientOneShot: false,
      loudnessLufs: -20,
      truePeakDbfs: -2.8,
      hasClipping: false,
      tags: ["bass", "dry", "game-score", "loop", "synthwave"],
      collectionId: "collection-neon-bass-pack",
      selectedForPack: false,
      favorite: false,
      duplicateOfVariantId: "loop-variant-bassline-120a",
    },
  ],
  pack: {
    collectionId: "collection-neon-bass-pack",
    name: "Neon bass starter pack",
    variantCount: 4,
    selectedVariantIds: [
      "sample-variant-bass-pluck-a",
      "sample-variant-bass-stab-c2",
      "loop-variant-bassline-120a",
    ],
    favoriteVariantIds: [
      "sample-variant-bass-pluck-a",
      "loop-variant-bassline-120a",
    ],
    loopVariantIds: [
      "loop-variant-bassline-120a",
      "loop-variant-bassline-drier",
    ],
    oneShotVariantIds: [
      "sample-variant-bass-pluck-a",
      "sample-variant-bass-stab-c2",
    ],
    exportFormats: ["wav", "flac"],
  },
  submission: {
    canSubmit: false,
    jobs: [
      {
        id: "job-recipe-samples-one-shots-reference",
        recipeId: "recipe-samples-one-shots-reference",
        kind: "generate-audio",
        status: "failed",
        outputVersionIds: [],
        error:
          "Manifest declares a packaged or installed model, but no verified cache/package evidence is attached.",
      },
      {
        id: "job-recipe-loops-four-bar-reference",
        recipeId: "recipe-loops-four-bar-reference",
        kind: "generate-audio",
        status: "failed",
        outputVersionIds: [],
        error:
          "Manifest declares a packaged or installed model, but no verified cache/package evidence is attached.",
      },
    ],
    recipes: [
      {
        id: "recipe-samples-one-shots-reference",
        workflow: "instrument-sample",
        outputAssetIds: [
          "asset-sample-variant-bass-pluck-a",
          "asset-sample-variant-bass-stab-c2",
        ],
      },
      {
        id: "recipe-loops-four-bar-reference",
        workflow: "loop",
        outputAssetIds: ["asset-loop-variant-bassline-120a"],
      },
    ],
    blockingReasons: [
      "Manifest declares a packaged or installed model, but no verified cache/package evidence is attached.",
    ],
    warnings: [
      "Reference audio is stored with provenance but unavailable as a provider input.",
      "BPM is stored with the recipe and validated after generation.",
    ],
  },
  savedOutputs: [
    {
      variantId: "sample-variant-bass-pluck-a",
      asset: {
        id: "asset-sample-variant-bass-pluck-a",
        kind: "instrument-sample",
        name: "Bass pluck A1",
        tags: ["bass", "game-score", "one-shot", "pluck", "synthwave"],
        currentVersionId: "version-sample-variant-bass-pluck-a-a",
      },
      version: {
        id: "version-sample-variant-bass-pluck-a-a",
        file: {
          storagePath:
            "soundworks-library/projects/project-demo/instrument-samples/asset-sample-variant-bass-pluck-a/version-sample-variant-bass-pluck-a-a/media.wav",
          format: "wav",
        },
        technical: {
          sampleRateHz: 48000,
          channels: 1,
          durationMs: 900,
          loudnessLufs: -18,
          bpm: null,
          musicalKey: "A1",
          loopPoints: null,
        },
      },
      exported: true,
      waveformPreviewReady: true,
    },
    {
      variantId: "sample-variant-bass-stab-c2",
      asset: {
        id: "asset-sample-variant-bass-stab-c2",
        kind: "instrument-sample",
        name: "Bass stab C2",
        tags: ["bass", "game-score", "one-shot", "stab", "synthwave"],
        currentVersionId: "version-sample-variant-bass-stab-c2-a",
      },
      version: {
        id: "version-sample-variant-bass-stab-c2-a",
        file: {
          storagePath:
            "soundworks-library/projects/project-demo/instrument-samples/asset-sample-variant-bass-stab-c2/version-sample-variant-bass-stab-c2-a/media.wav",
          format: "wav",
        },
        technical: {
          sampleRateHz: 48000,
          channels: 1,
          durationMs: 650,
          loudnessLufs: -17.2,
          bpm: null,
          musicalKey: "C2",
          loopPoints: null,
        },
      },
      exported: true,
      waveformPreviewReady: true,
    },
    {
      variantId: "loop-variant-bassline-120a",
      asset: {
        id: "asset-loop-variant-bassline-120a",
        kind: "loop",
        name: "Four-bar chase bassline",
        tags: ["bass", "four-bar", "game-score", "loop", "synthwave"],
        currentVersionId: "version-loop-variant-bassline-120a-a",
      },
      version: {
        id: "version-loop-variant-bassline-120a-a",
        file: {
          storagePath:
            "soundworks-library/projects/project-demo/loops/asset-loop-variant-bassline-120a/version-loop-variant-bassline-120a-a/media.wav",
          format: "wav",
        },
        technical: {
          sampleRateHz: 48000,
          channels: 2,
          durationMs: 8000,
          loudnessLufs: -19,
          bpm: 120,
          musicalKey: "A minor",
          loopPoints: { startSample: 0, endSample: 352800 },
        },
      },
      exported: true,
      waveformPreviewReady: true,
    },
  ],
  postProcessingActions: [
    {
      id: "trim-silence",
      operation: "trim",
      enabled: true,
      summary: "Trim one-shot heads/tails without damaging transients.",
    },
    {
      id: "normalize",
      operation: "normalize",
      enabled: true,
      summary: "Normalize samples and loops for audition/export consistency.",
    },
    {
      id: "loop-seam",
      operation: "fade",
      enabled: true,
      summary: "Check loop seam and apply short crossfade when required.",
    },
    {
      id: "pack-export",
      operation: "convert-format",
      enabled: true,
      summary:
        "Export sample-pack WAV/FLAC files with BPM/key/provenance sidecars.",
    },
  ],
  qaChecks: [
    {
      id: "samples.provider_capabilities",
      status: "passed",
      summary:
        "Instrument, tempo, key, loop, and batch controls come from provider capabilities.",
    },
    {
      id: "samples.isolation",
      status: "passed",
      summary: "One-shot variants track transient/sample isolation metadata.",
    },
    {
      id: "samples.loop_seam",
      status: "passed",
      summary:
        "Loop variants include BPM, key, bar count, and inspectable loop points.",
    },
    {
      id: "samples.audio_quality",
      status: "passed",
      summary:
        "Clipping, silence, loudness, and duration mismatch checks are represented.",
    },
  ],
};

export const fallbackSongStudio: SongStudioOverview = {
  schemaVersion: 1,
  draft: {
    id: "song-draft-city-lights",
    title: "City Lights Resolve",
    prompt:
      "Cinematic synth-pop song with a confident female lead, warm analog pads, tight electronic drums, and a final lift for the chorus.",
    lyrics:
      "Verse:\nStreetlights hum under rain on glass\nI keep the tempo of a moving train\n\nChorus:\nCity lights, carry me home\nTurn the static into gold",
    styleTags: ["synth-pop", "cinematic", "female-vocal", "120-bpm"],
    language: "en-US",
    vocalist: "vocal",
    singerHint: "clear alto lead with restrained vibrato",
    referenceAudioAssetIds: [
      "asset-reference-synth-pad",
      "asset-reference-drum-groove",
    ],
    sections: [
      {
        id: "intro",
        label: "Intro",
        bars: 8,
        lyrics: null,
        regenerateLocked: false,
      },
      {
        id: "verse-1",
        label: "Verse 1",
        bars: 16,
        lyrics:
          "Streetlights hum under rain on glass\nI keep the tempo of a moving train",
        regenerateLocked: false,
      },
      {
        id: "chorus-1",
        label: "Chorus 1",
        bars: 16,
        lyrics: "City lights, carry me home\nTurn the static into gold",
        regenerateLocked: false,
      },
      {
        id: "outro",
        label: "Outro",
        bars: 8,
        lyrics: null,
        regenerateLocked: false,
      },
    ],
  },
  controls: {
    bpm: 120,
    musicalKey: "A minor",
    timeSignature: "4/4",
    targetDurationMs: 96000,
    sectionLengthBars: 16,
    variationCount: 2,
    generateStems: true,
    requestedStems: ["full-mix", "vocals", "drums", "bass", "instruments"],
    allowReferenceAudio: true,
    promoteToProjectLibrary: true,
  },
  providerOptions: [
    {
      providerId: "soundworks-reference",
      modelId: "reference-generation-suite",
      modelVersion: "0.1.0",
      displayName:
        "SoundWorks Reference Capability Registry / Reference Audio Generation Suite",
      workflow: "song",
      runtime: "local",
      installStatus: "packaged",
      runnable: true,
      outputAssetKinds: ["song", "stem"],
      outputFormat: "wav",
      sampleRateHz: 48000,
      channelLayout: "stems",
      minDurationMs: 250,
      maxDurationMs: 300000,
      supportsLyrics: true,
      supportsStyleTags: true,
      supportsReferenceAudio: false,
      supportsStems: true,
      supportedStems: ["vocals", "drums", "bass", "instruments"],
      commercialUseAllowed: true,
      watermark: "sidecar-only",
      supportedControls: [
        "prompt",
        "section-structure",
        "vocal-mode",
        "singer-hint",
        "duration",
        "variants",
        "lyrics",
        "style-tags",
        "stems",
      ],
      limitations: [],
    },
  ],
  selectedProvider: {
    providerId: "soundworks-reference",
    modelId: "reference-generation-suite",
    modelVersion: "0.1.0",
    workflow: "song",
    runtime: "local",
    accepted: true,
    blocker: null,
  },
  providerScorecards: [
    {
      candidateId: "stable-audio-3",
      name: "Stable Audio 3",
      provider: "Stability AI",
      lanes: ["song", "sfx", "ambience"],
      status: "promising-spike",
      productEligibility: "needs-runtime-port",
      readiness: "needs-runtime-port",
      runtimePath: "external-executable",
      commercialUse: "provider-terms",
      recommended: false,
      blockers: [
        "Community license and enterprise threshold must be enforced before product enablement",
      ],
      notes:
        "Strong broad audio candidate, but SoundWorks needs runnable Mac/Windows packaging proof and license gating.",
    },
    {
      candidateId: "ace-step-1-5",
      name: "ACE-Step 1.5",
      provider: "ACE-Step",
      lanes: ["song", "loop"],
      status: "promising-spike",
      productEligibility: "needs-runtime-port",
      readiness: "needs-runtime-port",
      runtimePath: "external-executable",
      commercialUse: "allowed",
      recommended: true,
      blockers: [
        "Local runtime must be isolated as a provider package rather than bundled as Python product dependency",
      ],
      notes:
        "High-priority music spike because license is permissive and local execution is documented.",
    },
    {
      candidateId: "levo-2",
      name: "LeVo 2 / SongGeneration 2",
      provider: "Tencent AI Lab",
      lanes: ["song"],
      status: "promising-spike",
      productEligibility: "research-only",
      readiness: "research-only",
      runtimePath: "python-poc-only",
      commercialUse: "unknown",
      recommended: false,
      blockers: ["License terms and no-Python product path are unresolved"],
      notes:
        "Useful comparison target for complete-song quality; not product-eligible without license/runtime work.",
    },
    {
      candidateId: "yue",
      name: "YuE",
      provider: "M-A-P",
      lanes: ["song"],
      status: "promising-spike",
      productEligibility: "research-only",
      readiness: "research-only",
      runtimePath: "python-poc-only",
      commercialUse: "unknown",
      recommended: false,
      blockers: ["Compute footprint and product license must be resolved"],
      notes:
        "Long-form lyrics-to-song baseline; compare against faster candidates before product work.",
    },
    {
      candidateId: "diffrhythm-2",
      name: "DiffRhythm 2",
      provider: "ASLP Lab / Xiaomi Research",
      lanes: ["song"],
      status: "promising-spike",
      productEligibility: "research-only",
      readiness: "research-only",
      runtimePath: "python-poc-only",
      commercialUse: "unknown",
      recommended: false,
      blockers: ["No no-Python runtime path yet"],
      notes:
        "Candidate for fast complete-song smoke tests, but currently research-only for SoundWorks.",
    },
    {
      candidateId: "khala",
      name: "Khala",
      provider: "Khala Music AI",
      lanes: ["song"],
      status: "promising-spike",
      productEligibility: "research-only",
      readiness: "research-only",
      runtimePath: "python-poc-only",
      commercialUse: "unknown",
      recommended: false,
      blockers: ["License and runtime isolation are unresolved"],
      notes:
        "Promising full-song system for comparison, not a product candidate yet.",
    },
    {
      candidateId: "heartmula",
      name: "HeartMuLa",
      provider: "HeartMuLa",
      lanes: ["song", "loop"],
      status: "promising-spike",
      productEligibility: "research-only",
      readiness: "research-only",
      runtimePath: "python-poc-only",
      commercialUse: "unknown",
      recommended: false,
      blockers: ["Mac/Windows packaging evidence missing"],
      notes: "Model-family candidate for song and loop comparisons.",
    },
    {
      candidateId: "muse-song",
      name: "Muse",
      provider: "Muse authors",
      lanes: ["song"],
      status: "promising-spike",
      productEligibility: "research-only",
      readiness: "research-only",
      runtimePath: "python-poc-only",
      commercialUse: "unknown",
      recommended: false,
      blockers: ["License, runtime, and artifact packaging need validation"],
      notes: "Useful long-form style-control benchmark.",
    },
  ],
  arrangement: {
    sectionCount: 4,
    totalBars: 48,
    estimatedDurationMs: 96000,
    sections: [
      {
        id: "intro",
        label: "Intro",
        startBar: 0,
        bars: 8,
        hasLyrics: false,
        locked: false,
      },
      {
        id: "verse-1",
        label: "Verse 1",
        startBar: 8,
        bars: 16,
        hasLyrics: true,
        locked: false,
      },
      {
        id: "chorus-1",
        label: "Chorus 1",
        startBar: 24,
        bars: 16,
        hasLyrics: true,
        locked: false,
      },
      {
        id: "outro",
        label: "Outro",
        startBar: 40,
        bars: 8,
        hasLyrics: false,
        locked: false,
      },
    ],
  },
  variants: [
    {
      id: "song-variant-city-lights-main",
      label: "City Lights full mix",
      assetKind: "song",
      durationMs: 96000,
      bpm: 120,
      musicalKey: "A minor",
      vocalMode: "vocal",
      stemKinds: ["full-mix", "vocals", "drums", "bass", "instruments"],
      loudnessLufs: -14,
      truePeakDbfs: -1,
      lyricAlignmentScore: 86,
      structureMatchScore: 91,
      tags: [
        "120-bpm",
        "cinematic",
        "complete",
        "female-vocal",
        "song",
        "synth-pop",
      ],
      selectedForSave: true,
    },
    {
      id: "song-variant-city-lights-instrumental",
      label: "City Lights instrumental pass",
      assetKind: "music-clip",
      durationMs: 96000,
      bpm: 120,
      musicalKey: "A minor",
      vocalMode: "instrumental",
      stemKinds: ["drums", "bass", "instruments", "effects"],
      loudnessLufs: -15.5,
      truePeakDbfs: -1.4,
      lyricAlignmentScore: 0,
      structureMatchScore: 88,
      tags: [
        "120-bpm",
        "alternate",
        "cinematic",
        "female-vocal",
        "instrumental",
        "synth-pop",
      ],
      selectedForSave: true,
    },
  ],
  submission: {
    canSubmit: false,
    job: {
      id: "job-song-studio-reference",
      recipeId: "recipe-song-city-lights-reference",
      kind: "generate-audio",
      status: "failed",
      outputVersionIds: [],
      error: "No runnable complete-song provider is registered.",
    },
    recipe: {
      id: "recipe-song-city-lights-reference",
      workflow: "song",
      outputAssetIds: [
        "asset-song-variant-city-lights-main",
        "asset-song-variant-city-lights-instrumental",
      ],
    },
    blockingReasons: ["No runnable complete-song provider is registered."],
    warnings: [
      "Reference audio is stored with provenance but unavailable as a provider input.",
    ],
  },
  savedOutputs: [
    {
      variantId: "song-variant-city-lights-main",
      asset: {
        id: "asset-song-variant-city-lights-main",
        kind: "song",
        name: "City Lights full mix",
        tags: [
          "120-bpm",
          "cinematic",
          "complete",
          "female-vocal",
          "song",
          "synth-pop",
        ],
        currentVersionId: "version-song-variant-city-lights-main-a",
      },
      version: {
        id: "version-song-variant-city-lights-main-a",
        file: {
          storagePath:
            "soundworks-library/projects/project-demo/songs/asset-song-variant-city-lights-main/version-song-variant-city-lights-main-a/media.wav",
          format: "wav",
        },
        technical: {
          sampleRateHz: 48000,
          channels: 2,
          durationMs: 96000,
          loudnessLufs: -14,
          truePeakDbfs: -1,
          bpm: 120,
          musicalKey: "A minor",
        },
      },
      exportReady: true,
      waveformPreviewReady: true,
    },
    {
      variantId: "song-variant-city-lights-instrumental",
      asset: {
        id: "asset-song-variant-city-lights-instrumental",
        kind: "music-clip",
        name: "City Lights instrumental pass",
        tags: [
          "120-bpm",
          "alternate",
          "cinematic",
          "instrumental",
          "synth-pop",
        ],
        currentVersionId: "version-song-variant-city-lights-instrumental-a",
      },
      version: {
        id: "version-song-variant-city-lights-instrumental-a",
        file: {
          storagePath:
            "soundworks-library/projects/project-demo/music-clips/asset-song-variant-city-lights-instrumental/version-song-variant-city-lights-instrumental-a/media.wav",
          format: "wav",
        },
        technical: {
          sampleRateHz: 48000,
          channels: 2,
          durationMs: 96000,
          loudnessLufs: -15.5,
          truePeakDbfs: -1.4,
          bpm: 120,
          musicalKey: "A minor",
        },
      },
      exportReady: true,
      waveformPreviewReady: true,
    },
  ],
  exportTargets: [
    {
      id: "song-master",
      label: "Song master",
      formats: ["wav", "flac", "mp3"],
      includesStems: false,
      includesSidecar: true,
      summary:
        "Export mastered WAV/FLAC/MP3 with recipe, license, and disclosure sidecar.",
    },
    {
      id: "song-stems",
      label: "Stem bundle",
      formats: ["wav", "flac"],
      includesStems: true,
      includesSidecar: true,
      summary:
        "Export vocal, drums, bass, and instrument stems when provider or separator supports them.",
    },
    {
      id: "composition-source",
      label: "Send to multitrack",
      formats: ["wav"],
      includesStems: true,
      includesSidecar: true,
      summary:
        "Promote generated song and stems into the SoundWorks composition editor.",
    },
  ],
  qaChecks: [
    {
      id: "songs.capability_controls",
      status: "passed",
      summary:
        "Lyrics, style, reference audio, duration, and stem controls are gated by provider capabilities.",
    },
    {
      id: "songs.recipe_provenance",
      status: "passed",
      summary:
        "Song recipes preserve lyrics, section structure, references, seeds, provider metadata, and outputs.",
    },
    {
      id: "songs.preview_versioning",
      status: "passed",
      summary:
        "Output song variants are represented as previewable, versioned assets with waveform sidecars.",
    },
    {
      id: "songs.provider_gates",
      status: "warning",
      summary:
        "Stable Audio 3 and ACE-Step need runnable Mac/Windows smoke evidence before product enablement.",
    },
  ],
};
