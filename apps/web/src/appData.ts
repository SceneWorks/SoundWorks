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
  ],
};
