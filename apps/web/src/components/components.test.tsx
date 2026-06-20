import { describe, expect, it, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import {
  EmptyPanel,
  FeedbackLine,
  PlaybackControl,
  SegmentedControl,
  StatusBadge,
  SurfaceHeader,
  WorkerProgressCard,
} from "./index";
import { actionFeedback } from "../viewModel";

describe("shared component grammar", () => {
  it("SurfaceHeader renders eyebrow/title/blurb in the SceneWorks hero shape", () => {
    const { container } = render(
      <SurfaceHeader eyebrow="Studio" title="Text to speech" blurb="Synthesize" />,
    );
    const header = container.querySelector(".surface-header.hero");
    expect(header).not.toBeNull();
    expect(container.querySelector(".section-heading .eyebrow")?.textContent).toBe(
      "Studio",
    );
    expect(container.querySelector(".section-heading h2")?.textContent).toBe(
      "Text to speech",
    );
    expect(container.querySelector(".hero-blurb")?.textContent).toBe("Synthesize");
  });

  it("StatusBadge applies the tone modifier class", () => {
    const { container } = render(<StatusBadge tone="failed">Failed</StatusBadge>);
    const badge = container.querySelector(".status-badge");
    expect(badge?.classList.contains("failed")).toBe(true);
  });

  it("StatusBadge neutral tone adds no modifier", () => {
    const { container } = render(<StatusBadge>Ready</StatusBadge>);
    const badge = container.querySelector(".status-badge");
    expect(badge?.className.trim()).toBe("status-badge");
  });

  it("SegmentedControl marks the active option and fires onChange", () => {
    const onChange = vi.fn();
    render(
      <SegmentedControl
        ariaLabel="Mode"
        value="a"
        onChange={onChange}
        options={[
          { value: "a", label: "A" },
          { value: "b", label: "B" },
        ]}
      />,
    );
    const active = screen.getByRole("button", { pressed: true });
    expect(active.textContent).toBe("A");
    fireEvent.click(screen.getByText("B"));
    expect(onChange).toHaveBeenCalledWith("b");
  });

  it("EmptyPanel compact variant adds the compact-panel class", () => {
    const { container } = render(<EmptyPanel compact>Nothing here</EmptyPanel>);
    const panel = container.querySelector(".empty-panel");
    expect(panel?.classList.contains("compact-panel")).toBe(true);
  });

  it("WorkerProgressCard shows an actionable error and a determinate progress bar", () => {
    render(
      <WorkerProgressCard
        title="Install Kokoro"
        typeLabel="Install"
        statusLabel="Failed"
        tone="failed"
        percent={100}
        error={{ summary: "Install failed", recovery: "Resume the download." }}
      />,
    );
    expect(screen.getByRole("alert").textContent).toContain("Install failed");
    const bar = screen.getByRole("progressbar");
    expect(bar.getAttribute("aria-valuenow")).toBe("100");
  });

  it("FeedbackLine maps the action-feedback kind to a StatusBadge tone", () => {
    const { container } = render(
      <FeedbackLine feedback={actionFeedback.error("Export failed")} />,
    );
    const line = container.querySelector(".action-status");
    expect(line?.classList.contains("action-status-error")).toBe(true);
    expect(line?.textContent).toContain("Export failed");
    expect(container.querySelector(".status-badge.failed")).not.toBeNull();
  });

  it("FeedbackLine renders nothing for an empty message", () => {
    const { container } = render(
      <FeedbackLine feedback={actionFeedback.idle("")} />,
    );
    expect(container.querySelector(".action-status")).toBeNull();
  });

  it("PlaybackControl renders an audio element only for a playable clip", () => {
    const { container, rerender } = render(
      <PlaybackControl
        playback={{ itemId: "a", playable: true, path: "asset://clip.wav" }}
      />,
    );
    expect(container.querySelector("audio")).not.toBeNull();

    rerender(
      <PlaybackControl
        playback={{ itemId: "a", playable: false, reason: "No audio cached" }}
      />,
    );
    expect(container.querySelector("audio")).toBeNull();
    expect(container.querySelector(".playback-control.unavailable")?.textContent).toBe(
      "No audio cached",
    );

    rerender(<PlaybackControl playback={null} />);
    expect(container.querySelector(".playback-control")).toBeNull();
  });
});
