# Workspace And Global Library

`sc-6198` establishes SoundWorks as a project/workspace application with a reusable global asset library. The contract keeps SceneWorks-style project muscle memory while staying independent from SceneWorks internals.

Confidence: medium-high. The active project, global library, source picker, reuse actions, storage shape, and UI surface are represented. Real persistence services, file pickers, and cross-app import flows remain behind later service work.

## Contract

- `crates/soundworks-core/src/workspace.rs` defines `WorkspaceOverview`, active/recent project cards, global library state, scope controls, source picker policy, transfer actions, composition links, SceneWorks parity notes, and validation checks.
- The active workspace opens into `project-demo`, not a flat global editor.
- Project assets and global reusable assets are distinct scopes. The UI exposes Project Library and Global Library controls before the broader asset list.
- The global library is for reusable voices, references, loops, stems, music clips, presets, collections, and templates.
- Source pickers can use project-local assets and global assets. Link, copy, and promote flows are modeled separately.
- Composition links preserve original global asset IDs, version IDs, scope, usage, and provenance sidecar path.

## Reuse Semantics

`GlobalAssetImportMode` defines the three launch behaviors:

- `link`: use a global asset in a project without changing global ownership or creating a new asset ID.
- `copy`: create a project-local editable copy while retaining a source reuse event and provenance link to the global original.
- `promote-project-asset`: make a reusable project asset available globally while retaining source project, version, recipe, and provenance metadata.

All reuse actions must preserve provenance. The reference contract makes this testable through `WorkspaceTransferAction.preserves_provenance`, `creates_reuse_event`, and `CompositionAssetLink.provenance_sidecar_path`.

## App Boundary

The Tauri command `get_workspace_overview` exposes the workspace payload. The React workspace renders:

- active project identity and create/open project actions.
- project, project asset, global link, and global asset counts.
- recent project cards.
- global library card with reusable voice, preset, and collection counts.
- scope controls for project and global libraries.
- source picker targets and provenance requirements.
- link/copy/promote reuse actions.
- composition link provenance sidecar.
- SceneWorks parity notes and validation checks.

## Storage

Schema migration `workspace_global_library` adds:

- `workspace_records`
- `workspace_project_cards`
- `workspace_global_libraries`
- `workspace_scope_controls`
- `workspace_source_picker_policies`
- `workspace_transfer_actions`
- `workspace_composition_asset_links`
- `workspace_parity_notes`
- `workspace_validation_checks`

These tables keep nested source picker and scope payloads serialized while the concrete persistence service is still evolving.

## Validation

Rust tests verify:

- project and global scopes are both represented.
- active project create/open affordances are exposed.
- source picker policy allows global assets.
- link, copy, and promote actions preserve provenance.
- composition links retain original global asset identity and sidecar path.

Frontend tests verify the workspace panel renders create/open actions, project/global library scope, link/copy/promote actions, and global composition provenance.
