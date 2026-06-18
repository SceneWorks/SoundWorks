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
    expect(screen.getByRole("button", { name: /Convert/i })).toBeEnabled();
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
      screen.getByText("Score as speech-to-speech voice conversion, not text-to-speech."),
    ).toBeInTheDocument();
  });
});
