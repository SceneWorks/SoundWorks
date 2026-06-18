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
    expect(screen.getByText("Stem Separation")).toBeInTheDocument();
    expect(
      screen.getByRole("heading", { name: "Worker Runtime" }),
    ).toBeInTheDocument();
    expect(screen.getByText("Python runtime: blocked")).toBeInTheDocument();
    expect(screen.getByText("Reference Speech Suite")).toBeInTheDocument();
    expect(
      screen.getByText(/Reinstall the provider package/i),
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
    expect(screen.getByRole("button", { name: /Queue/i })).toBeEnabled();
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
    expect(screen.getByTitle("Queue SFX generation")).toBeEnabled();
    expect(screen.getAllByText("Tight hatch impact").length).toBeGreaterThan(0);
    expect(screen.getAllByText("Engine room bed").length).toBeGreaterThan(0);
    expect(screen.getByText("MOSS-SoundEffect")).toBeInTheDocument();
    expect(
      screen.getByText(
        "Best first SFX spike because an Apache-licensed MLX path exists for local Mac validation.",
      ),
    ).toBeInTheDocument();
    expect(
      screen.getByText(
        "AudioX, MMAudio, and ThinkSound remain deferred to the video-to-audio story.",
      ),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("heading", { name: "Neon bass starter pack" }),
    ).toBeInTheDocument();
    expect(screen.getByTitle("Queue sample and loop generation")).toBeEnabled();
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
    expect(screen.getByTitle("Queue song generation")).toBeEnabled();
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
      screen.getByRole("heading", { name: "Project and global audio assets" }),
    ).toBeInTheDocument();
    expect(screen.getByText("Global audio library")).toBeInTheDocument();
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
    expect(screen.getByText("DAW bundle")).toBeInTheDocument();
    expect(screen.getAllByText("Sidecars").length).toBeGreaterThan(0);
    expect(
      screen.getByText("Export presets cover WAV, FLAC, MP3, and OGG."),
    ).toBeInTheDocument();
    expect(
      screen.getByText(/Loop and sample pack exports preserve BPM, key/i),
    ).toBeInTheDocument();
    expect(screen.getByTitle("Commercial export gate")).toBeDisabled();
    expect(
      screen.getByText(
        "Guest voice conversion is blocked until the speaker consent record is completed.",
      ),
    ).toBeInTheDocument();
    expect(screen.getByText("ChatTTS")).toBeInTheDocument();
    expect(
      screen.getByText(
        "Noncommercial model terms block commercial SoundWorks export.",
      ),
    ).toBeInTheDocument();
    expect(
      screen.getByText(
        "Export sidecars include recipe, model, source media, rights, disclosure, and edit-chain fields.",
      ),
    ).toBeInTheDocument();
  });
});
