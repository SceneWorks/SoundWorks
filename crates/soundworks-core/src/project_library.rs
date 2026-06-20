use crate::asset_library::{
    AssetLibraryOverview, LibraryItemCard, LibraryItemDetail, LibraryItemType, LibraryOwnership,
    QuickAuditionState, WaveformThumbnail,
};
use crate::domain::{
    AssetCreation, AudioAsset, AudioAssetKind, AudioAssetVersion, AudioFileFormat,
    AudioFileReference, CommercialUseStatus, JobStatus, LibraryScope, LicenseStatus, LoopPoints,
    Project, RecipeSummary, RecipeWorkflow, RightsMetadata, TechnicalAudioMetadata,
    VoiceConsentStatus, WatermarkStatus, Workspace,
};
use crate::loudness;
use crate::manifests::CapabilityWorkflow;
use crate::runtime::{RuntimeArtifactKind, RuntimeJobStore};
use crate::storage::sanitized_join;
use crate::workspace::WorkspaceOverview;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
pub struct ProjectLibraryStore {
    root: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PersistedWorkspace {
    workspace: Workspace,
    active_project_id: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PersistedAssetRecord {
    item: LibraryItemCard,
    audio_path: Option<String>,
    metadata_sidecar_path: String,
    provenance_sidecar_path: String,
    updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateProjectRequest {
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportRuntimeArtifactRequest {
    pub job_id: String,
    pub project_id: Option<String>,
    pub name: Option<String>,
    pub scope: Option<LibraryScope>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum LibraryMutationAction {
    Favorite,
    Reject,
    Archive,
    Restore,
    PromoteToGlobal,
    AddTag,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryMutationRequest {
    pub item_id: String,
    pub action: LibraryMutationAction,
    pub tag: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectLibraryActionResult {
    pub workspace: WorkspaceOverview,
    pub asset_library: AssetLibraryOverview,
    pub selected_item: Option<LibraryItemDetail>,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryPlayback {
    pub item_id: String,
    pub playable: bool,
    pub path: Option<String>,
    pub mime_type: Option<String>,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveReviewEditRequest {
    pub item_id: String,
    pub start_ms: Option<u64>,
    pub end_ms: Option<u64>,
    pub fade_in_ms: Option<u64>,
    pub fade_out_ms: Option<u64>,
    pub normalize_loudness_lufs: Option<f32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewEditResult {
    pub library: ProjectLibraryActionResult,
    pub source_path: String,
    pub edited_path: String,
    pub provenance_sidecar_path: String,
    pub version_id: String,
    pub duration_ms: u64,
    pub loudness_lufs: Option<f32>,
    pub true_peak_dbfs: Option<f32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportLibraryItemRequest {
    pub item_id: String,
    pub preset_id: String,
    pub formats: Vec<AudioFileFormat>,
    pub scene_works_project_id: Option<String>,
    pub scene_works_video_asset_id: Option<String>,
    pub replace_existing_audio: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportedArtifact {
    pub path: String,
    pub format: Option<AudioFileFormat>,
    pub kind: String,
    pub bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportLibraryItemResult {
    pub item_id: String,
    pub preset_id: String,
    pub output_root: String,
    pub artifacts: Vec<ExportedArtifact>,
    pub sidecar_path: String,
    pub scene_works_manifest_path: String,
    pub can_attach_directly: bool,
    pub warnings: Vec<String>,
    pub validation_checks: Vec<ExportValidationEvidence>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportValidationEvidence {
    pub id: String,
    pub passed: bool,
    pub summary: String,
}

impl ProjectLibraryStore {
    pub fn default_root() -> PathBuf {
        if let Ok(root) = std::env::var("SOUNDWORKS_LIBRARY_ROOT") {
            return PathBuf::from(root);
        }
        std::env::var("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("."))
            .join("Library")
            .join("Application Support")
            .join("SoundWorks")
            .join("library")
    }

    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    pub fn default() -> Self {
        Self::new(Self::default_root())
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn workspace_overview(&self) -> io::Result<WorkspaceOverview> {
        let library = self.asset_library_overview(None)?;
        self.workspace_overview_with_library(&library)
    }

    /// F-018: build the workspace overview from an already-computed asset-library
    /// overview, so a mutation does not rescan `assets/` and rebuild the fixture
    /// catalog twice (once for the action result, once inside `workspace_overview`).
    fn workspace_overview_with_library(
        &self,
        library: &AssetLibraryOverview,
    ) -> io::Result<WorkspaceOverview> {
        let state = self.load_or_seed_workspace()?;
        let projects = self.read_projects()?;
        let active_project = projects
            .iter()
            .find(|project| project.id == state.active_project_id)
            .cloned()
            .unwrap_or_else(reference_project);
        let mut recent_projects = state
            .workspace
            .recent_project_ids
            .iter()
            .filter_map(|id| projects.iter().find(|project| &project.id == id).cloned())
            .collect::<Vec<_>>();
        if recent_projects.is_empty() {
            recent_projects.push(active_project.clone());
        }

        Ok(WorkspaceOverview::from_library(
            state.workspace,
            active_project,
            recent_projects,
            library,
        ))
    }

    pub fn asset_library_overview(
        &self,
        selected_item_id: Option<&str>,
    ) -> io::Result<AssetLibraryOverview> {
        let persisted: Vec<LibraryItemCard> = self
            .read_asset_records()?
            .into_iter()
            .map(|record| record.item)
            .collect();
        // F-009: production returns only persisted records. The fabricated demo
        // catalog is gated behind an opt-in flag so it is never merged on top of
        // (and inflating) real library data in a shipped build.
        if demo_library_enabled() {
            AssetLibraryOverview::reference_with_persisted_items(persisted, selected_item_id)
                .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error.to_string()))
        } else {
            Ok(AssetLibraryOverview::from_persisted_items(
                persisted,
                selected_item_id,
            ))
        }
    }

    pub fn create_project(
        &self,
        request: CreateProjectRequest,
    ) -> io::Result<ProjectLibraryActionResult> {
        let name = non_empty_or(request.name, "Untitled SoundWorks Project");
        let project_id = format!(
            "project-{}-{}-{}",
            slug(&name),
            timestamp_millis(),
            next_id_sequence()
        );
        let project_root = sanitized_join(&self.root, &["projects", &project_id])?;
        fs::create_dir_all(&project_root)?;

        let project = Project {
            id: project_id.clone(),
            name: name.clone(),
            storage_root: project_root.display().to_string(),
            asset_ids: vec![],
            composition_ids: vec![],
            recipe_ids: vec![],
            job_ids: vec![],
        };
        self.write_project(&project)?;

        let mut state = self.load_or_seed_workspace()?;
        state.active_project_id = project_id.clone();
        state
            .workspace
            .recent_project_ids
            .retain(|id| id != &project_id);
        state.workspace.recent_project_ids.insert(0, project_id);
        self.write_workspace(&state)?;

        self.action_result(
            None,
            format!("Created and opened persisted project \"{name}\"."),
        )
    }

    pub fn open_project(&self, project_id: &str) -> io::Result<ProjectLibraryActionResult> {
        let projects = self.read_projects()?;
        let project = projects
            .iter()
            .find(|project| project.id == project_id)
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::NotFound,
                    format!("project {project_id} does not exist"),
                )
            })?;
        let mut state = self.load_or_seed_workspace()?;
        state.active_project_id = project.id.clone();
        state
            .workspace
            .recent_project_ids
            .retain(|id| id != project_id);
        state
            .workspace
            .recent_project_ids
            .insert(0, project.id.clone());
        self.write_workspace(&state)?;
        self.action_result(
            None,
            format!("Opened persisted project \"{}\".", project.name),
        )
    }

    pub fn import_runtime_artifact(
        &self,
        request: ImportRuntimeArtifactRequest,
    ) -> io::Result<ProjectLibraryActionResult> {
        self.import_runtime_artifact_from_store(request, &RuntimeJobStore::default())
    }

    pub fn import_runtime_artifact_from_store(
        &self,
        request: ImportRuntimeArtifactRequest,
        runtime_store: &RuntimeJobStore,
    ) -> io::Result<ProjectLibraryActionResult> {
        let jobs = runtime_store.read_jobs()?;
        let job = jobs
            .iter()
            .find(|job| job.id == request.job_id)
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::NotFound,
                    format!("runtime job {} does not exist", request.job_id),
                )
            })?;
        if job.status != JobStatus::Succeeded {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "only succeeded runtime jobs can be saved to the library",
            ));
        }
        let artifact = job
            .artifacts
            .iter()
            .find(|artifact| artifact.kind == RuntimeArtifactKind::AudioPreview)
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "runtime job has no audio preview artifact",
                )
            })?;
        let output_manifest = job
            .artifacts
            .iter()
            .find(|artifact| artifact.kind == RuntimeArtifactKind::OutputManifest)
            .and_then(|artifact| read_json::<Value>(&artifact.path).ok());
        let source_path = PathBuf::from(&artifact.path);
        if !source_path.is_file() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("runtime artifact {} is missing", artifact.path),
            ));
        }

        let state = self.load_or_seed_workspace()?;
        let scope = request
            .scope
            .clone()
            .unwrap_or_else(|| LibraryScope::Project {
                project_id: request
                    .project_id
                    .clone()
                    .unwrap_or_else(|| state.active_project_id.clone()),
            });
        let project_id = match &scope {
            LibraryScope::Project { project_id } => Some(project_id.clone()),
            LibraryScope::GlobalLibrary => None,
        };
        let kind = kind_for_workflow(job.workflow);
        let asset_id = format!(
            "asset-{}-{}-{}",
            workflow_fragment(job.workflow),
            timestamp_millis(),
            next_id_sequence()
        );
        let version_id = format!("version-{}-a", asset_id.trim_start_matches("asset-"));
        let asset_root = self.asset_version_root(&scope, kind, &asset_id, &version_id)?;
        fs::create_dir_all(asset_root.join("metadata"))?;
        fs::create_dir_all(asset_root.join("previews"))?;
        let media_path = asset_root.join("media.wav");
        fs::copy(&source_path, &media_path)?;
        let byte_size = fs::metadata(&media_path)?.len();

        let tags = if request.tags.is_empty() {
            vec![
                workflow_fragment(job.workflow).to_string(),
                "runtime-artifact".to_string(),
            ]
        } else {
            request.tags
        };
        let name = request
            .name
            .unwrap_or_else(|| format!("{} runtime output", workflow_label(job.workflow)));
        let recipe = RecipeSummary {
            id: format!("recipe-{}", job.id),
            workflow: recipe_workflow_for(job.workflow),
            provider_id: job.provider_id.clone(),
            model_id: job.model_id.clone(),
            source_reference_count: 0,
            output_asset_count: 1,
            replayable: true,
        };
        let asset = AudioAsset {
            id: asset_id.clone(),
            scope: scope.clone(),
            kind,
            name: name.clone(),
            tags: tags.clone(),
            collection_ids: vec![],
            current_version_id: version_id.clone(),
            version_ids: vec![version_id.clone()],
            rights: RightsMetadata {
                license_status: LicenseStatus::ProviderLicensed,
                commercial_use: CommercialUseStatus::RequiresReview,
                voice_consent: if matches!(kind, AudioAssetKind::VoiceClip) {
                    VoiceConsentStatus::ProviderStockVoice
                } else {
                    VoiceConsentStatus::NotVoiceMaterial
                },
                ai_disclosure_required: true,
                watermark: WatermarkStatus::SidecarOnly,
                reference_media_ownership: Some("generated by SoundWorks runtime".to_string()),
            },
            provenance_ids: vec![format!("provenance-{asset_id}")],
        };
        let version = AudioAssetVersion {
            id: version_id,
            asset_id: asset_id.clone(),
            version_index: 1,
            file: AudioFileReference {
                storage_path: media_path.display().to_string(),
                format: AudioFileFormat::Wav,
                codec: Some("pcm_s16le".to_string()),
                byte_size: Some(byte_size),
                content_hash: None,
            },
            technical: TechnicalAudioMetadata {
                sample_rate_hz: output_manifest
                    .as_ref()
                    .and_then(|manifest| manifest.get("sampleRateHz"))
                    .and_then(Value::as_u64)
                    .and_then(|value| u32::try_from(value).ok())
                    .unwrap_or(48_000),
                bit_depth: Some(16),
                channels: output_manifest
                    .as_ref()
                    .and_then(|manifest| manifest.get("channels"))
                    .and_then(Value::as_u64)
                    .and_then(|value| u16::try_from(value).ok())
                    .unwrap_or(1),
                duration_ms: output_manifest
                    .as_ref()
                    .and_then(|manifest| manifest.get("durationMs"))
                    .and_then(Value::as_u64)
                    .unwrap_or(1_000),
                loudness_lufs: output_manifest
                    .as_ref()
                    .and_then(|manifest| manifest.get("loudnessLufs"))
                    .and_then(Value::as_f64)
                    .map(|value| value as f32),
                true_peak_dbfs: output_manifest
                    .as_ref()
                    .and_then(|manifest| manifest.get("truePeakDbfs"))
                    .and_then(Value::as_f64)
                    .map(|value| value as f32),
                has_clipping: false,
                bpm: output_manifest
                    .as_ref()
                    .and_then(|manifest| manifest.get("bpm"))
                    .and_then(Value::as_f64)
                    .map(|value| value as f32),
                musical_key: output_manifest
                    .as_ref()
                    .and_then(|manifest| manifest.get("musicalKey"))
                    .and_then(Value::as_str)
                    .map(str::to_string),
                loop_points: output_manifest.as_ref().and_then(loop_points_from_manifest),
            },
            created_by: AssetCreation::Generated {
                recipe_id: recipe.id.clone(),
                job_id: job.id.clone(),
            },
            waveform_preview_cache: Some(media_path.display().to_string()),
            spectrogram_preview_cache: None,
        };
        let item = library_item_from_asset(
            asset,
            version,
            Some(recipe),
            project_id.clone(),
            tags,
            LibraryOwnership::ProjectLocal,
            true,
        );
        let metadata_sidecar = asset_root.join("metadata").join("asset.json");
        let provenance_sidecar = asset_root.join("metadata").join("recipe-provenance.json");
        let record = PersistedAssetRecord {
            item: item.clone(),
            audio_path: Some(media_path.display().to_string()),
            metadata_sidecar_path: metadata_sidecar.display().to_string(),
            provenance_sidecar_path: provenance_sidecar.display().to_string(),
            updated_at: timestamp_string(),
        };
        write_json(&metadata_sidecar, &record)?;
        write_json(
            &provenance_sidecar,
            &serde_json::json!({
                "assetId": item.id,
                "jobId": job.id,
                "providerId": job.provider_id,
                "modelId": job.model_id,
                "runtimeRecipePath": format!("{}/recipe.json", job.record_root),
                "runtimeModelMetadataPath": format!("{}/model.json", job.record_root),
                "runtimeOutputManifest": output_manifest,
                "sourceArtifact": artifact.path,
                "copiedTo": media_path,
                "recipe": item.recipe,
            }),
        )?;
        self.write_asset_record(&record)?;
        if let Some(project_id) = project_id {
            self.add_asset_to_project(&project_id, &asset_id)?;
        }

        self.action_result(
            Some(asset_id),
            format!("Saved runtime audio artifact \"{name}\" to the library."),
        )
    }

    pub fn mutate_library_item(
        &self,
        request: LibraryMutationRequest,
    ) -> io::Result<ProjectLibraryActionResult> {
        let mut record = match self.read_asset_record(&request.item_id)? {
            Some(record) => record,
            None => {
                let library = self.asset_library_overview(Some(&request.item_id))?;
                let item = library
                    .items
                    .iter()
                    .find(|item| item.id == request.item_id)
                    .cloned()
                    .ok_or_else(|| {
                        io::Error::new(
                            io::ErrorKind::NotFound,
                            format!("library item {} does not exist", request.item_id),
                        )
                    })?;
                PersistedAssetRecord {
                    audio_path: item
                        .current_version
                        .as_ref()
                        .map(|version| version.file.storage_path.clone()),
                    metadata_sidecar_path: self
                        .asset_record_path(&request.item_id)?
                        .display()
                        .to_string(),
                    provenance_sidecar_path: sanitized_join(
                        &self.root,
                        &["assets", &request.item_id, "recipe-provenance.json"],
                    )?
                    .display()
                    .to_string(),
                    item,
                    updated_at: timestamp_string(),
                }
            }
        };

        match request.action {
            LibraryMutationAction::Favorite => record.item.favorite = !record.item.favorite,
            LibraryMutationAction::Reject => {
                record.item.rejected = true;
                record.item.archived = false;
            }
            LibraryMutationAction::Archive => record.item.archived = true,
            LibraryMutationAction::Restore => {
                record.item.rejected = false;
                record.item.archived = false;
            }
            LibraryMutationAction::PromoteToGlobal => {
                record.item.scope = LibraryScope::GlobalLibrary;
                record.item.project_id = None;
                record.item.ownership = LibraryOwnership::Global;
                if !record.item.tags.iter().any(|tag| tag == "global") {
                    record.item.tags.push("global".to_string());
                }
            }
            LibraryMutationAction::AddTag => {
                let tag = non_empty_or(request.tag.unwrap_or_default(), "reviewed");
                if !record.item.tags.iter().any(|existing| existing == &tag) {
                    record.item.tags.push(tag);
                }
            }
        }
        record.updated_at = timestamp_string();
        // F-002: never write to the persisted (and potentially untrusted) absolute
        // `metadata_sidecar_path` string. write_asset_record recomputes the target
        // from the validated item id under the store root.
        self.write_asset_record(&record)?;

        self.action_result(
            Some(record.item.id.clone()),
            format!("Updated persisted library item {}.", record.item.name),
        )
    }

    pub fn playback_for_item(&self, item_id: &str) -> io::Result<LibraryPlayback> {
        let library = self.asset_library_overview(Some(item_id))?;
        let Some(item) = library.items.iter().find(|item| item.id == item_id) else {
            return Ok(LibraryPlayback {
                item_id: item_id.to_string(),
                playable: false,
                path: None,
                mime_type: None,
                reason: Some("Library item does not exist.".to_string()),
            });
        };
        let Some(version) = &item.current_version else {
            return Ok(LibraryPlayback {
                item_id: item_id.to_string(),
                playable: false,
                path: None,
                mime_type: None,
                reason: Some("This library item has no attached audio version.".to_string()),
            });
        };
        let path = PathBuf::from(&version.file.storage_path);
        if path.is_file() {
            Ok(LibraryPlayback {
                item_id: item_id.to_string(),
                playable: true,
                path: Some(path.display().to_string()),
                mime_type: Some("audio/wav".to_string()),
                reason: None,
            })
        } else {
            Ok(LibraryPlayback {
                item_id: item_id.to_string(),
                playable: false,
                path: None,
                mime_type: Some("audio/wav".to_string()),
                reason: Some(format!(
                    "Audio file is not present on disk: {}",
                    version.file.storage_path
                )),
            })
        }
    }

    /// Resolve a library item's current audio version to interleaved `f32` PCM
    /// for offline composition mixdown (UX-NB1). Returns `Ok(None)` when the item,
    /// its version, or the file is absent, so the mixer can skip an unresolved
    /// clip with a logged warning rather than failing the whole render.
    pub fn load_item_pcm(&self, item_id: &str) -> io::Result<Option<(Vec<f32>, u16, u32)>> {
        let Some(record) = self.read_asset_record(item_id)? else {
            return Ok(None);
        };
        let Some(version) = record.item.current_version.clone() else {
            return Ok(None);
        };
        let path = PathBuf::from(
            record
                .audio_path
                .clone()
                .unwrap_or_else(|| version.file.storage_path.clone()),
        );
        if !path.is_file() {
            return Ok(None);
        }
        let wav = read_pcm16_wav(&path)?;
        let samples = wav
            .samples
            .iter()
            .map(|sample| *sample as f32 / i16::MAX as f32)
            .collect();
        Ok(Some((samples, wav.channels, wav.sample_rate)))
    }

    pub fn save_review_edit(&self, request: SaveReviewEditRequest) -> io::Result<ReviewEditResult> {
        let mut record = self
            .read_asset_record(&request.item_id)?
            .ok_or_else(|| persisted_item_required(&request.item_id))?;
        let source_version = record.item.current_version.clone().ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidInput, "item has no audio version")
        })?;
        let source_path = PathBuf::from(
            record
                .audio_path
                .clone()
                .unwrap_or_else(|| source_version.file.storage_path.clone()),
        );
        let wav = read_pcm16_wav(&source_path)?;
        let start_ms = request.start_ms.unwrap_or(0);
        let end_ms = request
            .end_ms
            .unwrap_or(source_version.technical.duration_ms)
            .min(source_version.technical.duration_ms);
        if end_ms <= start_ms {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "review edit end_ms must be after start_ms",
            ));
        }

        let start_frame = ms_to_frame(start_ms, wav.sample_rate);
        let end_frame = ms_to_frame(end_ms, wav.sample_rate).min(wav.frame_count());
        let mut samples = wav.slice_frames(start_frame, end_frame);
        apply_fade(
            &mut samples,
            wav.channels,
            wav.sample_rate,
            request.fade_in_ms.unwrap_or(60),
            true,
        );
        apply_fade(
            &mut samples,
            wav.channels,
            wav.sample_rate,
            request.fade_out_ms.unwrap_or(120),
            false,
        );
        if let Some(target_lufs) = request.normalize_loudness_lufs {
            normalize_to_lufs(&mut samples, wav.sample_rate, wav.channels, target_lufs);
        }
        let stats = loudness::analyze_i16(&samples, wav.sample_rate, wav.channels);

        let next_index = source_version.version_index + 1;
        let version_id = format!("version-{}-review-{next_index}", request.item_id);
        let asset = record.item.asset.as_ref().ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidInput, "item has no asset metadata")
        })?;
        let asset_root = self.asset_version_root(
            &record.item.scope,
            asset.kind,
            &request.item_id,
            &version_id,
        )?;
        fs::create_dir_all(asset_root.join("metadata"))?;
        let edited_path = asset_root.join("media.wav");
        write_pcm16_wav(&edited_path, wav.sample_rate, wav.channels, &samples)?;
        let duration_ms =
            samples.len() as u64 * 1000 / u64::from(wav.sample_rate) / u64::from(wav.channels);
        let byte_size = fs::metadata(&edited_path)?.len();

        let mut edited_asset = asset.clone();
        edited_asset.current_version_id = version_id.clone();
        if !edited_asset.version_ids.iter().any(|id| id == &version_id) {
            edited_asset.version_ids.push(version_id.clone());
        }
        let provenance_id = format!("provenance-{}-review-{next_index}", request.item_id);
        if !edited_asset
            .provenance_ids
            .iter()
            .any(|id| id == &provenance_id)
        {
            edited_asset.provenance_ids.push(provenance_id.clone());
        }

        let mut edited_version = source_version.clone();
        edited_version.id = version_id.clone();
        edited_version.version_index = next_index;
        edited_version.file = AudioFileReference {
            storage_path: edited_path.display().to_string(),
            format: AudioFileFormat::Wav,
            codec: Some("pcm_s16le".to_string()),
            byte_size: Some(byte_size),
            content_hash: None,
        };
        edited_version.technical.duration_ms = duration_ms;
        edited_version.technical.loudness_lufs = Some(stats.loudness_lufs);
        edited_version.technical.true_peak_dbfs = Some(stats.true_peak_dbfs);
        edited_version.technical.has_clipping = stats.true_peak_dbfs >= -0.01;
        edited_version.created_by = AssetCreation::Edited {
            source_version_id: source_version.id.clone(),
            edit_recipe_id: format!("recipe-review-edit-{}", request.item_id),
        };
        edited_version.waveform_preview_cache = Some(edited_path.display().to_string());

        record.item.asset = Some(edited_asset.clone());
        record.item.current_version = Some(edited_version.clone());
        record.item.duration_ms = Some(duration_ms);
        record.item.waveform_thumbnail = Some(WaveformThumbnail {
            preview_path: edited_path.display().to_string(),
            peak_count: 1,
            duration_ms,
            ready: true,
        });
        record.item.quick_audition.previewable = true;
        record.item.quick_audition.playable_range_ms = Some((0, duration_ms));
        if !record
            .item
            .badges
            .iter()
            .any(|badge| badge == "review-edited")
        {
            record.item.badges.push("review-edited".to_string());
        }
        if !record
            .item
            .generated_tags
            .iter()
            .any(|tag| tag == "edited-version")
        {
            record
                .item
                .generated_tags
                .push("edited-version".to_string());
        }
        record.audio_path = Some(edited_path.display().to_string());
        record.updated_at = timestamp_string();

        let provenance_sidecar = asset_root
            .join("metadata")
            .join("review-edit-provenance.json");
        write_json(
            &provenance_sidecar,
            &serde_json::json!({
                "assetId": request.item_id,
                "sourceVersionId": source_version.id,
                "editedVersionId": version_id,
                "sourcePath": source_path,
                "editedPath": edited_path,
                "operations": {
                    "trimStartMs": start_ms,
                    "trimEndMs": end_ms,
                    "fadeInMs": request.fade_in_ms.unwrap_or(60),
                    "fadeOutMs": request.fade_out_ms.unwrap_or(120),
                    "normalizeLoudnessLufs": request.normalize_loudness_lufs,
                },
                "technical": edited_version.technical,
                "nonDestructive": true,
            }),
        )?;
        // F-002: write only through write_asset_record (recomputed, validated path);
        // do not trust the persisted absolute `metadata_sidecar_path` for writes.
        self.write_asset_record(&record)?;

        let library = self.action_result(
            Some(request.item_id.clone()),
            format!("Saved non-destructive edited WAV version {next_index}."),
        )?;

        Ok(ReviewEditResult {
            library,
            source_path: source_path.display().to_string(),
            edited_path: edited_path.display().to_string(),
            provenance_sidecar_path: provenance_sidecar.display().to_string(),
            version_id,
            duration_ms,
            loudness_lufs: Some(stats.loudness_lufs),
            true_peak_dbfs: Some(stats.true_peak_dbfs),
        })
    }

    pub fn export_library_item(
        &self,
        request: ExportLibraryItemRequest,
    ) -> io::Result<ExportLibraryItemResult> {
        let record = self
            .read_asset_record(&request.item_id)?
            .ok_or_else(|| persisted_item_required(&request.item_id))?;
        // F-003: enforce the displayed rights/safety policy at export, not just at
        // generation. Refuse to export material whose stored rights are not cleared.
        if let Some(reason) = export_block_reason(&record) {
            return Err(io::Error::new(io::ErrorKind::PermissionDenied, reason));
        }
        let version = record.item.current_version.as_ref().ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidInput, "item has no audio version")
        })?;
        let source_path = PathBuf::from(
            record
                .audio_path
                .clone()
                .unwrap_or_else(|| version.file.storage_path.clone()),
        );
        if !source_path.is_file() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("audio file is missing on disk: {}", source_path.display()),
            ));
        }
        let source_audio = read_pcm16_wav(&source_path)?;
        let export_id = format!(
            "export-{}-{}-{}",
            request.item_id,
            timestamp_millis(),
            next_id_sequence()
        );
        let output_root = sanitized_join(&self.root, &["exports", &export_id])?;
        fs::create_dir_all(&output_root)?;

        let requested_formats = if request.formats.is_empty() {
            vec![AudioFileFormat::Wav]
        } else {
            request.formats.clone()
        };
        let mut artifacts = vec![];
        let mut warnings = vec![];
        for format in requested_formats {
            if format == AudioFileFormat::Wav {
                let audio_path = output_root.join(format!("{}.wav", slug(&record.item.name)));
                fs::copy(&source_path, &audio_path)?;
                artifacts.push(ExportedArtifact {
                    path: audio_path.display().to_string(),
                    format: Some(AudioFileFormat::Wav),
                    kind: "audio-file".to_string(),
                    bytes: fs::metadata(&audio_path)?.len(),
                });
            } else {
                warnings.push(format!(
                    "{} export is blocked until a local encoder is added and validated.",
                    format.extension().to_ascii_uppercase()
                ));
            }
        }
        if artifacts
            .iter()
            .all(|artifact| artifact.kind != "audio-file")
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "at least one supported export format is required; WAV is currently supported",
            ));
        }

        let sidecar_path = output_root.join("soundworks-export.json");
        let scene_works_manifest_path = output_root.join("sceneworks-handoff.json");
        write_json(
            &sidecar_path,
            &serde_json::json!({
                "exportId": export_id,
                "presetId": request.preset_id,
                "assetId": request.item_id,
                "assetName": record.item.name,
                "sourceAudioPath": source_path,
                "exportedArtifacts": artifacts,
                "recipe": record.item.recipe,
                "rights": record.item.asset.as_ref().map(|asset| &asset.rights),
                "technical": version.technical,
                "sourceProvenanceSidecar": record.provenance_sidecar_path,
                "warnings": warnings,
            }),
        )?;
        write_json(
            &scene_works_manifest_path,
            &serde_json::json!({
                "schema": "soundworks.sceneworks-audio-handoff.v1",
                "exportId": export_id,
                "soundWorksAssetId": request.item_id,
                "soundWorksVersionId": version.id,
                "renderedMixdownPath": artifacts.iter().find(|artifact| artifact.kind == "audio-file").map(|artifact| &artifact.path),
                "provenanceSidecarPath": sidecar_path,
                "target": {
                    "projectId": request.scene_works_project_id,
                    "videoAssetId": request.scene_works_video_asset_id,
                    "replaceExistingAudio": request.replace_existing_audio,
                    "directAttachmentSupported": false,
                },
                "audio": {
                    "durationMs": version.technical.duration_ms,
                    "sampleRateHz": source_audio.sample_rate,
                    "channels": source_audio.channels,
                    "loudnessLufs": version.technical.loudness_lufs,
                    "truePeakDbfs": version.technical.true_peak_dbfs,
                },
                "compatibility": [
                    {
                        "id": "source.file.exists",
                        "passed": true,
                        "summary": "SoundWorks source audio exists on disk."
                    },
                    {
                        "id": "sceneworks.direct.import",
                        "passed": false,
                        "summary": "SceneWorks has no verified standalone audio-track importer in this slice; package handoff is file-backed only."
                    }
                ]
            }),
        )?;
        artifacts.push(ExportedArtifact {
            path: sidecar_path.display().to_string(),
            format: None,
            kind: "metadata-sidecar".to_string(),
            bytes: fs::metadata(&sidecar_path)?.len(),
        });
        artifacts.push(ExportedArtifact {
            path: scene_works_manifest_path.display().to_string(),
            format: None,
            kind: "sceneworks-package-manifest".to_string(),
            bytes: fs::metadata(&scene_works_manifest_path)?.len(),
        });

        Ok(ExportLibraryItemResult {
            item_id: request.item_id,
            preset_id: request.preset_id,
            output_root: output_root.display().to_string(),
            artifacts,
            sidecar_path: sidecar_path.display().to_string(),
            scene_works_manifest_path: scene_works_manifest_path.display().to_string(),
            can_attach_directly: false,
            warnings,
            validation_checks: vec![
                ExportValidationEvidence {
                    id: "export.audio_file".to_string(),
                    passed: true,
                    summary: "Export wrote at least one real audio file to disk.".to_string(),
                },
                ExportValidationEvidence {
                    id: "export.sidecar".to_string(),
                    passed: true,
                    summary: "Export wrote a SoundWorks provenance sidecar.".to_string(),
                },
                ExportValidationEvidence {
                    id: "sceneworks.package_manifest".to_string(),
                    passed: true,
                    summary: "SceneWorks handoff manifest references the exported audio artifact.".to_string(),
                },
                ExportValidationEvidence {
                    id: "sceneworks.direct_attachment".to_string(),
                    passed: false,
                    summary: "Direct SceneWorks runtime attachment remains blocked until an importer exists.".to_string(),
                },
            ],
        })
    }

    fn action_result(
        &self,
        selected_item_id: Option<String>,
        message: String,
    ) -> io::Result<ProjectLibraryActionResult> {
        let asset_library = self.asset_library_overview(selected_item_id.as_deref())?;
        let workspace = self.workspace_overview_with_library(&asset_library)?;
        let selected_item = asset_library.selected_item.clone();
        Ok(ProjectLibraryActionResult {
            workspace,
            asset_library,
            selected_item,
            message,
        })
    }

    fn load_or_seed_workspace(&self) -> io::Result<PersistedWorkspace> {
        let path = self.workspace_path();
        if path.is_file() {
            return read_json(path);
        }
        fs::create_dir_all(self.root.join("projects"))?;
        fs::create_dir_all(self.root.join("assets"))?;
        // F-017: persist the default project on first run so imported assets can be
        // linked to it (its active_project_id is otherwise a record with no file).
        self.write_project(&reference_project())?;
        let state = PersistedWorkspace {
            workspace: Workspace {
                id: "workspace-local".to_string(),
                global_library_id: "global-library".to_string(),
                recent_project_ids: vec!["project-demo".to_string()],
            },
            active_project_id: "project-demo".to_string(),
        };
        self.write_workspace(&state)?;
        Ok(state)
    }

    fn write_workspace(&self, state: &PersistedWorkspace) -> io::Result<()> {
        fs::create_dir_all(&self.root)?;
        write_json(self.workspace_path(), state)
    }

    fn workspace_path(&self) -> PathBuf {
        self.root.join("workspace.json")
    }

    fn read_projects(&self) -> io::Result<Vec<Project>> {
        // F-017/F-018: prefer the persisted copy of each project. The demo project is
        // injected only as a fallback when nothing is on disk, so an updated
        // project-demo/project.json (e.g. with linked assets) is never shadowed by the
        // in-memory reference.
        let projects_root = self.root.join("projects");
        let mut projects: Vec<Project> = vec![];
        if projects_root.exists() {
            for entry in fs::read_dir(projects_root)? {
                let path = entry?.path().join("project.json");
                if path.is_file() {
                    projects.push(read_json(path)?);
                }
            }
        }
        if !projects.iter().any(|project| project.id == "project-demo") {
            projects.push(reference_project());
        }
        Ok(projects)
    }

    fn write_project(&self, project: &Project) -> io::Result<()> {
        let path = sanitized_join(&self.root, &["projects", &project.id, "project.json"])?;
        write_json(path, project)
    }

    fn add_asset_to_project(&self, project_id: &str, asset_id: &str) -> io::Result<()> {
        let mut projects = self.read_projects()?;
        if let Some(project) = projects.iter_mut().find(|project| project.id == project_id) {
            if !project.asset_ids.iter().any(|id| id == asset_id) {
                project.asset_ids.push(asset_id.to_string());
            }
            // F-017: always persist the link, including for the default project-demo
            // (which is now seeded on disk), so imported assets are never orphaned.
            self.write_project(project)?;
        }
        Ok(())
    }

    fn read_asset_records(&self) -> io::Result<Vec<PersistedAssetRecord>> {
        let assets_root = self.root.join("assets");
        if !assets_root.exists() {
            return Ok(vec![]);
        }
        let mut records = vec![];
        for entry in fs::read_dir(assets_root)? {
            let path = entry?.path().join("asset-record.json");
            if path.is_file() {
                records.push(read_json(path)?);
            }
        }
        Ok(records)
    }

    fn read_asset_record(&self, item_id: &str) -> io::Result<Option<PersistedAssetRecord>> {
        let path = self.asset_record_path(item_id)?;
        if path.is_file() {
            Ok(Some(read_json(path)?))
        } else {
            Ok(None)
        }
    }

    fn write_asset_record(&self, record: &PersistedAssetRecord) -> io::Result<()> {
        write_json(self.asset_record_path(&record.item.id)?, record)
    }

    fn asset_record_path(&self, item_id: &str) -> io::Result<PathBuf> {
        sanitized_join(&self.root, &["assets", item_id, "asset-record.json"])
    }

    fn asset_version_root(
        &self,
        scope: &LibraryScope,
        kind: AudioAssetKind,
        asset_id: &str,
        version_id: &str,
    ) -> io::Result<PathBuf> {
        let kind_dir = kind.storage_dir();
        match scope {
            LibraryScope::GlobalLibrary => {
                sanitized_join(&self.root, &["global", kind_dir, asset_id, version_id])
            }
            LibraryScope::Project { project_id } => sanitized_join(
                &self.root,
                &["projects", project_id, kind_dir, asset_id, version_id],
            ),
        }
    }
}

fn library_item_from_asset(
    asset: AudioAsset,
    version: AudioAssetVersion,
    recipe: Option<RecipeSummary>,
    project_id: Option<String>,
    tags: Vec<String>,
    ownership: LibraryOwnership,
    previewable: bool,
) -> LibraryItemCard {
    let item_type = LibraryItemType::from_asset_kind(asset.kind);
    LibraryItemCard {
        id: asset.id.clone(),
        name: asset.name.clone(),
        item_type,
        item_type_label: item_type.label().to_string(),
        asset: Some(asset.clone()),
        current_version: Some(version.clone()),
        scope: asset.scope.clone(),
        ownership,
        project_id,
        created_at: timestamp_string(),
        source_workflow: recipe.as_ref().map(|recipe| recipe.workflow),
        tags,
        generated_tags: vec!["runtime-artifact".to_string(), "persisted".to_string()],
        collection_ids: asset.collection_ids.clone(),
        duration_ms: Some(version.technical.duration_ms),
        bpm: version.technical.bpm,
        musical_key: version.technical.musical_key.clone(),
        language: matches!(asset.kind, AudioAssetKind::VoiceClip).then(|| "en-US".to_string()),
        voice_profile_id: None,
        provider_id: recipe.as_ref().map(|recipe| recipe.provider_id.clone()),
        model_id: recipe.as_ref().map(|recipe| recipe.model_id.clone()),
        license_status: asset.rights.license_status,
        commercial_use: asset.rights.commercial_use,
        favorite: false,
        rejected: false,
        archived: false,
        waveform_thumbnail: previewable.then(|| WaveformThumbnail {
            preview_path: version.file.storage_path.clone(),
            peak_count: 1,
            duration_ms: version.technical.duration_ms,
            ready: true,
        }),
        quick_audition: QuickAuditionState {
            previewable,
            playable_range_ms: previewable.then_some((0, version.technical.duration_ms)),
            shortcut: "Space".to_string(),
        },
        timeline_placeable: true,
        source_picker_eligible: true,
        composition_usage_count: 0,
        recipe,
        badges: vec![
            item_type.label().to_string(),
            "persisted".to_string(),
            "runtime".to_string(),
        ],
    }
}

/// F-009: opt-in flag that merges the fabricated demo asset catalog into the
/// library. Off by default so shipped builds show only persisted records; set
/// `SOUNDWORKS_DEMO_LIBRARY=1` (or `true`) for demos, screenshots, and walkthroughs.
fn demo_library_enabled() -> bool {
    let env_enabled = std::env::var("SOUNDWORKS_DEMO_LIBRARY")
        .map(|value| {
            let value = value.trim();
            value == "1" || value.eq_ignore_ascii_case("true")
        })
        .unwrap_or(false);
    // UX-15: the env var still works for CI/screenshots, but a durable user
    // preference (Settings demo toggle) enables it too.
    env_enabled || crate::ui_preferences::UiPreferencesStore::default().load().demo == Some(true)
}

fn reference_project() -> Project {
    Project {
        id: "project-demo".to_string(),
        name: "Demo SoundWorks Project".to_string(),
        storage_root: "soundworks-library/projects/project-demo".to_string(),
        asset_ids: vec![],
        composition_ids: vec!["composition-demo".to_string()],
        recipe_ids: vec![],
        job_ids: vec![],
    }
}

fn kind_for_workflow(workflow: CapabilityWorkflow) -> AudioAssetKind {
    match workflow {
        CapabilityWorkflow::Tts | CapabilityWorkflow::VoiceClone => AudioAssetKind::VoiceClip,
        CapabilityWorkflow::Sfx => AudioAssetKind::Sfx,
        CapabilityWorkflow::Ambience | CapabilityWorkflow::VideoToAudio => AudioAssetKind::Ambience,
        CapabilityWorkflow::InstrumentSample => AudioAssetKind::InstrumentSample,
        CapabilityWorkflow::Loop => AudioAssetKind::Loop,
        CapabilityWorkflow::Song => AudioAssetKind::Song,
        CapabilityWorkflow::StemSeparation => AudioAssetKind::Stem,
        CapabilityWorkflow::CompositionRender | CapabilityWorkflow::Edit => {
            AudioAssetKind::MixdownExport
        }
        CapabilityWorkflow::VoiceConversion => AudioAssetKind::VoiceClip,
    }
}

fn recipe_workflow_for(workflow: CapabilityWorkflow) -> RecipeWorkflow {
    match workflow {
        CapabilityWorkflow::Tts | CapabilityWorkflow::VoiceClone => RecipeWorkflow::Tts,
        CapabilityWorkflow::VoiceConversion => RecipeWorkflow::VoiceConversion,
        CapabilityWorkflow::Sfx | CapabilityWorkflow::Ambience => RecipeWorkflow::Sfx,
        CapabilityWorkflow::InstrumentSample => RecipeWorkflow::InstrumentSample,
        CapabilityWorkflow::Loop => RecipeWorkflow::Loop,
        CapabilityWorkflow::Song | CapabilityWorkflow::StemSeparation => RecipeWorkflow::Song,
        CapabilityWorkflow::VideoToAudio => RecipeWorkflow::VideoToAudio,
        CapabilityWorkflow::Edit => RecipeWorkflow::Edit,
        CapabilityWorkflow::CompositionRender => RecipeWorkflow::CompositionRender,
    }
}

fn workflow_fragment(workflow: CapabilityWorkflow) -> &'static str {
    match workflow {
        CapabilityWorkflow::Tts => "tts",
        CapabilityWorkflow::VoiceClone => "voice-clone",
        CapabilityWorkflow::VoiceConversion => "voice-conversion",
        CapabilityWorkflow::Sfx => "sfx",
        CapabilityWorkflow::Ambience => "ambience",
        CapabilityWorkflow::InstrumentSample => "sample",
        CapabilityWorkflow::Loop => "loop",
        CapabilityWorkflow::Song => "song",
        CapabilityWorkflow::StemSeparation => "stem",
        CapabilityWorkflow::VideoToAudio => "video-to-audio",
        CapabilityWorkflow::Edit => "edit",
        CapabilityWorkflow::CompositionRender => "composition-render",
    }
}

fn workflow_label(workflow: CapabilityWorkflow) -> &'static str {
    match workflow {
        CapabilityWorkflow::Tts => "TTS",
        CapabilityWorkflow::VoiceClone => "Voice clone",
        CapabilityWorkflow::VoiceConversion => "Voice conversion",
        CapabilityWorkflow::Sfx => "SFX",
        CapabilityWorkflow::Ambience => "Ambience",
        CapabilityWorkflow::InstrumentSample => "Sample",
        CapabilityWorkflow::Loop => "Loop",
        CapabilityWorkflow::Song => "Song",
        CapabilityWorkflow::StemSeparation => "Stem",
        CapabilityWorkflow::VideoToAudio => "Video-to-audio",
        CapabilityWorkflow::Edit => "Edit",
        CapabilityWorkflow::CompositionRender => "Composition render",
    }
}

fn loop_points_from_manifest(manifest: &Value) -> Option<LoopPoints> {
    let start_sample = manifest.get("loopStartSample")?.as_u64()?;
    let end_sample = manifest.get("loopEndSample")?.as_u64()?;
    (end_sample > start_sample).then_some(LoopPoints {
        start_sample,
        end_sample,
    })
}

fn slug(value: &str) -> String {
    let slug = value
        .to_ascii_lowercase()
        .chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("-");
    non_empty_or(slug, "project")
}

fn non_empty_or(value: String, fallback: &str) -> String {
    if value.trim().is_empty() {
        fallback.to_string()
    } else {
        value.trim().to_string()
    }
}

fn timestamp_millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or_default()
}

/// Process-wide monotonic counter appended to generated identifiers so two ids
/// created in the same millisecond (or under a faulted clock that reports 0) never
/// collide and clobber each other's on-disk directory.
fn next_id_sequence() -> u64 {
    static ID_SEQUENCE: AtomicU64 = AtomicU64::new(0);
    ID_SEQUENCE.fetch_add(1, Ordering::Relaxed)
}

fn timestamp_string() -> String {
    timestamp_millis().to_string()
}

fn write_json(path: impl AsRef<Path>, value: &impl Serialize) -> io::Result<()> {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let payload = serde_json::to_vec_pretty(value)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    write_atomic(path, &payload)
}

/// F-005: write durably — to a sibling temp file, fsync it, then atomically rename
/// over the destination, so a crash mid-write can never leave a truncated/corrupt
/// file. The parent directory is fsynced best-effort so the rename survives a crash.
fn write_atomic(path: &Path, payload: &[u8]) -> io::Result<()> {
    let mut temp_path = path.as_os_str().to_os_string();
    temp_path.push(".tmp");
    let temp_path = PathBuf::from(temp_path);
    {
        let mut file = fs::File::create(&temp_path)?;
        file.write_all(payload)?;
        file.sync_all()?;
    }
    fs::rename(&temp_path, path)?;
    if let Some(parent) = path.parent() {
        if let Ok(dir) = fs::File::open(parent) {
            let _ = dir.sync_all();
        }
    }
    Ok(())
}

fn read_json<T: DeserializeOwned>(path: impl AsRef<Path>) -> io::Result<T> {
    let payload = fs::read(path)?;
    serde_json::from_slice(&payload)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))
}

/// F-003 export gate: refuse to export material whose persisted rights are not
/// cleared — voice consent under review or prohibited, or commercial use
/// disallowed. Runtime-generated assets carry ProviderStockVoice/NotVoiceMaterial
/// consent and RequiresReview commercial use (a warn state, not a hard block), so
/// the working export path is unaffected. Returns the blocking reason, or None when
/// export may proceed.
fn export_block_reason(record: &PersistedAssetRecord) -> Option<String> {
    let asset = record.item.asset.as_ref()?;
    let rights = &asset.rights;
    match rights.voice_consent {
        VoiceConsentStatus::RequiresReview => {
            return Some(
                "Export blocked: voice consent for this asset is still under review. Record explicit speaker consent before exporting voice material."
                    .to_string(),
            );
        }
        VoiceConsentStatus::Prohibited => {
            return Some(
                "Export blocked: this asset references a prohibited (public-figure or unauthorized) voice and cannot be exported."
                    .to_string(),
            );
        }
        _ => {}
    }
    if rights.commercial_use == CommercialUseStatus::Disallowed {
        return Some(
            "Export blocked: commercial use of this asset is disallowed by its stored rights."
                .to_string(),
        );
    }
    None
}

fn persisted_item_required(item_id: &str) -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidInput,
        format!("item {item_id} must be a persisted runtime library item"),
    )
}

#[derive(Debug, Clone)]
struct Pcm16Wav {
    sample_rate: u32,
    channels: u16,
    samples: Vec<i16>,
}

impl Pcm16Wav {
    fn frame_count(&self) -> usize {
        self.samples.len() / usize::from(self.channels)
    }

    fn slice_frames(&self, start_frame: usize, end_frame: usize) -> Vec<i16> {
        let channels = usize::from(self.channels);
        let start = start_frame.min(self.frame_count()) * channels;
        let end = end_frame.min(self.frame_count()) * channels;
        self.samples[start..end].to_vec()
    }
}

/// Read a little-endian u16 at `at`, returning InvalidData (not a panic) when the
/// slice runs past the end of an untrusted buffer.
fn read_u16_le(bytes: &[u8], at: usize) -> io::Result<u16> {
    bytes
        .get(at..at + 2)
        .and_then(|slice| <[u8; 2]>::try_from(slice).ok())
        .map(u16::from_le_bytes)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "truncated WAV chunk field"))
}

/// Read a little-endian u32 at `at`, returning InvalidData on truncation.
fn read_u32_le(bytes: &[u8], at: usize) -> io::Result<u32> {
    bytes
        .get(at..at + 4)
        .and_then(|slice| <[u8; 4]>::try_from(slice).ok())
        .map(u32::from_le_bytes)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "truncated WAV chunk field"))
}

fn read_pcm16_wav(path: &Path) -> io::Result<Pcm16Wav> {
    let bytes = fs::read(path)?;
    if bytes.len() < 44 || &bytes[0..4] != b"RIFF" || &bytes[8..12] != b"WAVE" {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "only RIFF/WAVE PCM files are supported",
        ));
    }

    // F-016: every multi-byte read goes through read_u16_le/read_u32_le (checked),
    // chunk advancement saturates, and the fmt guard requires the declared body to
    // actually be present, so a crafted/truncated WAV returns InvalidData instead of
    // panicking the command thread on an out-of-range slice.
    let mut cursor = 12usize;
    let mut channels = None;
    let mut sample_rate = None;
    let mut bits_per_sample = None;
    let mut data_range = None;
    while cursor + 8 <= bytes.len() {
        let id = &bytes[cursor..cursor + 4];
        let size = read_u32_le(&bytes, cursor + 4)? as usize;
        let chunk_start = cursor + 8;
        let chunk_end = chunk_start.saturating_add(size).min(bytes.len());
        if id == b"fmt " {
            // The declared size may exceed the bytes actually present; require the
            // full 16-byte PCM fmt body before reading any field.
            if size < 16 || chunk_start + 16 > bytes.len() {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "invalid fmt chunk",
                ));
            }
            let audio_format = read_u16_le(&bytes, chunk_start)?;
            if audio_format != 1 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "only PCM WAV files are supported",
                ));
            }
            channels = Some(read_u16_le(&bytes, chunk_start + 2)?);
            sample_rate = Some(read_u32_le(&bytes, chunk_start + 4)?);
            bits_per_sample = Some(read_u16_le(&bytes, chunk_start + 14)?);
        } else if id == b"data" {
            data_range = Some((chunk_start, chunk_end));
        }
        // Advance past the chunk body plus RIFF word-alignment padding, saturating so
        // a huge declared size can never wrap the cursor and loop forever.
        cursor = chunk_start.saturating_add(size).saturating_add(size % 2);
    }

    let channels = channels.ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            "WAV fmt chunk is missing channels",
        )
    })?;
    if channels == 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "WAV channel count cannot be zero",
        ));
    }
    let sample_rate = sample_rate.ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            "WAV fmt chunk is missing sample rate",
        )
    })?;
    if bits_per_sample != Some(16) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "only 16-bit PCM WAV files are supported",
        ));
    }
    let (data_start, data_end) = data_range
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "WAV data chunk is missing"))?;
    let data = bytes
        .get(data_start..data_end)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "WAV data chunk is truncated"))?;
    let mut samples = Vec::with_capacity(data.len() / 2);
    for chunk in data.chunks_exact(2) {
        samples.push(i16::from_le_bytes([chunk[0], chunk[1]]));
    }

    Ok(Pcm16Wav {
        sample_rate,
        channels,
        samples,
    })
}

fn write_pcm16_wav(
    path: &Path,
    sample_rate: u32,
    channels: u16,
    samples: &[i16],
) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let data_bytes = samples.len() as u32 * 2;
    let mut bytes = Vec::with_capacity(44 + data_bytes as usize);
    bytes.extend_from_slice(b"RIFF");
    bytes.extend_from_slice(&(36 + data_bytes).to_le_bytes());
    bytes.extend_from_slice(b"WAVEfmt ");
    bytes.extend_from_slice(&16u32.to_le_bytes());
    bytes.extend_from_slice(&1u16.to_le_bytes());
    bytes.extend_from_slice(&channels.to_le_bytes());
    bytes.extend_from_slice(&sample_rate.to_le_bytes());
    bytes.extend_from_slice(&(sample_rate * u32::from(channels) * 2).to_le_bytes());
    bytes.extend_from_slice(&(channels * 2).to_le_bytes());
    bytes.extend_from_slice(&16u16.to_le_bytes());
    bytes.extend_from_slice(b"data");
    bytes.extend_from_slice(&data_bytes.to_le_bytes());
    for sample in samples {
        bytes.extend_from_slice(&sample.to_le_bytes());
    }
    fs::write(path, bytes)
}

fn ms_to_frame(ms: u64, sample_rate: u32) -> usize {
    (ms.saturating_mul(u64::from(sample_rate)) / 1000) as usize
}

fn apply_fade(
    samples: &mut [i16],
    channels: u16,
    sample_rate: u32,
    duration_ms: u64,
    fade_in: bool,
) {
    let channels = usize::from(channels);
    let fade_frames = ms_to_frame(duration_ms, sample_rate).min(samples.len() / channels);
    if fade_frames == 0 {
        return;
    }
    for frame in 0..fade_frames {
        let gain = frame as f32 / fade_frames as f32;
        let gain = if fade_in { gain } else { 1.0 - gain };
        let target_frame = if fade_in {
            frame
        } else {
            samples.len() / channels - 1 - frame
        };
        for channel in 0..channels {
            let index = target_frame * channels + channel;
            samples[index] = scale_sample(samples[index], gain);
        }
    }
}

/// Normalize toward `target_lufs` with a single-pass gain. A linear gain cannot
/// exactly hit a gated-LUFS target (the gated block set shifts with level), but
/// it lands within a fraction of a dB for steady material.
fn normalize_to_lufs(samples: &mut [i16], sample_rate: u32, channels: u16, target_lufs: f32) {
    let stats = loudness::analyze_i16(samples, sample_rate, channels);
    let gain = 10.0_f32.powf((target_lufs - stats.loudness_lufs) / 20.0);
    for sample in samples {
        *sample = scale_sample(*sample, gain);
    }
}

fn scale_sample(sample: i16, gain: f32) -> i16 {
    (sample as f32 * gain)
        .clamp(i16::MIN as f32, i16::MAX as f32)
        .round() as i16
}

#[cfg(test)]
mod tests {
    use super::{
        CreateProjectRequest, ExportLibraryItemRequest, ImportRuntimeArtifactRequest,
        LibraryMutationAction, LibraryMutationRequest, ProjectLibraryStore, SaveReviewEditRequest,
    };
    use crate::domain::{AudioFileFormat, JobKind, JobProgress, JobStatus};
    use crate::manifests::CapabilityWorkflow;
    use crate::runtime::{
        CancellationState, ProviderAdapterKind, RuntimeArtifactKind, RuntimeJobArtifact,
        RuntimeJobSnapshot, RuntimeJobStore,
    };
    use std::fs;
    use std::path::PathBuf;

    #[test]
    fn created_project_persists_and_reopens() {
        let store = ProjectLibraryStore::new(temp_root("project"));

        let created = store
            .create_project(CreateProjectRequest {
                name: "Recovery Project".to_string(),
            })
            .expect("project created");
        let project_id = created.workspace.active_project.project.id.clone();
        assert_eq!(
            created.workspace.active_project.project.name,
            "Recovery Project"
        );
        assert!(PathBuf::from(&created.workspace.active_project.project.storage_root).exists());

        let reopened = ProjectLibraryStore::new(store.root().to_path_buf())
            .open_project(&project_id)
            .expect("project reopened");
        assert_eq!(reopened.workspace.active_project.project.id, project_id);
    }

    #[test]
    fn export_blocks_assets_with_unreviewed_voice_consent() {
        // F-003 export gate: a clean runtime asset (ProviderStockVoice /
        // NotVoiceMaterial, RequiresReview commercial) exports fine; flipping the
        // persisted voice consent to RequiresReview blocks export.
        let store = ProjectLibraryStore::new(temp_root("export-gate"));
        let runtime_root = temp_root("export-gate-runtime");
        let runtime_store = RuntimeJobStore::new(&runtime_root);
        let job_id = seed_runtime_job(&runtime_root);
        let imported = store
            .import_runtime_artifact_from_store(
                ImportRuntimeArtifactRequest {
                    job_id,
                    project_id: None,
                    name: None,
                    scope: None,
                    tags: vec![],
                },
                &runtime_store,
            )
            .expect("runtime artifact imported");
        let item_id = imported
            .asset_library
            .selected_item
            .as_ref()
            .unwrap()
            .item
            .id
            .clone();

        let export_request = |item_id: String| ExportLibraryItemRequest {
            item_id,
            preset_id: "wav".to_string(),
            formats: vec![AudioFileFormat::Wav],
            scene_works_project_id: None,
            scene_works_video_asset_id: None,
            replace_existing_audio: false,
        };

        store
            .export_library_item(export_request(item_id.clone()))
            .expect("a cleared asset exports");

        // Flip the persisted rights to requires-review and confirm export is blocked.
        let record_path = store
            .root()
            .join("assets")
            .join(&item_id)
            .join("asset-record.json");
        let mut record: serde_json::Value =
            serde_json::from_slice(&fs::read(&record_path).expect("read record"))
                .expect("parse record");
        record["item"]["asset"]["rights"]["voiceConsent"] = serde_json::json!("requires-review");
        fs::write(
            &record_path,
            serde_json::to_vec_pretty(&record).expect("serialize record"),
        )
        .expect("write record");

        let blocked = store.export_library_item(export_request(item_id));
        assert!(
            blocked.is_err(),
            "export of unreviewed voice material must be blocked"
        );
        assert_eq!(
            blocked.unwrap_err().kind(),
            std::io::ErrorKind::PermissionDenied
        );
    }

    #[test]
    fn imported_asset_is_linked_to_the_default_project() {
        // F-017: an artifact imported into the out-of-the-box workspace must be linked
        // to the seeded project-demo, not orphaned.
        let store = ProjectLibraryStore::new(temp_root("orphan-link"));
        let runtime_root = temp_root("orphan-link-runtime");
        let runtime_store = RuntimeJobStore::new(&runtime_root);
        let job_id = seed_runtime_job(&runtime_root);
        let imported = store
            .import_runtime_artifact_from_store(
                ImportRuntimeArtifactRequest {
                    job_id,
                    project_id: None,
                    name: None,
                    scope: None,
                    tags: vec![],
                },
                &runtime_store,
            )
            .expect("runtime artifact imported");
        let asset_id = imported
            .asset_library
            .selected_item
            .as_ref()
            .unwrap()
            .item
            .id
            .clone();

        let workspace = store.workspace_overview().expect("workspace overview");
        assert_eq!(workspace.active_project.project.id, "project-demo");
        assert!(
            workspace
                .active_project
                .project
                .asset_ids
                .contains(&asset_id),
            "imported asset must be linked to project-demo, not orphaned"
        );
    }

    #[test]
    fn writes_leave_no_temp_files_behind() {
        // F-005: the atomic temp+rename writer must not leave .tmp files on success.
        let store = ProjectLibraryStore::new(temp_root("atomic-write"));
        store
            .create_project(CreateProjectRequest {
                name: "Atomic".to_string(),
            })
            .expect("project created");

        fn collect_tmp(dir: &std::path::Path, out: &mut Vec<PathBuf>) {
            if let Ok(entries) = fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_dir() {
                        collect_tmp(&path, out);
                    } else if path.extension().is_some_and(|ext| ext == "tmp") {
                        out.push(path);
                    }
                }
            }
        }
        let mut leftovers = vec![];
        collect_tmp(store.root(), &mut leftovers);
        assert!(
            leftovers.is_empty(),
            "no .tmp files should remain after atomic writes: {leftovers:?}"
        );
    }

    #[test]
    fn wav_parser_rejects_truncated_fmt_chunk_without_panicking() {
        // F-016: a fmt chunk that declares a 16-byte body but ends early must return
        // InvalidData, not panic on an out-of-range slice.
        let dir = temp_root("wav-malformed");
        fs::create_dir_all(&dir).expect("create dir");
        let path = dir.join("bad.wav");
        let mut bytes = Vec::new();
        bytes.extend_from_slice(b"RIFF");
        bytes.extend_from_slice(&36u32.to_le_bytes());
        bytes.extend_from_slice(b"WAVE");
        // A preceding JUNK chunk advances the cursor so the fmt chunk lands near EOF.
        bytes.extend_from_slice(b"JUNK");
        bytes.extend_from_slice(&12u32.to_le_bytes());
        bytes.extend_from_slice(&[0u8; 12]);
        // fmt chunk declares size 16 but only 4 body bytes are present before EOF.
        bytes.extend_from_slice(b"fmt ");
        bytes.extend_from_slice(&16u32.to_le_bytes());
        bytes.extend_from_slice(&[1u8, 0, 1, 0]);
        assert_eq!(bytes.len(), 44);
        fs::write(&path, &bytes).expect("write malformed wav");

        let result = super::read_pcm16_wav(&path);
        assert!(
            result.is_err(),
            "a truncated fmt chunk must error, not panic"
        );
    }

    #[test]
    fn library_mutation_rejects_traversal_item_id() {
        // F-001: a caller-supplied item_id that escapes the store root must be
        // rejected before any filesystem join, not fabricated into a write target.
        let store = ProjectLibraryStore::new(temp_root("traversal"));
        for malicious in [
            "../../etc/passwd",
            "..",
            "assets/../escape",
            "with space",
            "",
        ] {
            let result = store.mutate_library_item(LibraryMutationRequest {
                item_id: malicious.to_string(),
                action: LibraryMutationAction::Favorite,
                tag: None,
            });
            assert!(
                result.is_err(),
                "mutate_library_item must reject item_id {malicious:?}"
            );
        }
    }

    #[test]
    fn runtime_audio_artifact_is_copied_into_project_library() {
        let store = ProjectLibraryStore::new(temp_root("import"));
        let project = store
            .create_project(CreateProjectRequest {
                name: "Runtime Import".to_string(),
            })
            .expect("project created");
        let runtime_root = temp_root("runtime");
        let runtime_store = RuntimeJobStore::new(&runtime_root);
        let job_id = seed_runtime_job(&runtime_root);

        let result = store
            .import_runtime_artifact_from_store(
                ImportRuntimeArtifactRequest {
                    job_id,
                    project_id: Some(project.workspace.active_project.project.id.clone()),
                    name: Some("Persisted smoke clip".to_string()),
                    scope: None,
                    tags: vec!["smoke".to_string()],
                },
                &runtime_store,
            )
            .expect("runtime artifact imported");

        assert_eq!(
            result
                .asset_library
                .selected_item
                .as_ref()
                .unwrap()
                .item
                .name,
            "Persisted smoke clip"
        );
        let playback = store
            .playback_for_item(&result.asset_library.selected_item.as_ref().unwrap().item.id)
            .expect("playback state loads");
        assert!(playback.playable);
        assert!(PathBuf::from(playback.path.expect("playback path")).is_file());
    }

    #[test]
    fn library_mutations_persist_lifecycle_and_tags() {
        let store = ProjectLibraryStore::new(temp_root("mutation"));
        let runtime_root = temp_root("mutation-runtime");
        let runtime_store = RuntimeJobStore::new(&runtime_root);
        let job_id = seed_runtime_job(&runtime_root);
        let imported = store
            .import_runtime_artifact_from_store(
                ImportRuntimeArtifactRequest {
                    job_id,
                    project_id: None,
                    name: None,
                    scope: None,
                    tags: vec![],
                },
                &runtime_store,
            )
            .expect("runtime artifact imported");
        let item_id = imported
            .asset_library
            .selected_item
            .as_ref()
            .unwrap()
            .item
            .id
            .clone();

        store
            .mutate_library_item(LibraryMutationRequest {
                item_id: item_id.clone(),
                action: LibraryMutationAction::AddTag,
                tag: Some("keeper".to_string()),
            })
            .expect("tag added");
        let promoted = store
            .mutate_library_item(LibraryMutationRequest {
                item_id: item_id.clone(),
                action: LibraryMutationAction::PromoteToGlobal,
                tag: None,
            })
            .expect("promoted");

        assert!(promoted
            .asset_library
            .selected_item
            .as_ref()
            .unwrap()
            .item
            .tags
            .iter()
            .any(|tag| tag == "keeper"));
        assert!(promoted
            .asset_library
            .selected_item
            .as_ref()
            .unwrap()
            .item
            .project_id
            .is_none());
    }

    #[test]
    fn review_edit_writes_non_destructive_audio_version_and_sidecar() {
        let store = ProjectLibraryStore::new(temp_root("review-edit"));
        let runtime_root = temp_root("review-runtime");
        let runtime_store = RuntimeJobStore::new(&runtime_root);
        let job_id = seed_runtime_job(&runtime_root);
        let imported = store
            .import_runtime_artifact_from_store(
                ImportRuntimeArtifactRequest {
                    job_id,
                    project_id: None,
                    name: Some("Editable runtime clip".to_string()),
                    scope: None,
                    tags: vec!["review".to_string()],
                },
                &runtime_store,
            )
            .expect("runtime artifact imported");
        let item_id = imported
            .asset_library
            .selected_item
            .as_ref()
            .unwrap()
            .item
            .id
            .clone();

        let edit = store
            .save_review_edit(SaveReviewEditRequest {
                item_id: item_id.clone(),
                start_ms: Some(100),
                end_ms: Some(650),
                fade_in_ms: Some(25),
                fade_out_ms: Some(50),
                normalize_loudness_lufs: Some(-18.0),
            })
            .expect("review edit saved");

        assert_eq!(
            edit.library.selected_item.as_ref().unwrap().item.id,
            item_id
        );
        assert!(PathBuf::from(&edit.source_path).is_file());
        assert!(PathBuf::from(&edit.edited_path).is_file());
        assert!(PathBuf::from(&edit.provenance_sidecar_path).is_file());
        assert!(edit.duration_ms <= 560);
        assert_ne!(
            edit.source_path, edit.edited_path,
            "review edit must not overwrite the source audio"
        );
        assert_eq!(
            edit.library
                .selected_item
                .as_ref()
                .unwrap()
                .item
                .current_version
                .as_ref()
                .expect("edited current version")
                .version_index,
            2
        );
    }

    #[test]
    fn export_writes_real_wav_sidecar_and_sceneworks_manifest() {
        let store = ProjectLibraryStore::new(temp_root("export"));
        let runtime_root = temp_root("export-runtime");
        let runtime_store = RuntimeJobStore::new(&runtime_root);
        let job_id = seed_runtime_job(&runtime_root);
        let imported = store
            .import_runtime_artifact_from_store(
                ImportRuntimeArtifactRequest {
                    job_id,
                    project_id: None,
                    name: Some("Exportable runtime clip".to_string()),
                    scope: None,
                    tags: vec!["export".to_string()],
                },
                &runtime_store,
            )
            .expect("runtime artifact imported");

        let export = store
            .export_library_item(ExportLibraryItemRequest {
                item_id: imported
                    .asset_library
                    .selected_item
                    .as_ref()
                    .unwrap()
                    .item
                    .id
                    .clone(),
                preset_id: "preset-sceneworks-video-track".to_string(),
                formats: vec![AudioFileFormat::Wav, AudioFileFormat::Mp3],
                scene_works_project_id: Some("scene-project".to_string()),
                scene_works_video_asset_id: Some("video-asset".to_string()),
                replace_existing_audio: true,
            })
            .expect("export written");

        assert!(PathBuf::from(&export.output_root).is_dir());
        assert!(PathBuf::from(&export.sidecar_path).is_file());
        assert!(PathBuf::from(&export.scene_works_manifest_path).is_file());
        assert!(export.artifacts.iter().any(|artifact| {
            artifact.kind == "audio-file"
                && artifact.format == Some(AudioFileFormat::Wav)
                && artifact.bytes > 44
                && PathBuf::from(&artifact.path).is_file()
        }));
        assert!(export
            .warnings
            .iter()
            .any(|warning| warning.contains("MP3 export is blocked")));
        assert!(!export.can_attach_directly);
        assert!(export
            .validation_checks
            .iter()
            .any(|check| { check.id == "sceneworks.package_manifest" && check.passed }));
    }

    fn seed_runtime_job(root: &PathBuf) -> String {
        let job_id = "job-test-runtime".to_string();
        let job_root = root.join("jobs").join(&job_id);
        let artifact_path = job_root.join("artifacts").join("runtime-smoke.wav");
        fs::create_dir_all(artifact_path.parent().expect("artifact parent")).unwrap();
        write_test_wav(&artifact_path);
        let job = RuntimeJobSnapshot {
            id: job_id.clone(),
            kind: JobKind::GenerateAudio,
            status: JobStatus::Succeeded,
            provider_id: "soundworks-native".to_string(),
            model_id: "kokoro-82m".to_string(),
            workflow: CapabilityWorkflow::Tts,
            adapter: ProviderAdapterKind::NativeRust,
            progress: Some(JobProgress {
                percent: 100.0,
                message: Some("test".to_string()),
            }),
            cancellation: CancellationState::Completed,
            retry_count: 0,
            created_at: "1".to_string(),
            updated_at: "1".to_string(),
            record_root: format!("jobs/{job_id}"),
            log_tail: vec![],
            artifacts: vec![RuntimeJobArtifact {
                kind: RuntimeArtifactKind::AudioPreview,
                path: artifact_path.display().to_string(),
                mime_type: "audio/wav".to_string(),
                bytes: fs::metadata(&artifact_path).unwrap().len(),
                summary: "test audio".to_string(),
            }],
            actionable_error: None,
        };
        fs::write(
            job_root.join("job.json"),
            serde_json::to_vec_pretty(&job).unwrap(),
        )
        .unwrap();
        job_id
    }

    fn temp_root(label: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!(
            "soundworks-project-library-{label}-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&root);
        root
    }

    fn write_test_wav(path: &PathBuf) {
        let sample_rate = 16_000u32;
        let samples = sample_rate;
        let data_bytes = samples * 2;
        let mut bytes = Vec::with_capacity(44 + data_bytes as usize);
        bytes.extend_from_slice(b"RIFF");
        bytes.extend_from_slice(&(36 + data_bytes).to_le_bytes());
        bytes.extend_from_slice(b"WAVEfmt ");
        bytes.extend_from_slice(&16u32.to_le_bytes());
        bytes.extend_from_slice(&1u16.to_le_bytes());
        bytes.extend_from_slice(&1u16.to_le_bytes());
        bytes.extend_from_slice(&sample_rate.to_le_bytes());
        bytes.extend_from_slice(&(sample_rate * 2).to_le_bytes());
        bytes.extend_from_slice(&2u16.to_le_bytes());
        bytes.extend_from_slice(&16u16.to_le_bytes());
        bytes.extend_from_slice(b"data");
        bytes.extend_from_slice(&data_bytes.to_le_bytes());
        for index in 0..samples {
            let phase = index as f32 / sample_rate as f32 * 440.0 * std::f32::consts::TAU;
            let sample = (phase.sin() * i16::MAX as f32 * 0.3) as i16;
            bytes.extend_from_slice(&sample.to_le_bytes());
        }
        fs::write(path, bytes).unwrap();
    }
}
