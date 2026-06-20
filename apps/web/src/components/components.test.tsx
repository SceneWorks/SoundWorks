import { describe, expect, it, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import {
  ConfirmDialog,
  EmptyPanel,
  FeedbackLine,
  GenerationPanel,
  PlaybackControl,
  SegmentedControl,
  StatusBadge,
  SurfaceHeader,
  WorkerProgressCard,
} from "./index";
import { actionFeedback } from "../viewModel";
import type { RuntimeJobSnapshot } from "../types";

function fakeJob(overrides: Partial<RuntimeJobSnapshot>): RuntimeJobSnapshot {
  return {
    id: "job-1",
    kind: "generate-audio",
    status: "running",
    providerId: "p",
    modelId: "m",
    workflow: "tts",
    adapter: "native-rust",
    progress: { percent: 40, message: "Synthesizing" },
    cancellation: "cancellable",
    retryCount: 0,
    createdAt: "0",
    updatedAt: "0",
    recordRoot: "jobs/job-1",
    logTail: [],
    artifacts: [],
    ...overrides,
  };
}

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

  it("GenerationPanel shows progress + cancel for a running job it owns", () => {
    const onCancel = vi.fn();
    render(
      <GenerationPanel
        job={fakeJob({ status: "running" })}
        workflows={["tts"]}
        onCancel={onCancel}
        onRetry={vi.fn()}
      />,
    );
    expect(screen.getByRole("progressbar").getAttribute("aria-valuenow")).toBe(
      "40",
    );
    fireEvent.click(screen.getByRole("button", { name: "Cancel" }));
    expect(onCancel).toHaveBeenCalledWith("job-1");
  });

  it("GenerationPanel renders nothing for a job from another workflow", () => {
    const { container } = render(
      <GenerationPanel
        job={fakeJob({ workflow: "song" })}
        workflows={["tts"]}
        onCancel={vi.fn()}
        onRetry={vi.fn()}
      >
        <span>result</span>
      </GenerationPanel>,
    );
    expect(container.querySelector(".generation-panel")).toBeNull();
  });

  it("GenerationPanel shows the result slot + Retry only on the right states", () => {
    const onRetry = vi.fn();
    const { rerender } = render(
      <GenerationPanel
        job={fakeJob({ status: "succeeded", progress: { percent: 100 } })}
        workflows={["tts"]}
        onCancel={vi.fn()}
        onRetry={onRetry}
      >
        <span>saved-result</span>
      </GenerationPanel>,
    );
    expect(screen.getByText("saved-result")).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "Retry" })).toBeNull();

    rerender(
      <GenerationPanel
        job={fakeJob({
          status: "failed",
          progress: { percent: 100 },
          actionableError: {
            code: "x",
            summary: "Adapter failed",
            recovery: "Retry later",
          },
        })}
        workflows={["tts"]}
        onCancel={vi.fn()}
        onRetry={onRetry}
      >
        <span>saved-result</span>
      </GenerationPanel>,
    );
    expect(screen.queryByText("saved-result")).toBeNull();
    expect(screen.getByRole("alert").textContent).toContain("Adapter failed");
    fireEvent.click(screen.getByRole("button", { name: "Retry" }));
    expect(onRetry).toHaveBeenCalledWith("job-1");
  });

  it("ConfirmDialog renders only when open and routes confirm/cancel", () => {
    const onConfirm = vi.fn();
    const onCancel = vi.fn();
    const { rerender } = render(
      <ConfirmDialog
        open={false}
        title="Reject?"
        message="This cannot be undone."
        confirmLabel="Reject"
        onConfirm={onConfirm}
        onCancel={onCancel}
      />,
    );
    expect(screen.queryByRole("dialog")).toBeNull();

    rerender(
      <ConfirmDialog
        open
        title="Reject?"
        message="This cannot be undone."
        confirmLabel="Reject"
        onConfirm={onConfirm}
        onCancel={onCancel}
      />,
    );
    expect(screen.getByRole("dialog")).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Reject" }));
    expect(onConfirm).toHaveBeenCalledTimes(1);
    fireEvent.click(screen.getByRole("button", { name: "Cancel" }));
    expect(onCancel).toHaveBeenCalledTimes(1);
  });
});
