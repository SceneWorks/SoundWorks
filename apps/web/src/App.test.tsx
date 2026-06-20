import { fireEvent, render, screen, within } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { App } from "./App";

function navButton(label: string) {
  return within(
    screen.getByRole("navigation", { name: "SoundWorks views" }),
  ).getByRole("button", { name: label });
}

describe("App", () => {
  it("starts on a SceneWorks-style project workspace instead of rendering every surface", async () => {
    render(<App />);

    expect(screen.getByRole("main")).toBeInTheDocument();
    expect(screen.getAllByText("SoundWorks").length).toBeGreaterThan(0);
    expect(screen.getByText("Workspace")).toBeInTheDocument();
    expect(screen.getByText("Studios")).toBeInTheDocument();
    expect(screen.getByText("Review / Export")).toBeInTheDocument();
    expect(screen.getByText("System")).toBeInTheDocument();
    expect(
      screen.getByRole("heading", { name: "Project workspace", level: 1 }),
    ).toBeInTheDocument();
    expect(
      await screen.findByRole("heading", { name: "Demo SoundWorks Project" }),
    ).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /0 jobs/i })).toBeInTheDocument();
    expect(screen.queryByText("get_app_overview")).not.toBeInTheDocument();
    expect(
      screen.queryByRole("heading", { name: "Worker Runtime" }),
    ).not.toBeInTheDocument();
    expect(
      screen.queryByRole("heading", { name: "TTS Studio narration draft" }),
    ).not.toBeInTheDocument();
  });

  it("signals web-preview mock mode when not running in the Tauri shell", () => {
    render(<App />);

    const banner = screen.getByRole("status");
    expect(banner).toHaveTextContent(/Web preview/i);
    expect(banner).toHaveTextContent(/data is simulated/i);
  });

  it("changes active screens from the grouped left nav", async () => {
    render(<App />);

    const cases = [
      ["Library", "Project and global audio assets"],
      ["Multitrack", "Demo timeline"],
      ["TTS Studio", "Launch read"],
      ["Voice Lab", "Consented voice workflows"],
      ["SFX + Ambience", "Foley Impact"],
      ["Video Audio", "airlock-approach-silent.mp4"],
      ["Samples", "Neon bass starter pack"],
      ["Song Studio", "City Lights Resolve"],
      ["Review", "Dusty trip-hop drums"],
      ["Export", "Presets, stems, and handoff packages"],
      ["Rights", "SoundWorks launch rights policy"],
      ["Jobs", "Worker Runtime"],
      ["Models", "Model Manager"],
      ["Validation", "Release gate and demo matrix"],
      ["Settings", "Runtime Layers"],
    ] as const;

    for (const [buttonLabel, heading] of cases) {
      fireEvent.click(navButton(buttonLabel));
      expect(
        await screen.findByRole("heading", { name: heading }),
      ).toBeInTheDocument();
    }

    fireEvent.click(navButton("Library"));
    expect(
      await screen.findByRole("heading", {
        name: "Project and global audio assets",
      }),
    ).toBeInTheDocument();
    expect(
      screen.queryByRole("heading", { name: "Demo timeline" }),
    ).not.toBeInTheDocument();
    expect(
      screen.queryByRole("heading", { name: "Release gate and demo matrix" }),
    ).not.toBeInTheDocument();
  });

  it("keeps recovery action controls wired on their own screens", async () => {
    render(<App />);

    fireEvent.click(navButton("TTS Studio"));
    expect(await screen.findByTitle("Queue TTS generation")).toBeDisabled();
    expect(
      screen.getByText("Voice profile consent is required before generation."),
    ).toBeInTheDocument();

    fireEvent.click(navButton("Review"));
    expect(await screen.findByTitle("Save edited audio version")).toBeEnabled();
    expect(screen.getByTitle("Play or pause preview")).toBeEnabled();

    fireEvent.click(navButton("Export"));
    expect(
      await screen.findByTitle("Export selected composition"),
    ).toBeEnabled();
    expect(
      screen.getAllByText(/sceneworks-handoff\.json/).length,
    ).toBeGreaterThan(0);

    fireEvent.click(navButton("Models"));
    expect(await screen.findByTitle("Install Kokoro 82M")).toBeDisabled();
    expect(
      screen.getByText("MOSS-SoundEffect install failed cache verification."),
    ).toBeInTheDocument();

    fireEvent.click(navButton("Jobs"));
    expect(
      await screen.findByText("Python runtime: blocked"),
    ).toBeInTheDocument();
  });
});
