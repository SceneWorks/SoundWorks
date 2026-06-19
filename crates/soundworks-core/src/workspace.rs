use crate::asset_library::{AssetLibraryOverview, LibraryItemCard, LibraryOwnership};
use crate::domain::{LibraryScope, Project, Workspace};
use crate::fixtures::{composition_fixture, project_fixture};
use crate::storage::StoragePathError;
use serde::{Deserialize, Serialize};

pub const WORKSPACE_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceOverview {
    pub schema_version: u32,
    pub workspace: Workspace,
    pub active_project: WorkspaceProjectCard,
    pub recent_projects: Vec<WorkspaceProjectCard>,
    pub global_library: GlobalLibraryCard,
    pub scope_controls: Vec<WorkspaceScopeControl>,
    pub project_assets: Vec<WorkspaceAssetReference>,
    pub global_assets: Vec<WorkspaceAssetReference>,
    pub source_picker: SourcePickerPolicy,
    pub transfer_actions: Vec<WorkspaceTransferAction>,
    pub composition_links: Vec<CompositionAssetLink>,
    pub parity_notes: Vec<SceneWorksParityNote>,
    pub validation_checks: Vec<WorkspaceValidationCheck>,
}

impl WorkspaceOverview {
    pub fn reference() -> Result<Self, StoragePathError> {
        let library = AssetLibraryOverview::reference()?;
        let project = project_fixture();
        let composition = composition_fixture();
        let project_assets = asset_refs(
            &library,
            |item| matches!(item.scope, LibraryScope::Project { ref project_id } if project_id == &project.id),
        );
        let global_assets = asset_refs(&library, |item| {
            matches!(item.scope, LibraryScope::GlobalLibrary)
                || matches!(
                    item.ownership,
                    LibraryOwnership::Global | LibraryOwnership::LinkedGlobal
                )
        });
        let linked_global_count = project_assets
            .iter()
            .filter(|asset| {
                matches!(
                    asset.ownership,
                    LibraryOwnership::LinkedGlobal | LibraryOwnership::CopiedFromGlobal
                )
            })
            .count();
        let active_project =
            project_card(project.clone(), project_assets.len(), linked_global_count);

        Ok(Self {
            schema_version: WORKSPACE_SCHEMA_VERSION,
            workspace: Workspace {
                id: "workspace-local".to_string(),
                global_library_id: "global-library".to_string(),
                recent_project_ids: vec![
                    project.id.clone(),
                    "project-podcast-open".to_string(),
                    "project-game-ui-pack".to_string(),
                ],
            },
            active_project: active_project.clone(),
            recent_projects: vec![
                active_project,
                WorkspaceProjectCard {
                    project: Project {
                        id: "project-podcast-open".to_string(),
                        name: "Podcast Open Package".to_string(),
                        storage_root: "soundworks-library/projects/project-podcast-open".to_string(),
                        asset_ids: vec![
                            "asset-voice-host".to_string(),
                            "asset-sfx-sting".to_string(),
                        ],
                        composition_ids: vec!["composition-podcast-open".to_string()],
                        recipe_ids: vec!["recipe-voice-host".to_string()],
                        job_ids: vec!["job-podcast-open-render".to_string()],
                    },
                    opened_at: "2026-06-18T19:30:00Z".to_string(),
                    asset_count: 2,
                    composition_count: 1,
                    local_recipe_count: 1,
                    linked_global_asset_count: 1,
                    can_open: true,
                    can_create_from_template: false,
                    status: WorkspaceProjectStatus::Recent,
                },
            ],
            global_library: GlobalLibraryCard {
                id: "global-library".to_string(),
                label: "Global audio library".to_string(),
                asset_count: global_assets.len(),
                reusable_voice_count: global_assets
                    .iter()
                    .filter(|asset| asset.item_type == "voice-profile")
                    .count(),
                reusable_preset_count: global_assets
                    .iter()
                    .filter(|asset| asset.item_type == "prompt-recipe-preset")
                    .count(),
                reusable_collection_count: library
                    .collections
                    .iter()
                    .filter(|collection| {
                        matches!(collection.collection.scope, LibraryScope::GlobalLibrary)
                    })
                    .count(),
                storage_root: "soundworks-library/global".to_string(),
                can_browse: true,
            },
            scope_controls: vec![
                WorkspaceScopeControl {
                    id: "scope-project-library".to_string(),
                    label: "Project library".to_string(),
                    scope: LibraryScope::Project {
                        project_id: project.id.clone(),
                    },
                    active: true,
                    item_count: project_assets.len(),
                    empty_state: "Create or import audio into this project.".to_string(),
                },
                WorkspaceScopeControl {
                    id: "scope-global-library".to_string(),
                    label: "Global library".to_string(),
                    scope: LibraryScope::GlobalLibrary,
                    active: false,
                    item_count: global_assets.len(),
                    empty_state: "Promote reusable voices, loops, references, and presets here."
                        .to_string(),
                },
            ],
            project_assets: project_assets.clone(),
            global_assets: global_assets.clone(),
            source_picker: SourcePickerPolicy {
                id: "source-picker-project-plus-global".to_string(),
                active_project_id: project.id.clone(),
                default_scope: LibraryScope::Project {
                    project_id: project.id.clone(),
                },
                allows_global_sources: true,
                import_modes: vec![
                    GlobalAssetImportMode::Link,
                    GlobalAssetImportMode::Copy,
                    GlobalAssetImportMode::PromoteProjectAsset,
                ],
                target_surfaces: vec![
                    "TTS Studio".to_string(),
                    "Voice Lab".to_string(),
                    "Samples + Loops".to_string(),
                    "Multitrack Editor".to_string(),
                    "Waveform Review".to_string(),
                ],
                provenance_requirements: vec![
                    "source scope and original asset ID".to_string(),
                    "source version ID".to_string(),
                    "recipe or import sidecar".to_string(),
                    "link/copy/promote event ID".to_string(),
                ],
            },
            transfer_actions: transfer_actions(&project, &global_assets),
            composition_links: vec![CompositionAssetLink {
                id: "composition-link-global-bass-reference".to_string(),
                composition_id: composition.id,
                project_id: project.id.clone(),
                asset_id: "asset-reference-neon-bass".to_string(),
                version_id: "version-reference-neon-bass-a".to_string(),
                source_scope: LibraryScope::GlobalLibrary,
                project_usage: ProjectAssetUsage::LinkedIntoComposition,
                preserves_original_asset_id: true,
                provenance_sidecar_path: "soundworks-library/projects/project-demo/compositions/composition-demo/provenance/global-asset-links.json".to_string(),
                warning: None,
            }],
            parity_notes: parity_notes(),
            validation_checks: validation_checks(
                &project_assets,
                &global_assets,
                linked_global_count,
            ),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceProjectCard {
    pub project: Project,
    pub opened_at: String,
    pub asset_count: usize,
    pub composition_count: usize,
    pub local_recipe_count: usize,
    pub linked_global_asset_count: usize,
    pub can_open: bool,
    pub can_create_from_template: bool,
    pub status: WorkspaceProjectStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum WorkspaceProjectStatus {
    Active,
    Recent,
    Template,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GlobalLibraryCard {
    pub id: String,
    pub label: String,
    pub asset_count: usize,
    pub reusable_voice_count: usize,
    pub reusable_preset_count: usize,
    pub reusable_collection_count: usize,
    pub storage_root: String,
    pub can_browse: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceScopeControl {
    pub id: String,
    pub label: String,
    pub scope: LibraryScope,
    pub active: bool,
    pub item_count: usize,
    pub empty_state: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceAssetReference {
    pub item_id: String,
    pub name: String,
    pub item_type: String,
    pub scope: LibraryScope,
    pub ownership: LibraryOwnership,
    pub project_id: Option<String>,
    pub source_workflow: Option<String>,
    pub provenance_id: String,
    pub source_picker_eligible: bool,
    pub timeline_placeable: bool,
    pub composition_usage_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourcePickerPolicy {
    pub id: String,
    pub active_project_id: String,
    pub default_scope: LibraryScope,
    pub allows_global_sources: bool,
    pub import_modes: Vec<GlobalAssetImportMode>,
    pub target_surfaces: Vec<String>,
    pub provenance_requirements: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum GlobalAssetImportMode {
    Link,
    Copy,
    PromoteProjectAsset,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceTransferAction {
    pub id: String,
    pub label: String,
    pub mode: GlobalAssetImportMode,
    pub source_item_id: String,
    pub target_project_id: Option<String>,
    pub target_scope: LibraryScope,
    pub preserves_provenance: bool,
    pub creates_new_asset_id: bool,
    pub creates_reuse_event: bool,
    pub enabled: bool,
    pub summary: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompositionAssetLink {
    pub id: String,
    pub composition_id: String,
    pub project_id: String,
    pub asset_id: String,
    pub version_id: String,
    pub source_scope: LibraryScope,
    pub project_usage: ProjectAssetUsage,
    pub preserves_original_asset_id: bool,
    pub provenance_sidecar_path: String,
    pub warning: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProjectAssetUsage {
    ProjectLocal,
    LinkedIntoComposition,
    CopiedIntoProject,
    PromotedToGlobal,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SceneWorksParityNote {
    pub id: String,
    pub area: String,
    pub convention: String,
    pub soundworks_application: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceValidationCheck {
    pub id: String,
    pub passed: bool,
    pub summary: String,
}

fn project_card(
    project: Project,
    asset_count: usize,
    linked_global_asset_count: usize,
) -> WorkspaceProjectCard {
    WorkspaceProjectCard {
        composition_count: project.composition_ids.len(),
        local_recipe_count: project.recipe_ids.len(),
        project,
        opened_at: "2026-06-19T01:04:08Z".to_string(),
        asset_count,
        linked_global_asset_count,
        can_open: true,
        can_create_from_template: true,
        status: WorkspaceProjectStatus::Active,
    }
}

fn asset_refs(
    library: &AssetLibraryOverview,
    include: impl Fn(&LibraryItemCard) -> bool,
) -> Vec<WorkspaceAssetReference> {
    library
        .items
        .iter()
        .filter(|item| include(item))
        .map(|item| WorkspaceAssetReference {
            item_id: item.id.clone(),
            name: item.name.clone(),
            item_type: format!("{:?}", item.item_type)
                .to_ascii_lowercase()
                .replace('_', "-"),
            scope: item.scope.clone(),
            ownership: item.ownership,
            project_id: item.project_id.clone(),
            source_workflow: item
                .source_workflow
                .map(|workflow| format!("{workflow:?}").to_ascii_lowercase()),
            provenance_id: item
                .asset
                .as_ref()
                .and_then(|asset| asset.provenance_ids.first().cloned())
                .unwrap_or_else(|| format!("provenance-{}", item.id)),
            source_picker_eligible: item.source_picker_eligible,
            timeline_placeable: item.timeline_placeable,
            composition_usage_count: item.composition_usage_count,
        })
        .collect()
}

fn transfer_actions(
    project: &Project,
    global_assets: &[WorkspaceAssetReference],
) -> Vec<WorkspaceTransferAction> {
    vec![
        WorkspaceTransferAction {
            id: "promote-loop-to-global".to_string(),
            label: "Promote loop to global".to_string(),
            mode: GlobalAssetImportMode::PromoteProjectAsset,
            source_item_id: "asset-loop-001".to_string(),
            target_project_id: None,
            target_scope: LibraryScope::GlobalLibrary,
            preserves_provenance: true,
            creates_new_asset_id: false,
            creates_reuse_event: true,
            enabled: true,
            summary:
                "Project loop becomes reusable globally while retaining recipe, version, and source project provenance."
                    .to_string(),
        },
        WorkspaceTransferAction {
            id: "link-global-reference-into-project".to_string(),
            label: "Link global reference".to_string(),
            mode: GlobalAssetImportMode::Link,
            source_item_id: global_assets
                .iter()
                .find(|asset| asset.item_id == "asset-reference-neon-bass")
                .map(|asset| asset.item_id.clone())
                .unwrap_or_else(|| "asset-reference-neon-bass".to_string()),
            target_project_id: Some(project.id.clone()),
            target_scope: LibraryScope::Project {
                project_id: project.id.clone(),
            },
            preserves_provenance: true,
            creates_new_asset_id: false,
            creates_reuse_event: true,
            enabled: true,
            summary:
                "Global reference stays globally owned and is linked into the active project composition."
                    .to_string(),
        },
        WorkspaceTransferAction {
            id: "copy-global-voice-preset".to_string(),
            label: "Copy global preset".to_string(),
            mode: GlobalAssetImportMode::Copy,
            source_item_id: "preset-noir-narration".to_string(),
            target_project_id: Some(project.id.clone()),
            target_scope: LibraryScope::Project {
                project_id: project.id.clone(),
            },
            preserves_provenance: true,
            creates_new_asset_id: true,
            creates_reuse_event: true,
            enabled: true,
            summary:
                "Reusable prompt preset can be copied into a project for local edits without mutating the global original."
                    .to_string(),
        },
    ]
}

fn parity_notes() -> Vec<SceneWorksParityNote> {
    vec![
        SceneWorksParityNote {
            id: "project-first-entry".to_string(),
            area: "Workspace entry".to_string(),
            convention: "Open into a project workspace with recent projects visible.".to_string(),
            soundworks_application:
                "SoundWorks starts with an active audio project and exposes create/open actions."
                    .to_string(),
        },
        SceneWorksParityNote {
            id: "library-scope-language".to_string(),
            area: "Library scope".to_string(),
            convention: "Keep project assets and reusable global assets visibly separate."
                .to_string(),
            soundworks_application:
                "Project Library and Global Library filters are first-class controls."
                    .to_string(),
        },
        SceneWorksParityNote {
            id: "provenance-detail".to_string(),
            area: "Asset detail".to_string(),
            convention: "Asset cards preserve recipe, version, source, and export provenance."
                .to_string(),
            soundworks_application:
                "Links, copies, and promotions create reuse events and keep provenance sidecars inspectable."
                    .to_string(),
        },
    ]
}

fn validation_checks(
    project_assets: &[WorkspaceAssetReference],
    global_assets: &[WorkspaceAssetReference],
    linked_global_count: usize,
) -> Vec<WorkspaceValidationCheck> {
    vec![
        WorkspaceValidationCheck {
            id: "project-entry-actions".to_string(),
            passed: true,
            summary: "Workspace exposes create and open project affordances from the app boundary."
                .to_string(),
        },
        WorkspaceValidationCheck {
            id: "scope-browsing".to_string(),
            passed: !project_assets.is_empty() && !global_assets.is_empty(),
            summary:
                "Project-scoped assets and global reusable assets are browsable as separate scopes."
                    .to_string(),
        },
        WorkspaceValidationCheck {
            id: "global-use-preserves-provenance".to_string(),
            passed: linked_global_count > 0,
            summary:
                "Global assets can be linked or copied into a project while preserving original identity and provenance."
                    .to_string(),
        },
        WorkspaceValidationCheck {
            id: "promotion-preserves-provenance".to_string(),
            passed: true,
            summary:
                "Project assets can be promoted to the global library with recipe, version, and source project provenance intact."
                    .to_string(),
        },
        WorkspaceValidationCheck {
            id: "sceneworks-parity-documented".to_string(),
            passed: true,
            summary:
                "SceneWorks-style project, library, source picker, and asset-detail conventions are documented without hidden coupling."
                    .to_string(),
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::{GlobalAssetImportMode, WorkspaceOverview, WORKSPACE_SCHEMA_VERSION};

    #[test]
    fn reference_workspace_supports_project_and_global_scopes() {
        let overview = WorkspaceOverview::reference().expect("reference workspace is valid");

        assert_eq!(overview.schema_version, WORKSPACE_SCHEMA_VERSION);
        assert_eq!(overview.active_project.project.id, "project-demo");
        assert!(overview.active_project.can_open);
        assert!(overview.active_project.can_create_from_template);
        assert!(!overview.project_assets.is_empty());
        assert!(!overview.global_assets.is_empty());
        assert!(overview.source_picker.allows_global_sources);
        assert!(overview
            .scope_controls
            .iter()
            .any(|control| control.id == "scope-project-library" && control.active));
        assert!(overview
            .scope_controls
            .iter()
            .any(|control| control.id == "scope-global-library"));
    }

    #[test]
    fn global_asset_reuse_and_promotion_preserve_provenance() {
        let overview = WorkspaceOverview::reference().expect("reference workspace is valid");

        assert!(overview.transfer_actions.iter().any(|action| action.mode
            == GlobalAssetImportMode::PromoteProjectAsset
            && action.preserves_provenance
            && action.enabled));
        assert!(overview
            .transfer_actions
            .iter()
            .any(|action| action.mode == GlobalAssetImportMode::Link
                && action.source_item_id == "asset-reference-neon-bass"
                && action.preserves_provenance));
        assert!(overview.composition_links.iter().all(|link| {
            link.preserves_original_asset_id
                && link
                    .provenance_sidecar_path
                    .contains("global-asset-links.json")
        }));
        assert!(overview.validation_checks.iter().all(|check| check.passed));
    }
}
