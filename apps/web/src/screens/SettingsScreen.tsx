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
import { ACCENTS } from "@sceneworks/ui";
import { MainSurface, SectionHeading, SegmentedControl } from "../components";
import type { ThemeMode } from "../accents";
import { countFor, workflowLabel } from "../viewModel";
import { useAppContext } from "./context";

const THEME_OPTIONS: ReadonlyArray<{ value: ThemeMode; label: string }> = [
  { value: "light", label: "Light" },
  { value: "dark", label: "Dark" },
];

export function SettingsScreen() {
  const {
    overview,
    modelManager,
    theme,
    accent,
    changeTheme,
    changeAccent,
    demoMode,
    changeDemoMode,
  } = useAppContext();

  return (
    <section className="system-grid" aria-label="Settings">
      <MainSurface className="settings-prefs">
        <SectionHeading title="Appearance" eyebrow="theme + accent" />
        <div className="settings-field">
          <span className="settings-field-label">Theme</span>
          <SegmentedControl
            ariaLabel="Theme"
            value={theme}
            onChange={changeTheme}
            options={THEME_OPTIONS}
          />
        </div>
        <div className="settings-field">
          <span className="settings-field-label">Accent</span>
          <div className="accent-swatches" role="group" aria-label="Accent">
            {ACCENTS.map((option) => (
              <button
                key={option.id}
                type="button"
                className={
                  option.id === accent
                    ? "accent-swatch selected"
                    : "accent-swatch"
                }
                style={{ background: option.swatch }}
                aria-pressed={option.id === accent}
                aria-label={option.name}
                title={option.name}
                onClick={() => changeAccent(option.id)}
              />
            ))}
          </div>
        </div>
        <p className="field-hint">
          Theme and accent save automatically and persist across launches.
        </p>
      </MainSurface>

      <MainSurface className="settings-prefs">
        <SectionHeading title="Library" eyebrow="cache + demo data" />
        <div className="settings-field">
          <span className="settings-field-label">Model cache root</span>
          <code className="settings-value">{modelManager.cacheRoot}</code>
        </div>
        <label className="settings-field settings-toggle">
          <input
            type="checkbox"
            checked={demoMode}
            onChange={(event) => changeDemoMode(event.target.checked)}
          />
          <span>
            Demo library — merge the fabricated demo catalog into the library
            for screenshots and walkthroughs (off by default).
          </span>
        </label>
      </MainSurface>

      <details className="settings-advanced">
        <summary className="model-type-group-heading">
          <h3>Advanced diagnostics</h3>
          <span>{overview.commands.length}</span>
        </summary>
        <div className="system-grid">
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
        </div>
      </details>
    </section>
  );
}
