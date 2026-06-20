// DR-02: MVP Validation screen. Extracted from App.tsx and rebuilt on the shared
// grammar (SurfaceHeader hero + HeroStat for the header/metrics, MainSurface +
// SectionHeading for the sub-panels) in place of the bespoke mvp-validation-panel
// / mvp-header / mvp-metrics / tts-subpanel / subpanel-heading classes. The F-015
// inert release-gate status stays a non-button <div>, and all data bindings,
// statuses, and screen-specific list classes are preserved verbatim.
import { CircleAlert, ClipboardCheck, Gauge } from "lucide-react";
import {
  HeroStat,
  MainSurface,
  SectionHeading,
  SurfaceHeader,
} from "../components";
import { statusLabel, workflowLabel } from "../viewModel";
import { useAppContext } from "./context";

export function ValidationScreen() {
  const { mvpValidation, overview } = useAppContext();

  return (
    <section className="mvp-validation-panel" aria-label="MVP Validation">
      <SurfaceHeader
        eyebrow="MVP validation"
        title="Release gate and demo matrix"
        actions={
          <div
            className={
              mvpValidation.releaseGate.readyForMvp
                ? "primary-action validation-action is-inert"
                : "primary-action validation-action is-inert is-blocked"
            }
            title="MVP release gate status"
            role="status"
          >
            <ClipboardCheck aria-hidden="true" size={18} />
            <span>
              {mvpValidation.releaseGate.readyForMvp ? "Ready" : "Blocked"}
            </span>
          </div>
        }
        stats={
          <>
            <HeroStat
              label="workflows covered"
              value={`${mvpValidation.releaseGate.coveredWorkflowCount}/${mvpValidation.releaseGate.requiredWorkflowCount}`}
            />
            <HeroStat
              label="automated checks"
              value={`${mvpValidation.releaseGate.passedAutomatedCheckCount}/${mvpValidation.releaseGate.requiredAutomatedCheckCount}`}
            />
            <HeroStat
              label="manual scorecards"
              value={`${mvpValidation.releaseGate.passedManualScorecardCount}/${mvpValidation.releaseGate.requiredManualScorecardCount}`}
            />
            <HeroStat
              label="runtime evidence"
              value={`${mvpValidation.releaseGate.satisfiedRuntimeEvidenceCount}/${mvpValidation.releaseGate.requiredRuntimeEvidenceCount}`}
            />
            <HeroStat
              label="blocking items"
              value={overview.mvpValidation.blockingItemCount}
            />
          </>
        }
      />

      <div className="mvp-layout">
        <div className="mvp-main">
          <MainSurface ariaLabel="Golden demo workflows">
            <SectionHeading
              title="Golden demos"
              eyebrow={mvpValidation.demoWorkflows.length}
            />
            <div className="mvp-demo-grid">
              {mvpValidation.demoWorkflows.map((workflow) => (
                <article className="mvp-demo-card" key={workflow.id}>
                  <div className="sfx-variant-title">
                    <strong>{workflow.title}</strong>
                    <span>{workflowLabel(workflow.workflow)}</span>
                  </div>
                  <p>{workflow.goal}</p>
                  <div className="asset-tag-row">
                    {workflow.requiredArtifacts
                      .slice(0, 4)
                      .map((artifact, index) => (
                        <span key={`${workflow.id}-${index}`}>{artifact}</span>
                      ))}
                  </div>
                </article>
              ))}
            </div>
          </MainSurface>

          <MainSurface ariaLabel="Requirement coverage">
            <SectionHeading
              title="Epic requirement coverage"
              eyebrow={mvpValidation.requirementCoverage.length}
            />
            <div className="requirement-grid">
              {mvpValidation.requirementCoverage.map((coverage) => (
                <article
                  className={`requirement-card ${coverage.status}`}
                  key={coverage.requirementId}
                >
                  <div className="rights-card-title">
                    <strong>{coverage.requirementId}</strong>
                    <span>{statusLabel(coverage.status)}</span>
                  </div>
                  <p>{coverage.epicRequirement}</p>
                  <small>
                    {coverage.demoWorkflowIds.length} demos /{" "}
                    {coverage.fixtureIds.length} fixtures /{" "}
                    {coverage.checkIds.length} checks
                  </small>
                </article>
              ))}
            </div>
          </MainSurface>
        </div>

        <div className="mvp-side">
          <MainSurface ariaLabel="MVP blockers">
            <SectionHeading
              title="Release blockers"
              eyebrow={mvpValidation.releaseGate.blockingItems.length}
            />
            <ol className="voice-checks">
              {mvpValidation.releaseGate.blockingItems.map((item) => (
                <li className="blocked" key={item}>
                  <CircleAlert aria-hidden="true" size={16} />
                  <span>{item}</span>
                </li>
              ))}
            </ol>
          </MainSurface>

          <MainSurface ariaLabel="Automated validation checks">
            <SectionHeading
              title="Automated checks"
              eyebrow={mvpValidation.automatedChecks.length}
            />
            <ol className="voice-checks">
              {mvpValidation.automatedChecks.map((check) => (
                <li className={check.status} key={check.id}>
                  <ClipboardCheck aria-hidden="true" size={16} />
                  <span>
                    <strong>{statusLabel(check.category)}</strong>{" "}
                    {check.summary}
                  </span>
                </li>
              ))}
            </ol>
          </MainSurface>

          <MainSurface ariaLabel="Runtime evidence">
            <SectionHeading
              title="Runtime evidence"
              eyebrow={`${mvpValidation.releaseGate.satisfiedRuntimeEvidenceCount}/${mvpValidation.releaseGate.requiredRuntimeEvidenceCount}`}
            />
            <ol className="voice-checks">
              {mvpValidation.runtimeEvidence.map((evidence) => (
                <li className={evidence.status} key={evidence.id}>
                  <CircleAlert aria-hidden="true" size={16} />
                  <span>
                    <strong>{workflowLabel(evidence.workflow)}</strong>{" "}
                    {evidence.requirement}
                    <em>
                      {evidence.fixtureOnly ? "Fixture-only: " : "Evidence: "}
                      {evidence.evidence}
                    </em>
                    <em>{evidence.blocker}</em>
                  </span>
                </li>
              ))}
            </ol>
          </MainSurface>
        </div>
      </div>

      <div className="mvp-bottom-grid">
        <MainSurface ariaLabel="Regression fixtures">
          <SectionHeading
            title="Regression fixtures"
            eyebrow={mvpValidation.regressionFixtures.length}
          />
          <div className="fixture-list">
            {mvpValidation.regressionFixtures.map((fixture) => (
              <article key={fixture.id}>
                <strong>{fixture.name}</strong>
                <small>{workflowLabel(fixture.workflow)}</small>
                <p>{fixture.inputContract}</p>
              </article>
            ))}
          </div>
        </MainSurface>

        <MainSurface ariaLabel="Manual QA scorecards">
          <SectionHeading
            title="Manual QA"
            eyebrow={mvpValidation.manualScorecards.length}
          />
          <ol className="voice-checks">
            {mvpValidation.manualScorecards.slice(0, 6).map((scorecard) => (
              <li className={scorecard.status} key={scorecard.id}>
                <Gauge aria-hidden="true" size={16} />
                <span>
                  <strong>{workflowLabel(scorecard.workflow)}</strong>{" "}
                  {scorecard.passThreshold}
                </span>
              </li>
            ))}
          </ol>
        </MainSurface>

        <MainSurface ariaLabel="Stress cases and limitations">
          <SectionHeading
            title="Stress and limits"
            eyebrow={mvpValidation.stressCases.length}
          />
          <ol className="voice-checks">
            {mvpValidation.stressCases.map((stressCase) => (
              <li className={stressCase.status} key={stressCase.id}>
                <CircleAlert aria-hidden="true" size={16} />
                <span>
                  <strong>{stressCase.title}</strong>{" "}
                  {stressCase.expectedBehavior}
                </span>
              </li>
            ))}
          </ol>
          <div className="limitation-list">
            {mvpValidation.knownLimitations.map((limitation) => (
              <article
                className={limitation.blocksMvp ? "blocks" : ""}
                key={limitation.id}
              >
                <strong>{limitation.area}</strong>
                <p>{limitation.summary}</p>
              </article>
            ))}
          </div>
        </MainSurface>
      </div>
    </section>
  );
}
