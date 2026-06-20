// DR-02: Rights + Safety screen. Extracted from App.tsx and rebuilt on the
// shared grammar (SurfaceHeader hero + HeroStat for the header/metrics,
// MainSurface + SectionHeading for the sub-panels) in place of the bespoke
// samples-header / samples-metrics / tts-subpanel / subpanel-heading classes.
// The F-015 export-gate pill stays an inert role="status" div (not a button)
// and the data-bound rights-card / gate.status / check.status classes are kept
// verbatim. All bindings, labels, and aria attributes are preserved.
import { CircleCheck, ClipboardCheck, ShieldCheck } from "lucide-react";
import {
  HeroStat,
  MainSurface,
  SectionHeading,
  SurfaceHeader,
} from "../components";
import { statusLabel } from "../viewModel";
import { useAppContext } from "./context";

export function RightsScreen() {
  const { rightsSafety, overview } = useAppContext();

  return (
    <section className="rights-safety-panel" aria-label="Rights and Safety">
      <SurfaceHeader
        eyebrow="Rights + Safety"
        title={rightsSafety.policy.name}
        actions={
          <div
            className={
              overview.rightsSafety.canExport
                ? "primary-action safety-action is-inert"
                : "primary-action safety-action is-inert is-blocked"
            }
            title="Rights export gate status — run exports from the Export screen"
            role="status"
          >
            <ShieldCheck aria-hidden="true" size={18} />
            <span>
              {overview.rightsSafety.canExport ? "Ready" : "Blocked"}
            </span>
          </div>
        }
        stats={
          <>
            <HeroStat
              label="consent blocks"
              value={overview.rightsSafety.blockedConsentCount}
            />
            <HeroStat
              label="model blocks"
              value={overview.rightsSafety.blockedModelDecisionCount}
            />
            <HeroStat
              label="sidecars"
              value={overview.rightsSafety.sidecarCount}
            />
            <HeroStat
              label="disclosures"
              value={overview.rightsSafety.disclosureCount}
            />
          </>
        }
      />

      <div className="rights-layout">
        <div className="rights-main">
          <MainSurface ariaLabel="Consent checks">
            <SectionHeading
              title="Consent"
              eyebrow={rightsSafety.consentChecks.length}
            />
            <div className="rights-card-grid">
              {rightsSafety.consentChecks.map((check) => (
                <article
                  className={`rights-card ${check.decision}`}
                  key={check.id}
                >
                  <div className="rights-card-title">
                    <strong>{statusLabel(check.workflow)}</strong>
                    <span>{statusLabel(check.decision)}</span>
                  </div>
                  <small>
                    {check.voiceProfileId} /{" "}
                    {statusLabel(check.consentStatus)}
                  </small>
                  <p>{check.summary}</p>
                </article>
              ))}
            </div>
          </MainSurface>

          <MainSurface ariaLabel="Model export decisions">
            <SectionHeading
              title="Model export gates"
              eyebrow={rightsSafety.modelUseDecisions.length}
            />
            <div className="rights-card-grid model-gate-grid">
              {rightsSafety.modelUseDecisions.map((decision) => (
                <article
                  className={`rights-card ${decision.decision}`}
                  key={decision.candidateId}
                >
                  <div className="rights-card-title">
                    <strong>{decision.name}</strong>
                    <span>{statusLabel(decision.decision)}</span>
                  </div>
                  <small>
                    {statusLabel(decision.commercialUse)} /{" "}
                    {statusLabel(decision.productEligibility)}
                  </small>
                  <p>{decision.reasons[0]}</p>
                </article>
              ))}
            </div>
          </MainSurface>
        </div>

        <div className="rights-side">
          <MainSurface ariaLabel="Export provenance sidecars">
            <SectionHeading
              title="Sidecars"
              eyebrow={statusLabel(rightsSafety.policy.watermarkPolicy)}
            />
            {rightsSafety.exportSidecars.map((sidecar) => (
              <div className="output-card" key={sidecar.id}>
                <strong>{sidecar.assetId}</strong>
                <small>
                  {statusLabel(sidecar.target)} /{" "}
                  {statusLabel(sidecar.watermark)}
                </small>
                <p>{sidecar.path}</p>
              </div>
            ))}
          </MainSurface>

          <MainSurface ariaLabel="Policy requirements">
            <SectionHeading
              title="SoundWorks export"
              eyebrow={
                rightsSafety.policy.provenanceSidecarRequired
                  ? "sidecar"
                  : "manual"
              }
            />
            <ol className="policy-list">
              {rightsSafety.policy.exportRequires.map((requirement, index) => (
                <li key={index}>
                  <CircleCheck aria-hidden="true" size={16} />
                  <span>{requirement}</span>
                </li>
              ))}
            </ol>
          </MainSurface>
        </div>
      </div>

      <div className="rights-review-grid">
        <ol className="voice-checks" aria-label="Content policy gates">
          {rightsSafety.contentPolicyGates.map((gate) => (
            <li className={gate.status} key={gate.id}>
              <ShieldCheck aria-hidden="true" size={16} />
              <span>
                <strong>{statusLabel(gate.category)}</strong> {gate.summary}
              </span>
            </li>
          ))}
        </ol>
        <ol className="voice-checks" aria-label="Rights validation checks">
          {rightsSafety.validationChecks.map((check) => (
            <li className={check.status} key={check.id}>
              <ClipboardCheck aria-hidden="true" size={16} />
              <span>{check.summary}</span>
            </li>
          ))}
        </ol>
      </div>
    </section>
  );
}
