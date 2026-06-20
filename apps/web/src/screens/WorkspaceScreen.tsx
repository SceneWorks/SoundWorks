// DR-02: Workspace (landing) screen. Extracted from App.tsx (F-010) and rebuilt
// on the shared component grammar (SurfaceHeader + HeroStat, MainSurface,
// SectionHeading, StatusBadge) instead of the bespoke project-workspace-panel /
// subpanel-heading classes. This is the template the other screens follow.
import { useState } from "react";
import {
  CircleCheck,
  FolderOpen,
  FolderPlus,
  Rocket,
  ShieldCheck,
  Sparkles,
} from "lucide-react";
import {
  FeedbackLine,
  HeroStat,
  MainSurface,
  SectionHeading,
  StatusBadge,
  SurfaceHeader,
} from "../components";
import { statusLabel, studioIconById, studioViewById } from "../viewModel";
import { useAppContext } from "./context";

const ONBOARDING_KEY = "soundworks-onboarding-dismissed";

export function WorkspaceScreen() {
  const {
    workspace,
    overview,
    createProject,
    openProject,
    openRecentProject,
    setActiveView,
    libraryActionStatus,
  } = useAppContext();

  const [newName, setNewName] = useState("");
  const firstRun = workspace.recentProjects.length === 0;
  const [guideOpen, setGuideOpen] = useState(() => {
    if (!firstRun) {
      return false;
    }
    try {
      return localStorage.getItem(ONBOARDING_KEY) !== "1";
    } catch {
      return true;
    }
  });
  const dismissGuide = () => {
    setGuideOpen(false);
    try {
      localStorage.setItem(ONBOARDING_KEY, "1");
    } catch {
      // ignore (private mode)
    }
  };
  const trimmedName = newName.trim();

  return (
    <>
      {guideOpen ? (
        <MainSurface className="onboarding-card" ariaLabel="Getting started">
          <SectionHeading
            eyebrow="Welcome to SoundWorks"
            title="Generate, review, and hand off audio for your video projects"
          />
          <p>
            Start by creating a project, then generate speech, SFX, samples, or a
            mixdown in the studios. Generation needs a model — check the Models
            page to see what is installed.
          </p>
          <div className="onboarding-actions">
            <button
              type="button"
              className="primary-action"
              onClick={() => {
                createProject(trimmedName || undefined);
                dismissGuide();
              }}
            >
              <FolderPlus aria-hidden="true" size={18} />
              <span>Create your first project</span>
            </button>
            <button
              type="button"
              className="secondary-action"
              onClick={() => setActiveView("models")}
            >
              Check model readiness
            </button>
            <button
              type="button"
              className="secondary-action"
              onClick={dismissGuide}
            >
              Dismiss
            </button>
          </div>
        </MainSurface>
      ) : null}

      <MainSurface
        className="workspace-overview"
        ariaLabel="Project workspace"
      >
        <SurfaceHeader
          eyebrow="Project workspace"
          title={workspace.activeProject.project.name}
          actions={
            <>
              <input
                className="field-input workspace-name-input"
                type="text"
                value={newName}
                onChange={(event) => setNewName(event.target.value)}
                placeholder="New project name…"
                aria-label="New project name"
                onKeyDown={(event) => {
                  if (
                    event.key === "Enter" &&
                    overview.workspace.canCreateProject
                  ) {
                    createProject(trimmedName || undefined);
                    setNewName("");
                  }
                }}
              />
              <button
                className="primary-action workspace-action"
                disabled={!overview.workspace.canCreateProject}
                onClick={() => {
                  createProject(trimmedName || undefined);
                  setNewName("");
                }}
                title="Create SoundWorks project"
                type="button"
              >
                <FolderPlus aria-hidden="true" size={18} />
                <span>Create</span>
              </button>
              <button
                className="secondary-icon-action"
                disabled={!overview.workspace.canOpenProject}
                onClick={openRecentProject}
                title="Open most recent SoundWorks project"
                type="button"
              >
                <FolderOpen aria-hidden="true" size={18} />
              </button>
              <button
                className="secondary-icon-action"
                onClick={() => setGuideOpen(true)}
                title="Show the getting-started guide"
                type="button"
              >
                <Rocket aria-hidden="true" size={18} />
              </button>
            </>
          }
          stats={
            <>
              <HeroStat
                label="Projects"
                value={overview.workspace.projectCount}
              />
              <HeroStat
                label="Project assets"
                value={overview.workspace.projectAssetCount}
              />
              <HeroStat
                label="Global links"
                value={overview.workspace.linkedGlobalAssetCount}
              />
              <HeroStat
                label="Global assets"
                value={overview.workspace.globalAssetCount}
              />
            </>
          }
        />

        <FeedbackLine feedback={libraryActionStatus} />

        <div className="workspace-layout">
          <section className="workspace-projects" aria-label="Recent projects">
            {workspace.recentProjects.length === 0 ? (
              <p className="field-hint">
                No projects yet — name one above and Create to get started.
              </p>
            ) : null}
            {workspace.recentProjects.map((project) => (
              <article
                className={
                  project.status === "active"
                    ? "workspace-project active"
                    : "workspace-project"
                }
                key={project.project.id}
              >
                <div className="workspace-project-title">
                  <strong>{project.project.name}</strong>
                  <StatusBadge
                    tone={project.status === "active" ? "installed" : "neutral"}
                  >
                    {statusLabel(project.status)}
                  </StatusBadge>
                </div>
                <small>{project.project.storageRoot}</small>
                <div className="asset-tag-row">
                  <span>{project.assetCount} assets</span>
                  <span>{project.compositionCount} composition</span>
                  <span>{project.linkedGlobalAssetCount} global link</span>
                </div>
                <button
                  type="button"
                  className="secondary-action"
                  disabled={project.status === "active"}
                  onClick={() => openProject(project.project.id)}
                  title={`Open ${project.project.name}`}
                >
                  {project.status === "active" ? "Current" : "Open"}
                </button>
              </article>
            ))}
          </section>

          <MainSurface
            className="workspace-global-card"
            ariaLabel="Global asset library"
          >
            <SectionHeading
              title={workspace.globalLibrary.label}
              eyebrow={`${workspace.globalLibrary.assetCount} assets`}
            />
            <p>{workspace.globalLibrary.storageRoot}</p>
            <div className="asset-tag-row detail-tags">
              <span>{workspace.globalLibrary.reusableVoiceCount} voice</span>
              <span>{workspace.globalLibrary.reusablePresetCount} preset</span>
              <span>
                {workspace.globalLibrary.reusableCollectionCount} collection
              </span>
            </div>
            <div className="workspace-scope-grid">
              {workspace.scopeControls.map((scope) => (
                <div
                  className={
                    scope.active
                      ? "workspace-scope-button active is-inert"
                      : "workspace-scope-button is-inert"
                  }
                  key={scope.id}
                  title={scope.emptyState}
                >
                  <span>{scope.label}</span>
                  <strong>{scope.itemCount}</strong>
                </div>
              ))}
            </div>
          </MainSurface>
        </div>

        <div className="workspace-bottom-grid">
          <MainSurface aria-label="Source picker policy">
            <SectionHeading
              title="Source picker"
              eyebrow={`${workspace.sourcePicker.targetSurfaces.length} surfaces`}
            />
            <div className="asset-tag-row detail-tags">
              {workspace.sourcePicker.targetSurfaces.map((target) => (
                <span key={target}>{target}</span>
              ))}
            </div>
            <ol className="voice-checks">
              {workspace.sourcePicker.provenanceRequirements.map(
                (requirement, index) => (
                  <li className="passed" key={index}>
                    <ShieldCheck aria-hidden="true" size={16} />
                    <span>{requirement}</span>
                  </li>
                ),
              )}
            </ol>
            <div className="workspace-link-list">
              {workspace.compositionLinks.map((link) => (
                <article key={link.id}>
                  <strong>{link.assetId}</strong>
                  <small>
                    {statusLabel(link.projectUsage)} / {link.versionId}
                  </small>
                  <p>{link.provenanceSidecarPath}</p>
                </article>
              ))}
            </div>
          </MainSurface>

          <MainSurface aria-label="Global reuse actions">
            <SectionHeading
              title="Reuse actions"
              eyebrow={`${workspace.transferActions.length} actions`}
            />
            <div className="workspace-action-list">
              {workspace.transferActions.map((action) => (
                <article key={action.id}>
                  <strong>{action.label}</strong>
                  <small>
                    {statusLabel(action.mode)} / {action.sourceItemId}
                  </small>
                  <p>{action.summary}</p>
                </article>
              ))}
            </div>
          </MainSurface>

          <MainSurface aria-label="Workspace validation">
            <SectionHeading
              title="Validation"
              eyebrow={`${workspace.validationChecks.length} checks`}
            />
            <ol className="voice-checks">
              {workspace.validationChecks.map((check) => (
                <li
                  className={check.passed ? "passed" : "failed"}
                  key={check.id}
                >
                  <CircleCheck aria-hidden="true" size={16} />
                  <span>{check.summary}</span>
                </li>
              ))}
            </ol>
          </MainSurface>
        </div>

        <div className="workspace-parity-strip" aria-label="SceneWorks parity">
          {workspace.parityNotes.map((note) => (
            <article key={note.id}>
              <strong>{note.area}</strong>
              <span>{note.soundworksApplication}</span>
            </article>
          ))}
        </div>
      </MainSurface>

      <section className="studio-grid" aria-label="Studios">
        {overview.studios.map((studio) => {
          const Icon = studioIconById[studio.id] ?? Sparkles;
          const viewId = studioViewById[studio.id];

          return (
            <button
              className="studio-card"
              key={studio.id}
              disabled={!viewId}
              onClick={() => {
                if (viewId) {
                  setActiveView(viewId);
                }
              }}
              type="button"
            >
              <span className="icon-badge">
                <Icon aria-hidden="true" size={22} />
              </span>
              <span className="studio-copy">
                <strong>{studio.name}</strong>
                <small>{studio.route}</small>
              </span>
              <StatusBadge
                tone={studio.status === "scaffolded" ? "installed" : "neutral"}
              >
                {studio.status}
              </StatusBadge>
            </button>
          );
        })}
      </section>
    </>
  );
}
