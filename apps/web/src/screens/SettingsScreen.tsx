// DR-02: System Settings screen. Extracted from App.tsx and rebuilt on the
// shared grammar (MainSurface + SectionHeading) in place of the bespoke .panel /
// .panel-heading wrappers. This is the architecture/coverage surface, so the
// Provider Coverage + Evaluation Scorecard panels (previously shared with the
// Models page via `showSettings || showModels`) live here — matching DR-03's
// "move provider-coverage/evaluation off the Models surface". Data bindings are
// preserved verbatim.
import {
  Boxes,
  CircleAlert,
  CircleCheck,
  ClipboardCheck,
  Cpu,
} from "lucide-react";
import { MainSurface, SectionHeading } from "../components";
import { countFor, workflowLabel } from "../viewModel";
import { useAppContext } from "./context";

export function SettingsScreen() {
  const { overview } = useAppContext();

  return (
    <section className="system-grid" aria-label="Architecture">
      <MainSurface>
        <SectionHeading
          title="Runtime Layers"
          eyebrow={`${overview.architecture.layers.length} layers`}
        />
        <ol className="layer-list">
          {overview.architecture.layers.map((layer) => (
            <li key={layer.id}>
              <span className={`layer-dot ${layer.status}`} />
              <div>
                <strong>{layer.name}</strong>
                <p>{layer.responsibility}</p>
              </div>
            </li>
          ))}
        </ol>
      </MainSurface>

      <MainSurface>
        <SectionHeading
          title="Command Boundary"
          eyebrow={`${overview.commands.length} commands`}
        />
        <div className="command-list">
          {overview.commands.map((command) => (
            <article className="command-row" key={command.name}>
              <strong>{command.name}</strong>
              <span>{command.direction}</span>
              <p>{command.purpose}</p>
            </article>
          ))}
        </div>
      </MainSurface>

      <MainSurface className="provider-panel">
        <SectionHeading
          title="Provider Coverage"
          eyebrow={`${overview.providerCatalog.capabilityCount} capabilities`}
        />
        <div className="provider-metrics" aria-label="Provider catalog">
          <div>
            <strong>{overview.providerCatalog.providerCount}</strong>
            <span>providers</span>
          </div>
          <div>
            <strong>{overview.providerCatalog.modelCount}</strong>
            <span>models</span>
          </div>
        </div>
        <ol className="workflow-list">
          {overview.providerCatalog.workflows.map((workflow) => (
            <li key={workflow.workflow}>
              <Cpu aria-hidden="true" size={16} />
              <span>{workflowLabel(workflow.workflow)}</span>
              <small>{workflow.defaultModelId}</small>
            </li>
          ))}
        </ol>
      </MainSurface>

      <MainSurface className="evaluation-panel">
        <SectionHeading
          title="Evaluation Scorecard"
          eyebrow={`${overview.modelEvaluation.candidateCount} candidates`}
        />
        <div className="evaluation-summary" aria-label="Model evaluation">
          <div>
            <ClipboardCheck aria-hidden="true" size={18} />
            <strong>{overview.modelEvaluation.sourceCount}</strong>
            <span>sources</span>
          </div>
          <div>
            <Boxes aria-hidden="true" size={18} />
            <strong>{overview.modelEvaluation.fixtureCount}</strong>
            <span>fixtures</span>
          </div>
          <div>
            <CircleCheck aria-hidden="true" size={18} />
            <strong>
              {countFor(
                overview.modelEvaluation.productEligibilityCounts,
                "product-candidate",
              )}
            </strong>
            <span>product candidates</span>
          </div>
          <div>
            <CircleAlert aria-hidden="true" size={18} />
            <strong>
              {countFor(overview.modelEvaluation.statusCounts, "blocked")}
            </strong>
            <span>blocked</span>
          </div>
        </div>
        <ol className="recommendation-list" aria-label="Recommended spikes">
          {overview.modelEvaluation.recommendedCandidateIds.map(
            (candidateId) => (
              <li key={candidateId}>
                <span>{candidateId}</span>
              </li>
            ),
          )}
        </ol>
      </MainSurface>
    </section>
  );
}
