// DR-03: Models page rebuilt on SceneWorks' model-manager grammar.
// - Candidates grouped into collapsible model-type-group sections by capability
//   lane, Recommended vs Additional, one ModelCard per model with a single
//   StatusBadge (replaces the 28-row QA-console cache list + 5 status vocabularies).
// - The in-flight operation is routed through WorkerProgressCard (a failure shows
//   an actionable error + Retry, not a dead red banner).
// - Honest install (F-014): there is no in-app downloader, so the always-failing
//   Download button is gone; cards show the manual install command + a Revalidate
//   action that actually works. Raw blocker strings are demoted to a disclosure.
// - Provider coverage + evaluation scorecard moved to Settings (DR-02f); lane
//   readiness + validation checks moved into a Diagnostics disclosure (off the
//   main surface).
import { useEffect, useState } from "react";
import { Copy, Search } from "lucide-react";
import {
  ModelCard,
  ModelGrid,
  StatusBadge,
  SurfaceHeader,
  HeroStat,
  WorkerProgressCard,
} from "../components";
import type { StatusTone } from "../components";
import { formatMb, isHttpUrl, statusLabel, workflowLabel } from "../viewModel";
import { useAppContext } from "./context";
import type { ModelManagerOverview } from "../types";

type Candidate = ModelManagerOverview["candidates"][number];

// Single status vocabulary for the card badge (replaces installState/
// evaluationStatus/productEligibility/evidenceLevel/runtimePath competing pills).
function installTone(state: Candidate["installState"]): StatusTone {
  switch (state) {
    case "installed":
      return "installed";
    case "missing-cache":
    case "needs-runtime-port":
      return "warning";
    case "blocked":
      return "failed";
    default:
      return "neutral";
  }
}

function operationTone(status: string): StatusTone {
  if (status === "failed") return "failed";
  if (status === "succeeded") return "completed";
  return "neutral";
}

// Group candidates under their primary capability lane, preserving the lane order
// the backend declares (laneReadiness is already lane-ordered).
function groupByLane(
  laneOrder: string[],
  candidates: readonly Candidate[],
): Array<{ lane: string; candidates: Candidate[] }> {
  const groups = new Map<string, Candidate[]>();
  for (const candidate of candidates) {
    const lane = candidate.lanes[0] ?? "other";
    const bucket = groups.get(lane);
    if (bucket) {
      bucket.push(candidate);
    } else {
      groups.set(lane, [candidate]);
    }
  }
  const ordered = [...laneOrder.filter((lane) => groups.has(lane)), ...groups.keys()];
  const seen = new Set<string>();
  return ordered
    .filter((lane) => !seen.has(lane) && seen.add(lane))
    .map((lane) => ({ lane, candidates: groups.get(lane) ?? [] }));
}

function CandidateCard({
  candidate,
  recommended,
  onRevalidate,
}: {
  candidate: Candidate;
  recommended: boolean;
  onRevalidate: (candidateId: string) => void;
}) {
  const installed = candidate.installState === "installed";
  const meta = [
    {
      label: "Cache",
      value: `${candidate.cache.presentFileCount}/${candidate.cache.expectedFileCount} files`,
    },
    { label: "Evidence", value: candidate.cache.evidence },
  ];
  if (candidate.cache.diskUsageMb) {
    meta.push({ label: "On disk", value: formatMb(candidate.cache.diskUsageMb) });
  }

  return (
    <ModelCard
      title={candidate.name}
      status={
        <StatusBadge tone={installTone(candidate.installState)}>
          {statusLabel(candidate.installState)}
        </StatusBadge>
      }
      description={`${candidate.provider} · ${candidate.licenseLabel}${recommended ? " · Recommended" : ""}`}
      meta={meta}
      actions={
        <>
          {candidate.actions.includes("revalidate") ? (
            <button
              className="secondary-action"
              onClick={() => onRevalidate(candidate.candidateId)}
              type="button"
            >
              Revalidate cache
            </button>
          ) : null}
          {isHttpUrl(candidate.sourceUrl) ? (
            <a
              className="model-source-link"
              href={candidate.sourceUrl}
              rel="noopener noreferrer"
              target="_blank"
            >
              {candidate.sourceLabel}
            </a>
          ) : null}
        </>
      }
    >
      {!installed ? (
        <div className="model-gated-notice">
          <span>
            Manual install required — SoundWorks has no in-app downloader yet.
          </span>
          <div className="command-hint-row">
            <code>{candidate.downloadPlan.commandHint}</code>
            <button
              type="button"
              className="icon-action"
              title="Copy install command"
              onClick={() => {
                void navigator.clipboard?.writeText(
                  candidate.downloadPlan.commandHint,
                );
              }}
            >
              <Copy aria-hidden="true" size={14} />
            </button>
          </div>
          {candidate.downloadPlan.requiresLicenseAcceptance ? (
            <small>Requires accepting the model license first.</small>
          ) : null}
          {candidate.blockers.length > 0 ? (
            <details className="model-blockers">
              <summary>
                {candidate.blockers.length} blocker
                {candidate.blockers.length === 1 ? "" : "s"}
              </summary>
              <ul>
                {candidate.blockers.map((blocker, index) => (
                  <li key={index}>{blocker}</li>
                ))}
              </ul>
            </details>
          ) : null}
        </div>
      ) : null}
    </ModelCard>
  );
}

export function ModelsScreen() {
  const {
    modelManager,
    modelManagerOperation,
    runModelManagerAction,
    modelFocus,
    clearModelFocus,
  } = useAppContext();

  const [query, setQuery] = useState("");
  // UX-14: a studio deep-link pre-filters the grid to its lane, then clears the
  // one-shot focus so manual edits aren't overwritten.
  useEffect(() => {
    if (modelFocus) {
      setQuery(modelFocus);
      clearModelFocus();
    }
  }, [modelFocus, clearModelFocus]);

  const recommendedIds = new Set(
    modelManager.laneReadiness.map((lane) => lane.recommendedCandidateId),
  );
  const laneOrder = modelManager.laneReadiness.map((lane) => lane.lane);
  const normalizedQuery = query.trim().toLowerCase();
  const filteredCandidates = normalizedQuery
    ? modelManager.candidates.filter((candidate) =>
        [
          candidate.name,
          candidate.provider,
          candidate.candidateId,
          candidate.licenseLabel,
          ...candidate.lanes,
          ...candidate.lanes.map(workflowLabel),
        ]
          .join(" ")
          .toLowerCase()
          .includes(normalizedQuery),
      )
    : modelManager.candidates;
  const groups = groupByLane(laneOrder, filteredCandidates);
  const revalidate = (candidateId: string) =>
    runModelManagerAction(candidateId, "revalidate");

  return (
    <section className="models-screen" aria-label="Model manager">
      <SurfaceHeader
        eyebrow="Model manager"
        title="Models"
        blurb="No model is installed until its required files verify on disk. Install models manually into the cache, then revalidate."
        stats={
          <>
            <HeroStat
              label="Verified"
              value={modelManager.summary.verifiedInstalledCount}
            />
            <HeroStat
              label="Installable"
              value={modelManager.summary.installableCount}
            />
            <HeroStat
              label="Missing cache"
              value={modelManager.summary.missingCacheCount}
            />
            <HeroStat
              label="Cache root"
              value={modelManager.cacheRoot}
            />
          </>
        }
      />

      {modelManagerOperation ? (
        <WorkerProgressCard
          typeLabel={statusLabel(modelManagerOperation.action)}
          title={modelManagerOperation.candidateId}
          statusLabel={statusLabel(modelManagerOperation.status)}
          tone={operationTone(modelManagerOperation.status)}
          percent={modelManagerOperation.progressPercent}
          message={
            modelManagerOperation.status === "failed"
              ? undefined
              : modelManagerOperation.summary
          }
          logTail={modelManagerOperation.logTail}
          error={
            modelManagerOperation.status === "failed"
              ? {
                  summary: modelManagerOperation.summary,
                  recovery: modelManagerOperation.recovery,
                }
              : null
          }
          actions={
            <button
              className="secondary-action"
              onClick={() => revalidate(modelManagerOperation.candidateId)}
              type="button"
            >
              Retry (revalidate)
            </button>
          }
        />
      ) : null}

      <div className="models-search" role="search">
        <Search aria-hidden="true" size={18} />
        <input
          type="search"
          className="field-input"
          value={query}
          onChange={(event) => setQuery(event.target.value)}
          placeholder="Search models by name, provider, or lane…"
          aria-label="Search models"
        />
      </div>

      {groups.length === 0 ? (
        <p className="field-hint">No models match “{query.trim()}”.</p>
      ) : null}

      {groups.map(({ lane, candidates }) => {
        const recommended = candidates.filter((candidate) =>
          recommendedIds.has(candidate.candidateId),
        );
        const additional = candidates.filter(
          (candidate) => !recommendedIds.has(candidate.candidateId),
        );
        return (
          <details className="model-type-group" key={lane} open>
            <summary className="model-type-group-heading">
              <h3>{workflowLabel(lane)}</h3>
              <span>{candidates.length}</span>
            </summary>
            {recommended.length > 0 ? (
              <>
                <h4 className="model-subgroup-heading">Recommended</h4>
                <ModelGrid>
                  {recommended.map((candidate) => (
                    <CandidateCard
                      key={candidate.candidateId}
                      candidate={candidate}
                      recommended
                      onRevalidate={revalidate}
                    />
                  ))}
                </ModelGrid>
              </>
            ) : null}
            {additional.length > 0 ? (
              <>
                <h4 className="model-subgroup-heading">Additional</h4>
                <ModelGrid>
                  {additional.map((candidate) => (
                    <CandidateCard
                      key={candidate.candidateId}
                      candidate={candidate}
                      recommended={false}
                      onRevalidate={revalidate}
                    />
                  ))}
                </ModelGrid>
              </>
            ) : null}
          </details>
        );
      })}

      <details className="model-type-group">
        <summary className="model-type-group-heading">
          <h3>Diagnostics</h3>
          <span>{modelManager.validationChecks.length}</span>
        </summary>
        <div className="model-diagnostics">
          <div className="runtime-stack">
            <h4 className="model-subgroup-heading">Lane readiness</h4>
            <ol className="runtime-list">
              {modelManager.laneReadiness.map((lane) => (
                <li key={`${lane.lane}-${lane.recommendedCandidateId}`}>
                  <span className={`runtime-dot ${lane.state}`} />
                  <div>
                    <strong>{workflowLabel(lane.lane)}</strong>
                    <small>
                      {lane.recommendedCandidateId} / {statusLabel(lane.state)}
                    </small>
                    <em>{lane.summary}</em>
                    {lane.blocker ? <em>{lane.blocker}</em> : null}
                  </div>
                </li>
              ))}
            </ol>
          </div>
          <div className="runtime-stack">
            <h4 className="model-subgroup-heading">Validation checks</h4>
            <ol className="validation-list" aria-label="Model manager checks">
              {modelManager.validationChecks.map((check) => (
                <li
                  className={check.passed ? "passed" : "failed"}
                  key={check.id}
                >
                  <span>
                    {check.summary}
                    {check.recovery ? <em>{check.recovery}</em> : null}
                  </span>
                </li>
              ))}
            </ol>
          </div>
        </div>
      </details>
    </section>
  );
}
