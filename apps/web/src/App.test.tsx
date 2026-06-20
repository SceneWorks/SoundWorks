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

    const banner = screen.getByRole("status", { name: "Preview mode" });
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
      ["Models", "Models"],
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
    // UX-05/S1a: the generate button is gated on an installed model and now
    // carries the disabled-reason as its tooltip (no model in web preview).
    const ttsGenerate = await screen.findByRole("button", { name: "Blocked" });
    expect(ttsGenerate).toBeDisabled();
    expect(ttsGenerate).toHaveAttribute(
      "title",
      "Install a TTS model to generate.",
    );
    expect(
      screen.getByText("Voice profile consent is required before generation."),
    ).toBeInTheDocument();

    fireEvent.click(navButton("Review"));
    expect(await screen.findByTitle("Save edited audio version")).toBeEnabled();
    expect(screen.getByTitle("Play or pause preview")).toBeEnabled();

    fireEvent.click(navButton("Export"));
    // UX-S4: the export button now opens a confirm summary first; its tooltip
    // reflects whether export is allowed.
    expect(
      await screen.findByTitle("Review and export selected composition"),
    ).toBeEnabled();
    expect(
      screen.getAllByText(/sceneworks-handoff\.json/).length,
    ).toBeGreaterThan(0);

    fireEvent.click(navButton("Models"));
    // DR-03: there is no in-app downloader, so the always-failing Download button
    // is gone. The failed operation is shown as an actionable WorkerProgressCard
    // (Retry, not a dead red banner), and each card offers a working Revalidate.
    expect(
      await screen.findByText("Kokoro install failed cache verification."),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: "Retry (revalidate)" }),
    ).toBeInTheDocument();
    expect(
      screen.getAllByRole("button", { name: "Revalidate cache" }).length,
    ).toBeGreaterThan(0);
    expect(screen.queryByTitle("Install Kokoro 82M")).not.toBeInTheDocument();

    fireEvent.click(navButton("Jobs"));
    expect(
      await screen.findByText("Python runtime: blocked"),
    ).toBeInTheDocument();
  });

  it("deep-links studio cards to their view by id, not array position", async () => {
    render(<App />);

    const studios = screen.getByRole("region", { name: "Studios" });
    fireEvent.click(
      within(studios).getByRole("button", { name: /Multitrack Editor/ }),
    );
    expect(
      await screen.findByRole("heading", { name: "Demo timeline" }),
    ).toBeInTheDocument();
  });

  it("wires Voice/Video/Song generation and demotes genuinely inert controls", async () => {
    render(<App />);

    // Previously-dead Generate/Convert buttons are now real buttons, gated on
    // an installed runtime model (none in preview, so they read Blocked).
    fireEvent.click(navButton("Voice Lab"));
    const convert = await screen.findByRole("button", { name: "Blocked" });
    expect(convert.tagName).toBe("BUTTON");
    expect(convert).toBeDisabled();
    expect(convert).toHaveAttribute(
      "title",
      "Install a voice-conversion model to generate.",
    );
    // DR-03: with no installed model (web preview), the studio shows an
    // actionable availability gate instead of leaving only a dead Blocked button.
    expect(
      screen.getByRole("button", { name: "Open Model Manager" }),
    ).toBeInTheDocument();
    // UX-08: consent capture is available per voice profile.
    expect(
      screen.getAllByRole("button", { name: "Record consent" }).length,
    ).toBeGreaterThan(0);

    // UX-S1b: Video + Song are honestly gated (no model in preview) and the
    // generate button now carries the disabled-reason as its tooltip.
    fireEvent.click(navButton("Video Audio"));
    const videoGenerate = await screen.findByRole("button", { name: "Blocked" });
    expect(videoGenerate).toBeDisabled();
    expect(videoGenerate).toHaveAttribute(
      "title",
      "Install a video-to-audio model to generate.",
    );

    fireEvent.click(navButton("Song Studio"));
    const songGenerate = await screen.findByRole("button", { name: "Blocked" });
    expect(songGenerate).toBeDisabled();
    expect(songGenerate).toHaveAttribute(
      "title",
      "Install a song model to generate.",
    );

    // UX-10: Render Mixdown is now a real button wired to the composition-render
    // job (was an inert div under F-015); it is enabled when the render plan is
    // ready in the fixture.
    fireEvent.click(navButton("Multitrack"));
    await screen.findByRole("heading", { name: "Demo timeline" });
    const renderMixdown = screen.getByRole("button", { name: /Render Mixdown/i });
    expect(renderMixdown).toBeEnabled();
  });
});
