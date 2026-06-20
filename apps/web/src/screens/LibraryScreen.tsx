// DR-02: Asset Library screen. Extracted from App.tsx (F-010) and rebuilt on the
// shared component grammar (SurfaceHeader hero + HeroStat for the header/metrics,
// MainSurface + SectionHeading for the bespoke subpanels) in place of the
// library-header / library-metrics / tts-subpanel / subpanel-heading classes.
// Screen-specific layout/list classes (asset-library-panel, library-layout,
// library-item, collection-grid, etc.) and every handler, guard, and data
// binding are preserved verbatim.
import { CircleCheck, Play, Search, ShieldCheck } from "lucide-react";
import {
  FeedbackLine,
  HeroStat,
  MainSurface,
  PlaybackControl,
  SectionHeading,
  SurfaceHeader,
} from "../components";
import {
  actionFeedback,
  formatDuration,
  statusLabel,
  toLibraryMutationAction,
} from "../viewModel";
import { useAppContext } from "./context";

export function LibraryScreen() {
  const {
    assetLibrary,
    overview,
    libraryPlayback,
    libraryActionStatus,
    setLibraryActionStatus,
    latestImportableRuntimeJob,
    importLatestRuntimeArtifact,
    mutateSelectedLibraryItem,
    previewLibraryItem,
  } = useAppContext();

  return (
    <section className="asset-library-panel" aria-label="Asset Library">
      <SurfaceHeader
        eyebrow="Asset Library"
        title="Project and global audio assets"
        actions={
          <>
            <div className="library-search" role="search">
              <Search aria-hidden="true" size={18} />
              <span>{assetLibrary.selectedFilter.searchText}</span>
            </div>
            <button
              className="secondary-action"
              disabled={!latestImportableRuntimeJob}
              onClick={importLatestRuntimeArtifact}
              title="Save latest runtime audio artifact to library"
              type="button"
            >
              Save latest output
            </button>
          </>
        }
        stats={
          <>
            <HeroStat label="items" value={overview.assetLibrary.itemCount} />
            <HeroStat
              label="previewable"
              value={overview.assetLibrary.previewableItemCount}
            />
            <HeroStat
              label="collections"
              value={overview.assetLibrary.collectionCount}
            />
            <HeroStat
              label="asset types"
              value={overview.assetLibrary.supportedTypeCount}
            />
          </>
        }
      />
      <FeedbackLine feedback={libraryActionStatus} />
      <PlaybackControl
        playback={libraryPlayback}
        className="library-audio-preview"
      />

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
                item.id === assetLibrary.selectedItem?.item.id
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
                    .map((tag, index) => (
                      <span key={`${item.id}-${index}`}>{tag}</span>
                    ))}
                </div>
              </div>
              <button
                className="icon-action"
                disabled={!item.quickAudition.previewable}
                onClick={() => previewLibraryItem(item.id)}
                title={`Preview ${item.name}`}
                type="button"
              >
                <Play aria-hidden="true" size={16} />
              </button>
            </article>
          ))}
        </div>

        {assetLibrary.selectedItem ? (
          <div className="library-detail" aria-label="Asset detail">
            <MainSurface className="tts-subpanel">
              <SectionHeading
                title={assetLibrary.selectedItem.item.name}
                eyebrow={assetLibrary.selectedItem.item.itemTypeLabel}
              />
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
            </MainSurface>

            <MainSurface className="tts-subpanel">
              <SectionHeading
                title="Version history"
                eyebrow={assetLibrary.selectedItem.versionCount}
              />
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
            </MainSurface>

            <MainSurface className="tts-subpanel">
              <SectionHeading
                title="Recipe provenance"
                eyebrow={
                  assetLibrary.selectedItem.recipe?.workflow ?? "manual"
                }
              />
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
            </MainSurface>
          </div>
        ) : (
          <div className="library-detail" aria-label="Asset detail">
            <MainSurface className="tts-subpanel">
              <SectionHeading title="No asset selected" />
              <p>
                This library has no saved assets yet. Generate audio in a studio
                or import a runtime artifact to populate it.
              </p>
            </MainSurface>
          </div>
        )}
      </div>

      <div className="library-bottom-grid">
        <MainSurface className="tts-subpanel" ariaLabel="Collections">
          <SectionHeading
            title="Collections"
            eyebrow={assetLibrary.collections.length}
          />
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
        </MainSurface>

        <MainSurface className="tts-subpanel" ariaLabel="Lifecycle actions">
          <SectionHeading
            title="Lifecycle"
            eyebrow={assetLibrary.lifecycleActions.length}
          />
          <div className="lifecycle-actions">
            {assetLibrary.lifecycleActions.map((action) => (
              <button
                className="secondary-action"
                key={action.id}
                onClick={() => {
                  const mutation = toLibraryMutationAction(action.id);
                  if (!mutation) {
                    setLibraryActionStatus(
                      actionFeedback.error(
                        `Unknown library action "${action.id}".`,
                      ),
                    );
                    return;
                  }
                  mutateSelectedLibraryItem(mutation);
                }}
                type="button"
              >
                {action.label}
              </button>
            ))}
          </div>
        </MainSurface>

        <MainSurface className="tts-subpanel" ariaLabel="Library validation">
          <SectionHeading
            title="Validation"
            eyebrow={assetLibrary.validationChecks.length}
          />
          <ol className="voice-checks">
            {assetLibrary.validationChecks.map((check) => (
              <li className={check.passed ? "passed" : "failed"} key={check.id}>
                <CircleCheck aria-hidden="true" size={16} />
                <span>{check.summary}</span>
              </li>
            ))}
          </ol>
        </MainSurface>
      </div>
    </section>
  );
}
