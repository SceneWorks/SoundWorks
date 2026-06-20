// DR-02: shared view-model types + pure helpers, lifted out of App.tsx so the
// per-ActiveView screen components (and App's shell) can import them without an
// App <-> screens import cycle. No React state lives here — only the view id
// union, the studio maps, and the formatting/derivation helpers screens reuse.
import {
  Boxes,
  FileVideo,
  Mic2,
  Music2,
  Play,
  Radio,
  ShieldCheck,
  SlidersHorizontal,
  Waves,
} from "lucide-react";
import type { LucideIcon } from "lucide-react";
import type {
  LibraryMutationAction,
  ModelManagerOperation,
  RuntimeJobRequest,
  RuntimeOverview,
} from "./types";

// UX-F2: shared action-feedback tri-state. Replaces the bare status strings so
// screens can distinguish in-flight (pending) from success/error and style them
// (the FeedbackLine renderer maps each kind to a StatusBadge tone). `idle` is the
// neutral resting message.
export type ActionFeedbackKind = "idle" | "pending" | "success" | "error";
export interface ActionFeedback {
  kind: ActionFeedbackKind;
  message: string;
}
export const actionFeedback = {
  idle: (message: string): ActionFeedback => ({ kind: "idle", message }),
  pending: (message: string): ActionFeedback => ({ kind: "pending", message }),
  success: (message: string): ActionFeedback => ({ kind: "success", message }),
  error: (message: string): ActionFeedback => ({ kind: "error", message }),
};

export type ActiveView =
  | "workspace"
  | "library"
  | "multitrack"
  | "tts"
  | "voice"
  | "sfx"
  | "video-audio"
  | "samples"
  | "song"
  | "review"
  | "export"
  | "rights"
  | "jobs"
  | "models"
  | "validation"
  | "settings";

export type NavItem = {
  id: ActiveView;
  label: string;
  title: string;
  blurb: string;
  icon: LucideIcon;
};

/** Runtime model row the studios gate Generate on. */
export type RuntimeModelState = RuntimeOverview["modelStates"][number];

// Explicit studio-id -> destination view + icon so the workspace studio cards
// stay decoupled from nav ordering (F-028).
export const studioViewById: Record<string, ActiveView> = {
  tts: "tts",
  "voice-lab": "voice",
  sfx: "sfx",
  loops: "samples",
  songs: "song",
  review: "review",
  "rights-safety": "rights",
  "composition-editor": "multitrack",
  "video-to-audio": "video-audio",
};

export const studioIconById: Record<string, LucideIcon> = {
  tts: Mic2,
  "voice-lab": Radio,
  sfx: Waves,
  loops: Boxes,
  songs: Music2,
  review: Play,
  "rights-safety": ShieldCheck,
  "composition-editor": SlidersHorizontal,
  "video-to-audio": FileVideo,
};

// The set of recognized library lifecycle actions. An action id from the
// backend is mapped to the typed union explicitly; unknown ids are rejected
// rather than silently treated as "add-tag" (F-027).
export const LIBRARY_MUTATION_ACTIONS: readonly LibraryMutationAction[] = [
  "favorite",
  "reject",
  "archive",
  "restore",
  "promote-to-global",
  "add-tag",
];

export function toLibraryMutationAction(
  id: string,
): LibraryMutationAction | null {
  return (LIBRARY_MUTATION_ACTIONS as readonly string[]).includes(id)
    ? (id as LibraryMutationAction)
    : null;
}

// Only render externally-controlled URLs as links when they use a safe
// http(s) scheme; anything else (javascript:, data:, etc.) renders as plain
// text. Links also get rel="noopener noreferrer" at the call site (F-036).
export function isHttpUrl(value: string): boolean {
  try {
    const parsed = new URL(value);
    return parsed.protocol === "http:" || parsed.protocol === "https:";
  } catch {
    return false;
  }
}

export function workflowLabel(workflow: string) {
  return workflow
    .split("-")
    .map((part) => part.charAt(0).toUpperCase() + part.slice(1))
    .join(" ");
}

export function statusLabel(status: string) {
  return status
    .split("-")
    .map((part) => part.charAt(0).toUpperCase() + part.slice(1))
    .join(" ");
}

export function formatMb(value?: number | null) {
  if (!value) {
    return "n/a";
  }

  return value >= 1024 ? `${Math.round(value / 1024)} GB` : `${value} MB`;
}

export function formatDuration(ms: number) {
  const seconds = Math.round(ms / 100) / 10;

  return `${seconds}s`;
}

export function countFor(counts: Record<string, number>, key: string) {
  return counts[key] ?? 0;
}

export function visibleModelManagerOperation(
  operations: ModelManagerOperation[],
) {
  return (
    operations.find((operation) => operation.status === "failed") ??
    operations[0] ??
    null
  );
}

export function scopeLabel(scope: { kind: string; projectId?: string }) {
  return scope.kind === "globalLibrary"
    ? "Global"
    : (scope.projectId ?? "Project");
}

export function runtimeModelFor(runtime: RuntimeOverview, workflow: string) {
  return (
    runtime.modelStates.find(
      (model) =>
        model.availability === "installed" &&
        model.cache.verified &&
        model.workflows.includes(workflow as RuntimeJobRequest["workflow"]),
    ) ?? null
  );
}
