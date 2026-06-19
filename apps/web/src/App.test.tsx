import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { App } from "./App";

describe("App", () => {
  it("renders the scaffolded SoundWorks workspace", async () => {
    render(<App />);

    expect(screen.getByRole("main")).toBeInTheDocument();
    expect(screen.getAllByText("SoundWorks").length).toBeGreaterThan(0);
    expect(
      await screen.findByRole("button", { name: /TTS Studio/i }),
    ).toBeInTheDocument();
    expect(screen.getByText("get_app_overview")).toBeInTheDocument();
    expect(screen.getByText("Provider Coverage")).toBeInTheDocument();
    expect(screen.getAllByText("Stem Separation").length).toBeGreaterThan(0);
    expect(
      screen.getByRole("heading", { name: "Worker Runtime" }),
    ).toBeInTheDocument();
    expect(screen.getByText("Python runtime: blocked")).toBeInTheDocument();
    expect(screen.getByText("Reference Speech Suite")).toBeInTheDocument();
    expect(
      screen.getAllByText(
        "manifest-only; on-disk cache/package has not been verified",
      ).length,
    ).toBeGreaterThan(0);
    expect(screen.getByText("No runtime jobs")).toBeInTheDocument();
    expect(
      screen.getByText(/Manifest-only packaged\/install states cannot count/i),
    ).toBeInTheDocument();
    expect(
      screen.getByText(/Fixture\/demo actions are blocked/i),
    ).toBeInTheDocument();
    expect(
      screen.getByText(/Inspect the on-disk model cache\/package/i),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("heading", { name: "Evaluation Scorecard" }),
    ).toBeInTheDocument();
    expect(screen.getByText("moss-soundeffect")).toBeInTheDocument();
    expect(
      screen.getByRole("heading", { name: "Launch read" }),
    ).toBeInTheDocument();
    expect(screen.getAllByText("Producer").length).toBeGreaterThan(0);
    expect(
      screen.getByText("Voice profile consent is required before generation."),
    ).toBeInTheDocument();
    expect(screen.getByText("TTS Studio narration draft")).toBeInTheDocument();
    expect(screen.getByTitle("Queue TTS generation")).toBeDisabled();
    expect(
      screen.getByRole("heading", { name: "Consented voice workflows" }),
    ).toBeInTheDocument();
    expect(screen.getByTitle("Queue voice conversion")).toBeEnabled();
    expect(screen.getByText("Zero-shot clone")).toBeInTheDocument();
    expect(screen.getByText("Few-shot fine-tune")).toBeInTheDocument();
    expect(screen.getByText("Voice conversion")).toBeInTheDocument();
    expect(screen.getByText("Narrator converted read")).toBeInTheDocument();
    expect(
      screen.getByText(
        "RVC-style conversion requires source audio and a target voice profile.",
      ),
    ).toBeInTheDocument();
    expect(
      screen.getByText(
        "Score as speech-to-speech voice conversion, not text-to-speech.",
      ),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("heading", { name: "Foley Impact" }),
    ).toBeInTheDocument();
    expect(screen.getByTitle("Queue SFX generation")).toBeDisabled();
    expect(screen.getAllByText("Tight hatch impact").length).toBeGreaterThan(0);
    expect(screen.getAllByText("Engine room bed").length).toBeGreaterThan(0);
    expect(screen.getAllByText("MOSS-SoundEffect").length).toBeGreaterThan(0);
    expect(
      screen.getAllByText(
        "Best first SFX spike because an Apache-licensed MLX path exists for local Mac validation.",
      ).length,
    ).toBeGreaterThan(0);
    expect(
      screen.getByText(
        "AudioX, MMAudio, and ThinkSound remain deferred to the video-to-audio story.",
      ),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("heading", { name: "airlock-approach-silent.mp4" }),
    ).toBeInTheDocument();
    expect(screen.getByTitle("Queue video-to-audio generation")).toBeEnabled();
    expect(screen.getAllByText("Door servo").length).toBeGreaterThan(0);
    expect(screen.getAllByText("Boot steps").length).toBeGreaterThan(0);
    expect(screen.getAllByText("Pressure seal").length).toBeGreaterThan(0);
    expect(screen.getAllByText("MMAudio").length).toBeGreaterThan(0);
    expect(screen.getAllByText("AudioX").length).toBeGreaterThan(0);
    expect(screen.getAllByText("ThinkSound").length).toBeGreaterThan(0);
    expect(
      screen.getByText(
        "Text-to-SFX candidate only; use for Foley bed comparison, not video-conditioned sync.",
      ),
    ).toBeInTheDocument();
    expect(screen.getByText("Airlock synchronized Foley")).toBeInTheDocument();
    expect(
      screen.getByText(
        "Source video is user-owned and cleared for generated Foley.",
      ),
    ).toBeInTheDocument();
    expect(
      screen.getByText(
        "soundworks-exports/project-demo/airlock-approach/video-to-audio-provenance.json",
      ),
    ).toBeInTheDocument();
    expect(
      screen.getByText(
        "Real provider adapters and generated audio bytes still require later runnable model integration.",
      ),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("heading", { name: "Neon bass starter pack" }),
    ).toBeInTheDocument();
    expect(
      screen.getByTitle("Queue sample and loop generation"),
    ).toBeDisabled();
    expect(screen.getAllByText("Bass pluck A1").length).toBeGreaterThan(0);
    expect(
      screen.getAllByText("Four-bar chase bassline").length,
    ).toBeGreaterThan(0);
    expect(screen.getAllByText("ACE-Step 1.5").length).toBeGreaterThan(0);
    expect(
      screen.getByText(
        "Loop variants include BPM, key, bar count, and inspectable loop points.",
      ),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("heading", { name: "City Lights Resolve" }),
    ).toBeInTheDocument();
    expect(screen.getByTitle("Queue song generation")).toBeDisabled();
    expect(screen.getByText("Verse 1")).toBeInTheDocument();
    expect(screen.getAllByText("City Lights full mix").length).toBeGreaterThan(
      0,
    );
    expect(
      screen.getAllByText("City Lights instrumental pass").length,
    ).toBeGreaterThan(0);
    expect(screen.getAllByText("Stable Audio 3").length).toBeGreaterThan(0);
    expect(screen.getAllByText("DiffRhythm 2").length).toBeGreaterThan(0);
    expect(
      screen.getByText(
        "Stable Audio 3 and ACE-Step need runnable Mac/Windows smoke evidence before product enablement.",
      ),
    ).toBeInTheDocument();
    expect(
      screen.getAllByRole("heading", { name: "Dusty trip-hop drums" }).length,
    ).toBeGreaterThan(0);
    expect(screen.getByTitle("Save edited audio version")).toBeEnabled();
    expect(screen.getByText("Trim selection")).toBeInTheDocument();
    expect(screen.getByText("Loop crossfade")).toBeInTheDocument();
    expect(screen.getByText("Version comparison")).toBeInTheDocument();
    expect(
      screen.getAllByText("version-loop-001-b-review-edit").length,
    ).toBeGreaterThan(0);
    expect(
      screen.getByText(
        "Edit recipe, source version, generated source recipe, and provenance sidecar remain inspectable.",
      ),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("heading", { name: "SoundWorks launch rights policy" }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("heading", { name: "Demo SoundWorks Project" }),
    ).toBeInTheDocument();
    expect(screen.getByTitle("Create SoundWorks project")).toBeEnabled();
    expect(screen.getByTitle("Open SoundWorks project")).toBeEnabled();
    expect(screen.getByText("Project library")).toBeInTheDocument();
    expect(screen.getByText("Link global reference")).toBeInTheDocument();
    expect(screen.getByText("Copy global preset")).toBeInTheDocument();
    expect(
      screen.getByText(/Global assets can be linked or copied into a project/i),
    ).toBeInTheDocument();
    expect(
      screen.getByText(
        "soundworks-library/projects/project-demo/compositions/composition-demo/provenance/global-asset-links.json",
      ),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("heading", { name: "Project and global audio assets" }),
    ).toBeInTheDocument();
    expect(screen.getAllByText("Global audio library").length).toBeGreaterThan(
      0,
    );
    expect(screen.getByText("Prompt/Recipe preset")).toBeInTheDocument();
    expect(screen.getByText("Version history")).toBeInTheDocument();
    expect(screen.getByText("Recipe provenance")).toBeInTheDocument();
    expect(screen.getByText("Promote to global")).toBeInTheDocument();
    expect(
      screen.getByText(/Filter model includes type, tags, duration, BPM/i),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("heading", {
        name: "Presets, stems, and handoff packages",
      }),
    ).toBeInTheDocument();
    expect(screen.getByTitle("Export selected composition")).toBeEnabled();
    expect(screen.getByText("Podcast/dialogue")).toBeInTheDocument();
    expect(screen.getByText("Sample pack")).toBeInTheDocument();
    expect(
      screen.getAllByText("SceneWorks video track").length,
    ).toBeGreaterThan(0);
    expect(screen.getAllByText("DAW bundle").length).toBeGreaterThan(0);
    expect(screen.getAllByText("Sidecars").length).toBeGreaterThan(0);
    expect(
      screen.getByText("Export presets cover WAV, FLAC, MP3, and OGG."),
    ).toBeInTheDocument();
    expect(
      screen.getByText(/Loop and sample pack exports preserve BPM, key/i),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("heading", { name: "Demo timeline" }),
    ).toBeInTheDocument();
    expect(screen.getByTitle("Render composition mixdown")).toBeEnabled();
    expect(screen.getByText("clip-voice-intro")).toBeInTheDocument();
    expect(screen.getByText("Reference cue bed")).toBeInTheDocument();
    expect(screen.getByText("City Lights vocal stem")).toBeInTheDocument();
    expect(screen.getAllByText("Metal hatch impact").length).toBeGreaterThan(0);
    expect(
      screen.getByText(
        "Timeline clips preserve project/global source identity and version IDs for reopen safety.",
      ),
    ).toBeInTheDocument();
    expect(
      screen.getByText("composition sits at -16.2 LUFS with -1.1 dBTP peak"),
    ).toBeInTheDocument();
    expect(screen.getAllByText("waveform-playlist").length).toBeGreaterThan(0);
    expect(
      screen.getByText(
        "Spike first; do not hard-depend in product code until runtime/export behavior is proven.",
      ),
    ).toBeInTheDocument();
    expect(
      screen.getByText(
        "SoundWorks can render a SceneWorks handoff package; direct attachment waits for a SceneWorks-side importer.",
      ),
    ).toBeInTheDocument();
    expect(
      screen.getAllByText(/sceneworks-handoff\.json/).length,
    ).toBeGreaterThan(0);
    expect(
      screen.getByRole("heading", { name: "SceneWorks compatibility" }),
    ).toBeInTheDocument();
    expect(
      screen.getByText(
        /Current SceneWorks project imports do not accept standalone audio files/i,
      ),
    ).toBeInTheDocument();
    expect(
      screen.getByText("Attach or replace the video's audio track"),
    ).toBeInTheDocument();
    expect(screen.getByTitle("SoundWorks export gate")).toBeDisabled();
    expect(
      screen.getByText(
        "Guest voice conversion is blocked until the speaker consent record is completed.",
      ),
    ).toBeInTheDocument();
    expect(screen.getByText("ChatTTS")).toBeInTheDocument();
    expect(
      screen.getByText(
        "Noncommercial model terms fit SoundWorks' non-commercial posture when other export gates pass.",
      ),
    ).toBeInTheDocument();
    expect(
      screen.getByText(
        "Export sidecars include recipe, model, source media, rights, disclosure, and edit-chain fields.",
      ),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("heading", { name: "Release gate and demo matrix" }),
    ).toBeInTheDocument();
    expect(screen.getByTitle("MVP release gate")).toBeDisabled();
    expect(screen.getByText("12/12")).toBeInTheDocument();
    expect(screen.getAllByText("0/5").length).toBeGreaterThan(0);
    expect(screen.getByText("Runtime evidence")).toBeInTheDocument();
    expect(screen.getByText("Narrate short script")).toBeInTheDocument();
    expect(
      screen.getByText("Generate complete song from lyrics and structure"),
    ).toBeInTheDocument();
    expect(
      screen.getByText("Prototype silent video Foley"),
    ).toBeInTheDocument();
    expect(screen.getByText("Silent video Foley map")).toBeInTheDocument();
    expect(screen.getByText("Failed model download")).toBeInTheDocument();
    expect(
      screen.getByText(
        "Reference fixtures define contracts but do not yet prove generated audio quality from real selected providers.",
      ),
    ).toBeInTheDocument();
    expect(
      screen.getByText(
        "Required manual audio-quality scorecards are not all passed.",
      ),
    ).toBeInTheDocument();
    expect(
      screen.getByText(
        "Runtime evidence is missing; fixture/demo data cannot satisfy generated audio, playback, edit, or export criteria.",
      ),
    ).toBeInTheDocument();
    expect(
      screen.getByText(
        "Installed model counts must come from verified cache/package files, not static provider manifests.",
      ),
    ).toBeInTheDocument();
    expect(
      screen.getByText(
        "SC-6467 must implement model download/cache verification before any model can be counted as installed.",
      ),
    ).toBeInTheDocument();
  });
});
