import type { AppOverview, RuntimeOverview } from "./types";

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
    { id: "tts", name: "TTS Studio", route: "/studios/tts", status: "planned" },
    {
      id: "voice-lab",
      name: "Voice Lab",
      route: "/studios/voice-lab",
      status: "planned",
    },
    {
      id: "sfx",
      name: "SFX + Ambience",
      route: "/studios/sfx",
      status: "planned",
    },
    {
      id: "loops",
      name: "Samples + Loops",
      route: "/studios/loops",
      status: "planned",
    },
    {
      id: "songs",
      name: "Song Studio",
      route: "/studios/songs",
      status: "planned",
    },
    {
      id: "video-to-audio",
      name: "Video to Audio",
      route: "/studios/video-to-audio",
      status: "planned",
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
      name: "get_runtime_overview",
      direction: "ui-to-backend",
      purpose:
        "Report worker runtime policy, device/model state, job progress, and cancellation readiness.",
    },
    {
      name: "get_model_evaluation_catalog",
      direction: "ui-to-backend",
      purpose:
        "Load source-backed model scorecards, fixtures, recommendation status, and product eligibility gates.",
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
    installed: 3,
    available: 0,
    unavailable: 0,
  },
  modelStates: [
    {
      providerId: "soundworks-reference",
      modelId: "reference-speech-suite",
      modelName: "Reference Speech Suite",
      availability: "installed",
      installStatus: "packaged",
      health: "ready",
      workflows: ["tts", "voice-clone", "voice-conversion"],
      reasons: [],
      cache: {
        status: "ready",
        expectedSizeMb: 2048,
        diskUsageMb: 2048,
        warmup: "cold",
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
      availability: "installed",
      installStatus: "packaged",
      health: "ready",
      workflows: [
        "sfx",
        "ambience",
        "instrument-sample",
        "loop",
        "song",
        "video-to-audio",
      ],
      reasons: [],
      cache: {
        status: "ready",
        expectedSizeMb: 8192,
        diskUsageMb: 8192,
        warmup: "cold",
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
      availability: "installed",
      installStatus: "packaged",
      health: "ready",
      workflows: ["stem-separation", "edit", "composition-render"],
      reasons: [],
      cache: {
        status: "ready",
        expectedSizeMb: 1024,
        diskUsageMb: 1024,
        warmup: "cold",
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
  jobs: [
    {
      id: "job-runtime-reference-generate",
      kind: "generate-audio",
      status: "running",
      providerId: "soundworks-reference",
      modelId: "reference-speech-suite",
      progress: {
        percent: 42,
        message: "Generating preview audio from queued worker contract.",
      },
      cancellation: "cancellable",
      retryCount: 0,
      logTail: [
        "claimed job from local queue",
        "loaded model package from cache",
        "streamed progress event 42%",
      ],
      actionableError: null,
    },
    {
      id: "job-runtime-reference-cache-repair",
      kind: "evaluate-model",
      status: "failed",
      providerId: "soundworks-reference",
      modelId: "reference-speech-suite",
      progress: {
        percent: 0,
        message: "Runtime validation detected a repairable package issue.",
      },
      cancellation: "not-cancellable",
      retryCount: 0,
      logTail: [
        "verified package manifest",
        "detected cache checksum mismatch",
      ],
      actionableError: {
        code: "runtime.cache_mismatch",
        summary: "Model package cache needs repair",
        recovery:
          "Reinstall the provider package or clear the model cache entry before retrying.",
      },
    },
  ],
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
      id: "runtime.devices",
      status: "passed",
      summary: "Runtime device inventory can report available accelerators.",
    },
  ],
};
