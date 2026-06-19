use crate::asset_library::{
    AssetLibraryOverview, LibraryItemCard, LibraryItemDetail, LibraryItemType, LibraryOwnership,
    QuickAuditionState, WaveformThumbnail,
};
use crate::domain::{
    AssetCreation, AudioAsset, AudioAssetKind, AudioAssetVersion, AudioFileFormat,
    AudioFileReference, CommercialUseStatus, JobStatus, LibraryScope, LicenseStatus, Project,
    RecipeSummary, RecipeWorkflow, RightsMetadata, TechnicalAudioMetadata, VoiceConsentStatus,
    WatermarkStatus, Workspace,
};
use crate::manifests::CapabilityWorkflow;
use crate::runtime::{RuntimeArtifactKind, RuntimeJobStore};
use crate::workspace::WorkspaceOverview;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
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
    pub selected_item: LibraryItemDetail,
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
        let library = self.asset_library_overview(None)?;

        Ok(WorkspaceOverview::from_library(
            state.workspace,
            active_project,
            recent_projects,
            &library,
        ))
    }

    pub fn asset_library_overview(
        &self,
        selected_item_id: Option<&str>,
    ) -> io::Result<AssetLibraryOverview> {
        AssetLibraryOverview::reference_with_persisted_items(
            self.read_asset_records()?
                .into_iter()
                .map(|record| record.item)
                .collect(),
            selected_item_id,
        )
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error.to_string()))
    }

    pub fn create_project(
        &self,
        request: CreateProjectRequest,
    ) -> io::Result<ProjectLibraryActionResult> {
        let name = non_empty_or(request.name, "Untitled SoundWorks Project");
        let project_id = format!("project-{}-{}", slug(&name), timestamp_millis());
        let project_root = self.root.join("projects").join(&project_id);
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
            "asset-{}-{}",
            workflow_fragment(job.workflow),
            timestamp_millis()
        );
        let version_id = format!("version-{}-a", asset_id.trim_start_matches("asset-"));
        let asset_root = self.asset_version_root(&scope, kind, &asset_id, &version_id);
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
                loudness_lufs: None,
                true_peak_dbfs: None,
                has_clipping: false,
                bpm: None,
                musical_key: None,
                loop_points: None,
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
                "runtimeRecipePath": job.recipe_path,
                "runtimeModelMetadataPath": job.model_metadata_path,
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
                        .asset_record_path(&request.item_id)
                        .display()
                        .to_string(),
                    provenance_sidecar_path: self
                        .root
                        .join("assets")
                        .join(&request.item_id)
                        .join("recipe-provenance.json")
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
        write_json(&record.metadata_sidecar_path, &record)?;
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

    fn action_result(
        &self,
        selected_item_id: Option<String>,
        message: String,
    ) -> io::Result<ProjectLibraryActionResult> {
        let asset_library = self.asset_library_overview(selected_item_id.as_deref())?;
        let workspace = self.workspace_overview()?;
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
        let projects_root = self.root.join("projects");
        let mut projects = vec![reference_project()];
        if !projects_root.exists() {
            return Ok(projects);
        }
        for entry in fs::read_dir(projects_root)? {
            let path = entry?.path().join("project.json");
            if path.is_file() {
                let project: Project = read_json(path)?;
                if !projects.iter().any(|existing| existing.id == project.id) {
                    projects.push(project);
                }
            }
        }
        Ok(projects)
    }

    fn write_project(&self, project: &Project) -> io::Result<()> {
        let path = self
            .root
            .join("projects")
            .join(&project.id)
            .join("project.json");
        write_json(path, project)
    }

    fn add_asset_to_project(&self, project_id: &str, asset_id: &str) -> io::Result<()> {
        let mut projects = self.read_projects()?;
        if let Some(project) = projects.iter_mut().find(|project| project.id == project_id) {
            if !project.asset_ids.iter().any(|id| id == asset_id) {
                project.asset_ids.push(asset_id.to_string());
            }
            if project.id != "project-demo" {
                self.write_project(project)?;
            }
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
        let path = self.asset_record_path(item_id);
        if path.is_file() {
            Ok(Some(read_json(path)?))
        } else {
            Ok(None)
        }
    }

    fn write_asset_record(&self, record: &PersistedAssetRecord) -> io::Result<()> {
        write_json(self.asset_record_path(&record.item.id), record)
    }

    fn asset_record_path(&self, item_id: &str) -> PathBuf {
        self.root
            .join("assets")
            .join(item_id)
            .join("asset-record.json")
    }

    fn asset_version_root(
        &self,
        scope: &LibraryScope,
        kind: AudioAssetKind,
        asset_id: &str,
        version_id: &str,
    ) -> PathBuf {
        let scope_path = match scope {
            LibraryScope::GlobalLibrary => self.root.join("global"),
            LibraryScope::Project { project_id } => self.root.join("projects").join(project_id),
        };
        scope_path
            .join(kind.storage_dir())
            .join(asset_id)
            .join(version_id)
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

fn timestamp_string() -> String {
    timestamp_millis().to_string()
}

fn write_json(path: impl AsRef<Path>, value: &impl Serialize) -> io::Result<()> {
    if let Some(parent) = path.as_ref().parent() {
        fs::create_dir_all(parent)?;
    }
    let payload = serde_json::to_vec_pretty(value)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    fs::write(path, payload)
}

fn read_json<T: DeserializeOwned>(path: impl AsRef<Path>) -> io::Result<T> {
    let payload = fs::read(path)?;
    serde_json::from_slice(&payload)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))
}

#[cfg(test)]
mod tests {
    use super::{
        CreateProjectRequest, ImportRuntimeArtifactRequest, LibraryMutationAction,
        LibraryMutationRequest, ProjectLibraryStore,
    };
    use crate::domain::{JobKind, JobProgress, JobStatus};
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
            result.asset_library.selected_item.item.name,
            "Persisted smoke clip"
        );
        let playback = store
            .playback_for_item(&result.asset_library.selected_item.item.id)
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
        let item_id = imported.asset_library.selected_item.item.id.clone();

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
            .item
            .tags
            .iter()
            .any(|tag| tag == "keeper"));
        assert!(promoted
            .asset_library
            .selected_item
            .item
            .project_id
            .is_none());
    }

    fn seed_runtime_job(root: &PathBuf) -> String {
        let job_id = "job-test-runtime".to_string();
        let job_root = root.join("jobs").join(&job_id);
        let artifact_path = job_root.join("artifacts").join("runtime-smoke.wav");
        fs::create_dir_all(artifact_path.parent().expect("artifact parent")).unwrap();
        fs::write(&artifact_path, b"RIFFtestWAVEfmt ").unwrap();
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
            record_root: job_root.display().to_string(),
            recipe_path: job_root.join("recipe.json").display().to_string(),
            model_metadata_path: job_root.join("model.json").display().to_string(),
            events_path: job_root.join("events.jsonl").display().to_string(),
            log_tail: vec![],
            artifacts: vec![RuntimeJobArtifact {
                kind: RuntimeArtifactKind::AudioPreview,
                path: artifact_path.display().to_string(),
                mime_type: "audio/wav".to_string(),
                bytes: 16,
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
}
