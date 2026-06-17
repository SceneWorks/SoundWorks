import type { AppOverview } from "./types";

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
        status: "planned",
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
};
