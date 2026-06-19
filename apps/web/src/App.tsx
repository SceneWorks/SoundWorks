import {
  Activity,
  Archive,
  Boxes,
  CircleAlert,
  CircleCheck,
  ClipboardCheck,
  Cpu,
  Disc3,
  Download,
  FileAudio,
  FileVideo,
  FolderOpen,
  FolderPlus,
  Gauge,
  HardDrive,
  Link2,
  Library,
  Mic2,
  Music2,
  PackageCheck,
  Play,
  Radio,
  Save,
  Search,
  ShieldCheck,
  SlidersHorizontal,
  Sparkles,
  Waves,
} from "lucide-react";
import { useEffect, useMemo, useState } from "react";
import {
  fallbackOverview,
  fallbackAssetLibrary,
  fallbackCompositionEditor,
  fallbackExportWorkflow,
  fallbackMvpValidation,
  fallbackModelManager,
  fallbackRightsSafety,
  fallbackReviewWorkspace,
  fallbackRuntime,
  fallbackSamplesStudio,
  fallbackSongStudio,
  fallbackSfxStudio,
  fallbackTtsStudio,
  fallbackVideoToAudio,
  fallbackVoiceLab,
  fallbackWorkspace,
} from "./appData";
import {
  loadAppOverview,
  loadAssetLibraryOverview,
  loadCompositionEditorOverview,
  loadExportWorkflowOverview,
  installModelCandidate,
  loadModelManagerOverview,
  loadMvpValidationOverview,
  loadRightsSafetyOverview,
  loadReviewWorkspaceOverview,
  loadRuntimeOverview,
  loadSamplesStudioOverview,
  loadSongStudioOverview,
  loadSfxStudioOverview,
  loadTtsStudioOverview,
  loadVideoToAudioOverview,
  loadVoiceLabOverview,
  loadWorkspaceOverview,
  revalidateModelCandidate,
} from "./tauri";
import type {
  AppOverview,
  AssetLibraryOverview,
  CompositionEditorOverview,
  ExportWorkflowOverview,
  ModelManagerOperation,
  ModelManagerOverview,
  MvpValidationOverview,
  RightsSafetyOverview,
  ReviewWorkspaceOverview,
  RuntimeOverview,
  SamplesStudioOverview,
  SongStudioOverview,
  SfxStudioOverview,
  TtsStudioOverview,
  VideoToAudioOverview,
  VoiceLabOverview,
  WorkspaceOverview,
} from "./types";

const navItems = [
  { label: "Studios", icon: Sparkles },
  { label: "Library", icon: Library },
  { label: "Mixer", icon: SlidersHorizontal },
  { label: "Jobs", icon: Activity },
];

const studioIcons = [
  Mic2,
  Radio,
  Waves,
  Boxes,
  Music2,
  ClipboardCheck,
  Sparkles,
];

function workflowLabel(workflow: string) {
  return workflow
    .split("-")
    .map((part) => part.charAt(0).toUpperCase() + part.slice(1))
    .join(" ");
}

function statusLabel(status: string) {
  return status
    .split("-")
    .map((part) => part.charAt(0).toUpperCase() + part.slice(1))
    .join(" ");
}

function formatMb(value?: number | null) {
  if (!value) {
    return "n/a";
  }

  return value >= 1024 ? `${Math.round(value / 1024)} GB` : `${value} MB`;
}

function formatDuration(ms: number) {
  const seconds = Math.round(ms / 100) / 10;

  return `${seconds}s`;
}

function countFor(counts: Record<string, number>, key: string) {
  return counts[key] ?? 0;
}

function visibleModelManagerOperation(operations: ModelManagerOperation[]) {
  return (
    operations.find((operation) => operation.status === "failed") ??
    operations[0] ??
    null
  );
}

function scopeLabel(scope: { kind: string; projectId?: string }) {
  return scope.kind === "globalLibrary"
    ? "Global"
    : (scope.projectId ?? "Project");
}

export function App() {
  const [overview, setOverview] = useState<AppOverview>(fallbackOverview);
  const [runtime, setRuntime] = useState<RuntimeOverview>(fallbackRuntime);
  const [modelManager, setModelManager] =
    useState<ModelManagerOverview>(fallbackModelManager);
  const [modelManagerOperation, setModelManagerOperation] =
    useState<ModelManagerOperation | null>(
      visibleModelManagerOperation(fallbackModelManager.operations),
    );
  const [workspace, setWorkspace] =
    useState<WorkspaceOverview>(fallbackWorkspace);
  const [assetLibrary, setAssetLibrary] =
    useState<AssetLibraryOverview>(fallbackAssetLibrary);
  const [exportWorkflow, setExportWorkflow] = useState<ExportWorkflowOverview>(
    fallbackExportWorkflow,
  );
  const [compositionEditor, setCompositionEditor] =
    useState<CompositionEditorOverview>(fallbackCompositionEditor);
  const [mvpValidation, setMvpValidation] = useState<MvpValidationOverview>(
    fallbackMvpValidation,
  );
  const [ttsStudio, setTtsStudio] =
    useState<TtsStudioOverview>(fallbackTtsStudio);
  const [voiceLab, setVoiceLab] = useState<VoiceLabOverview>(fallbackVoiceLab);
  const [sfxStudio, setSfxStudio] =
    useState<SfxStudioOverview>(fallbackSfxStudio);
  const [samplesStudio, setSamplesStudio] = useState<SamplesStudioOverview>(
    fallbackSamplesStudio,
  );
  const [songStudio, setSongStudio] =
    useState<SongStudioOverview>(fallbackSongStudio);
  const [reviewWorkspace, setReviewWorkspace] =
    useState<ReviewWorkspaceOverview>(fallbackReviewWorkspace);
  const [rightsSafety, setRightsSafety] =
    useState<RightsSafetyOverview>(fallbackRightsSafety);
  const [videoToAudio, setVideoToAudio] =
    useState<VideoToAudioOverview>(fallbackVideoToAudio);

  useEffect(() => {
    let active = true;

    loadAppOverview().then((nextOverview) => {
      if (active) {
        setOverview(nextOverview);
      }
    });

    loadRuntimeOverview().then((nextRuntime) => {
      if (active) {
        setRuntime(nextRuntime);
      }
    });

    loadModelManagerOverview().then((nextModelManager) => {
      if (active) {
        setModelManager(nextModelManager);
        setModelManagerOperation(
          visibleModelManagerOperation(nextModelManager.operations),
        );
      }
    });

    loadWorkspaceOverview().then((nextWorkspace) => {
      if (active) {
        setWorkspace(nextWorkspace);
      }
    });

    loadAssetLibraryOverview().then((nextAssetLibrary) => {
      if (active) {
        setAssetLibrary(nextAssetLibrary);
      }
    });

    loadExportWorkflowOverview().then((nextExportWorkflow) => {
      if (active) {
        setExportWorkflow(nextExportWorkflow);
      }
    });

    loadCompositionEditorOverview().then((nextCompositionEditor) => {
      if (active) {
        setCompositionEditor(nextCompositionEditor);
      }
    });

    loadMvpValidationOverview().then((nextMvpValidation) => {
      if (active) {
        setMvpValidation(nextMvpValidation);
      }
    });

    loadTtsStudioOverview().then((nextTtsStudio) => {
      if (active) {
        setTtsStudio(nextTtsStudio);
      }
    });

    loadVoiceLabOverview().then((nextVoiceLab) => {
      if (active) {
        setVoiceLab(nextVoiceLab);
      }
    });

    loadSfxStudioOverview().then((nextSfxStudio) => {
      if (active) {
        setSfxStudio(nextSfxStudio);
      }
    });

    loadSamplesStudioOverview().then((nextSamplesStudio) => {
      if (active) {
        setSamplesStudio(nextSamplesStudio);
      }
    });

    loadSongStudioOverview().then((nextSongStudio) => {
      if (active) {
        setSongStudio(nextSongStudio);
      }
    });

    loadReviewWorkspaceOverview().then((nextReviewWorkspace) => {
      if (active) {
        setReviewWorkspace(nextReviewWorkspace);
      }
    });

    loadRightsSafetyOverview().then((nextRightsSafety) => {
      if (active) {
        setRightsSafety(nextRightsSafety);
      }
    });

    loadVideoToAudioOverview().then((nextVideoToAudio) => {
      if (active) {
        setVideoToAudio(nextVideoToAudio);
      }
    });

    return () => {
      active = false;
    };
  }, []);

  function runModelManagerAction(
    candidateId: string,
    action: "install" | "revalidate",
  ) {
    const runner =
      action === "install" ? installModelCandidate : revalidateModelCandidate;
    runner(candidateId).then((operation) => {
      setModelManagerOperation(operation);
    });
  }

  const scaffoldedLayerCount = useMemo(
    () =>
      overview.architecture.layers.filter(
        (layer) => layer.status === "scaffolded",
      ).length,
    [overview.architecture.layers],
  );
  const voiceCandidateFocus = useMemo(
    () =>
      voiceLab.providerScorecards.filter((scorecard) =>
        ["chatterbox", "rvc", "chatterbox-turbo"].includes(
          scorecard.candidateId,
        ),
      ),
    [voiceLab.providerScorecards],
  );
  const sfxCandidateFocus = useMemo(
    () =>
      sfxStudio.providerScorecards.filter((scorecard) =>
        ["moss-soundeffect", "stable-audio-open-1", "mmaudio"].includes(
          scorecard.candidateId,
        ),
      ),
    [sfxStudio.providerScorecards],
  );
  const videoCandidateFocus = useMemo(
    () =>
      videoToAudio.providerScorecards.filter((scorecard) =>
        ["mmaudio", "audiox", "thinksound", "moss-soundeffect"].includes(
          scorecard.candidateId,
        ),
      ),
    [videoToAudio.providerScorecards],
  );
  const samplesCandidateFocus = useMemo(
    () =>
      samplesStudio.providerScorecards.filter((scorecard) =>
        ["ace-step-1-5", "stable-audio-open-1", "heartmula"].includes(
          scorecard.candidateId,
        ),
      ),
    [samplesStudio.providerScorecards],
  );
  const songCandidateFocus = useMemo(
    () =>
      songStudio.providerScorecards.filter((scorecard) =>
        ["ace-step-1-5", "stable-audio-3", "diffrhythm-2"].includes(
          scorecard.candidateId,
        ),
      ),
    [songStudio.providerScorecards],
  );

  return (
    <main className="app-shell">
      <aside className="sidebar" aria-label="Primary">
        <div className="brand-mark" aria-label={overview.productName}>
          <Waves aria-hidden="true" size={28} />
          <span>{overview.productName}</span>
        </div>

        <nav className="nav-list">
          {navItems.map((item) => (
            <button
              className="nav-button"
              key={item.label}
              type="button"
              title={item.label}
            >
              <item.icon aria-hidden="true" size={18} />
              <span>{item.label}</span>
            </button>
          ))}
        </nav>
      </aside>

      <section className="workspace" aria-label="Workspace">
        <header className="workspace-header">
          <div>
            <p className="eyebrow">Local workspace</p>
            <h1>{overview.productName}</h1>
          </div>
          <div className="status-strip" aria-label="Scaffold status">
            <strong>{scaffoldedLayerCount}</strong>
            <span>active layers</span>
          </div>
        </header>

        <section
          className="project-workspace-panel"
          aria-label="Project workspace"
        >
          <div className="project-workspace-header">
            <div>
              <p className="eyebrow">Project workspace</p>
              <h2>{workspace.activeProject.project.name}</h2>
            </div>
            <div className="workspace-actions">
              <button
                className="primary-action workspace-action"
                disabled={!overview.workspace.canCreateProject}
                title="Create SoundWorks project"
                type="button"
              >
                <FolderPlus aria-hidden="true" size={18} />
                <span>Create</span>
              </button>
              <button
                className="secondary-icon-action"
                disabled={!overview.workspace.canOpenProject}
                title="Open SoundWorks project"
                type="button"
              >
                <FolderOpen aria-hidden="true" size={18} />
              </button>
            </div>
          </div>

          <div className="workspace-metrics" aria-label="Workspace status">
            <div>
              <FolderOpen aria-hidden="true" size={18} />
              <strong>{overview.workspace.projectCount}</strong>
              <span>projects</span>
            </div>
            <div>
              <Library aria-hidden="true" size={18} />
              <strong>{overview.workspace.projectAssetCount}</strong>
              <span>project assets</span>
            </div>
            <div>
              <Link2 aria-hidden="true" size={18} />
              <strong>{overview.workspace.linkedGlobalAssetCount}</strong>
              <span>global links</span>
            </div>
            <div>
              <Boxes aria-hidden="true" size={18} />
              <strong>{overview.workspace.globalAssetCount}</strong>
              <span>global assets</span>
            </div>
          </div>

          <div className="workspace-layout">
            <section
              className="workspace-projects"
              aria-label="Recent projects"
            >
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
                    <span>{statusLabel(project.status)}</span>
                  </div>
                  <small>{project.project.storageRoot}</small>
                  <div className="asset-tag-row">
                    <span>{project.assetCount} assets</span>
                    <span>{project.compositionCount} composition</span>
                    <span>{project.linkedGlobalAssetCount} global link</span>
                  </div>
                </article>
              ))}
            </section>

            <section
              className="workspace-global-card"
              aria-label="Global asset library"
            >
              <div className="subpanel-heading">
                <h3>{workspace.globalLibrary.label}</h3>
                <span>{workspace.globalLibrary.assetCount}</span>
              </div>
              <p>{workspace.globalLibrary.storageRoot}</p>
              <div className="asset-tag-row detail-tags">
                <span>{workspace.globalLibrary.reusableVoiceCount} voice</span>
                <span>
                  {workspace.globalLibrary.reusablePresetCount} preset
                </span>
                <span>
                  {workspace.globalLibrary.reusableCollectionCount} collection
                </span>
              </div>
              <div className="workspace-scope-grid">
                {workspace.scopeControls.map((scope) => (
                  <button
                    className={
                      scope.active
                        ? "workspace-scope-button active"
                        : "workspace-scope-button"
                    }
                    key={scope.id}
                    title={scope.emptyState}
                    type="button"
                  >
                    <span>{scope.label}</span>
                    <strong>{scope.itemCount}</strong>
                  </button>
                ))}
              </div>
            </section>
          </div>

          <div className="workspace-bottom-grid">
            <section className="tts-subpanel" aria-label="Source picker policy">
              <div className="subpanel-heading">
                <h3>Source picker</h3>
                <span>{workspace.sourcePicker.targetSurfaces.length}</span>
              </div>
              <div className="asset-tag-row detail-tags">
                {workspace.sourcePicker.targetSurfaces.map((target) => (
                  <span key={target}>{target}</span>
                ))}
              </div>
              <ol className="voice-checks">
                {workspace.sourcePicker.provenanceRequirements.map(
                  (requirement) => (
                    <li className="passed" key={requirement}>
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
            </section>

            <section className="tts-subpanel" aria-label="Global reuse actions">
              <div className="subpanel-heading">
                <h3>Reuse actions</h3>
                <span>{workspace.transferActions.length}</span>
              </div>
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
            </section>

            <section className="tts-subpanel" aria-label="Workspace validation">
              <div className="subpanel-heading">
                <h3>Validation</h3>
                <span>{workspace.validationChecks.length}</span>
              </div>
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
            </section>
          </div>

          <div
            className="workspace-parity-strip"
            aria-label="SceneWorks parity"
          >
            {workspace.parityNotes.map((note) => (
              <article key={note.id}>
                <strong>{note.area}</strong>
                <span>{note.soundworksApplication}</span>
              </article>
            ))}
          </div>
        </section>

        <section className="studio-grid" aria-label="Studios">
          {overview.studios.map((studio, index) => {
            const Icon = studioIcons[index % studioIcons.length];

            return (
              <button className="studio-card" key={studio.id} type="button">
                <span className="icon-badge">
                  <Icon aria-hidden="true" size={22} />
                </span>
                <span className="studio-copy">
                  <strong>{studio.name}</strong>
                  <small>{studio.route}</small>
                </span>
                <span className={`state-pill ${studio.status}`}>
                  {studio.status}
                </span>
              </button>
            );
          })}
        </section>

        <section className="asset-library-panel" aria-label="Asset Library">
          <div className="library-header">
            <div>
              <p className="eyebrow">Asset Library</p>
              <h2>Project and global audio assets</h2>
            </div>
            <div className="library-search" role="search">
              <Search aria-hidden="true" size={18} />
              <span>{assetLibrary.selectedFilter.searchText}</span>
            </div>
          </div>

          <div className="library-metrics" aria-label="Library status">
            <div>
              <Library aria-hidden="true" size={18} />
              <strong>{overview.assetLibrary.itemCount}</strong>
              <span>items</span>
            </div>
            <div>
              <Play aria-hidden="true" size={18} />
              <strong>{overview.assetLibrary.previewableItemCount}</strong>
              <span>previewable</span>
            </div>
            <div>
              <Boxes aria-hidden="true" size={18} />
              <strong>{overview.assetLibrary.collectionCount}</strong>
              <span>collections</span>
            </div>
            <div>
              <Archive aria-hidden="true" size={18} />
              <strong>{overview.assetLibrary.supportedTypeCount}</strong>
              <span>asset types</span>
            </div>
          </div>

          <div className="library-scope-row" aria-label="Library scopes">
            {assetLibrary.scopes.map((scope) => (
              <article className="library-scope" key={scope.id}>
                <div>
                  <strong>{scope.label}</strong>
                  <small>{statusLabel(scope.ownership)}</small>
                </div>
                <span>{scope.assetCount} assets</span>
              </article>
            ))}
          </div>

          <div className="library-filter-strip" aria-label="Asset filters">
            {assetLibrary.filters.facets.map((facet) => (
              <section className="filter-chip-group" key={facet.id}>
                <strong>{facet.label}</strong>
                <div>
                  {facet.options.slice(0, 3).map((option) => (
                    <span
                      className={
                        option.selected ? "filter-chip selected" : "filter-chip"
                      }
                      key={option.id}
                    >
                      {option.label} {option.count}
                    </span>
                  ))}
                </div>
              </section>
            ))}
          </div>

          <div className="library-layout">
            <div className="library-item-list" aria-label="Library assets">
              {assetLibrary.items.map((item) => (
                <article
                  className={
                    item.id === assetLibrary.selectedItem.item.id
                      ? "library-item selected"
                      : "library-item"
                  }
                  key={item.id}
                >
                  <div className="waveform-thumb" aria-hidden="true">
                    {Array.from({ length: 18 }).map((_, index) => (
                      <span
                        key={`${item.id}-${index}`}
                        style={{ height: `${18 + ((index * 11) % 34)}px` }}
                      />
                    ))}
                  </div>
                  <div className="library-item-main">
                    <div className="library-item-title">
                      <strong>{item.name}</strong>
                      <span>{item.itemTypeLabel}</span>
                    </div>
                    <small>
                      {item.bpm ? `${item.bpm} BPM / ` : ""}
                      {item.musicalKey ? `${item.musicalKey} / ` : ""}
                      {item.durationMs
                        ? formatDuration(item.durationMs)
                        : "metadata"}
                    </small>
                    <div className="asset-tag-row">
                      {[...item.tags, ...item.generatedTags]
                        .slice(0, 5)
                        .map((tag) => (
                          <span key={`${item.id}-${tag}`}>{tag}</span>
                        ))}
                    </div>
                  </div>
                  <button
                    className="icon-action"
                    disabled={!item.quickAudition.previewable}
                    title={`Preview ${item.name}`}
                    type="button"
                  >
                    <Play aria-hidden="true" size={16} />
                  </button>
                </article>
              ))}
            </div>

            <div className="library-detail" aria-label="Asset detail">
              <section className="tts-subpanel">
                <div className="subpanel-heading">
                  <h3>{assetLibrary.selectedItem.item.name}</h3>
                  <span>{assetLibrary.selectedItem.item.itemTypeLabel}</span>
                </div>
                <p>
                  {assetLibrary.selectedItem.item.ownership} /{" "}
                  {assetLibrary.selectedItem.item.licenseStatus} /{" "}
                  {assetLibrary.selectedItem.item.commercialUse}
                </p>
                <div className="asset-tag-row detail-tags">
                  {assetLibrary.selectedItem.item.badges.map((badge) => (
                    <span key={badge}>{badge}</span>
                  ))}
                </div>
              </section>

              <section className="tts-subpanel">
                <div className="subpanel-heading">
                  <h3>Version history</h3>
                  <span>{assetLibrary.selectedItem.versionCount}</span>
                </div>
                <ol className="version-list">
                  {assetLibrary.selectedItem.versionHistory.map((version) => (
                    <li key={version.versionId}>
                      <CircleCheck aria-hidden="true" size={16} />
                      <div>
                        <strong>{version.label}</strong>
                        <small>{version.versionId}</small>
                      </div>
                    </li>
                  ))}
                </ol>
              </section>

              <section className="tts-subpanel">
                <div className="subpanel-heading">
                  <h3>Recipe provenance</h3>
                  <span>
                    {assetLibrary.selectedItem.recipe?.workflow ?? "manual"}
                  </span>
                </div>
                <p>
                  {assetLibrary.selectedItem.recipe
                    ? `${assetLibrary.selectedItem.recipe.id} / ${assetLibrary.selectedItem.recipe.modelId}`
                    : "No generation recipe attached."}
                </p>
                <ol className="policy-list">
                  {assetLibrary.selectedItem.provenanceLinks.map((link) => (
                    <li key={link.id}>
                      <ShieldCheck aria-hidden="true" size={16} />
                      <span>{link.label}</span>
                    </li>
                  ))}
                </ol>
              </section>
            </div>
          </div>

          <div className="library-bottom-grid">
            <section className="tts-subpanel" aria-label="Collections">
              <div className="subpanel-heading">
                <h3>Collections</h3>
                <span>{assetLibrary.collections.length}</span>
              </div>
              <div className="collection-grid">
                {assetLibrary.collections.map((collection) => (
                  <article
                    className="collection-card"
                    key={collection.collection.id}
                  >
                    <strong>{collection.collection.name}</strong>
                    <small>
                      {statusLabel(collection.collectionType)} /{" "}
                      {collection.itemCount} items
                    </small>
                    <p>{collection.description}</p>
                  </article>
                ))}
              </div>
            </section>

            <section className="tts-subpanel" aria-label="Lifecycle actions">
              <div className="subpanel-heading">
                <h3>Lifecycle</h3>
                <span>{assetLibrary.lifecycleActions.length}</span>
              </div>
              <div className="lifecycle-actions">
                {assetLibrary.lifecycleActions.map((action) => (
                  <button
                    className="secondary-action"
                    key={action.id}
                    type="button"
                  >
                    {action.label}
                  </button>
                ))}
              </div>
            </section>

            <section className="tts-subpanel" aria-label="Library validation">
              <div className="subpanel-heading">
                <h3>Validation</h3>
                <span>{assetLibrary.validationChecks.length}</span>
              </div>
              <ol className="voice-checks">
                {assetLibrary.validationChecks.map((check) => (
                  <li
                    className={check.passed ? "passed" : "failed"}
                    key={check.id}
                  >
                    <CircleCheck aria-hidden="true" size={16} />
                    <span>{check.summary}</span>
                  </li>
                ))}
              </ol>
            </section>
          </div>
        </section>

        <section className="export-workflow-panel" aria-label="Export Workflow">
          <div className="export-header">
            <div>
              <p className="eyebrow">Export</p>
              <h2>Presets, stems, and handoff packages</h2>
            </div>
            <button
              className="primary-action export-action"
              disabled={!exportWorkflow.selectedExport.canExport}
              title="Export selected composition"
              type="button"
            >
              <Download aria-hidden="true" size={18} />
              <span>
                {exportWorkflow.selectedExport.canExport ? "Export" : "Blocked"}
              </span>
            </button>
          </div>

          <div className="export-metrics" aria-label="Export status">
            <div>
              <FileAudio aria-hidden="true" size={18} />
              <strong>{overview.exportWorkflow.presetCount}</strong>
              <span>presets</span>
            </div>
            <div>
              <ShieldCheck aria-hidden="true" size={18} />
              <strong>{overview.exportWorkflow.sidecarCount}</strong>
              <span>sidecars</span>
            </div>
            <div>
              <Boxes aria-hidden="true" size={18} />
              <strong>{overview.exportWorkflow.readyTargetCount}</strong>
              <span>targets ready</span>
            </div>
            <div>
              <Save aria-hidden="true" size={18} />
              <strong>{overview.exportWorkflow.selectedFormatCount}</strong>
              <span>formats selected</span>
            </div>
          </div>

          <div className="export-layout">
            <div className="export-preset-grid" aria-label="Export presets">
              {exportWorkflow.presets.map((preset) => (
                <article className="export-preset-card" key={preset.preset.id}>
                  <div className="export-preset-topline">
                    <strong>{preset.preset.name}</strong>
                    <span>{statusLabel(preset.preset.target)}</span>
                  </div>
                  <p>{preset.description}</p>
                  <div className="asset-tag-row">
                    {preset.formats.map((format) => (
                      <span key={`${preset.preset.id}-${format}`}>
                        {format}
                      </span>
                    ))}
                    {preset.preset.includeStems ? <span>stems</span> : null}
                    {preset.writesSidecar ? <span>sidecar</span> : null}
                  </div>
                </article>
              ))}
            </div>

            <div className="export-detail" aria-label="Selected export detail">
              <section className="tts-subpanel">
                <div className="subpanel-heading">
                  <h3>Selected export</h3>
                  <span>
                    {statusLabel(exportWorkflow.selectedExport.sourceKind)}
                  </span>
                </div>
                <p>
                  {exportWorkflow.selectedExport.presetId} /{" "}
                  {exportWorkflow.selectedExport.sourceId}
                </p>
                <ol className="version-list">
                  {exportWorkflow.selectedExport.outputPaths.map((path) => (
                    <li key={path}>
                      <CircleCheck aria-hidden="true" size={16} />
                      <div>
                        <strong>{path.split("/").pop()}</strong>
                        <small>{path}</small>
                      </div>
                    </li>
                  ))}
                </ol>
              </section>

              <section className="tts-subpanel">
                <div className="subpanel-heading">
                  <h3>DAW bundle</h3>
                  <span>{exportWorkflow.dawHandoff.stemKinds.length}</span>
                </div>
                <p>{exportWorkflow.dawHandoff.packagePath}</p>
                <div className="asset-tag-row detail-tags">
                  <span>zip bundle</span>
                  <span>cue markers</span>
                  <span>loop markers</span>
                  <span>BPM/key</span>
                  <span>lyrics</span>
                </div>
              </section>

              <section className="tts-subpanel">
                <div className="subpanel-heading">
                  <h3>SceneWorks handoff</h3>
                  <span>
                    {formatDuration(
                      exportWorkflow.sceneWorksHandoff.durationMs,
                    )}
                  </span>
                </div>
                <p>{exportWorkflow.sceneWorksHandoff.packagePath}</p>
                <small>
                  {exportWorkflow.sceneWorksHandoff.sampleRateHz} Hz /{" "}
                  {exportWorkflow.sceneWorksHandoff.channels} channels /{" "}
                  {exportWorkflow.sceneWorksHandoff.markerCount} marker /{" "}
                  {statusLabel(exportWorkflow.sceneWorksHandoff.importStrategy)}
                </small>
                <div className="asset-tag-row detail-tags">
                  <span>
                    {exportWorkflow.sceneWorksHandoff.sceneWorksAssetType}
                  </span>
                  <span>
                    {exportWorkflow.sceneWorksHandoff.sceneWorksMimeType}
                  </span>
                  <span>
                    {exportWorkflow.sceneWorksHandoff.replaceExistingAudio
                      ? "replace enabled"
                      : "attach only"}
                  </span>
                </div>
                <p>{exportWorkflow.sceneWorksHandoff.packageManifestPath}</p>
                <small>
                  {exportWorkflow.sceneWorksHandoff.intendedProjectId} /{" "}
                  {exportWorkflow.sceneWorksHandoff.intendedVideoAssetId}
                </small>
              </section>
            </div>
          </div>

          <div className="export-bottom-grid">
            <section className="tts-subpanel" aria-label="Export targets">
              <div className="subpanel-heading">
                <h3>Targets</h3>
                <span>{exportWorkflow.targets.length}</span>
              </div>
              <ol className="voice-checks">
                {exportWorkflow.targets.map((target) => (
                  <li
                    className={target.ready ? "passed" : "failed"}
                    key={target.target}
                  >
                    <CircleCheck aria-hidden="true" size={16} />
                    <span>
                      <strong>{target.label}</strong> {target.notes[0]}
                    </span>
                  </li>
                ))}
              </ol>
            </section>

            <section className="tts-subpanel" aria-label="Export sidecars">
              <div className="subpanel-heading">
                <h3>Sidecars</h3>
                <span>{exportWorkflow.sidecars.length}</span>
              </div>
              <div className="sidecar-list">
                {exportWorkflow.sidecars.map((sidecar) => (
                  <article key={sidecar.id}>
                    <strong>{sidecar.assetId}</strong>
                    <small>
                      {statusLabel(sidecar.target)} / {sidecar.eventCount}{" "}
                      events
                    </small>
                    <p>{sidecar.path}</p>
                  </article>
                ))}
              </div>
            </section>

            <section className="tts-subpanel" aria-label="Export validation">
              <div className="subpanel-heading">
                <h3>Validation</h3>
                <span>{exportWorkflow.validationChecks.length}</span>
              </div>
              <ol className="voice-checks">
                {exportWorkflow.validationChecks.map((check) => (
                  <li
                    className={check.passed ? "passed" : "failed"}
                    key={check.id}
                  >
                    <ClipboardCheck aria-hidden="true" size={16} />
                    <span>{check.summary}</span>
                  </li>
                ))}
              </ol>
            </section>

            <section
              className="tts-subpanel"
              aria-label="SceneWorks compatibility"
            >
              <div className="subpanel-heading">
                <h3>SceneWorks compatibility</h3>
                <span>
                  {exportWorkflow.sceneWorksHandoff.compatibilityChecks.length}
                </span>
              </div>
              <ol className="voice-checks">
                {exportWorkflow.sceneWorksHandoff.compatibilityChecks.map(
                  (check) => (
                    <li
                      className={
                        check.status === "blocked" ? "failed" : "passed"
                      }
                      key={check.id}
                    >
                      <ClipboardCheck aria-hidden="true" size={16} />
                      <span>
                        <strong>{statusLabel(check.status)}</strong>{" "}
                        {check.summary}
                      </span>
                    </li>
                  ),
                )}
              </ol>
            </section>

            <section
              className="tts-subpanel"
              aria-label="SceneWorks attachment steps"
            >
              <div className="subpanel-heading">
                <h3>SceneWorks attachment</h3>
                <span>
                  {exportWorkflow.sceneWorksHandoff.attachmentSteps.length}
                </span>
              </div>
              <ol className="version-list">
                {exportWorkflow.sceneWorksHandoff.attachmentSteps.map(
                  (step) => (
                    <li key={step.id}>
                      <CircleCheck aria-hidden="true" size={16} />
                      <div>
                        <strong>{step.label}</strong>
                        <small>
                          {step.source}
                          {" -> "}
                          {step.target}
                        </small>
                      </div>
                    </li>
                  ),
                )}
              </ol>
            </section>
          </div>
        </section>

        <section
          className="composition-editor-panel"
          aria-label="Multitrack Composition Editor"
        >
          <div className="composition-header">
            <div>
              <p className="eyebrow">Multitrack editor</p>
              <h2>{compositionEditor.composition.name}</h2>
            </div>
            <button
              className="primary-action composition-action"
              disabled={!compositionEditor.exportPlan.canRenderMixdown}
              title="Render composition mixdown"
              type="button"
            >
              <Disc3 aria-hidden="true" size={18} />
              <span>
                {compositionEditor.exportPlan.canRenderMixdown
                  ? "Render"
                  : "Blocked"}
              </span>
            </button>
          </div>

          <div className="composition-metrics" aria-label="Editor status">
            <div>
              <SlidersHorizontal aria-hidden="true" size={18} />
              <strong>{overview.compositionEditor.trackCount}</strong>
              <span>tracks</span>
            </div>
            <div>
              <FileAudio aria-hidden="true" size={18} />
              <strong>{overview.compositionEditor.clipCount}</strong>
              <span>clips</span>
            </div>
            <div>
              <Library aria-hidden="true" size={18} />
              <strong>{overview.compositionEditor.assetBinCount}</strong>
              <span>assets</span>
            </div>
            <div>
              <Gauge aria-hidden="true" size={18} />
              <strong>{compositionEditor.timeline.zoomPercent}%</strong>
              <span>{compositionEditor.timeline.snapGridMs}ms grid</span>
            </div>
          </div>

          <div className="composition-layout">
            <div className="composition-main">
              <section
                className="composition-toolbar"
                aria-label="Editor tools"
              >
                {compositionEditor.tools.map((tool) => (
                  <button
                    className={
                      tool.id === compositionEditor.timeline.selectedTool
                        ? "tool-button selected"
                        : "tool-button"
                    }
                    disabled={!tool.enabled}
                    key={tool.id}
                    title={tool.label}
                    type="button"
                  >
                    <span>{tool.label}</span>
                  </button>
                ))}
              </section>

              <section className="timeline-board" aria-label="Timeline tracks">
                <div
                  className="timeline-selection"
                  aria-label="Timeline selection"
                >
                  <span>{compositionEditor.timeline.selectedClipId}</span>
                  <span>
                    cursor{" "}
                    {formatDuration(
                      compositionEditor.timeline.playbackCursorMs,
                    )}
                  </span>
                  <span>
                    loop{" "}
                    {formatDuration(compositionEditor.timeline.loopRange.endMs)}
                  </span>
                </div>
                <div className="timeline-ruler" aria-label="Timeline ruler">
                  {compositionEditor.timeline.gridLabels.map((label) => (
                    <span key={label}>{label}</span>
                  ))}
                </div>
                {compositionEditor.tracks.map((track) => (
                  <article className="timeline-track" key={track.trackId}>
                    <div className="track-strip">
                      <strong>{track.name}</strong>
                      <small>
                        {statusLabel(track.role)} / {track.gainDb} dB / pan{" "}
                        {track.pan}
                      </small>
                      <span>
                        {track.muted ? "Muted" : "Live"} /{" "}
                        {track.soloed ? "Solo" : "Mix"}
                      </span>
                    </div>
                    <div className="clip-lane">
                      {track.clips.map((clip) => (
                        <button
                          className={
                            clip.clipId ===
                            compositionEditor.timeline.selectedClipId
                              ? "timeline-clip selected"
                              : "timeline-clip"
                          }
                          key={clip.clipId}
                          style={{
                            marginLeft: `${Math.min(
                              68,
                              clip.timelineStartMs / 420,
                            )}%`,
                            width: `${Math.max(
                              16,
                              Math.min(
                                38,
                                (clip.sourceRange.endMs -
                                  clip.sourceRange.startMs) /
                                  520,
                              ),
                            )}%`,
                          }}
                          title={clip.assetName}
                          type="button"
                        >
                          <strong>{clip.assetName}</strong>
                          <span>{statusLabel(clip.assetKind)}</span>
                        </button>
                      ))}
                    </div>
                  </article>
                ))}
              </section>

              <section className="tts-subpanel" aria-label="Editor validation">
                <div className="subpanel-heading">
                  <h3>Validation</h3>
                  <span>{compositionEditor.validationChecks.length}</span>
                </div>
                <ol className="voice-checks">
                  {compositionEditor.validationChecks.map((check) => (
                    <li
                      className={check.passed ? "passed" : "failed"}
                      key={check.id}
                    >
                      <ClipboardCheck aria-hidden="true" size={16} />
                      <span>{check.summary}</span>
                    </li>
                  ))}
                </ol>
              </section>
            </div>

            <div className="composition-side">
              <section className="tts-subpanel" aria-label="Timeline assets">
                <div className="subpanel-heading">
                  <h3>Asset bin</h3>
                  <span>{compositionEditor.assetBin.length}</span>
                </div>
                <div className="asset-bin-list">
                  {compositionEditor.assetBin.map((asset) => (
                    <article key={asset.assetId}>
                      <strong>{asset.name}</strong>
                      <small>
                        {statusLabel(asset.kind)} / {scopeLabel(asset.scope)}
                      </small>
                      <div className="asset-tag-row">
                        <span>{formatDuration(asset.durationMs)}</span>
                        <span>{statusLabel(asset.sourceWorkflow)}</span>
                        {asset.draggableToTimeline ? (
                          <span>placeable</span>
                        ) : null}
                      </div>
                    </article>
                  ))}
                </div>
              </section>

              <section className="tts-subpanel" aria-label="Mixer state">
                <div className="subpanel-heading">
                  <h3>Mixer</h3>
                  <span>{compositionEditor.mixer.targetLufs} LUFS</span>
                </div>
                <p>{compositionEditor.mixer.loudnessCheck}</p>
                <div className="mixer-list">
                  {compositionEditor.mixer.trackStates.map((track) => (
                    <article key={track.trackId}>
                      <strong>{track.label}</strong>
                      <small>
                        {track.gainDb} dB / pan {track.pan}
                      </small>
                      <div className="asset-tag-row">
                        {track.effectChain.map((effect) => (
                          <span key={`${track.trackId}-${effect}`}>
                            {effect}
                          </span>
                        ))}
                        {track.sendTargets.map((send) => (
                          <span key={`${track.trackId}-${send}`}>{send}</span>
                        ))}
                      </div>
                    </article>
                  ))}
                </div>
              </section>
            </div>
          </div>

          <div className="composition-bottom-grid">
            <section
              className="tts-subpanel"
              aria-label="Generated asset flows"
            >
              <div className="subpanel-heading">
                <h3>Studio flows</h3>
                <span>{compositionEditor.sourceFlows.length}</span>
              </div>
              <ol className="voice-checks">
                {compositionEditor.sourceFlows.map((flow) => (
                  <li
                    className={flow.status === "ready" ? "passed" : "failed"}
                    key={`${flow.workflow}-${flow.assetKind}`}
                  >
                    <CircleCheck aria-hidden="true" size={16} />
                    <span>
                      <strong>{flow.label}</strong>{" "}
                      {statusLabel(flow.assetKind)}
                    </span>
                  </li>
                ))}
              </ol>
            </section>

            <section className="tts-subpanel" aria-label="Render plan">
              <div className="subpanel-heading">
                <h3>Render plan</h3>
                <span>
                  {compositionEditor.exportPlan.canRenderMixdown
                    ? "ready"
                    : "blocked"}
                </span>
              </div>
              <p>{compositionEditor.exportPlan.mixdownPath}</p>
              <div className="asset-tag-row detail-tags">
                {compositionEditor.exportPlan.presetIds.map((preset) => (
                  <span key={preset}>{preset}</span>
                ))}
              </div>
              <small>{compositionEditor.exportPlan.sceneWorksWarning}</small>
            </section>

            <section
              className="tts-subpanel"
              aria-label="Editor component decision"
            >
              <div className="subpanel-heading">
                <h3>Component decision</h3>
                <span>{overview.compositionEditor.recommendedComponentId}</span>
              </div>
              <div className="component-decision-list">
                {compositionEditor.componentDecisions.map((decision) => (
                  <article
                    className={
                      decision.fit === "strong-prototype-candidate"
                        ? "recommended"
                        : ""
                    }
                    key={decision.id}
                  >
                    <strong>{decision.name}</strong>
                    <small>
                      {decision.license} / {statusLabel(decision.fit)}
                    </small>
                    <p>{decision.prototypeEvidence}</p>
                    <p>{decision.decision}</p>
                    <a href={decision.sourceUrl}>{decision.sourceUrl}</a>
                  </article>
                ))}
              </div>
            </section>
          </div>
        </section>

        <section className="mvp-validation-panel" aria-label="MVP Validation">
          <div className="mvp-header">
            <div>
              <p className="eyebrow">MVP validation</p>
              <h2>Release gate and demo matrix</h2>
            </div>
            <button
              className="primary-action validation-action"
              disabled={!mvpValidation.releaseGate.readyForMvp}
              title="MVP release gate"
              type="button"
            >
              <ClipboardCheck aria-hidden="true" size={18} />
              <span>
                {mvpValidation.releaseGate.readyForMvp ? "Ready" : "Blocked"}
              </span>
            </button>
          </div>

          <div className="mvp-metrics" aria-label="Validation status">
            <div>
              <CircleCheck aria-hidden="true" size={18} />
              <strong>
                {mvpValidation.releaseGate.coveredWorkflowCount}/
                {mvpValidation.releaseGate.requiredWorkflowCount}
              </strong>
              <span>workflows covered</span>
            </div>
            <div>
              <ClipboardCheck aria-hidden="true" size={18} />
              <strong>
                {mvpValidation.releaseGate.passedAutomatedCheckCount}/
                {mvpValidation.releaseGate.requiredAutomatedCheckCount}
              </strong>
              <span>automated checks</span>
            </div>
            <div>
              <Gauge aria-hidden="true" size={18} />
              <strong>
                {mvpValidation.releaseGate.passedManualScorecardCount}/
                {mvpValidation.releaseGate.requiredManualScorecardCount}
              </strong>
              <span>manual scorecards</span>
            </div>
            <div>
              <HardDrive aria-hidden="true" size={18} />
              <strong>
                {mvpValidation.releaseGate.satisfiedRuntimeEvidenceCount}/
                {mvpValidation.releaseGate.requiredRuntimeEvidenceCount}
              </strong>
              <span>runtime evidence</span>
            </div>
            <div>
              <CircleAlert aria-hidden="true" size={18} />
              <strong>{overview.mvpValidation.blockingItemCount}</strong>
              <span>blocking items</span>
            </div>
          </div>

          <div className="mvp-layout">
            <div className="mvp-main">
              <section
                className="tts-subpanel"
                aria-label="Golden demo workflows"
              >
                <div className="subpanel-heading">
                  <h3>Golden demos</h3>
                  <span>{mvpValidation.demoWorkflows.length}</span>
                </div>
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
                          .map((artifact) => (
                            <span key={`${workflow.id}-${artifact}`}>
                              {artifact}
                            </span>
                          ))}
                      </div>
                    </article>
                  ))}
                </div>
              </section>

              <section
                className="tts-subpanel"
                aria-label="Requirement coverage"
              >
                <div className="subpanel-heading">
                  <h3>Epic requirement coverage</h3>
                  <span>{mvpValidation.requirementCoverage.length}</span>
                </div>
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
              </section>
            </div>

            <div className="mvp-side">
              <section className="tts-subpanel" aria-label="MVP blockers">
                <div className="subpanel-heading">
                  <h3>Release blockers</h3>
                  <span>{mvpValidation.releaseGate.blockingItems.length}</span>
                </div>
                <ol className="voice-checks">
                  {mvpValidation.releaseGate.blockingItems.map((item) => (
                    <li className="blocked" key={item}>
                      <CircleAlert aria-hidden="true" size={16} />
                      <span>{item}</span>
                    </li>
                  ))}
                </ol>
              </section>

              <section
                className="tts-subpanel"
                aria-label="Automated validation checks"
              >
                <div className="subpanel-heading">
                  <h3>Automated checks</h3>
                  <span>{mvpValidation.automatedChecks.length}</span>
                </div>
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
              </section>

              <section className="tts-subpanel" aria-label="Runtime evidence">
                <div className="subpanel-heading">
                  <h3>Runtime evidence</h3>
                  <span>
                    {mvpValidation.releaseGate.satisfiedRuntimeEvidenceCount}/
                    {mvpValidation.releaseGate.requiredRuntimeEvidenceCount}
                  </span>
                </div>
                <ol className="voice-checks">
                  {mvpValidation.runtimeEvidence.map((evidence) => (
                    <li className={evidence.status} key={evidence.id}>
                      <CircleAlert aria-hidden="true" size={16} />
                      <span>
                        <strong>{workflowLabel(evidence.workflow)}</strong>{" "}
                        {evidence.requirement}
                        <em>
                          {evidence.fixtureOnly
                            ? "Fixture-only: "
                            : "Evidence: "}
                          {evidence.evidence}
                        </em>
                        <em>{evidence.blocker}</em>
                      </span>
                    </li>
                  ))}
                </ol>
              </section>
            </div>
          </div>

          <div className="mvp-bottom-grid">
            <section className="tts-subpanel" aria-label="Regression fixtures">
              <div className="subpanel-heading">
                <h3>Regression fixtures</h3>
                <span>{mvpValidation.regressionFixtures.length}</span>
              </div>
              <div className="fixture-list">
                {mvpValidation.regressionFixtures.map((fixture) => (
                  <article key={fixture.id}>
                    <strong>{fixture.name}</strong>
                    <small>{workflowLabel(fixture.workflow)}</small>
                    <p>{fixture.inputContract}</p>
                  </article>
                ))}
              </div>
            </section>

            <section className="tts-subpanel" aria-label="Manual QA scorecards">
              <div className="subpanel-heading">
                <h3>Manual QA</h3>
                <span>{mvpValidation.manualScorecards.length}</span>
              </div>
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
            </section>

            <section
              className="tts-subpanel"
              aria-label="Stress cases and limitations"
            >
              <div className="subpanel-heading">
                <h3>Stress and limits</h3>
                <span>{mvpValidation.stressCases.length}</span>
              </div>
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
            </section>
          </div>
        </section>

        <section className="tts-studio-panel" aria-label="TTS Studio">
          <div className="tts-header">
            <div>
              <p className="eyebrow">TTS Studio</p>
              <h2>{ttsStudio.script.title}</h2>
            </div>
            <button
              className="primary-action"
              disabled={!ttsStudio.submission.canSubmit}
              type="button"
              title="Queue TTS generation"
            >
              <Play aria-hidden="true" size={18} />
              <span>
                {ttsStudio.submission.canSubmit ? "Queue" : "Blocked"}
              </span>
            </button>
          </div>

          <div className="tts-metrics" aria-label="TTS workflow status">
            <div>
              <Mic2 aria-hidden="true" size={18} />
              <strong>{overview.ttsStudio.segmentCount}</strong>
              <span>segments</span>
            </div>
            <div>
              <ShieldCheck aria-hidden="true" size={18} />
              <strong>{overview.ttsStudio.speakerCount}</strong>
              <span>voices</span>
            </div>
            <div>
              <Gauge aria-hidden="true" size={18} />
              <strong>
                {formatDuration(
                  ttsStudio.generationPlan.estimatedTotalDurationMs,
                )}
              </strong>
              <span>estimate</span>
            </div>
            <div>
              <Save aria-hidden="true" size={18} />
              <strong>{ttsStudio.savedOutput.version.file.format}</strong>
              <span>{ttsStudio.savedOutput.asset.kind}</span>
            </div>
          </div>

          <div className="tts-layout">
            <div className="tts-script" aria-label="Script segments">
              {ttsStudio.script.segments.map((segment) => (
                <article className="segment-row" key={segment.id}>
                  <span className="segment-index">{segment.position}</span>
                  <div>
                    <div className="segment-meta">
                      <strong>{segment.speakerLabel}</strong>
                      <span>{segment.sceneLabel}</span>
                      <small>
                        {formatDuration(segment.targetDurationMs ?? 0)}
                      </small>
                    </div>
                    <p>{segment.text}</p>
                  </div>
                </article>
              ))}
            </div>

            <div className="tts-side">
              <section className="tts-subpanel" aria-label="Voice consent">
                <div className="subpanel-heading">
                  <h3>Voices</h3>
                  <span>{ttsStudio.voiceProfiles.length}</span>
                </div>
                <ol className="voice-list">
                  {ttsStudio.speakers.map((speaker) => (
                    <li key={speaker.voiceProfileId}>
                      <ShieldCheck aria-hidden="true" size={16} />
                      <div>
                        <strong>{speaker.label}</strong>
                        <small>
                          {speaker.language} /{" "}
                          {statusLabel(speaker.consentStatus)}
                        </small>
                      </div>
                    </li>
                  ))}
                </ol>
              </section>

              <section className="tts-subpanel" aria-label="Provider limits">
                <div className="subpanel-heading">
                  <h3>Provider</h3>
                  <span>{ttsStudio.providerOptions.length}</span>
                </div>
                {ttsStudio.providerOptions.map((provider) => (
                  <article className="provider-option" key={provider.modelId}>
                    <strong>{provider.modelId}</strong>
                    <small>
                      {statusLabel(provider.installStatus)} /{" "}
                      {statusLabel(provider.runtime)} / {provider.sampleRateHz}{" "}
                      Hz
                    </small>
                    <ul>
                      {provider.limitations.map((limitation) => (
                        <li key={limitation}>{limitation}</li>
                      ))}
                    </ul>
                  </article>
                ))}
              </section>

              <section className="tts-subpanel" aria-label="Saved output">
                <div className="subpanel-heading">
                  <h3>Output</h3>
                  <span>{ttsStudio.submission.job.status}</span>
                </div>
                <div className="output-card">
                  <strong>{ttsStudio.savedOutput.asset.name}</strong>
                  <small>{ttsStudio.savedOutput.asset.currentVersionId}</small>
                  <p>{ttsStudio.savedOutput.version.file.storagePath}</p>
                </div>
              </section>
            </div>
          </div>

          <ol className="tts-checks" aria-label="TTS checks">
            {ttsStudio.validationChecks.map((check) => (
              <li key={check.id}>
                <CircleCheck aria-hidden="true" size={16} />
                <span>{check.summary}</span>
              </li>
            ))}
          </ol>
        </section>

        <section className="voice-lab-panel" aria-label="Voice Lab">
          <div className="voice-lab-header">
            <div>
              <p className="eyebrow">Voice Lab</p>
              <h2>Consented voice workflows</h2>
            </div>
            <button
              className="primary-action voice-action"
              disabled={!voiceLab.selectedConversion.canSubmit}
              type="button"
              title="Queue voice conversion"
            >
              <Radio aria-hidden="true" size={18} />
              <span>
                {voiceLab.selectedConversion.canSubmit ? "Convert" : "Blocked"}
              </span>
            </button>
          </div>

          <div className="voice-lab-metrics" aria-label="Voice Lab status">
            <div>
              <Mic2 aria-hidden="true" size={18} />
              <strong>{overview.voiceLab.modeCount}</strong>
              <span>modes</span>
            </div>
            <div>
              <ShieldCheck aria-hidden="true" size={18} />
              <strong>{overview.voiceLab.profileCount}</strong>
              <span>profiles</span>
            </div>
            <div>
              <ClipboardCheck aria-hidden="true" size={18} />
              <strong>{overview.voiceLab.providerCount}</strong>
              <span>scorecards</span>
            </div>
            <div>
              <Save aria-hidden="true" size={18} />
              <strong>{overview.voiceLab.savedAssetKind}</strong>
              <span>{overview.voiceLab.selectedConversionCandidateId}</span>
            </div>
          </div>

          <div className="voice-mode-grid" aria-label="Voice modes">
            {voiceLab.modes.map((mode) => (
              <article className="voice-mode-card" key={mode.mode}>
                <div className="voice-mode-title">
                  <strong>{mode.label}</strong>
                  <span>{workflowLabel(mode.workflow)}</span>
                </div>
                <small>
                  {mode.inputAssetKinds.map(statusLabel).join(" / ")} to{" "}
                  {statusLabel(mode.outputAssetKind)}
                </small>
                <div className="candidate-strip">
                  {mode.providerCandidateIds.slice(0, 4).map((candidateId) => (
                    <span key={candidateId}>{candidateId}</span>
                  ))}
                </div>
              </article>
            ))}
          </div>

          <div className="voice-lab-layout">
            <div className="voice-profile-column" aria-label="Voice profiles">
              {voiceLab.voiceProfiles.map((profile) => (
                <article
                  className="voice-profile-card"
                  key={profile.profile.id}
                >
                  <div className="voice-profile-topline">
                    <div>
                      <strong>{profile.profile.displayName}</strong>
                      <small>
                        {profile.language} /{" "}
                        {statusLabel(profile.profile.consent)}
                      </small>
                    </div>
                    <span
                      className={
                        profile.commercialUseAllowed
                          ? "voice-approval approved"
                          : "voice-approval review"
                      }
                    >
                      {profile.commercialUseAllowed ? "licensed" : "review"}
                    </span>
                  </div>
                  <p>{profile.safetySummary}</p>
                  <ol className="mode-readiness">
                    {profile.modeReadiness.map((readiness) => (
                      <li key={readiness.mode}>
                        <span
                          className={
                            readiness.ready
                              ? "readiness-dot ready"
                              : "readiness-dot blocked"
                          }
                        />
                        <span>{statusLabel(readiness.mode)}</span>
                        <small>{readiness.reason ?? "ready"}</small>
                      </li>
                    ))}
                  </ol>
                </article>
              ))}
            </div>

            <div className="voice-side">
              <section className="tts-subpanel" aria-label="Conversion source">
                <div className="subpanel-heading">
                  <h3>Conversion</h3>
                  <span>{voiceLab.selectedConversion.job.status}</span>
                </div>
                <div className="conversion-source">
                  <strong>{voiceLab.conversionSource.name}</strong>
                  <small>
                    {statusLabel(voiceLab.conversionSource.kind)} /{" "}
                    {formatDuration(voiceLab.conversionSource.durationMs)}
                  </small>
                  <p>{voiceLab.conversionSource.assetId}</p>
                </div>
                <div className="output-card">
                  <strong>{voiceLab.savedOutput.asset.name}</strong>
                  <small>{voiceLab.savedOutput.asset.currentVersionId}</small>
                  <p>{voiceLab.savedOutput.version.file.storagePath}</p>
                </div>
              </section>

              <section className="tts-subpanel" aria-label="Voice providers">
                <div className="subpanel-heading">
                  <h3>Providers</h3>
                  <span>{voiceLab.providerScorecards.length}</span>
                </div>
                <div className="voice-provider-list">
                  {voiceCandidateFocus.map((scorecard) => (
                    <article
                      className={`voice-provider ${scorecard.readiness}`}
                      key={scorecard.candidateId}
                    >
                      <div>
                        <strong>{scorecard.name}</strong>
                        <small>
                          {statusLabel(scorecard.readiness)} /{" "}
                          {scorecard.lanes.map(workflowLabel).join(" / ")}
                        </small>
                        <p>{scorecard.notes}</p>
                        {scorecard.blockers[0] ? (
                          <p>{scorecard.blockers[0]}</p>
                        ) : null}
                      </div>
                      {scorecard.recommended ? <span>pick</span> : null}
                    </article>
                  ))}
                </div>
              </section>
            </div>
          </div>

          <div className="voice-review-grid">
            <ol className="voice-checks" aria-label="Voice safety gates">
              {voiceLab.safetyGates.map((gate) => (
                <li className={gate.status} key={gate.id}>
                  <ShieldCheck aria-hidden="true" size={16} />
                  <span>{gate.summary}</span>
                </li>
              ))}
            </ol>
            <ol className="voice-checks" aria-label="Voice QA checks">
              {voiceLab.qaChecks.map((check) => (
                <li className={check.status} key={check.id}>
                  <ClipboardCheck aria-hidden="true" size={16} />
                  <span>
                    <strong>{check.label}</strong> {check.target}
                  </span>
                </li>
              ))}
            </ol>
          </div>
        </section>

        <section className="sfx-studio-panel" aria-label="SFX and Ambience">
          <div className="sfx-header">
            <div>
              <p className="eyebrow">SFX + Ambience</p>
              <h2>{statusLabel(sfxStudio.prompt.category)}</h2>
            </div>
            <button
              className="primary-action sfx-action"
              disabled={!sfxStudio.submission.canSubmit}
              type="button"
              title="Queue SFX generation"
            >
              <Play aria-hidden="true" size={18} />
              <span>
                {sfxStudio.submission.canSubmit ? "Generate" : "Blocked"}
              </span>
            </button>
          </div>

          <div className="sfx-metrics" aria-label="SFX workflow status">
            <div>
              <Waves aria-hidden="true" size={18} />
              <strong>{overview.sfxStudio.variantCount}</strong>
              <span>variants</span>
            </div>
            <div>
              <Save aria-hidden="true" size={18} />
              <strong>{overview.sfxStudio.savedOutputCount}</strong>
              <span>saved</span>
            </div>
            <div>
              <ClipboardCheck aria-hidden="true" size={18} />
              <strong>{overview.sfxStudio.scorecardCount}</strong>
              <span>scorecards</span>
            </div>
            <div>
              <Gauge aria-hidden="true" size={18} />
              <strong>{formatDuration(sfxStudio.controls.durationMs)}</strong>
              <span>
                {sfxStudio.controls.loopable ? "loopable" : "one-shot"}
              </span>
            </div>
          </div>

          <div className="sfx-layout">
            <div className="sfx-main">
              <section className="sfx-prompt-panel" aria-label="SFX prompt">
                <div className="subpanel-heading">
                  <h3>Prompt</h3>
                  <span>{sfxStudio.prompt.tags.length}</span>
                </div>
                <p>{sfxStudio.prompt.text}</p>
                <small>{sfxStudio.prompt.negativePrompt}</small>
                <div className="candidate-strip">
                  {sfxStudio.prompt.tags.map((tag) => (
                    <span key={tag}>{tag}</span>
                  ))}
                </div>
              </section>

              <div className="sfx-control-grid" aria-label="SFX controls">
                <div>
                  <strong>{sfxStudio.controls.variationCount}</strong>
                  <span>batch</span>
                </div>
                <div>
                  <strong>{sfxStudio.controls.intensity}</strong>
                  <span>intensity</span>
                </div>
                <div>
                  <strong>{sfxStudio.controls.realism}</strong>
                  <span>realism</span>
                </div>
                <div>
                  <strong>{sfxStudio.controls.loopCrossfadeMs}ms</strong>
                  <span>crossfade</span>
                </div>
              </div>

              <div className="sfx-variant-grid" aria-label="Generated variants">
                {sfxStudio.variants.map((variant) => (
                  <article
                    className={
                      variant.selectedForSave
                        ? "sfx-variant selected"
                        : "sfx-variant"
                    }
                    key={variant.id}
                  >
                    <div className="sfx-variant-title">
                      <strong>{variant.label}</strong>
                      <span>{statusLabel(variant.assetKind)}</span>
                    </div>
                    <small>
                      {formatDuration(variant.durationMs)} /{" "}
                      {variant.loudnessLufs} LUFS / {variant.truePeakDbfs} dBTP
                    </small>
                    <p>
                      {variant.loopPoints
                        ? `loop ${variant.loopPoints.startSample}-${variant.loopPoints.endSample}`
                        : "one-shot preview"}
                    </p>
                    <div className="candidate-strip">
                      {variant.tags.slice(0, 4).map((tag) => (
                        <span key={tag}>{tag}</span>
                      ))}
                    </div>
                  </article>
                ))}
              </div>
            </div>

            <div className="sfx-side">
              <section
                className="tts-subpanel"
                aria-label="SFX provider options"
              >
                <div className="subpanel-heading">
                  <h3>Providers</h3>
                  <span>{sfxStudio.providerOptions.length}</span>
                </div>
                {sfxStudio.providerOptions.map((provider) => (
                  <article
                    className="sfx-provider-option"
                    key={`${provider.workflow}-${provider.modelId}`}
                  >
                    <strong>{workflowLabel(provider.workflow)}</strong>
                    <small>
                      {statusLabel(provider.installStatus)} /{" "}
                      {statusLabel(provider.outputAssetKind)} /{" "}
                      {provider.sampleRateHz} Hz
                    </small>
                    <p>
                      {provider.supportedControls.map(statusLabel).join(" / ")}
                    </p>
                  </article>
                ))}
              </section>

              <section
                className="tts-subpanel"
                aria-label="SFX provider scorecards"
              >
                <div className="subpanel-heading">
                  <h3>Scorecards</h3>
                  <span>{sfxStudio.providerScorecards.length}</span>
                </div>
                <div className="voice-provider-list">
                  {sfxCandidateFocus.map((scorecard) => (
                    <article
                      className={`voice-provider ${scorecard.readiness}`}
                      key={scorecard.candidateId}
                    >
                      <div>
                        <strong>{scorecard.name}</strong>
                        <small>
                          {statusLabel(scorecard.readiness)} /{" "}
                          {scorecard.lanes.map(workflowLabel).join(" / ")}
                        </small>
                        <p>{scorecard.notes}</p>
                        {scorecard.blockers[0] ? (
                          <p>{scorecard.blockers[0]}</p>
                        ) : null}
                      </div>
                      {scorecard.recommended ? <span>pick</span> : null}
                    </article>
                  ))}
                </div>
              </section>

              <section className="tts-subpanel" aria-label="Saved SFX outputs">
                <div className="subpanel-heading">
                  <h3>Outputs</h3>
                  <span>{sfxStudio.submission.job.status}</span>
                </div>
                {sfxStudio.savedOutputs.map((output) => (
                  <div className="output-card" key={output.variantId}>
                    <strong>{output.asset.name}</strong>
                    <small>
                      {output.asset.kind} / {output.asset.currentVersionId}
                    </small>
                    <p>{output.version.file.storagePath}</p>
                  </div>
                ))}
              </section>
            </div>
          </div>

          <div className="sfx-review-grid">
            <ol className="voice-checks" aria-label="SFX post-processing">
              {sfxStudio.postProcessingActions.map((action) => (
                <li
                  className={action.enabled ? "ready" : "warning"}
                  key={action.id}
                >
                  <SlidersHorizontal aria-hidden="true" size={16} />
                  <span>{action.summary}</span>
                </li>
              ))}
            </ol>
            <ol className="voice-checks" aria-label="SFX validation checks">
              {sfxStudio.validationChecks.map((check) => (
                <li className={check.status} key={check.id}>
                  <CircleCheck aria-hidden="true" size={16} />
                  <span>{check.summary}</span>
                </li>
              ))}
            </ol>
          </div>
        </section>

        <section className="video-to-audio-panel" aria-label="Video to Audio">
          <div className="video-header">
            <div>
              <p className="eyebrow">Video to Audio</p>
              <h2>{videoToAudio.source.filename}</h2>
            </div>
            <button
              className="primary-action video-action"
              disabled={!videoToAudio.submission.canSubmit}
              type="button"
              title="Queue video-to-audio generation"
            >
              <FileVideo aria-hidden="true" size={18} />
              <span>
                {videoToAudio.submission.canSubmit ? "Generate" : "Blocked"}
              </span>
            </button>
          </div>

          <div className="video-metrics" aria-label="Video-to-audio status">
            <div>
              <FileVideo aria-hidden="true" size={18} />
              <strong>{formatDuration(videoToAudio.source.durationMs)}</strong>
              <span>{videoToAudio.source.frameRate}</span>
            </div>
            <div>
              <SlidersHorizontal aria-hidden="true" size={18} />
              <strong>{overview.videoToAudio.targetRangeCount}</strong>
              <span>ranges</span>
            </div>
            <div>
              <Activity aria-hidden="true" size={18} />
              <strong>{overview.videoToAudio.syncPointCount}</strong>
              <span>sync points</span>
            </div>
            <div>
              <ClipboardCheck aria-hidden="true" size={18} />
              <strong>{overview.videoToAudio.scorecardCount}</strong>
              <span>scorecards</span>
            </div>
          </div>

          <div className="video-layout">
            <div className="video-main">
              <section
                className="video-source-panel"
                aria-label="Video source and direction"
              >
                <div className="subpanel-heading">
                  <h3>Source and direction</h3>
                  <span>{videoToAudio.source.resolution}</span>
                </div>
                <p>{videoToAudio.direction.prompt}</p>
                <small>{videoToAudio.direction.negativePrompt}</small>
                <div className="candidate-strip">
                  <span>{statusLabel(videoToAudio.direction.syncMode)}</span>
                  <span>
                    {videoToAudio.source.hasSourceAudio
                      ? "source audio"
                      : "silent video"}
                  </span>
                  <span>
                    {videoToAudio.source.imageReferenceIds.length} keyframe
                  </span>
                  <span>
                    {videoToAudio.source.referenceAudioAssetIds.length}{" "}
                    reference audio
                  </span>
                </div>
              </section>

              <div className="video-range-list" aria-label="Target ranges">
                {videoToAudio.targetRanges.map((range) => (
                  <article className="video-range" key={range.id}>
                    <div>
                      <strong>{range.label}</strong>
                      <small>
                        {formatDuration(range.range.startMs)}-
                        {formatDuration(range.range.endMs)}
                      </small>
                    </div>
                    <p>{range.requestedAction}</p>
                    <span>{range.objectLabel ?? "full frame"}</span>
                  </article>
                ))}
              </div>

              <div className="sync-timeline" aria-label="Sync preview">
                {videoToAudio.syncPreview.segments.map((segment) => (
                  <article
                    className="sync-segment"
                    key={segment.id}
                    style={{
                      marginLeft: `${(segment.range.startMs / videoToAudio.syncPreview.durationMs) * 100}%`,
                      width: `${Math.max(
                        8,
                        ((segment.range.endMs - segment.range.startMs) /
                          videoToAudio.syncPreview.durationMs) *
                          100,
                      )}%`,
                    }}
                  >
                    <strong>{segment.label}</strong>
                    <span>{Math.round(segment.syncConfidence * 100)}%</span>
                  </article>
                ))}
              </div>

              <div className="video-event-grid" aria-label="Detected events">
                {videoToAudio.detectedEvents.map((event) => (
                  <div className="video-event" key={event.id}>
                    <strong>{event.label}</strong>
                    <span>{formatDuration(event.atMs)}</span>
                    <small>{event.requestedSound}</small>
                  </div>
                ))}
              </div>
            </div>

            <div className="video-side">
              <section
                className="tts-subpanel"
                aria-label="Video-to-audio provider options"
              >
                <div className="subpanel-heading">
                  <h3>Provider</h3>
                  <span>{videoToAudio.providerOptions.length}</span>
                </div>
                {videoToAudio.providerOptions.map((provider) => (
                  <article
                    className="sfx-provider-option"
                    key={`${provider.workflow}-${provider.modelId}`}
                  >
                    <strong>{provider.displayName}</strong>
                    <small>
                      {statusLabel(provider.installStatus)} /{" "}
                      {provider.sampleRateHz} Hz /{" "}
                      {statusLabel(provider.channelLayout)}
                    </small>
                    <p>
                      {[
                        provider.supportsVideo ? "video" : null,
                        provider.supportsText ? "text" : null,
                        provider.supportsRangeRefinement ? "ranges" : null,
                        provider.supportsObjectRegions ? "regions" : null,
                      ]
                        .filter(Boolean)
                        .join(" / ")}
                    </p>
                  </article>
                ))}
              </section>

              <section
                className="tts-subpanel"
                aria-label="Video-to-audio scorecards"
              >
                <div className="subpanel-heading">
                  <h3>Scorecards</h3>
                  <span>{videoToAudio.providerScorecards.length}</span>
                </div>
                <div className="voice-provider-list">
                  {videoCandidateFocus.map((scorecard) => (
                    <article
                      className={`voice-provider ${scorecard.readiness}`}
                      key={scorecard.candidateId}
                    >
                      <div>
                        <strong>{scorecard.name}</strong>
                        <small>
                          {statusLabel(scorecard.readiness)} /{" "}
                          {scorecard.supports.map(statusLabel).join(" / ")}
                        </small>
                        <p>{scorecard.notes}</p>
                        {scorecard.blockers[0] ? (
                          <p>{scorecard.blockers[0]}</p>
                        ) : null}
                      </div>
                      {scorecard.recommended ? <span>pick</span> : null}
                    </article>
                  ))}
                </div>
              </section>

              <section className="tts-subpanel" aria-label="Video output">
                <div className="subpanel-heading">
                  <h3>Output</h3>
                  <span>{videoToAudio.submission.job.status}</span>
                </div>
                <div className="output-card">
                  <strong>{videoToAudio.savedOutput.asset.name}</strong>
                  <small>
                    {videoToAudio.savedOutput.asset.kind} /{" "}
                    {videoToAudio.savedOutput.asset.currentVersionId}
                  </small>
                  <p>{videoToAudio.savedOutput.version.file.storagePath}</p>
                </div>
              </section>
            </div>
          </div>

          <div className="video-review-grid">
            <ol className="voice-checks" aria-label="Video-to-audio gates">
              {videoToAudio.safetyGates.map((gate) => (
                <li className={gate.status} key={gate.id}>
                  <ShieldCheck aria-hidden="true" size={16} />
                  <span>{gate.summary}</span>
                </li>
              ))}
            </ol>
            <section className="video-sidecar" aria-label="Video sidecar">
              <div className="subpanel-heading">
                <h3>Sidecar</h3>
                <span>{videoToAudio.exportPackage.requiredFields.length}</span>
              </div>
              <p>{videoToAudio.exportPackage.sidecarPath}</p>
              <div className="candidate-strip">
                {videoToAudio.exportPackage.destinationTargets.map((target) => (
                  <span key={target}>{target}</span>
                ))}
              </div>
            </section>
            <ol className="voice-checks" aria-label="Video-to-audio validation">
              {videoToAudio.validationChecks.map((check) => (
                <li className={check.status} key={check.id}>
                  <CircleCheck aria-hidden="true" size={16} />
                  <span>{check.summary}</span>
                </li>
              ))}
            </ol>
          </div>
        </section>

        <section
          className="samples-studio-panel"
          aria-label="Samples and Loops"
        >
          <div className="samples-header">
            <div>
              <p className="eyebrow">Samples + Loops</p>
              <h2>{samplesStudio.pack.name}</h2>
            </div>
            <button
              className="primary-action samples-action"
              disabled={!samplesStudio.submission.canSubmit}
              type="button"
              title="Queue sample and loop generation"
            >
              <Disc3 aria-hidden="true" size={18} />
              <span>
                {samplesStudio.submission.canSubmit ? "Generate" : "Blocked"}
              </span>
            </button>
          </div>

          <div className="samples-metrics" aria-label="Samples workflow status">
            <div>
              <Boxes aria-hidden="true" size={18} />
              <strong>{overview.samplesStudio.variantCount}</strong>
              <span>variants</span>
            </div>
            <div>
              <Save aria-hidden="true" size={18} />
              <strong>{overview.samplesStudio.savedOutputCount}</strong>
              <span>saved</span>
            </div>
            <div>
              <Gauge aria-hidden="true" size={18} />
              <strong>{samplesStudio.controls.bpm}</strong>
              <span>{samplesStudio.controls.musicalKey}</span>
            </div>
            <div>
              <ClipboardCheck aria-hidden="true" size={18} />
              <strong>{overview.samplesStudio.scorecardCount}</strong>
              <span>scorecards</span>
            </div>
          </div>

          <div className="samples-layout">
            <div className="samples-main">
              <section className="sfx-prompt-panel" aria-label="Sample prompt">
                <div className="subpanel-heading">
                  <h3>{statusLabel(samplesStudio.prompt.instrumentFamily)}</h3>
                  <span>{samplesStudio.prompt.genreTags.length}</span>
                </div>
                <p>{samplesStudio.prompt.text}</p>
                <small>{samplesStudio.prompt.negativePrompt}</small>
                <div className="candidate-strip">
                  {samplesStudio.prompt.genreTags.map((tag) => (
                    <span key={tag}>{tag}</span>
                  ))}
                </div>
              </section>

              <div
                className="samples-control-grid"
                aria-label="Sample controls"
              >
                <div>
                  <strong>{samplesStudio.controls.bars} bars</strong>
                  <span>{samplesStudio.controls.beats}/4 grid</span>
                </div>
                <div>
                  <strong>{samplesStudio.controls.batchSize}</strong>
                  <span>batch size</span>
                </div>
                <div>
                  <strong>{samplesStudio.controls.velocityEnergy}</strong>
                  <span>velocity</span>
                </div>
                <div>
                  <strong>{samplesStudio.controls.dryWetAmbience}</strong>
                  <span>ambience</span>
                </div>
              </div>

              <div
                className="samples-variant-grid"
                aria-label="Sample variants"
              >
                {samplesStudio.variants.map((variant) => (
                  <article
                    className={
                      variant.selectedForPack
                        ? "samples-variant selected"
                        : "samples-variant"
                    }
                    key={variant.id}
                  >
                    <div className="sfx-variant-title">
                      <strong>{variant.label}</strong>
                      <span>{statusLabel(variant.assetKind)}</span>
                    </div>
                    <small>
                      {formatDuration(variant.durationMs)} /{" "}
                      {variant.bpm ? `${variant.bpm} BPM` : "one-shot"} /{" "}
                      {variant.musicalKey ?? "unpitched"}
                    </small>
                    <p>
                      {variant.loopPoints
                        ? `loop ${variant.loopPoints.startSample}-${variant.loopPoints.endSample}`
                        : variant.articulation}
                    </p>
                    <div className="candidate-strip">
                      {variant.tags.slice(0, 4).map((tag) => (
                        <span key={tag}>{tag}</span>
                      ))}
                    </div>
                  </article>
                ))}
              </div>
            </div>

            <div className="samples-side">
              <section className="tts-subpanel" aria-label="Sample providers">
                <div className="subpanel-heading">
                  <h3>Providers</h3>
                  <span>{samplesStudio.providerOptions.length}</span>
                </div>
                {samplesStudio.providerOptions.map((provider) => (
                  <article
                    className="sfx-provider-option"
                    key={`${provider.workflow}-${provider.modelId}`}
                  >
                    <strong>{workflowLabel(provider.workflow)}</strong>
                    <small>
                      {statusLabel(provider.installStatus)} /{" "}
                      {provider.sampleRateHz} Hz /{" "}
                      {provider.supportsLoopPoints ? "loop points" : "metadata"}
                    </small>
                    <p>
                      {provider.supportedControls.map(statusLabel).join(" / ")}
                    </p>
                  </article>
                ))}
              </section>

              <section
                className="tts-subpanel"
                aria-label="Sample provider scorecards"
              >
                <div className="subpanel-heading">
                  <h3>Scorecards</h3>
                  <span>{samplesStudio.providerScorecards.length}</span>
                </div>
                <div className="voice-provider-list">
                  {samplesCandidateFocus.map((scorecard) => (
                    <article
                      className={`voice-provider ${scorecard.readiness}`}
                      key={scorecard.candidateId}
                    >
                      <div>
                        <strong>{scorecard.name}</strong>
                        <small>
                          {statusLabel(scorecard.readiness)} /{" "}
                          {scorecard.lanes.map(workflowLabel).join(" / ")}
                        </small>
                        <p>{scorecard.notes}</p>
                      </div>
                      {scorecard.recommended ? <span>pick</span> : null}
                    </article>
                  ))}
                </div>
              </section>

              <section
                className="tts-subpanel"
                aria-label="Sample pack outputs"
              >
                <div className="subpanel-heading">
                  <h3>Pack</h3>
                  <span>{samplesStudio.pack.exportFormats.join(" / ")}</span>
                </div>
                {samplesStudio.savedOutputs.map((output) => (
                  <div className="output-card" key={output.variantId}>
                    <strong>{output.asset.name}</strong>
                    <small>
                      {output.asset.kind} /{" "}
                      {output.version.technical.bpm
                        ? `${output.version.technical.bpm} BPM`
                        : output.version.technical.musicalKey}
                    </small>
                    <p>{output.version.file.storagePath}</p>
                  </div>
                ))}
              </section>
            </div>
          </div>

          <div className="samples-review-grid">
            <ol className="voice-checks" aria-label="Sample post-processing">
              {samplesStudio.postProcessingActions.map((action) => (
                <li
                  className={action.enabled ? "ready" : "warning"}
                  key={action.id}
                >
                  <SlidersHorizontal aria-hidden="true" size={16} />
                  <span>{action.summary}</span>
                </li>
              ))}
            </ol>
            <ol className="voice-checks" aria-label="Sample QA checks">
              {samplesStudio.qaChecks.map((check) => (
                <li className={check.status} key={check.id}>
                  <CircleCheck aria-hidden="true" size={16} />
                  <span>{check.summary}</span>
                </li>
              ))}
            </ol>
          </div>
        </section>

        <section
          className="samples-studio-panel song-studio-panel"
          aria-label="Song Studio"
        >
          <div className="samples-header">
            <div>
              <p className="eyebrow">Song Studio</p>
              <h2>{songStudio.draft.title}</h2>
            </div>
            <button
              className="primary-action samples-action"
              disabled={!songStudio.submission.canSubmit}
              type="button"
              title="Queue song generation"
            >
              <Music2 aria-hidden="true" size={18} />
              <span>
                {songStudio.submission.canSubmit ? "Generate" : "Blocked"}
              </span>
            </button>
          </div>

          <div className="samples-metrics" aria-label="Song workflow status">
            <div>
              <Music2 aria-hidden="true" size={18} />
              <strong>{overview.songStudio.sectionCount}</strong>
              <span>sections</span>
            </div>
            <div>
              <Save aria-hidden="true" size={18} />
              <strong>{overview.songStudio.savedOutputCount}</strong>
              <span>saved</span>
            </div>
            <div>
              <Gauge aria-hidden="true" size={18} />
              <strong>{songStudio.controls.bpm}</strong>
              <span>{songStudio.controls.musicalKey}</span>
            </div>
            <div>
              <ClipboardCheck aria-hidden="true" size={18} />
              <strong>{overview.songStudio.scorecardCount}</strong>
              <span>scorecards</span>
            </div>
          </div>

          <div className="samples-layout">
            <div className="samples-main">
              <section className="sfx-prompt-panel" aria-label="Song prompt">
                <div className="subpanel-heading">
                  <h3>{songStudio.draft.language}</h3>
                  <span>{songStudio.draft.styleTags.length}</span>
                </div>
                <p>{songStudio.draft.prompt}</p>
                <small>{songStudio.draft.singerHint}</small>
                <div className="candidate-strip">
                  {songStudio.draft.styleTags.map((tag) => (
                    <span key={tag}>{tag}</span>
                  ))}
                </div>
              </section>

              <div className="samples-control-grid" aria-label="Song controls">
                <div>
                  <strong>{songStudio.arrangement.totalBars} bars</strong>
                  <span>{songStudio.controls.timeSignature}</span>
                </div>
                <div>
                  <strong>
                    {formatDuration(songStudio.arrangement.estimatedDurationMs)}
                  </strong>
                  <span>arranged</span>
                </div>
                <div>
                  <strong>{songStudio.controls.variationCount}</strong>
                  <span>variants</span>
                </div>
                <div>
                  <strong>{songStudio.controls.requestedStems.length}</strong>
                  <span>stems</span>
                </div>
              </div>

              <div className="samples-variant-grid" aria-label="Song sections">
                {songStudio.arrangement.sections.map((section) => (
                  <article
                    className="samples-variant selected"
                    key={section.id}
                  >
                    <div className="sfx-variant-title">
                      <strong>{section.label}</strong>
                      <span>{section.bars} bars</span>
                    </div>
                    <small>
                      starts bar {section.startBar + 1} /{" "}
                      {section.hasLyrics ? "lyrics" : "instrumental"}
                    </small>
                    <p>{section.locked ? "locked" : "regeneratable"}</p>
                  </article>
                ))}
              </div>

              <div className="samples-variant-grid" aria-label="Song variants">
                {songStudio.variants.map((variant) => (
                  <article
                    className={
                      variant.selectedForSave
                        ? "samples-variant selected"
                        : "samples-variant"
                    }
                    key={variant.id}
                  >
                    <div className="sfx-variant-title">
                      <strong>{variant.label}</strong>
                      <span>{statusLabel(variant.assetKind)}</span>
                    </div>
                    <small>
                      {formatDuration(variant.durationMs)} / {variant.bpm} BPM /{" "}
                      {variant.musicalKey}
                    </small>
                    <p>
                      lyric {variant.lyricAlignmentScore} / structure{" "}
                      {variant.structureMatchScore}
                    </p>
                    <div className="candidate-strip">
                      {variant.stemKinds.slice(0, 5).map((stem) => (
                        <span key={stem}>{statusLabel(stem)}</span>
                      ))}
                    </div>
                  </article>
                ))}
              </div>
            </div>

            <div className="samples-side">
              <section className="tts-subpanel" aria-label="Song providers">
                <div className="subpanel-heading">
                  <h3>Providers</h3>
                  <span>{songStudio.providerOptions.length}</span>
                </div>
                {songStudio.providerOptions.map((provider) => (
                  <article
                    className="sfx-provider-option"
                    key={`${provider.workflow}-${provider.modelId}`}
                  >
                    <strong>{provider.modelId}</strong>
                    <small>
                      {statusLabel(provider.installStatus)} /{" "}
                      {provider.supportsStems ? "stems" : "mixdown"} /{" "}
                      {provider.sampleRateHz} Hz
                    </small>
                    <p>
                      {provider.supportedControls.map(statusLabel).join(" / ")}
                    </p>
                  </article>
                ))}
              </section>

              <section
                className="tts-subpanel"
                aria-label="Song provider scorecards"
              >
                <div className="subpanel-heading">
                  <h3>Scorecards</h3>
                  <span>{songStudio.providerScorecards.length}</span>
                </div>
                <div className="voice-provider-list">
                  {songCandidateFocus.map((scorecard) => (
                    <article
                      className={`voice-provider ${scorecard.readiness}`}
                      key={scorecard.candidateId}
                    >
                      <div>
                        <strong>{scorecard.name}</strong>
                        <small>
                          {statusLabel(scorecard.readiness)} /{" "}
                          {scorecard.lanes.map(workflowLabel).join(" / ")}
                        </small>
                        <p>{scorecard.notes}</p>
                      </div>
                      {scorecard.recommended ? <span>pick</span> : null}
                    </article>
                  ))}
                </div>
              </section>

              <section className="tts-subpanel" aria-label="Song outputs">
                <div className="subpanel-heading">
                  <h3>Outputs</h3>
                  <span>{songStudio.submission.job.status}</span>
                </div>
                {songStudio.savedOutputs.map((output) => (
                  <div className="output-card" key={output.variantId}>
                    <strong>{output.asset.name}</strong>
                    <small>
                      {output.asset.kind} /{" "}
                      {output.version.technical.bpm
                        ? `${output.version.technical.bpm} BPM`
                        : output.asset.currentVersionId}
                    </small>
                    <p>{output.version.file.storagePath}</p>
                  </div>
                ))}
              </section>
            </div>
          </div>

          <div className="samples-review-grid">
            <ol className="voice-checks" aria-label="Song export targets">
              {songStudio.exportTargets.map((target) => (
                <li className="ready" key={target.id}>
                  <Save aria-hidden="true" size={16} />
                  <span>{target.summary}</span>
                </li>
              ))}
            </ol>
            <ol className="voice-checks" aria-label="Song QA checks">
              {songStudio.qaChecks.map((check) => (
                <li className={check.status} key={check.id}>
                  <CircleCheck aria-hidden="true" size={16} />
                  <span>{check.summary}</span>
                </li>
              ))}
            </ol>
          </div>
        </section>

        <section
          className="review-workspace-panel"
          aria-label="Waveform Review"
        >
          <div className="samples-header">
            <div>
              <p className="eyebrow">Waveform Review</p>
              <h2>{reviewWorkspace.selectedAsset.asset.name}</h2>
            </div>
            <button
              className="primary-action review-action"
              disabled={!reviewWorkspace.editSubmission.canSave}
              type="button"
              title="Save edited audio version"
            >
              <Save aria-hidden="true" size={18} />
              <span>
                {reviewWorkspace.editSubmission.canSave
                  ? "Save version"
                  : "Blocked"}
              </span>
            </button>
          </div>

          <div className="samples-metrics" aria-label="Waveform review status">
            <div>
              <Library aria-hidden="true" size={18} />
              <strong>{overview.reviewWorkspace.assetCount}</strong>
              <span>assets</span>
            </div>
            <div>
              <Waves aria-hidden="true" size={18} />
              <strong>{overview.reviewWorkspace.previewableAssetCount}</strong>
              <span>previewable</span>
            </div>
            <div>
              <SlidersHorizontal aria-hidden="true" size={18} />
              <strong>{overview.reviewWorkspace.editActionCount}</strong>
              <span>edit actions</span>
            </div>
            <div>
              <ClipboardCheck aria-hidden="true" size={18} />
              <strong>{overview.reviewWorkspace.comparisonCount}</strong>
              <span>comparison</span>
            </div>
          </div>

          <div className="review-layout">
            <div className="review-main">
              <section
                className="review-transport"
                aria-label="Waveform transport"
              >
                <div className="review-transport-topline">
                  <button
                    className="icon-control"
                    type="button"
                    title="Play or pause preview"
                  >
                    <Play aria-hidden="true" size={18} />
                  </button>
                  <strong>
                    {formatDuration(reviewWorkspace.transport.positionMs)} /{" "}
                    {formatDuration(reviewWorkspace.transport.durationMs)}
                  </strong>
                  <span>
                    {reviewWorkspace.transport.zoomPixelsPerSecond}px/s
                  </span>
                </div>
                <div
                  className="waveform-strip"
                  aria-label="Cached waveform preview"
                >
                  {reviewWorkspace.waveform.peaks.map((peak, index) => (
                    <span
                      aria-hidden="true"
                      className="waveform-bar"
                      key={`${peak.min}-${peak.max}-${index}`}
                      style={{ height: `${Math.max(20, peak.max * 86)}%` }}
                    />
                  ))}
                </div>
                <div className="transport-meta">
                  <span>
                    selection{" "}
                    {formatDuration(
                      reviewWorkspace.transport.selection?.startMs ?? 0,
                    )}
                    -
                    {formatDuration(
                      reviewWorkspace.transport.selection?.endMs ?? 0,
                    )}
                  </span>
                  <span>
                    loop{" "}
                    {formatDuration(
                      reviewWorkspace.transport.loopRegion?.startMs ?? 0,
                    )}
                    -
                    {formatDuration(
                      reviewWorkspace.transport.loopRegion?.endMs ?? 0,
                    )}
                  </span>
                  <span>{reviewWorkspace.waveform.cachePath}</span>
                </div>
              </section>

              <div className="review-asset-grid" aria-label="Reviewable assets">
                {reviewWorkspace.assets.map((asset) => (
                  <article
                    className={
                      asset.asset.id === reviewWorkspace.selectedAsset.asset.id
                        ? "review-asset selected"
                        : "review-asset"
                    }
                    key={asset.asset.id}
                  >
                    <div className="sfx-variant-title">
                      <strong>{asset.asset.name}</strong>
                      <span>{statusLabel(asset.asset.kind)}</span>
                    </div>
                    <small>
                      {statusLabel(asset.sourceWorkflow)} /{" "}
                      {asset.versions.length} version
                    </small>
                    <p>
                      {asset.canPreview
                        ? "waveform and spectrogram cached"
                        : "preview pending"}
                    </p>
                  </article>
                ))}
              </div>

              <div
                className="edit-action-grid"
                aria-label="Lightweight edit actions"
              >
                {reviewWorkspace.editActions.map((action) => (
                  <button
                    className={
                      action.enabled ? "edit-action enabled" : "edit-action"
                    }
                    key={action.id}
                    type="button"
                    title={action.label}
                  >
                    <SlidersHorizontal aria-hidden="true" size={16} />
                    <span>{action.label}</span>
                  </button>
                ))}
              </div>
            </div>

            <div className="review-side">
              <section className="tts-subpanel" aria-label="Version comparison">
                <div className="subpanel-heading">
                  <h3>Version comparison</h3>
                  <span>{reviewWorkspace.versionComparison.mode}</span>
                </div>
                <div className="comparison-grid">
                  {[
                    reviewWorkspace.versionComparison.left,
                    reviewWorkspace.versionComparison.right,
                  ].map((side) => (
                    <article key={side.versionId}>
                      <strong>{side.label}</strong>
                      <small>{side.versionId}</small>
                      <p>
                        {formatDuration(side.durationMs)} / {side.loudnessLufs}{" "}
                        LUFS / {side.truePeakDbfs} dBTP
                      </p>
                    </article>
                  ))}
                </div>
                <div className="comparison-metrics">
                  <span>
                    {reviewWorkspace.versionComparison.metrics.durationDeltaMs}
                    ms
                  </span>
                  <span>
                    {
                      reviewWorkspace.versionComparison.metrics
                        .loudnessDeltaLufs
                    }{" "}
                    LUFS
                  </span>
                  <span>
                    diff{" "}
                    {
                      reviewWorkspace.versionComparison.metrics
                        .waveformDifferenceScore
                    }
                  </span>
                </div>
              </section>

              <section className="tts-subpanel" aria-label="Edited version">
                <div className="subpanel-heading">
                  <h3>Edited version</h3>
                  <span>{reviewWorkspace.editSubmission.job.status}</span>
                </div>
                <div className="output-card">
                  <strong>
                    {reviewWorkspace.editSubmission.savedVersion.id}
                  </strong>
                  <small>
                    v{reviewWorkspace.editSubmission.savedVersion.versionIndex}{" "}
                    / {reviewWorkspace.editSubmission.savedVersion.file.format}
                  </small>
                  <p>
                    {
                      reviewWorkspace.editSubmission.savedVersion.file
                        .storagePath
                    }
                  </p>
                </div>
              </section>

              <section className="tts-subpanel" aria-label="Recipe provenance">
                <div className="subpanel-heading">
                  <h3>Provenance</h3>
                  <span>
                    {reviewWorkspace.provenance.inspectable
                      ? "inspectable"
                      : "blocked"}
                  </span>
                </div>
                <div className="output-card">
                  <strong>{reviewWorkspace.provenance.editRecipe.id}</strong>
                  <small>
                    {statusLabel(
                      reviewWorkspace.provenance.originalRecipe.workflow,
                    )}{" "}
                    to{" "}
                    {statusLabel(
                      reviewWorkspace.provenance.editRecipe.workflow,
                    )}
                  </small>
                  <p>{reviewWorkspace.provenance.sidecarPath}</p>
                </div>
              </section>
            </div>
          </div>

          <div className="samples-review-grid">
            <ol className="voice-checks" aria-label="Review validation checks">
              {reviewWorkspace.validationChecks.map((check) => (
                <li className={check.status} key={check.id}>
                  <CircleCheck aria-hidden="true" size={16} />
                  <span>{check.summary}</span>
                </li>
              ))}
            </ol>
            <ol className="voice-checks" aria-label="Review shortcuts">
              {reviewWorkspace.transport.keyboardShortcuts.map((shortcut) => (
                <li className="ready" key={shortcut.id}>
                  <ClipboardCheck aria-hidden="true" size={16} />
                  <span>
                    <strong>{shortcut.keys}</strong> {shortcut.action}
                  </span>
                </li>
              ))}
            </ol>
          </div>
        </section>

        <section className="rights-safety-panel" aria-label="Rights and Safety">
          <div className="samples-header">
            <div>
              <p className="eyebrow">Rights + Safety</p>
              <h2>{rightsSafety.policy.name}</h2>
            </div>
            <button
              className="primary-action safety-action"
              disabled={!overview.rightsSafety.canExport}
              type="button"
              title="SoundWorks export gate"
            >
              <ShieldCheck aria-hidden="true" size={18} />
              <span>
                {overview.rightsSafety.canExport ? "Export" : "Blocked"}
              </span>
            </button>
          </div>

          <div className="samples-metrics" aria-label="Rights workflow status">
            <div>
              <ShieldCheck aria-hidden="true" size={18} />
              <strong>{overview.rightsSafety.blockedConsentCount}</strong>
              <span>consent blocks</span>
            </div>
            <div>
              <CircleAlert aria-hidden="true" size={18} />
              <strong>{overview.rightsSafety.blockedModelDecisionCount}</strong>
              <span>model blocks</span>
            </div>
            <div>
              <Save aria-hidden="true" size={18} />
              <strong>{overview.rightsSafety.sidecarCount}</strong>
              <span>sidecars</span>
            </div>
            <div>
              <ClipboardCheck aria-hidden="true" size={18} />
              <strong>{overview.rightsSafety.disclosureCount}</strong>
              <span>disclosures</span>
            </div>
          </div>

          <div className="rights-layout">
            <div className="rights-main">
              <section className="tts-subpanel" aria-label="Consent checks">
                <div className="subpanel-heading">
                  <h3>Consent</h3>
                  <span>{rightsSafety.consentChecks.length}</span>
                </div>
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
              </section>

              <section
                className="tts-subpanel"
                aria-label="Model export decisions"
              >
                <div className="subpanel-heading">
                  <h3>Model export gates</h3>
                  <span>{rightsSafety.modelUseDecisions.length}</span>
                </div>
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
              </section>
            </div>

            <div className="rights-side">
              <section
                className="tts-subpanel"
                aria-label="Export provenance sidecars"
              >
                <div className="subpanel-heading">
                  <h3>Sidecars</h3>
                  <span>
                    {statusLabel(rightsSafety.policy.watermarkPolicy)}
                  </span>
                </div>
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
              </section>

              <section
                className="tts-subpanel"
                aria-label="Policy requirements"
              >
                <div className="subpanel-heading">
                  <h3>SoundWorks export</h3>
                  <span>
                    {rightsSafety.policy.provenanceSidecarRequired
                      ? "sidecar"
                      : "manual"}
                  </span>
                </div>
                <ol className="policy-list">
                  {rightsSafety.policy.exportRequires.map((requirement) => (
                    <li key={requirement}>
                      <CircleCheck aria-hidden="true" size={16} />
                      <span>{requirement}</span>
                    </li>
                  ))}
                </ol>
              </section>
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

        <section className="system-grid" aria-label="Architecture">
          <div className="panel">
            <div className="panel-heading">
              <h2>Runtime Layers</h2>
              <span>{overview.architecture.layers.length}</span>
            </div>
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
          </div>

          <div className="panel">
            <div className="panel-heading">
              <h2>Command Boundary</h2>
              <span>{overview.commands.length}</span>
            </div>
            <div className="command-list">
              {overview.commands.map((command) => (
                <article className="command-row" key={command.name}>
                  <strong>{command.name}</strong>
                  <span>{command.direction}</span>
                  <p>{command.purpose}</p>
                </article>
              ))}
            </div>
          </div>

          <div className="panel provider-panel">
            <div className="panel-heading">
              <h2>Provider Coverage</h2>
              <span>{overview.providerCatalog.capabilityCount}</span>
            </div>
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
          </div>

          <div className="panel evaluation-panel">
            <div className="panel-heading">
              <h2>Evaluation Scorecard</h2>
              <span>{overview.modelEvaluation.candidateCount}</span>
            </div>
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
          </div>

          <div className="panel model-manager-panel">
            <div className="panel-heading">
              <h2>Model Manager</h2>
              <span>{modelManager.summary.candidateCount}</span>
            </div>
            <div className="runtime-summary" aria-label="Model manager status">
              <div>
                <PackageCheck aria-hidden="true" size={18} />
                <strong>{modelManager.summary.verifiedInstalledCount}</strong>
                <span>verified</span>
              </div>
              <div>
                <Download aria-hidden="true" size={18} />
                <strong>{modelManager.summary.installableCount}</strong>
                <span>installable</span>
              </div>
              <div>
                <HardDrive aria-hidden="true" size={18} />
                <strong>{modelManager.summary.missingCacheCount}</strong>
                <span>missing cache</span>
              </div>
              <div>
                <CircleAlert aria-hidden="true" size={18} />
                <strong>{modelManager.summary.failedOperationCount}</strong>
                <span>failed ops</span>
              </div>
            </div>

            <div className="runtime-policy">
              <strong>{modelManager.cacheRoot}</strong>
              <span>
                No model is installed until required files verify on disk.
              </span>
              {modelManagerOperation ? (
                <small>
                  {statusLabel(modelManagerOperation.action)}:{" "}
                  {statusLabel(modelManagerOperation.status)}
                </small>
              ) : null}
            </div>

            <div className="model-manager-grid">
              <div className="runtime-stack">
                <h3>Lane readiness</h3>
                <ol className="runtime-list">
                  {modelManager.laneReadiness.map((lane) => (
                    <li key={`${lane.lane}-${lane.recommendedCandidateId}`}>
                      <span className={`runtime-dot ${lane.state}`} />
                      <div>
                        <strong>{workflowLabel(lane.lane)}</strong>
                        <small>
                          {lane.recommendedCandidateId} /{" "}
                          {statusLabel(lane.state)}
                        </small>
                        <em>{lane.summary}</em>
                        {lane.blocker ? <em>{lane.blocker}</em> : null}
                      </div>
                    </li>
                  ))}
                </ol>
              </div>

              <div className="runtime-stack">
                <h3>Candidate cache</h3>
                <ol className="runtime-list model-cache-list">
                  {modelManager.candidates.slice(0, 10).map((candidate) => (
                    <li key={candidate.candidateId}>
                      <span
                        className={`runtime-dot ${candidate.installState}`}
                      />
                      <div>
                        <strong>{candidate.name}</strong>
                        <small>
                          {candidate.candidateId} /{" "}
                          {statusLabel(candidate.installState)} /{" "}
                          {candidate.cache.presentFileCount} of{" "}
                          {candidate.cache.expectedFileCount}
                        </small>
                        <em>{candidate.cache.evidence}</em>
                        {candidate.cache.missingRequiredFiles[0] ? (
                          <em>
                            missing{" "}
                            {candidate.cache.missingRequiredFiles.join(", ")}
                          </em>
                        ) : null}
                        <div className="model-manager-actions">
                          <button
                            className="icon-button small"
                            disabled={!candidate.actions.includes("install")}
                            onClick={() =>
                              runModelManagerAction(
                                candidate.candidateId,
                                "install",
                              )
                            }
                            title={`Install ${candidate.name}`}
                            type="button"
                          >
                            <Download aria-hidden="true" size={15} />
                          </button>
                          <button
                            className="icon-button small"
                            onClick={() =>
                              runModelManagerAction(
                                candidate.candidateId,
                                "revalidate",
                              )
                            }
                            title={`Revalidate ${candidate.name}`}
                            type="button"
                          >
                            <CircleCheck aria-hidden="true" size={15} />
                          </button>
                        </div>
                      </div>
                    </li>
                  ))}
                </ol>
              </div>
            </div>

            {modelManagerOperation ? (
              <div
                className={`operation-banner ${modelManagerOperation.status}`}
              >
                <strong>{modelManagerOperation.summary}</strong>
                {modelManagerOperation.recovery ? (
                  <span>{modelManagerOperation.recovery}</span>
                ) : null}
              </div>
            ) : null}

            <ol className="validation-list" aria-label="Model manager checks">
              {modelManager.validationChecks.map((check) => (
                <li
                  className={check.passed ? "passed" : "failed"}
                  key={check.id}
                >
                  <CircleCheck aria-hidden="true" size={16} />
                  <span>
                    {check.summary}
                    {check.recovery ? <em>{check.recovery}</em> : null}
                  </span>
                </li>
              ))}
            </ol>
          </div>

          <div className="panel runtime-panel">
            <div className="panel-heading">
              <h2>Worker Runtime</h2>
              <span>{runtime.schemaVersion}</span>
            </div>
            <div className="runtime-summary" aria-label="Runtime status">
              <div>
                <PackageCheck aria-hidden="true" size={18} />
                <strong>{runtime.statusCounts.installed}</strong>
                <span>verified installs</span>
              </div>
              <div>
                <HardDrive aria-hidden="true" size={18} />
                <strong>{runtime.statusCounts.available}</strong>
                <span>available</span>
              </div>
              <div>
                <CircleAlert aria-hidden="true" size={18} />
                <strong>{runtime.statusCounts.unavailable}</strong>
                <span>unavailable</span>
              </div>
              <div>
                <Cpu aria-hidden="true" size={18} />
                <strong>{runtime.devices.length}</strong>
                <span>devices</span>
              </div>
            </div>

            <div className="runtime-policy">
              <strong>{runtime.packagingPolicy.name}</strong>
              <span>
                Python runtime:{" "}
                {runtime.packagingPolicy.productRuntimeAllowsPython
                  ? "allowed"
                  : "blocked"}
              </span>
              <small>
                {runtime.packagingPolicy.shippedPlatforms
                  .map(statusLabel)
                  .join(" / ")}
              </small>
            </div>

            <div className="runtime-columns">
              <div className="runtime-stack">
                <h3>Models</h3>
                <ol className="runtime-list">
                  {runtime.modelStates.map((model) => (
                    <li key={`${model.providerId}-${model.modelId}`}>
                      <span className={`runtime-dot ${model.availability}`} />
                      <div>
                        <strong>{model.modelName}</strong>
                        <small>
                          {statusLabel(model.availability)} /{" "}
                          {statusLabel(model.installStatus)} /{" "}
                          {formatMb(model.cache.diskUsageMb)}
                        </small>
                        <em>{model.cache.evidence}</em>
                        {model.reasons[0] ? <em>{model.reasons[0]}</em> : null}
                      </div>
                    </li>
                  ))}
                </ol>
              </div>

              <div className="runtime-stack">
                <h3>Jobs</h3>
                <ol className="runtime-list">
                  {runtime.jobs.length === 0 ? (
                    <li>
                      <span className="runtime-dot unavailable" />
                      <div>
                        <strong>No runtime jobs</strong>
                        <small>
                          Fixture/demo actions are blocked until provider
                          execution is wired.
                        </small>
                      </div>
                    </li>
                  ) : null}
                  {runtime.jobs.map((job) => (
                    <li key={job.id}>
                      <span className={`runtime-dot ${job.status}`} />
                      <div>
                        <strong>{statusLabel(job.kind)}</strong>
                        <small>
                          {statusLabel(job.status)} /{" "}
                          {Math.round(job.progress?.percent ?? 0)}% /{" "}
                          {statusLabel(job.cancellation)}
                        </small>
                        {job.actionableError ? (
                          <em>{job.actionableError.recovery}</em>
                        ) : null}
                      </div>
                    </li>
                  ))}
                </ol>
              </div>
            </div>

            <ol className="validation-list" aria-label="Runtime checks">
              {runtime.validationChecks.map((check) => (
                <li className={check.status} key={check.id}>
                  <CircleCheck aria-hidden="true" size={16} />
                  <span>
                    {check.summary}
                    {check.recovery ? <em>{check.recovery}</em> : null}
                  </span>
                </li>
              ))}
            </ol>
          </div>
        </section>
      </section>
    </main>
  );
}
