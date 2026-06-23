use crate::domain::{JobKind, JobProgress, JobStatus, ModelRuntime, VoiceConsentStatus};
use crate::evaluation::{EvaluationLane, ProductEligibility, ProductRuntimePath};
use crate::loudness;
use crate::manifests::{
    CapabilityWorkflow, DeviceAccelerator, ModelInstallStatus, ModelManifest, ProviderCatalog,
};
use crate::model_manager::{
    CandidateInstallState, ModelCandidateInstallState, ModelManagerOverview,
};
use crate::rights::{PolicyDecision, RightsSafetyOverview};
use crate::storage::sanitized_join;
use kokoro_en::{KokoroTts, Voice};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, HashMap};
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

pub const RUNTIME_SCHEMA_VERSION: u32 = 2;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeOverview {
    pub schema_version: u32,
    pub packaging_policy: RuntimePackagingPolicy,
    pub devices: Vec<DeviceReport>,
    pub status_counts: RuntimeStatusCounts,
    pub model_states: Vec<ModelRuntimeState>,
    pub jobs: Vec<RuntimeJobSnapshot>,
    pub validation_checks: Vec<RuntimeValidationCheck>,
}

impl RuntimeOverview {
    pub fn reference() -> Self {
        Self::from_catalog(
            &ProviderCatalog::reference(),
            &DeviceInventory::reference_mac(),
            RuntimePackagingPolicy::shipped_desktop(),
        )
    }

    pub fn from_catalog(
        catalog: &ProviderCatalog,
        inventory: &DeviceInventory,
        policy: RuntimePackagingPolicy,
    ) -> Self {
        let mut validation_checks = policy.validate_catalog(catalog);
        validation_checks.extend(inventory.validation_checks());

        let model_states: Vec<ModelRuntimeState> = catalog
            .providers
            .iter()
            .flat_map(|provider| {
                provider.models.iter().map(|model| {
                    ModelRuntimeState::from_manifest(&provider.id, model, inventory, &policy)
                })
            })
            .collect();

        Self {
            schema_version: RUNTIME_SCHEMA_VERSION,
            packaging_policy: policy,
            devices: inventory.devices.clone(),
            status_counts: RuntimeStatusCounts::from_model_states(&model_states),
            jobs: vec![],
            model_states,
            validation_checks,
        }
    }

    pub fn from_model_manager(
        manager: &ModelManagerOverview,
        inventory: &DeviceInventory,
        policy: RuntimePackagingPolicy,
        store: &RuntimeJobStore,
    ) -> Self {
        let mut validation_checks = policy.validate_catalog(&ProviderCatalog {
            schema_version: 1,
            providers: vec![],
        });
        validation_checks.extend(inventory.validation_checks());
        validation_checks.extend(store.validation_checks());

        let mut model_states: Vec<ModelRuntimeState> = manager
            .candidates
            .iter()
            .filter(|candidate| candidate.install_state == CandidateInstallState::Installed)
            .map(|candidate| ModelRuntimeState::from_candidate(candidate, inventory))
            .collect();
        // F-024: inject the always-available native models from the single
        // registry instead of repeating their ids inline.
        for native in NativeModel::injected() {
            let already_present = model_states.iter().any(|state| {
                state.provider_id == "soundworks-native" && state.model_id == native.model_id()
            });
            if !already_present {
                if let Some(state) = native.injected_state(inventory) {
                    model_states.push(state);
                }
            }
        }
        let mut jobs = store.read_jobs().unwrap_or_default();
        jobs.sort_by(|left, right| right.created_at.cmp(&left.created_at));

        Self {
            schema_version: RUNTIME_SCHEMA_VERSION,
            packaging_policy: policy,
            devices: inventory.devices.clone(),
            status_counts: RuntimeStatusCounts::from_model_states(&model_states),
            jobs,
            model_states,
            validation_checks,
        }
    }

    pub fn admit_job(
        &self,
        provider_id: &str,
        model_id: &str,
        kind: JobKind,
    ) -> RuntimeJobAdmission {
        match self
            .model_states
            .iter()
            .find(|state| state.provider_id == provider_id && state.model_id == model_id)
        {
            Some(state)
                if matches!(
                    state.availability,
                    RuntimeAvailability::Installed | RuntimeAvailability::Available
                ) =>
            {
                RuntimeJobAdmission {
                    accepted: true,
                    provider_id: provider_id.to_string(),
                    model_id: model_id.to_string(),
                    kind,
                    reason: "model runtime is available for queued execution".to_string(),
                    actionable_error: None,
                }
            }
            Some(state) => RuntimeJobAdmission {
                accepted: false,
                provider_id: provider_id.to_string(),
                model_id: model_id.to_string(),
                kind,
                reason: state
                    .reasons
                    .first()
                    .cloned()
                    .unwrap_or_else(|| "model runtime is unavailable".to_string()),
                actionable_error: Some(ActionableRuntimeError {
                    code: "runtime.unavailable".to_string(),
                    summary: format!("{} cannot accept jobs", state.model_name),
                    recovery: state
                        .reasons
                        .first()
                        .cloned()
                        .unwrap_or_else(|| "Review install state and device compatibility.".to_string()),
                }),
            },
            None => RuntimeJobAdmission {
                accepted: false,
                provider_id: provider_id.to_string(),
                model_id: model_id.to_string(),
                kind,
                reason: "provider/model pair is not registered in the runtime catalog".to_string(),
                actionable_error: Some(ActionableRuntimeError {
                    code: "runtime.model_not_found".to_string(),
                    summary: "Model is not registered".to_string(),
                    recovery:
                        "Refresh provider manifests or install a provider package that declares this model."
                            .to_string(),
                }),
            },
        }
    }

    pub fn cancel_job(&self, job_id: &str) -> Option<RuntimeJobSnapshot> {
        self.jobs.iter().find(|job| job.id == job_id).map(|job| {
            let mut cancelled = job.clone();
            if matches!(
                cancelled.cancellation,
                CancellationState::Cancellable | CancellationState::Requested
            ) {
                cancelled.status = JobStatus::Cancelled;
                cancelled.cancellation = CancellationState::Completed;
                cancelled.progress = Some(JobProgress {
                    percent: cancelled.progress.map_or(0.0, |progress| progress.percent),
                    message: Some("Cancellation acknowledged by worker boundary.".to_string()),
                });
            }
            cancelled
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimePackagingPolicy {
    pub name: String,
    pub product_runtime_allows_python: bool,
    pub shipped_platforms: Vec<DesktopPlatform>,
    pub allowed_product_runtimes: Vec<ProductRuntimeMode>,
    pub worker_process: WorkerProcessMode,
    pub dependency_isolation: DependencyIsolation,
    pub model_cache_roots: Vec<ModelCacheRoot>,
}

impl RuntimePackagingPolicy {
    pub fn shipped_desktop() -> Self {
        Self {
            name: "SoundWorks shipped desktop runtime".to_string(),
            product_runtime_allows_python: false,
            shipped_platforms: vec![DesktopPlatform::MacOs, DesktopPlatform::Windows],
            allowed_product_runtimes: vec![
                ProductRuntimeMode::RustNative,
                ProductRuntimeMode::NativeLibraryBinding,
                ProductRuntimeMode::ExternalExecutable,
                ProductRuntimeMode::ManagedApi,
            ],
            worker_process: WorkerProcessMode::AppManagedSidecar,
            dependency_isolation: DependencyIsolation {
                strategy: DependencyIsolationStrategy::PerProvider,
                cache_namespace: "soundworks/providers".to_string(),
                prevents_stack_poisoning: true,
            },
            model_cache_roots: vec![
                ModelCacheRoot {
                    platform: DesktopPlatform::MacOs,
                    path_hint: "~/Library/Application Support/SoundWorks/models".to_string(),
                    purpose: "macOS packaged and user-installed model cache".to_string(),
                },
                ModelCacheRoot {
                    platform: DesktopPlatform::Windows,
                    path_hint: "%APPDATA%\\SoundWorks\\models".to_string(),
                    purpose: "Windows packaged and user-installed model cache".to_string(),
                },
            ],
        }
    }

    pub fn validate_catalog(&self, catalog: &ProviderCatalog) -> Vec<RuntimeValidationCheck> {
        let mut checks = vec![
            RuntimeValidationCheck::passed(
                "runtime.platforms",
                "Packaging policy targets macOS and Windows desktop builds.",
            ),
            RuntimeValidationCheck::passed(
                "runtime.isolation",
                "Provider dependencies are isolated per provider package.",
            ),
        ];

        let python_blockers: Vec<String> = catalog
            .providers
            .iter()
            .flat_map(|provider| {
                provider.models.iter().filter_map(move |model| {
                    if !self.product_runtime_allows_python && model_requires_python(model) {
                        Some(format!("{}:{}", provider.id, model.id))
                    } else {
                        None
                    }
                })
            })
            .collect();

        if python_blockers.is_empty() {
            checks.push(RuntimeValidationCheck::passed(
                "runtime.no_python",
                "Product-enabled runtime manifests do not require Python.",
            ));
        } else {
            checks.push(RuntimeValidationCheck::failed(
                "runtime.no_python",
                format!(
                    "Python runtime dependencies are blocked for shipped builds: {}.",
                    python_blockers.join(", ")
                ),
                "Mark these models research-only/API-only or replace them with Rust, native-library, external-executable, or managed API providers.",
            ));
        }

        let manifest_only_models: Vec<String> = catalog
            .providers
            .iter()
            .flat_map(|provider| {
                provider.models.iter().filter_map(move |model| {
                    if matches!(
                        model.install.status,
                        ModelInstallStatus::Installed | ModelInstallStatus::Packaged
                    ) {
                        Some(format!("{}:{}", provider.id, model.id))
                    } else {
                        None
                    }
                })
            })
            .collect();

        if manifest_only_models.is_empty() {
            checks.push(RuntimeValidationCheck::passed(
                "runtime.cache_evidence",
                "Installed model state is derived from verified cache/package evidence.",
            ));
        } else {
            checks.push(RuntimeValidationCheck::failed(
                "runtime.cache_evidence",
                format!(
                    "Manifest-only packaged/install states cannot count as verified runtime installs: {}.",
                    manifest_only_models.join(", ")
                ),
                "Inspect the on-disk model cache/package and attach file evidence before marking models installed.",
            ));
        }

        checks
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DesktopPlatform {
    MacOs,
    Windows,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProductRuntimeMode {
    RustNative,
    NativeLibraryBinding,
    ExternalExecutable,
    ManagedApi,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum WorkerProcessMode {
    InProcess,
    AppManagedSidecar,
    ExternalService,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DependencyIsolation {
    pub strategy: DependencyIsolationStrategy,
    pub cache_namespace: String,
    pub prevents_stack_poisoning: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DependencyIsolationStrategy {
    Shared,
    PerProvider,
    PerModel,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelCacheRoot {
    pub platform: DesktopPlatform,
    pub path_hint: String,
    pub purpose: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceInventory {
    pub devices: Vec<DeviceReport>,
}

impl DeviceInventory {
    pub fn reference_mac() -> Self {
        Self {
            devices: vec![
                DeviceReport {
                    accelerator: DeviceAccelerator::Cpu,
                    name: "Apple Silicon CPU".to_string(),
                    memory_mb: Some(32_768),
                    available: true,
                    driver: None,
                },
                DeviceReport {
                    accelerator: DeviceAccelerator::Mps,
                    name: "Apple Metal Performance Shaders".to_string(),
                    memory_mb: Some(32_768),
                    available: true,
                    driver: Some("Metal".to_string()),
                },
            ],
        }
    }

    fn validation_checks(&self) -> Vec<RuntimeValidationCheck> {
        if self.devices.iter().any(|device| device.available) {
            vec![RuntimeValidationCheck::passed(
                "runtime.devices",
                "Runtime device inventory can report available accelerators.",
            )]
        } else {
            vec![RuntimeValidationCheck::warning(
                "runtime.devices",
                "No runtime accelerators are currently available.",
                "Keep models unavailable until CPU, GPU, or managed API capability is detected.",
            )]
        }
    }

    fn compatible_accelerator(&self, requirements: &[DeviceAccelerator]) -> Option<DeviceReport> {
        self.devices
            .iter()
            .find(|device| device.available && requirements.contains(&device.accelerator))
            .cloned()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceReport {
    pub accelerator: DeviceAccelerator,
    pub name: String,
    pub memory_mb: Option<u32>,
    pub available: bool,
    pub driver: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeStatusCounts {
    pub installed: usize,
    pub available: usize,
    pub unavailable: usize,
}

impl RuntimeStatusCounts {
    fn from_model_states(states: &[ModelRuntimeState]) -> Self {
        Self {
            installed: states
                .iter()
                .filter(|state| state.availability == RuntimeAvailability::Installed)
                .count(),
            available: states
                .iter()
                .filter(|state| state.availability == RuntimeAvailability::Available)
                .count(),
            unavailable: states
                .iter()
                .filter(|state| state.availability == RuntimeAvailability::Unavailable)
                .count(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelRuntimeState {
    pub provider_id: String,
    pub model_id: String,
    pub model_name: String,
    pub runtime: ModelRuntime,
    pub execution_strategy: ExecutionStrategy,
    pub workflows: Vec<CapabilityWorkflow>,
    pub availability: RuntimeAvailability,
    pub install_status: ModelInstallStatus,
    pub cache: ModelCacheState,
    pub compatibility: RuntimeCompatibility,
    pub health: RuntimeHealth,
    pub reasons: Vec<String>,
}

impl ModelRuntimeState {
    fn from_manifest(
        provider_id: &str,
        model: &ModelManifest,
        inventory: &DeviceInventory,
        policy: &RuntimePackagingPolicy,
    ) -> Self {
        let compatibility = RuntimeCompatibility::from_model(model, inventory);
        let cache = ModelCacheState::from_manifest(model);
        let mut reasons = vec![];

        if !policy.product_runtime_allows_python && model_requires_python(model) {
            reasons.push(
                "Python runtime dependency is not allowed in shipped SoundWorks builds."
                    .to_string(),
            );
        }

        if model.runtime == ModelRuntime::ResearchOnly {
            reasons.push("Research-only models cannot be product-enabled.".to_string());
        }

        if !compatibility.supported {
            reasons.extend(compatibility.reasons.clone());
        }

        if matches!(
            model.install.status,
            ModelInstallStatus::Installed | ModelInstallStatus::Packaged
        ) && !cache.verified
        {
            reasons.push(
                "Manifest declares a packaged or installed model, but no verified cache/package evidence is attached."
                    .to_string(),
            );
        }

        let availability =
            if !reasons.is_empty() || model.install.status == ModelInstallStatus::Unavailable {
                RuntimeAvailability::Unavailable
            } else {
                match model.install.status {
                    ModelInstallStatus::Installed | ModelInstallStatus::Packaged => {
                        RuntimeAvailability::Installed
                    }
                    ModelInstallStatus::Installable | ModelInstallStatus::External => {
                        RuntimeAvailability::Available
                    }
                    ModelInstallStatus::Unavailable => RuntimeAvailability::Unavailable,
                }
            };

        let health = match availability {
            RuntimeAvailability::Installed => RuntimeHealth::Ready,
            RuntimeAvailability::Available => RuntimeHealth::PendingInstall,
            RuntimeAvailability::Unavailable => RuntimeHealth::Blocked,
        };

        Self {
            provider_id: provider_id.to_string(),
            model_id: model.id.clone(),
            model_name: model.name.clone(),
            runtime: model.runtime,
            execution_strategy: ExecutionStrategy::for_model(&model.id, model.runtime),
            workflows: model
                .capabilities
                .iter()
                .map(|capability| capability.workflow)
                .collect(),
            availability,
            install_status: model.install.status,
            cache,
            compatibility,
            health,
            reasons,
        }
    }

    fn from_candidate(candidate: &ModelCandidateInstallState, inventory: &DeviceInventory) -> Self {
        let workflows: Vec<CapabilityWorkflow> = candidate
            .lanes
            .iter()
            .filter_map(lane_to_workflow)
            .collect();
        let runtime = match candidate.runtime_path {
            ProductRuntimePath::ManagedApi => ModelRuntime::ExternalApi,
            ProductRuntimePath::PythonPocOnly | ProductRuntimePath::Unknown => {
                ModelRuntime::ResearchOnly
            }
            ProductRuntimePath::RustNative
            | ProductRuntimePath::NativeLibraryBinding
            | ProductRuntimePath::ExternalExecutable => ModelRuntime::Local,
        };
        let availability = if candidate.install_state == CandidateInstallState::Installed {
            RuntimeAvailability::Installed
        } else {
            RuntimeAvailability::Unavailable
        };
        let health = if availability == RuntimeAvailability::Installed {
            RuntimeHealth::Ready
        } else {
            RuntimeHealth::Blocked
        };
        let device = first_available_device(inventory);
        let selected = device.map(|device| device.accelerator);
        let mut reasons = candidate.blockers.clone();
        if candidate.product_eligibility == ProductEligibility::ResearchOnly {
            reasons.push("Research-only model cannot be product-enabled.".to_string());
        }

        Self {
            provider_id: provider_id_for_candidate(candidate),
            model_id: candidate.candidate_id.clone(),
            model_name: candidate.name.clone(),
            runtime,
            execution_strategy: ExecutionStrategy::for_model(&candidate.candidate_id, runtime),
            workflows,
            availability,
            install_status: if availability == RuntimeAvailability::Installed {
                ModelInstallStatus::Installed
            } else {
                ModelInstallStatus::Unavailable
            },
            cache: ModelCacheState {
                cache_path: Some(candidate.cache.cache_path.clone()),
                package_id: candidate.download_plan.repository_id.clone(),
                status: if candidate.cache.verified {
                    CacheStatus::Ready
                } else {
                    CacheStatus::Missing
                },
                expected_size_mb: candidate.download_plan.expected_size_mb,
                disk_usage_mb: candidate.cache.disk_usage_mb,
                verified: candidate.cache.verified,
                evidence: candidate.cache.evidence.clone(),
                license: LicenseAcceptanceState::Accepted,
                warmup: WarmupStatus::Cold,
            },
            compatibility: RuntimeCompatibility {
                supported: true,
                selected_accelerator: selected,
                min_memory_mb: None,
                available_memory_mb: device.and_then(|device| device.memory_mb),
                requires_network: false,
                reasons: vec![],
            },
            health,
            reasons,
        }
    }

    /// Build the shared `ModelRuntimeState` for a built-in native model. The two
    /// native constructors differ only in id/name/package/evidence/workflows; the
    /// runtime, availability, cache, and accelerator compatibility are identical.
    fn native_state(
        model_id: String,
        model_name: &str,
        package_id: &str,
        evidence: &str,
        workflows: Vec<CapabilityWorkflow>,
        inventory: &DeviceInventory,
    ) -> Self {
        let device = first_available_device(inventory);
        Self {
            provider_id: "soundworks-native".to_string(),
            model_id,
            model_name: model_name.to_string(),
            runtime: ModelRuntime::Local,
            execution_strategy: ExecutionStrategy::NativeRust,
            workflows,
            availability: RuntimeAvailability::Installed,
            install_status: ModelInstallStatus::Installed,
            cache: ModelCacheState {
                cache_path: None,
                package_id: Some(package_id.to_string()),
                status: CacheStatus::Ready,
                expected_size_mb: Some(1),
                disk_usage_mb: Some(1),
                verified: true,
                evidence: evidence.to_string(),
                license: LicenseAcceptanceState::Accepted,
                warmup: WarmupStatus::Cold,
            },
            compatibility: RuntimeCompatibility {
                supported: true,
                selected_accelerator: device.map(|device| device.accelerator),
                min_memory_mb: None,
                available_memory_mb: device.and_then(|device| device.memory_mb),
                requires_network: false,
                reasons: vec![],
            },
            health: RuntimeHealth::Ready,
            reasons: vec![],
        }
    }

    fn native_procedural_sfx(inventory: &DeviceInventory) -> Self {
        Self::native_state(
            NativeModel::ProceduralSfx.model_id().to_string(),
            "SoundWorks native procedural SFX",
            "soundworks-native-procedural-sfx",
            "built into the Rust runtime; no Python, model cache, or network call required",
            vec![CapabilityWorkflow::Sfx, CapabilityWorkflow::Ambience],
            inventory,
        )
    }

    fn native_procedural_music(inventory: &DeviceInventory) -> Self {
        Self::native_state(
            NativeModel::ProceduralMusic.model_id().to_string(),
            "SoundWorks native procedural samples and loops",
            "soundworks-native-procedural-music",
            "built into the Rust runtime; generates procedural one-shots and tempo-aligned loops without Python, model cache, or network calls",
            vec![
                CapabilityWorkflow::InstrumentSample,
                CapabilityWorkflow::Loop,
            ],
            inventory,
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RuntimeAvailability {
    Installed,
    Available,
    Unavailable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RuntimeHealth {
    Ready,
    PendingInstall,
    Blocked,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelCacheState {
    pub cache_path: Option<String>,
    pub package_id: Option<String>,
    pub status: CacheStatus,
    pub expected_size_mb: Option<u32>,
    pub disk_usage_mb: Option<u32>,
    pub verified: bool,
    pub evidence: String,
    pub license: LicenseAcceptanceState,
    pub warmup: WarmupStatus,
}

impl ModelCacheState {
    fn from_manifest(model: &ModelManifest) -> Self {
        let status = match model.install.status {
            ModelInstallStatus::Installed | ModelInstallStatus::Packaged => CacheStatus::Missing,
            ModelInstallStatus::Installable | ModelInstallStatus::Unavailable => {
                CacheStatus::Missing
            }
            ModelInstallStatus::External => CacheStatus::External,
        };
        let verified = false;

        Self {
            cache_path: None,
            package_id: model.install.package_id.clone(),
            status,
            expected_size_mb: model.install.installed_size_mb,
            disk_usage_mb: None,
            verified,
            evidence: match model.install.status {
                ModelInstallStatus::Installed | ModelInstallStatus::Packaged => {
                    "manifest-only; on-disk cache/package has not been verified".to_string()
                }
                ModelInstallStatus::Installable => {
                    "installable manifest; no local cache expected yet".to_string()
                }
                ModelInstallStatus::External => {
                    "external provider state; managed outside the local model cache".to_string()
                }
                ModelInstallStatus::Unavailable => {
                    "unavailable manifest; no local cache exists".to_string()
                }
            },
            license: match status {
                CacheStatus::Ready | CacheStatus::External => LicenseAcceptanceState::Accepted,
                CacheStatus::Missing => LicenseAcceptanceState::Required,
            },
            warmup: match status {
                CacheStatus::Ready => WarmupStatus::Cold,
                CacheStatus::Missing => WarmupStatus::NotAvailable,
                CacheStatus::External => WarmupStatus::ManagedByProvider,
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CacheStatus {
    Ready,
    Missing,
    External,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum LicenseAcceptanceState {
    Accepted,
    Required,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum WarmupStatus {
    Cold,
    NotAvailable,
    ManagedByProvider,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeCompatibility {
    pub supported: bool,
    pub selected_accelerator: Option<DeviceAccelerator>,
    pub min_memory_mb: Option<u32>,
    pub available_memory_mb: Option<u32>,
    pub requires_network: bool,
    pub reasons: Vec<String>,
}

impl RuntimeCompatibility {
    fn from_model(model: &ModelManifest, inventory: &DeviceInventory) -> Self {
        let selected = inventory.compatible_accelerator(&model.requirements.accelerators);
        let available_memory = selected.as_ref().and_then(|device| device.memory_mb);
        let memory_ok = match (model.requirements.min_memory_mb, available_memory) {
            (Some(required), Some(available)) => available >= required,
            (Some(_), None) => false,
            (None, _) => true,
        };

        let mut reasons = vec![];
        if selected.is_none() {
            reasons.push("No compatible accelerator is currently available.".to_string());
        }
        if !memory_ok {
            reasons.push("Detected device memory is below model requirements.".to_string());
        }

        Self {
            supported: reasons.is_empty(),
            selected_accelerator: selected.as_ref().map(|device| device.accelerator),
            min_memory_mb: model.requirements.min_memory_mb,
            available_memory_mb: available_memory,
            requires_network: model.requirements.requires_network,
            reasons,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeJobSnapshot {
    pub id: String,
    pub kind: JobKind,
    pub status: JobStatus,
    pub provider_id: String,
    pub model_id: String,
    pub workflow: CapabilityWorkflow,
    pub adapter: ProviderAdapterKind,
    pub progress: Option<JobProgress>,
    pub cancellation: CancellationState,
    pub retry_count: u8,
    pub created_at: String,
    pub updated_at: String,
    /// Store-relative job record directory (e.g. `jobs/<id>`). Absolute paths are
    /// derived on demand from the store root via `sanitized_join`, so payloads no
    /// longer carry the user's home directory or redundant derivable paths (F-030).
    pub record_root: String,
    pub log_tail: Vec<String>,
    pub artifacts: Vec<RuntimeJobArtifact>,
    pub actionable_error: Option<ActionableRuntimeError>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeJobArtifact {
    pub kind: RuntimeArtifactKind,
    pub path: String,
    pub mime_type: String,
    pub bytes: u64,
    pub summary: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RuntimeArtifactKind {
    AudioPreview,
    OutputManifest,
    ErrorReport,
    Log,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProviderAdapterKind {
    NativeRust,
    LocalExecutable,
    ManagedApi,
    ResearchOnly,
}

/// How the runtime will execute a model, stored on every `ModelRuntimeState`
/// (F-007). Dispatch reads this as data rather than re-deriving the adapter by
/// matching the model id in multiple places, so the runtime catalog itself
/// declares how each model runs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ExecutionStrategy {
    /// A built-in native Rust adapter (Kokoro ONNX or procedural synthesis).
    NativeRust,
    /// A locally-installed executable/library adapter (declared but not yet
    /// runnable in the shipped build).
    LocalExecutable,
    /// A managed external API adapter.
    ManagedApi,
    /// Research-only; never product-executable.
    ResearchOnly,
}

impl ExecutionStrategy {
    /// Resolve the strategy from the model id (via the native registry) and the
    /// declared runtime kind. Native built-ins always win — they are executable
    /// regardless of how their catalog runtime is labelled.
    fn for_model(model_id: &str, runtime: ModelRuntime) -> Self {
        if NativeModel::from_model_id(model_id).is_some() {
            return ExecutionStrategy::NativeRust;
        }
        match runtime {
            ModelRuntime::Local => ExecutionStrategy::LocalExecutable,
            ModelRuntime::ExternalApi => ExecutionStrategy::ManagedApi,
            ModelRuntime::ResearchOnly => ExecutionStrategy::ResearchOnly,
        }
    }

    fn adapter_kind(self) -> ProviderAdapterKind {
        match self {
            ExecutionStrategy::NativeRust => ProviderAdapterKind::NativeRust,
            ExecutionStrategy::LocalExecutable => ProviderAdapterKind::LocalExecutable,
            ExecutionStrategy::ManagedApi => ProviderAdapterKind::ManagedApi,
            ExecutionStrategy::ResearchOnly => ProviderAdapterKind::ResearchOnly,
        }
    }
}

/// The built-in models SoundWorks executes natively in-process (no Python, no
/// external process). This enum is the single source of truth for native model
/// identity and dispatch (F-024): adding or renaming a native model is one edit
/// here, and both catalog injection and execution read it instead of repeating
/// literal id strings across `from_model_manager`, `adapter_for_model`, and
/// `run_adapter`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeModel {
    KokoroTts,
    ProceduralSfx,
    ProceduralMusic,
    Smoke,
}

impl NativeModel {
    const ALL: [NativeModel; 4] = [
        NativeModel::KokoroTts,
        NativeModel::ProceduralSfx,
        NativeModel::ProceduralMusic,
        NativeModel::Smoke,
    ];

    pub const fn model_id(self) -> &'static str {
        match self {
            NativeModel::KokoroTts => "kokoro-82m",
            NativeModel::ProceduralSfx => "native-procedural-sfx",
            NativeModel::ProceduralMusic => "native-procedural-music",
            NativeModel::Smoke => "native-smoke",
        }
    }

    pub fn from_model_id(model_id: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|model| model.model_id() == model_id)
    }

    /// The always-available native models the runtime injects into the catalog
    /// when a verified install for them is not already present.
    const fn injected() -> [NativeModel; 2] {
        [NativeModel::ProceduralSfx, NativeModel::ProceduralMusic]
    }

    fn injected_state(self, inventory: &DeviceInventory) -> Option<ModelRuntimeState> {
        match self {
            NativeModel::ProceduralSfx => Some(ModelRuntimeState::native_procedural_sfx(inventory)),
            NativeModel::ProceduralMusic => {
                Some(ModelRuntimeState::native_procedural_music(inventory))
            }
            NativeModel::KokoroTts | NativeModel::Smoke => None,
        }
    }

    /// Execute this native model's adapter against a claimed job.
    fn run(
        self,
        store: &RuntimeJobStore,
        job: &mut RuntimeJobSnapshot,
        request: &RuntimeJobRequest,
        ctx: &ExecutionContext,
    ) -> io::Result<()> {
        match self {
            NativeModel::KokoroTts => store.write_kokoro_tts_audio(job, request, ctx),
            NativeModel::ProceduralSfx => store.write_native_sfx_audio(job, request, ctx),
            NativeModel::ProceduralMusic => store.write_native_music_audio(job, request, ctx),
            NativeModel::Smoke => store.write_smoke_audio(job, request),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CancellationState {
    Cancellable,
    Requested,
    NotCancellable,
    Completed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionableRuntimeError {
    pub code: String,
    pub summary: String,
    pub recovery: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeJobAdmission {
    pub accepted: bool,
    pub provider_id: String,
    pub model_id: String,
    pub kind: JobKind,
    pub reason: String,
    pub actionable_error: Option<ActionableRuntimeError>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeJobRequest {
    pub provider_id: String,
    pub model_id: String,
    pub kind: JobKind,
    pub workflow: CapabilityWorkflow,
    pub prompt: String,
    pub source_surface: String,
    #[serde(default)]
    pub parameters: BTreeMap<String, Value>,
}

/// Shared execution backend for the runtime job store (F-006 / F-019).
///
/// Holds the single process-wide Tokio runtime, a warm cache of loaded Kokoro
/// models, and the per-job cancellation registry. The desktop keeps one instance
/// in Tauri-managed state so every queued job reuses the same runtime + warm
/// model rather than rebuilding a runtime and re-reading the ONNX model from disk
/// on each request.
pub struct RuntimeEngine {
    runtime: tokio::runtime::Runtime,
    kokoro: Mutex<Vec<KokoroCacheEntry>>,
    cancellations: Mutex<HashMap<String, Arc<AtomicBool>>>,
}

struct KokoroCacheEntry {
    key: String,
    tts: Arc<KokoroTts>,
}

/// Upper bound on distinct Kokoro models held warm at once. Each entry pins an
/// ONNX session + voice table in memory, so the cache is small and evicts the
/// oldest entry once full.
const KOKORO_CACHE_CAPACITY: usize = 4;

impl RuntimeEngine {
    pub fn new() -> io::Result<Self> {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .build()
            .map_err(io::Error::other)?;
        Ok(Self {
            runtime,
            kokoro: Mutex::new(Vec::new()),
            cancellations: Mutex::new(HashMap::new()),
        })
    }

    fn lock_kokoro(&self) -> std::sync::MutexGuard<'_, Vec<KokoroCacheEntry>> {
        self.kokoro
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
    }

    fn lock_cancellations(&self) -> std::sync::MutexGuard<'_, HashMap<String, Arc<AtomicBool>>> {
        self.cancellations
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
    }

    /// Get-or-create the cancellation token for a job id.
    fn cancel_token(&self, job_id: &str) -> Arc<AtomicBool> {
        self.lock_cancellations()
            .entry(job_id.to_string())
            .or_insert_with(|| Arc::new(AtomicBool::new(false)))
            .clone()
    }

    /// Signal an in-flight job to cancel. Returns true when a worker token
    /// existed (i.e. the job had been queued/started in this process).
    pub fn request_cancel(&self, job_id: &str) -> bool {
        match self.lock_cancellations().get(job_id) {
            Some(token) => {
                token.store(true, Ordering::SeqCst);
                true
            }
            None => false,
        }
    }

    /// Drop the cancellation token once a job reaches a terminal state.
    pub fn forget(&self, job_id: &str) {
        self.lock_cancellations().remove(job_id);
    }

    /// Bundle the engine with a job's cancellation token for the duration of a run.
    pub fn context_for(&self, job_id: &str) -> ExecutionContext<'_> {
        ExecutionContext {
            engine: self,
            cancel: self.cancel_token(job_id),
        }
    }

    /// Load a Kokoro model, reusing a warm in-process instance keyed by the model
    /// and voices directory (F-019). `KokoroTts` loads every voice in the directory
    /// at construction, so the cache key is the cache root, not the per-call voice.
    /// Returns `(tts, warm)` where `warm` reports a cache hit.
    fn kokoro(&self, model_path: &Path, voices_path: &Path) -> io::Result<(Arc<KokoroTts>, bool)> {
        let key = format!("{}|{}", model_path.display(), voices_path.display());
        if let Some(entry) = self.lock_kokoro().iter().find(|entry| entry.key == key) {
            return Ok((entry.tts.clone(), true));
        }
        let tts = self.runtime.block_on(async {
            KokoroTts::new(model_path, voices_path)
                .await
                .map_err(|error| io::Error::other(error.to_string()))
        })?;
        let tts = Arc::new(tts);
        let mut cache = self.lock_kokoro();
        // Re-check: another thread may have loaded the same model concurrently.
        if let Some(entry) = cache.iter().find(|entry| entry.key == key) {
            return Ok((entry.tts.clone(), true));
        }
        if cache.len() >= KOKORO_CACHE_CAPACITY {
            cache.remove(0);
        }
        cache.push(KokoroCacheEntry {
            key,
            tts: tts.clone(),
        });
        Ok((tts, false))
    }
}

impl std::fmt::Debug for RuntimeEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RuntimeEngine")
            .field("warm_kokoro_models", &self.lock_kokoro().len())
            .field("tracked_cancellations", &self.lock_cancellations().len())
            .finish()
    }
}

/// A single job's execution handle: the shared engine plus that job's cancellation
/// token. Passed to the adapters so they can run on the shared runtime / warm
/// model cache and observe cooperative cancellation at checkpoints.
pub struct ExecutionContext<'a> {
    engine: &'a RuntimeEngine,
    cancel: Arc<AtomicBool>,
}

impl ExecutionContext<'_> {
    fn is_cancelled(&self) -> bool {
        self.cancel.load(Ordering::SeqCst)
    }
}

#[derive(Debug, Clone)]
pub struct RuntimeJobStore {
    root: PathBuf,
}

impl RuntimeJobStore {
    pub fn default_root() -> PathBuf {
        if let Ok(root) = std::env::var("SOUNDWORKS_RUNTIME_ROOT") {
            return PathBuf::from(root);
        }
        std::env::var("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("."))
            .join("Library")
            .join("Application Support")
            .join("SoundWorks")
            .join("runtime")
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

    pub fn enqueue(
        &self,
        overview: &RuntimeOverview,
        request: RuntimeJobRequest,
    ) -> io::Result<RuntimeJobSnapshot> {
        let job_id = format!(
            "job-{}-{}-{}",
            request.workflow_id_fragment(),
            timestamp_millis(),
            next_job_sequence()
        );
        let record_root = sanitized_join(&self.root, &["jobs", &job_id])?;
        fs::create_dir_all(record_root.join("artifacts"))?;
        let created_at = timestamp_string();
        let adapter = adapter_for_model(overview, &request);
        // UX-NB1: composition mixdown is not a model-backed job — it sums existing
        // library clips on the native offline mixer — so it bypasses model
        // availability admission (a real model id is never required).
        let admission = if request.kind == JobKind::RenderComposition {
            RuntimeJobAdmission {
                accepted: true,
                provider_id: request.provider_id.clone(),
                model_id: request.model_id.clone(),
                kind: request.kind,
                reason: "composition mixdown runs on the native offline mixer".to_string(),
                actionable_error: None,
            }
        } else {
            overview.admit_job(&request.provider_id, &request.model_id, request.kind)
        };
        let request_gate = validate_request_gates(&request);

        write_json(record_root.join("recipe.json"), &request)?;
        write_json(
            record_root.join("model.json"),
            &serde_json::json!({
                "providerId": request.provider_id,
                "modelId": request.model_id,
                "workflow": request.workflow,
                "adapter": adapter,
                "admissionAccepted": admission.accepted,
                "admissionReason": admission.reason,
            }),
        )?;

        let mut job = RuntimeJobSnapshot {
            id: job_id.clone(),
            kind: request.kind,
            status: JobStatus::Queued,
            provider_id: request.provider_id.clone(),
            model_id: request.model_id.clone(),
            workflow: request.workflow,
            adapter,
            progress: Some(JobProgress {
                percent: 0.0,
                message: Some("Job persisted in the local runtime queue.".to_string()),
            }),
            cancellation: CancellationState::Cancellable,
            retry_count: 0,
            created_at: created_at.clone(),
            updated_at: created_at,
            record_root: format!("jobs/{job_id}"),
            log_tail: vec!["persisted job record".to_string()],
            artifacts: vec![],
            actionable_error: None,
        };
        self.append_event(&job, "queued", "persisted job record")?;

        if !admission.accepted || request_gate.is_some() {
            job.status = JobStatus::Failed;
            job.progress = Some(JobProgress {
                percent: 100.0,
                message: Some("Runtime rejected the job before adapter execution.".to_string()),
            });
            job.cancellation = CancellationState::NotCancellable;
            job.updated_at = timestamp_string();
            if let Some(error) = request_gate {
                job.log_tail.push(error.summary.clone());
                job.actionable_error = Some(error);
            } else {
                job.log_tail.push(admission.reason.clone());
                job.actionable_error = admission.actionable_error;
            }
            self.write_error_report(&mut job)?;
            self.append_event(&job, "failed", "runtime admission rejected job")?;
            self.write_job(&job)?;
            return Ok(job);
        }

        // F-006: persist the job as `Queued` and return immediately. Synthesis is
        // driven off the calling thread by `run_job` (the desktop spawns a worker;
        // tests call `enqueue_and_run`). This is what makes the queue/worker/cancel
        // vocabulary honest (F-037): the caller is no longer blocked for the full
        // generation, and a job can be cancelled while it is genuinely Queued.
        self.write_job(&job)?;
        Ok(job)
    }

    /// Execute a previously `enqueue`d job to a terminal state. Idempotent for
    /// non-`Queued` jobs (returns them unchanged) and absent jobs (`Ok(None)`).
    /// The worker observes cooperative cancellation through `ctx` and runs Kokoro
    /// on the shared runtime + warm model cache.
    pub fn run_job(
        &self,
        job_id: &str,
        ctx: &ExecutionContext,
    ) -> io::Result<Option<RuntimeJobSnapshot>> {
        let Some(job) = self.read_job(job_id)? else {
            return Ok(None);
        };
        if job.status != JobStatus::Queued {
            return Ok(Some(job));
        }
        // F-004: recompute the recipe path from the validated id, never trust the
        // persisted string.
        let recipe_path = sanitized_join(&self.root, &["jobs", &job.id, "recipe.json"])?;
        let request: RuntimeJobRequest = read_json(&recipe_path)?;
        Ok(Some(self.run_adapter(job, &request, ctx)?))
    }

    /// Convenience for tests and synchronous callers: enqueue, then run to a
    /// terminal state on the current thread. Rejected jobs short-circuit without
    /// running an adapter.
    pub fn enqueue_and_run(
        &self,
        engine: &RuntimeEngine,
        overview: &RuntimeOverview,
        request: RuntimeJobRequest,
    ) -> io::Result<RuntimeJobSnapshot> {
        let job = self.enqueue(overview, request)?;
        if job.status != JobStatus::Queued {
            return Ok(job);
        }
        let ctx = engine.context_for(&job.id);
        let result = self.run_job(&job.id, &ctx)?.unwrap_or(job);
        engine.forget(&result.id);
        Ok(result)
    }

    pub fn cancel(&self, job_id: &str) -> io::Result<Option<RuntimeJobSnapshot>> {
        let Some(mut job) = self.read_job(job_id)? else {
            return Ok(None);
        };
        if matches!(
            job.cancellation,
            CancellationState::Cancellable | CancellationState::Requested
        ) {
            job.status = JobStatus::Cancelled;
            job.cancellation = CancellationState::Completed;
            job.progress = Some(JobProgress {
                percent: job
                    .progress
                    .as_ref()
                    .map_or(0.0, |progress| progress.percent),
                message: Some("Cancellation persisted by runtime job store.".to_string()),
            });
            job.updated_at = timestamp_string();
            job.log_tail.push("cancelled by user request".to_string());
            self.append_event(&job, "cancelled", "cancelled by user request")?;
            self.write_job(&job)?;
        }
        Ok(Some(job))
    }

    pub fn retry(
        &self,
        overview: &RuntimeOverview,
        job_id: &str,
    ) -> io::Result<Option<RuntimeJobSnapshot>> {
        let Some(job) = self.read_job(job_id)? else {
            return Ok(None);
        };
        // F-004 (second-order): do not trust the persisted recipe_path string;
        // recompute it from the validated job id under the store root.
        let recipe_path = sanitized_join(&self.root, &["jobs", &job.id, "recipe.json"])?;
        let request: RuntimeJobRequest = read_json(&recipe_path)?;
        let mut retried = self.enqueue(overview, request)?;
        retried.retry_count = job.retry_count.saturating_add(1);
        retried.log_tail.push(format!("retry of {}", job.id));
        self.write_job(&retried)?;
        Ok(Some(retried))
    }

    pub fn artifacts(&self, job_id: &str) -> io::Result<Vec<RuntimeJobArtifact>> {
        Ok(self
            .read_job(job_id)?
            .map(|job| job.artifacts)
            .unwrap_or_default())
    }

    /// Read a single job snapshot by id so the UI can poll an in-flight worker
    /// (UX-F1). Returns `Ok(None)` for an unknown id. Path-validated through the
    /// same `read_job` traversal guard (F-004) as cancel/retry/artifacts.
    pub fn job(&self, job_id: &str) -> io::Result<Option<RuntimeJobSnapshot>> {
        self.read_job(job_id)
    }

    pub fn read_jobs(&self) -> io::Result<Vec<RuntimeJobSnapshot>> {
        let jobs_root = self.root.join("jobs");
        if !jobs_root.exists() {
            return Ok(vec![]);
        }
        let mut jobs = vec![];
        for entry in fs::read_dir(jobs_root)? {
            let path = entry?.path().join("job.json");
            if path.is_file() {
                jobs.push(read_json(&path)?);
            }
        }
        Ok(jobs)
    }

    fn read_job(&self, job_id: &str) -> io::Result<Option<RuntimeJobSnapshot>> {
        // F-004: validate the caller-supplied job_id before joining it into a path.
        // cancel/retry/artifacts all funnel through read_job, so this one guard
        // closes the read-traversal across the whole job store.
        let path = sanitized_join(&self.root, &["jobs", job_id, "job.json"])?;
        if path.is_file() {
            Ok(Some(read_json(path)?))
        } else {
            Ok(None)
        }
    }

    // UX-NB1: render the composition described in the request parameters into a
    // single mixed WAV. Clip PCM is resolved from the project library; unresolved
    // clips are skipped only when at least one playable clip remains.
    fn write_composition_mixdown(
        &self,
        job: &mut RuntimeJobSnapshot,
        request: &RuntimeJobRequest,
        ctx: &ExecutionContext,
    ) -> io::Result<()> {
        use crate::composition_mixdown::{db_to_linear, mix, MixClip, MixRequest};

        let params = &request.parameters;
        let sample_rate = params
            .get("sampleRateHz")
            .and_then(Value::as_u64)
            .unwrap_or(48_000) as u32;
        let channels = params
            .get("channels")
            .and_then(Value::as_u64)
            .unwrap_or(2)
            .clamp(1, 2) as u16;
        let duration_ms = params
            .get("durationMs")
            .and_then(Value::as_u64)
            .unwrap_or(0);
        let master_gain = db_to_linear(
            params
                .get("masterGainDb")
                .and_then(Value::as_f64)
                .unwrap_or(0.0) as f32,
        );

        if duration_ms == 0 {
            job.status = JobStatus::Failed;
            job.progress = Some(JobProgress {
                percent: 100.0,
                message: Some("Composition has no duration to render.".to_string()),
            });
            job.cancellation = CancellationState::NotCancellable;
            job.actionable_error = Some(ActionableRuntimeError {
                code: "composition.empty".to_string(),
                summary: "Composition has no duration".to_string(),
                recovery: "Add at least one clip to the timeline before rendering a mixdown."
                    .to_string(),
            });
            self.write_error_report(job)?;
            return Ok(());
        }

        job.status = JobStatus::Running;
        job.progress = Some(JobProgress {
            percent: 45.0,
            message: Some("Mixing composition clips into a rendered WAV.".to_string()),
        });
        self.append_event(job, "running", "composition mixdown started")?;
        self.write_job(job)?;
        if ctx.is_cancelled() {
            mark_cancelled(job);
            return Ok(());
        }

        let library = crate::ProjectLibraryStore::default();
        let mut clips: Vec<MixClip> = vec![];
        let mut resolved = 0usize;
        let mut skipped = 0usize;
        if let Some(entries) = params.get("clips").and_then(Value::as_array) {
            for entry in entries {
                let asset_id = entry.get("assetId").and_then(Value::as_str).unwrap_or("");
                match library.load_item_pcm(asset_id).ok().flatten() {
                    Some((samples, source_channels, source_sample_rate)) => {
                        let source_frames =
                            samples.len() as u64 / u64::from(source_channels.max(1));
                        let default_end =
                            source_frames * 1000 / u64::from(source_sample_rate.max(1));
                        clips.push(MixClip {
                            samples,
                            source_channels,
                            source_sample_rate,
                            timeline_start_ms: entry
                                .get("timelineStartMs")
                                .and_then(Value::as_u64)
                                .unwrap_or(0),
                            source_start_ms: entry
                                .get("sourceStartMs")
                                .and_then(Value::as_u64)
                                .unwrap_or(0),
                            source_end_ms: entry
                                .get("sourceEndMs")
                                .and_then(Value::as_u64)
                                .unwrap_or(default_end),
                            gain: db_to_linear(
                                entry.get("gainDb").and_then(Value::as_f64).unwrap_or(0.0) as f32,
                            ),
                            pan: entry.get("pan").and_then(Value::as_f64).unwrap_or(0.0) as f32,
                            fade_in_ms: entry.get("fadeInMs").and_then(Value::as_u64).unwrap_or(0),
                            fade_out_ms: entry
                                .get("fadeOutMs")
                                .and_then(Value::as_u64)
                                .unwrap_or(0),
                        });
                        resolved += 1;
                    }
                    None => {
                        skipped += 1;
                        job.log_tail
                            .push(format!("skipped unresolved clip asset {asset_id}"));
                    }
                }
            }
        }

        if resolved == 0 && skipped > 0 {
            job.status = JobStatus::Failed;
            job.progress = Some(JobProgress {
                percent: 100.0,
                message: Some("Composition clips could not be resolved.".to_string()),
            });
            job.cancellation = CancellationState::NotCancellable;
            job.actionable_error = Some(ActionableRuntimeError {
                code: "composition.unresolved_clips".to_string(),
                summary: "Composition mixdown has no playable clips".to_string(),
                recovery:
                    "Re-link or import the requested timeline clips before rendering the mixdown."
                        .to_string(),
            });
            self.write_error_report(job)?;
            return Ok(());
        }

        let samples = mix(&MixRequest {
            sample_rate,
            channels,
            duration_ms,
            master_gain,
            clips,
        });
        if ctx.is_cancelled() {
            mark_cancelled(job);
            return Ok(());
        }

        let record_root = sanitized_join(&self.root, &["jobs", &job.id])?;
        let audio_path = record_root
            .join("artifacts")
            .join("composition-mixdown.wav");
        write_pcm16_wav_channels(&audio_path, &samples, sample_rate, channels)?;
        let frame_count = samples.len() as u64 / u64::from(channels.max(1));
        let manifest_path = record_root.join("output-manifest.json");
        write_json(
            &manifest_path,
            &serde_json::json!({
                "jobId": job.id,
                "workflow": request.workflow,
                "kind": request.kind,
                "durationMs": duration_ms,
                "sampleRateHz": sample_rate,
                "channels": channels,
                "clipsResolved": resolved,
                "clipsSkipped": skipped,
                "artifact": audio_path,
                "note": "Native offline composition mixdown (sum of library clip PCM with per-clip gain/pan/fade).",
            }),
        )?;

        job.status = JobStatus::Succeeded;
        job.progress = Some(JobProgress {
            percent: 100.0,
            message: Some(format!(
                "Rendered composition mixdown ({resolved} clip(s) mixed, {skipped} skipped)."
            )),
        });
        job.cancellation = CancellationState::Completed;
        job.log_tail.push(format!(
            "wrote composition-mixdown.wav with {frame_count} frames"
        ));
        job.artifacts = vec![
            artifact(
                RuntimeArtifactKind::AudioPreview,
                &audio_path,
                "audio/wav",
                "Rendered composition mixdown WAV",
            )?,
            artifact(
                RuntimeArtifactKind::OutputManifest,
                &manifest_path,
                "application/json",
                "Composition mixdown manifest and provenance",
            )?,
        ];
        Ok(())
    }

    fn run_adapter(
        &self,
        mut job: RuntimeJobSnapshot,
        request: &RuntimeJobRequest,
        ctx: &ExecutionContext,
    ) -> io::Result<RuntimeJobSnapshot> {
        job.status = JobStatus::Running;
        job.progress = Some(JobProgress {
            percent: 35.0,
            message: Some("Provider adapter claimed the job.".to_string()),
        });
        job.updated_at = timestamp_string();
        job.log_tail.push(format!(
            "{:?} adapter claimed {}:{}",
            job.adapter, job.provider_id, job.model_id
        ));
        self.append_event(&job, "running", "provider adapter claimed job")?;
        // Persist the Running transition so a concurrent `cancel` (and any UI
        // poll) observes the job in flight before synthesis begins.
        self.write_job(&job)?;

        // F-006: real cooperative cancellation. If the worker boundary was asked
        // to cancel before synthesis started, stop here with no orphaned output.
        if ctx.is_cancelled() {
            mark_cancelled(&mut job);
            job.updated_at = timestamp_string();
            self.append_event(&job, "cancelled", "worker observed cancellation")?;
            self.write_job(&job)?;
            return Ok(job);
        }

        // UX-NB1: composition mixdown is offline mixing of existing clips, not a
        // generative model, so it runs before (and instead of) the model adapter
        // dispatch.
        if request.kind == JobKind::RenderComposition {
            self.write_composition_mixdown(&mut job, request, ctx)?;
            job.updated_at = timestamp_string();
            self.append_event(&job, status_event(&job.status), "adapter finished")?;
            self.write_job(&job)?;
            return Ok(job);
        }

        match job.adapter {
            ProviderAdapterKind::NativeRust => {
                // F-024: resolve the native model once from the registry. An
                // unrecognised native id falls back to the smoke adapter rather
                // than silently mis-routing through a literal match arm.
                NativeModel::from_model_id(&job.model_id)
                    .unwrap_or(NativeModel::Smoke)
                    .run(self, &mut job, request, ctx)?
            }
            ProviderAdapterKind::LocalExecutable
            | ProviderAdapterKind::ManagedApi
            | ProviderAdapterKind::ResearchOnly => {
                job.status = JobStatus::Failed;
                job.progress = Some(JobProgress {
                    percent: 100.0,
                    message: Some(
                        "Provider adapter is not executable in the shipped runtime yet."
                            .to_string(),
                    ),
                });
                job.cancellation = CancellationState::NotCancellable;
                job.actionable_error = Some(ActionableRuntimeError {
                    code: "runtime.adapter_not_executable".to_string(),
                    summary: "Provider adapter cannot run".to_string(),
                    recovery: "Install or port a product-safe adapter before retrying this model."
                        .to_string(),
                });
                self.write_error_report(&mut job)?;
            }
        }

        job.updated_at = timestamp_string();
        self.append_event(&job, status_event(&job.status), "adapter finished")?;
        self.write_job(&job)?;
        Ok(job)
    }

    fn write_smoke_audio(
        &self,
        job: &mut RuntimeJobSnapshot,
        request: &RuntimeJobRequest,
    ) -> io::Result<()> {
        let record_root = sanitized_join(&self.root, &["jobs", &job.id])?;
        let audio_path = record_root.join("artifacts").join("runtime-smoke.wav");
        write_smoke_wav(&audio_path)?;
        let manifest_path = record_root.join("output-manifest.json");
        write_json(
            &manifest_path,
            &serde_json::json!({
                "jobId": job.id,
                "workflow": request.workflow,
                "providerId": request.provider_id,
                "modelId": request.model_id,
                "prompt": request.prompt,
                "artifact": audio_path,
                "note": "Native Rust smoke artifact proves durable job execution only; real model audio is owned by later workflow stories.",
            }),
        )?;
        job.status = JobStatus::Succeeded;
        job.progress = Some(JobProgress {
            percent: 100.0,
            message: Some("Native Rust adapter wrote an auditable smoke artifact.".to_string()),
        });
        job.cancellation = CancellationState::Completed;
        job.log_tail.push("wrote runtime-smoke.wav".to_string());
        job.artifacts = vec![
            artifact(
                RuntimeArtifactKind::AudioPreview,
                &audio_path,
                "audio/wav",
                "Runtime smoke WAV artifact",
            )?,
            artifact(
                RuntimeArtifactKind::OutputManifest,
                &manifest_path,
                "application/json",
                "Output manifest and provenance",
            )?,
        ];
        Ok(())
    }

    fn write_kokoro_tts_audio(
        &self,
        job: &mut RuntimeJobSnapshot,
        request: &RuntimeJobRequest,
        ctx: &ExecutionContext,
    ) -> io::Result<()> {
        let cache_root = request
            .parameters
            .get("cachePath")
            .and_then(Value::as_str)
            .map(PathBuf::from)
            .unwrap_or_else(kokoro_cache_root);
        let model_path = cache_root.join("onnx").join("model.onnx");
        let voices_path = cache_root.join("voices");
        let voice = request
            .parameters
            .get("voice")
            .and_then(Value::as_str)
            .unwrap_or("af_heart");

        if !model_path.is_file() || !voices_path.join(format!("{voice}.bin")).is_file() {
            job.status = JobStatus::Failed;
            job.progress = Some(JobProgress {
                percent: 100.0,
                message: Some("Kokoro cache verification failed at adapter execution.".to_string()),
            });
            job.cancellation = CancellationState::NotCancellable;
            job.actionable_error = Some(ActionableRuntimeError {
                code: "tts.kokoro_cache_missing".to_string(),
                summary: "Kokoro model files are missing".to_string(),
                recovery: format!(
                    "Install onnx/model.onnx and voices/{voice}.bin under {} before retrying.",
                    cache_root.display()
                ),
            });
            self.write_error_report(job)?;
            return Ok(());
        }

        let text = request.prompt.trim();
        if text.is_empty() {
            job.status = JobStatus::Failed;
            job.progress = Some(JobProgress {
                percent: 100.0,
                message: Some("TTS script is empty.".to_string()),
            });
            job.cancellation = CancellationState::NotCancellable;
            job.actionable_error = Some(ActionableRuntimeError {
                code: "tts.empty_script".to_string(),
                summary: "TTS script is empty".to_string(),
                recovery: "Enter script text before queueing speech generation.".to_string(),
            });
            self.write_error_report(job)?;
            return Ok(());
        }

        job.progress = Some(JobProgress {
            percent: 65.0,
            message: Some("Kokoro is synthesizing speech from the script.".to_string()),
        });
        job.log_tail
            .push(format!("loading Kokoro cache {}", cache_root.display()));
        self.append_event(job, "running", "kokoro synthesis started")?;

        let speed = request
            .parameters
            .get("speed")
            .and_then(Value::as_f64)
            .map(|speed| speed as f32)
            .unwrap_or(1.0);
        // F-019: reuse a warm Kokoro model + the shared Tokio runtime instead of
        // rebuilding a runtime and re-reading the ONNX model from disk per request.
        let (tts, warm) = match ctx.engine.kokoro(&model_path, &voices_path) {
            Ok(loaded) => loaded,
            Err(error) => {
                job.status = JobStatus::Failed;
                job.progress = Some(JobProgress {
                    percent: 100.0,
                    message: Some("Kokoro model failed to load.".to_string()),
                });
                job.cancellation = CancellationState::NotCancellable;
                job.actionable_error = Some(ActionableRuntimeError {
                    code: "tts.kokoro_load_failed".to_string(),
                    summary: "Kokoro model failed to load".to_string(),
                    recovery: error.to_string(),
                });
                self.write_error_report(job)?;
                return Ok(());
            }
        };
        job.log_tail.push(
            if warm {
                "served Kokoro from a warm in-process model cache"
            } else {
                "cold-loaded Kokoro model into the shared warm cache"
            }
            .to_string(),
        );
        let synth = ctx.engine.runtime.block_on(async {
            tts.synth(text, Voice::new(voice).with_speed(speed))
                .await
                .map_err(|error| io::Error::other(error.to_string()))
        });
        let (samples, took) = match synth {
            Ok(result) => result,
            Err(error) => {
                job.status = JobStatus::Failed;
                job.progress = Some(JobProgress {
                    percent: 100.0,
                    message: Some("Kokoro synthesis failed.".to_string()),
                });
                job.cancellation = CancellationState::NotCancellable;
                job.actionable_error = Some(ActionableRuntimeError {
                    code: "tts.kokoro_synthesis_failed".to_string(),
                    summary: "Kokoro synthesis failed".to_string(),
                    recovery: error.to_string(),
                });
                self.write_error_report(job)?;
                return Ok(());
            }
        };

        // ONNX inference itself is not preemptible, but if cancellation arrived
        // while it ran we discard the result rather than commit a "succeeded"
        // artifact the user already cancelled.
        if ctx.is_cancelled() {
            mark_cancelled(job);
            return Ok(());
        }

        let record_root = sanitized_join(&self.root, &["jobs", &job.id])?;
        let audio_path = record_root.join("artifacts").join("kokoro-tts.wav");
        write_pcm16_wav(&audio_path, &samples, 24_000)?;
        let duration_ms = samples.len() as u64 * 1000 / 24_000;
        let manifest_path = record_root.join("output-manifest.json");
        write_json(
            &manifest_path,
            &serde_json::json!({
                "jobId": job.id,
                "workflow": request.workflow,
                "providerId": request.provider_id,
                "modelId": request.model_id,
                "modelVersion": request.parameters.get("modelVersion"),
                "language": request.parameters.get("language"),
                "voice": voice,
                "speakerLabels": request.parameters.get("speakerLabels"),
                "voiceProfileIds": request.parameters.get("voiceProfileIds"),
                "seed": request.parameters.get("seed"),
                "inputText": text,
                "sampleRateHz": 24_000,
                "channels": 1,
                "durationMs": duration_ms,
                "sampleCount": samples.len(),
                "synthesisMs": took.as_millis(),
                "warmStart": warm,
                "artifact": audio_path,
                "note": "Real Kokoro speech synthesis from verified local cache; no Python runtime was used.",
            }),
        )?;
        job.status = JobStatus::Succeeded;
        job.progress = Some(JobProgress {
            percent: 100.0,
            message: Some("Kokoro wrote a real generated speech WAV.".to_string()),
        });
        job.cancellation = CancellationState::Completed;
        job.log_tail.push(format!(
            "wrote kokoro-tts.wav with {} samples in {} ms",
            samples.len(),
            took.as_millis()
        ));
        job.artifacts = vec![
            artifact(
                RuntimeArtifactKind::AudioPreview,
                &audio_path,
                "audio/wav",
                "Generated Kokoro speech WAV",
            )?,
            artifact(
                RuntimeArtifactKind::OutputManifest,
                &manifest_path,
                "application/json",
                "Generated speech manifest and provenance",
            )?,
        ];
        Ok(())
    }

    fn write_native_sfx_audio(
        &self,
        job: &mut RuntimeJobSnapshot,
        request: &RuntimeJobRequest,
        ctx: &ExecutionContext,
    ) -> io::Result<()> {
        let prompt = request.prompt.trim();
        if prompt.is_empty() {
            job.status = JobStatus::Failed;
            job.progress = Some(JobProgress {
                percent: 100.0,
                message: Some("SFX prompt is empty.".to_string()),
            });
            job.cancellation = CancellationState::NotCancellable;
            job.actionable_error = Some(ActionableRuntimeError {
                code: "sfx.empty_prompt".to_string(),
                summary: "SFX prompt is empty".to_string(),
                recovery: "Enter a sound-effect or ambience prompt before queueing generation."
                    .to_string(),
            });
            self.write_error_report(job)?;
            return Ok(());
        }

        let sample_rate = 48_000u32;
        let channels = 2u16;
        let duration_ms = request
            .parameters
            .get("durationMs")
            .and_then(Value::as_u64)
            .unwrap_or(8_000)
            .clamp(250, 30_000);
        let loopable = request
            .parameters
            .get("loopable")
            .and_then(Value::as_bool)
            .unwrap_or(matches!(request.workflow, CapabilityWorkflow::Ambience));
        let category = request
            .parameters
            .get("category")
            .and_then(Value::as_str)
            .unwrap_or(if loopable {
                "ambience-bed"
            } else {
                "foley-impact"
            });
        let intensity = request
            .parameters
            .get("intensity")
            .and_then(Value::as_u64)
            .unwrap_or(70)
            .clamp(1, 100) as f32
            / 100.0;
        let realism = request
            .parameters
            .get("realism")
            .and_then(Value::as_u64)
            .unwrap_or(65)
            .clamp(1, 100) as f32
            / 100.0;

        job.progress = Some(JobProgress {
            percent: 65.0,
            message: Some("Rust native SFX generator is writing prompt-derived audio.".to_string()),
        });
        job.log_tail.push(format!(
            "generating native SFX: {duration_ms} ms, category {category}, loopable {loopable}"
        ));
        self.append_event(job, "running", "native SFX generation started")?;

        let samples = synthesize_native_sfx(
            prompt,
            request.workflow,
            duration_ms,
            sample_rate,
            intensity,
            realism,
            loopable,
        );
        if ctx.is_cancelled() {
            mark_cancelled(job);
            return Ok(());
        }
        let stats = loudness::analyze_f32(&samples, sample_rate, channels);
        let record_root = sanitized_join(&self.root, &["jobs", &job.id])?;
        let audio_path = record_root.join("artifacts").join("native-sfx.wav");
        write_pcm16_wav_channels(&audio_path, &samples, sample_rate, channels)?;
        let frame_count = samples.len() as u64 / channels as u64;
        let loop_start_sample = loopable.then_some(sample_rate as u64 / 2);
        let loop_end_sample =
            loopable.then_some(frame_count.saturating_sub(sample_rate as u64 / 2));
        let manifest_path = record_root.join("output-manifest.json");
        write_json(
            &manifest_path,
            &serde_json::json!({
                "jobId": job.id,
                "workflow": request.workflow,
                "providerId": request.provider_id,
                "modelId": request.model_id,
                "prompt": prompt,
                "negativePrompt": request.parameters.get("negativePrompt"),
                "category": category,
                "tags": request.parameters.get("tags"),
                "durationMs": duration_ms,
                "sampleRateHz": sample_rate,
                "channels": channels,
                "loudnessLufs": stats.loudness_lufs,
                "truePeakDbfs": stats.true_peak_dbfs,
                "loopable": loopable,
                "loopStartSample": loop_start_sample,
                "loopEndSample": loop_end_sample,
                "artifact": audio_path,
                "note": "Real Rust-native procedural SFX/ambience generation. MOSS-SoundEffect remains blocked until verified cache and product-safe adapter evidence are present.",
                "sourceEvidence": {
                    "mossMlx": "https://huggingface.co/mlx-community/MOSS-SoundEffect-v2.0-4bit",
                    "mossUpstream": "https://github.com/OpenMOSS/MOSS-TTS/blob/main/moss_soundeffect_v2/README.md"
                }
            }),
        )?;
        job.status = JobStatus::Succeeded;
        job.progress = Some(JobProgress {
            percent: 100.0,
            message: Some("Native SFX generator wrote a real playable WAV.".to_string()),
        });
        job.cancellation = CancellationState::Completed;
        job.log_tail
            .push(format!("wrote native-sfx.wav with {frame_count} frames"));
        job.artifacts = vec![
            artifact(
                RuntimeArtifactKind::AudioPreview,
                &audio_path,
                "audio/wav",
                "Generated native SFX/ambience WAV",
            )?,
            artifact(
                RuntimeArtifactKind::OutputManifest,
                &manifest_path,
                "application/json",
                "Generated SFX manifest, loop metadata, and provenance",
            )?,
        ];
        Ok(())
    }

    fn write_native_music_audio(
        &self,
        job: &mut RuntimeJobSnapshot,
        request: &RuntimeJobRequest,
        ctx: &ExecutionContext,
    ) -> io::Result<()> {
        let prompt = request.prompt.trim();
        if prompt.is_empty() {
            job.status = JobStatus::Failed;
            job.progress = Some(JobProgress {
                percent: 100.0,
                message: Some("Sample or loop prompt is empty.".to_string()),
            });
            job.cancellation = CancellationState::NotCancellable;
            job.actionable_error = Some(ActionableRuntimeError {
                code: "music.empty_prompt".to_string(),
                summary: "Sample or loop prompt is empty".to_string(),
                recovery: "Enter a musical sample or loop prompt before queueing generation."
                    .to_string(),
            });
            self.write_error_report(job)?;
            return Ok(());
        }

        let sample_rate = 48_000u32;
        let channels = 2u16;
        let bpm = request
            .parameters
            .get("bpm")
            .and_then(Value::as_f64)
            .map(|value| value as f32)
            .unwrap_or(120.0)
            .clamp(40.0, 240.0);
        let bars = request
            .parameters
            .get("bars")
            .and_then(Value::as_u64)
            .unwrap_or(4)
            .clamp(1, 16) as u16;
        let beats = request
            .parameters
            .get("beats")
            .and_then(Value::as_u64)
            .unwrap_or(4)
            .clamp(1, 12) as u16;
        let loopable = request
            .parameters
            .get("loopable")
            .and_then(Value::as_bool)
            .unwrap_or(matches!(request.workflow, CapabilityWorkflow::Loop));
        let musical_key = request
            .parameters
            .get("musicalKey")
            .and_then(Value::as_str)
            .unwrap_or("A minor");
        let instrument_family = request
            .parameters
            .get("instrumentFamily")
            .and_then(Value::as_str)
            .unwrap_or("synth-bass");
        let articulation = request
            .parameters
            .get("articulation")
            .and_then(Value::as_str)
            .unwrap_or("pulsed sequence");
        let velocity = request
            .parameters
            .get("velocityEnergy")
            .and_then(Value::as_u64)
            .unwrap_or(76)
            .clamp(1, 100) as f32
            / 100.0;
        let duration_ms = request
            .parameters
            .get("durationMs")
            .and_then(Value::as_u64)
            .unwrap_or_else(|| {
                if matches!(request.workflow, CapabilityWorkflow::Loop) {
                    loop_duration_ms(bpm, bars, beats)
                } else {
                    900
                }
            })
            .clamp(150, 120_000);

        job.progress = Some(JobProgress {
            percent: 65.0,
            message: Some("Rust native music generator is writing tempo-aware audio.".to_string()),
        });
        job.log_tail.push(format!(
            "generating native music: {duration_ms} ms, {bpm} BPM, key {musical_key}, loopable {loopable}"
        ));
        self.append_event(job, "running", "native music generation started")?;

        let samples = synthesize_native_music(
            prompt,
            request.workflow,
            duration_ms,
            sample_rate,
            bpm,
            bars,
            beats,
            musical_key,
            instrument_family,
            articulation,
            velocity,
        );
        if ctx.is_cancelled() {
            mark_cancelled(job);
            return Ok(());
        }
        let stats = loudness::analyze_f32(&samples, sample_rate, channels);
        let record_root = sanitized_join(&self.root, &["jobs", &job.id])?;
        let audio_path = record_root.join("artifacts").join(match request.workflow {
            CapabilityWorkflow::InstrumentSample => "native-sample.wav",
            _ => "native-loop.wav",
        });
        write_pcm16_wav_channels(&audio_path, &samples, sample_rate, channels)?;
        let frame_count = samples.len() as u64 / channels as u64;
        let loop_start_sample = loopable.then_some(0);
        let loop_end_sample = loopable.then_some(frame_count);
        let manifest_path = record_root.join("output-manifest.json");
        write_json(
            &manifest_path,
            &serde_json::json!({
                "jobId": job.id,
                "workflow": request.workflow,
                "providerId": request.provider_id,
                "modelId": request.model_id,
                "prompt": prompt,
                "negativePrompt": request.parameters.get("negativePrompt"),
                "instrumentFamily": instrument_family,
                "articulation": articulation,
                "tags": request.parameters.get("tags"),
                "durationMs": duration_ms,
                "sampleRateHz": sample_rate,
                "channels": channels,
                "loudnessLufs": stats.loudness_lufs,
                "truePeakDbfs": stats.true_peak_dbfs,
                "bpm": bpm,
                "musicalKey": musical_key,
                "bars": bars,
                "beats": beats,
                "loopable": loopable,
                "loopStartSample": loop_start_sample,
                "loopEndSample": loop_end_sample,
                "stemCapable": false,
                "artifact": audio_path,
                "note": "Real Rust-native procedural sample/loop generation. Full-song ML support remains blocked until a product-safe adapter, license, and runtime cache are verified.",
                "sourceEvidence": {
                    "stableAudio3": "https://stability.ai/news-updates/meet-stable-audio-3-the-model-family-built-for-artistic-experimentation-with-open-weight-models",
                    "aceStep15": "https://github.com/ace-step/ACE-Step-1.5",
                    "stableAudioOpen": "https://huggingface.co/stabilityai/stable-audio-open-1.0",
                    "diffRhythm2": "https://huggingface.co/ASLP-lab/DiffRhythm2",
                    "levo2": "https://github.com/tencent-ailab/SongGeneration",
                    "heartMuLa": "https://github.com/HeartMuLa/heartlib"
                }
            }),
        )?;
        job.status = JobStatus::Succeeded;
        job.progress = Some(JobProgress {
            percent: 100.0,
            message: Some("Native music generator wrote a real playable WAV.".to_string()),
        });
        job.cancellation = CancellationState::Completed;
        job.log_tail
            .push(format!("wrote {} frames of native music", frame_count));
        job.artifacts = vec![
            artifact(
                RuntimeArtifactKind::AudioPreview,
                &audio_path,
                "audio/wav",
                "Generated native sample/loop WAV",
            )?,
            artifact(
                RuntimeArtifactKind::OutputManifest,
                &manifest_path,
                "application/json",
                "Generated sample/loop manifest, BPM/key/loop metadata, and provenance",
            )?,
        ];
        Ok(())
    }

    fn write_error_report(&self, job: &mut RuntimeJobSnapshot) -> io::Result<()> {
        let path = sanitized_join(&self.root, &["jobs", &job.id, "error.json"])?;
        write_json(
            &path,
            &serde_json::json!({
                "jobId": job.id,
                "providerId": job.provider_id,
                "modelId": job.model_id,
                "error": job.actionable_error,
                "logTail": job.log_tail,
            }),
        )?;
        job.artifacts = vec![artifact(
            RuntimeArtifactKind::ErrorReport,
            &path,
            "application/json",
            "Actionable runtime error report",
        )?];
        Ok(())
    }

    fn write_job(&self, job: &RuntimeJobSnapshot) -> io::Result<()> {
        // F-002/F-030: recompute the write target from the validated job id rather
        // than trusting the persisted (possibly stale/untrusted) record_root string.
        let path = sanitized_join(&self.root, &["jobs", &job.id, "job.json"])?;
        write_json(path, job)
    }

    fn append_event(&self, job: &RuntimeJobSnapshot, event: &str, message: &str) -> io::Result<()> {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(sanitized_join(
                &self.root,
                &["jobs", &job.id, "events.jsonl"],
            )?)?;
        writeln!(
            file,
            "{}",
            serde_json::json!({
                "at": timestamp_string(),
                "jobId": job.id,
                "event": event,
                "status": job.status,
                "message": message,
            })
        )
    }

    fn validation_checks(&self) -> Vec<RuntimeValidationCheck> {
        if self.root.join("jobs").exists() {
            vec![RuntimeValidationCheck::passed(
                "runtime.job_store",
                "Runtime jobs are read from the durable local job store.",
            )]
        } else {
            vec![RuntimeValidationCheck::warning(
                "runtime.job_store",
                "No persisted runtime job store exists yet.",
                "Queue a generation job to create durable job, recipe, event, manifest, and artifact records.",
            )]
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeValidationCheck {
    pub id: String,
    pub status: ValidationStatus,
    pub summary: String,
    pub recovery: Option<String>,
}

impl RuntimeValidationCheck {
    fn passed(id: &str, summary: &str) -> Self {
        Self {
            id: id.to_string(),
            status: ValidationStatus::Passed,
            summary: summary.to_string(),
            recovery: None,
        }
    }

    fn warning(id: &str, summary: &str, recovery: &str) -> Self {
        Self {
            id: id.to_string(),
            status: ValidationStatus::Warning,
            summary: summary.to_string(),
            recovery: Some(recovery.to_string()),
        }
    }

    fn failed(id: &str, summary: String, recovery: &str) -> Self {
        Self {
            id: id.to_string(),
            status: ValidationStatus::Failed,
            summary,
            recovery: Some(recovery.to_string()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ValidationStatus {
    Passed,
    Warning,
    Failed,
}

fn model_requires_python(model: &ModelManifest) -> bool {
    model.requirements.dependencies.iter().any(|dependency| {
        let dependency = dependency.to_ascii_lowercase();
        dependency.contains("python") || dependency.contains("torch")
    })
}

impl RuntimeJobRequest {
    fn workflow_id_fragment(&self) -> &'static str {
        match self.workflow {
            CapabilityWorkflow::Tts => "tts",
            CapabilityWorkflow::VoiceClone => "voice-clone",
            CapabilityWorkflow::VoiceConversion => "voice-conversion",
            CapabilityWorkflow::Sfx => "sfx",
            CapabilityWorkflow::Ambience => "ambience",
            CapabilityWorkflow::InstrumentSample => "sample",
            CapabilityWorkflow::Loop => "loop",
            CapabilityWorkflow::Song => "song",
            CapabilityWorkflow::StemSeparation => "stem",
            CapabilityWorkflow::VideoToAudio => "video-audio",
            CapabilityWorkflow::Edit => "edit",
            CapabilityWorkflow::CompositionRender => "composition",
        }
    }
}

fn lane_to_workflow(lane: &EvaluationLane) -> Option<CapabilityWorkflow> {
    Some(match lane {
        EvaluationLane::Tts => CapabilityWorkflow::Tts,
        EvaluationLane::VoiceClone => CapabilityWorkflow::VoiceClone,
        EvaluationLane::VoiceConversion => CapabilityWorkflow::VoiceConversion,
        EvaluationLane::Sfx => CapabilityWorkflow::Sfx,
        EvaluationLane::Ambience => CapabilityWorkflow::Ambience,
        EvaluationLane::InstrumentSample => CapabilityWorkflow::InstrumentSample,
        EvaluationLane::Loop => CapabilityWorkflow::Loop,
        EvaluationLane::Song => CapabilityWorkflow::Song,
        EvaluationLane::StemSeparation => CapabilityWorkflow::StemSeparation,
        EvaluationLane::VideoToAudio => CapabilityWorkflow::VideoToAudio,
    })
}

fn provider_id_for_candidate(candidate: &ModelCandidateInstallState) -> String {
    candidate
        .provider
        .to_ascii_lowercase()
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character
            } else {
                '-'
            }
        })
        .collect::<String>()
        .split('-')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

fn adapter_for_model(
    overview: &RuntimeOverview,
    request: &RuntimeJobRequest,
) -> ProviderAdapterKind {
    // F-007: dispatch on the catalog's declared execution strategy, not on a
    // re-derived match over literal model ids.
    overview
        .model_states
        .iter()
        .find(|state| {
            state.provider_id == request.provider_id && state.model_id == request.model_id
        })
        .map(|state| state.execution_strategy.adapter_kind())
        .unwrap_or(ProviderAdapterKind::ResearchOnly)
}

/// Collect the voice-profile ids a request references, across the known parameter
/// keys. A built-in provider voice (e.g. Kokoro's `voice` param) is NOT a profile
/// and is intentionally excluded — it carries no consent obligation.
fn referenced_voice_profile_ids(request: &RuntimeJobRequest) -> Vec<String> {
    let mut ids = vec![];
    for key in [
        "voiceProfileIds",
        "voiceProfileId",
        "targetVoiceId",
        "sourceVoiceId",
    ] {
        match request.parameters.get(key) {
            Some(Value::String(value)) if !value.is_empty() => ids.push(value.clone()),
            Some(Value::Array(values)) => {
                for value in values {
                    if let Some(text) = value.as_str() {
                        if !text.is_empty() {
                            ids.push(text.to_string());
                        }
                    }
                }
            }
            _ => {}
        }
    }
    ids.sort();
    ids.dedup();
    ids
}

/// Policy-aware admission gate (F-003). Enforces, before any synthesis runs:
/// (1) voice consent resolved from the persisted voice profile — NOT a
///     caller-supplied boolean — for voice-identity workflows and any request
///     that references a voice profile; a `Prohibited` profile (public figure /
///     unauthorized) and any non-`ExplicitConsentRecorded` profile are rejected.
/// (2) model-use policy — a model whose `ModelUseDecision` is `Blocked`
///     (incompatible commercial-use / research-only / Python runtime) can never run.
fn validate_request_gates(request: &RuntimeJobRequest) -> Option<ActionableRuntimeError> {
    let referenced_profiles = referenced_voice_profile_ids(request);
    let consent_required = matches!(
        request.workflow,
        CapabilityWorkflow::VoiceClone | CapabilityWorkflow::VoiceConversion
    ) || !referenced_profiles.is_empty();

    if consent_required {
        if referenced_profiles.is_empty() {
            return Some(ActionableRuntimeError {
                code: "voice.consent_required".to_string(),
                summary: "Voice generation requires a consented voice profile".to_string(),
                recovery:
                    "Select a voice profile with explicit, recorded consent before running voice cloning or conversion."
                        .to_string(),
            });
        }
        for profile_id in &referenced_profiles {
            match crate::voice_lab::profile_consent(profile_id) {
                Some(VoiceConsentStatus::ExplicitConsentRecorded) => {}
                Some(VoiceConsentStatus::Prohibited) => {
                    return Some(ActionableRuntimeError {
                        code: "voice.consent_prohibited".to_string(),
                        summary: format!("Voice profile {profile_id} is prohibited for cloning"),
                        recovery:
                            "Public-figure or unauthorized voice references cannot be used. Create a new, consented voice profile."
                                .to_string(),
                    });
                }
                _ => {
                    return Some(ActionableRuntimeError {
                        code: "voice.consent_required".to_string(),
                        summary: format!("Voice profile {profile_id} has no recorded consent"),
                        recovery:
                            "Record explicit speaker consent on this voice profile before running the workflow."
                                .to_string(),
                    });
                }
            }
        }
    }

    let rights = RightsSafetyOverview::reference();
    if request.kind != JobKind::RenderComposition
        && NativeModel::from_model_id(&request.model_id).is_none()
    {
        let Some(decision) = rights
            .model_use_decisions
            .iter()
            .find(|decision| decision.candidate_id == request.model_id)
        else {
            return Some(ActionableRuntimeError {
                code: "model.use_unreviewed".to_string(),
                summary: format!(
                    "Model {} has no SoundWorks model-use decision",
                    request.model_id
                ),
                recovery:
                    "Add the model to the evaluated catalog and rights policy before runtime use."
                        .to_string(),
            });
        };

        if decision.decision == PolicyDecision::Blocked {
            return Some(ActionableRuntimeError {
                code: "model.use_blocked".to_string(),
                summary: format!(
                    "Model {} is not an allowed SoundWorks export model",
                    request.model_id
                ),
                recovery: if decision.reasons.is_empty() {
                    "Choose a model whose license and runtime are cleared for SoundWorks."
                        .to_string()
                } else {
                    decision.reasons.join(" ")
                },
            });
        }
    }

    None
}

fn kokoro_cache_root() -> PathBuf {
    if let Ok(root) = std::env::var("SOUNDWORKS_MODEL_CACHE") {
        return PathBuf::from(root).join("kokoro-82m");
    }
    std::env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("Library")
        .join("Application Support")
        .join("SoundWorks")
        .join("models")
        .join("kokoro-82m")
}

/// Move an in-flight job into the cancelled terminal state without committing any
/// output. The caller is responsible for persisting the snapshot and appending the
/// `cancelled` event afterwards.
fn mark_cancelled(job: &mut RuntimeJobSnapshot) {
    job.status = JobStatus::Cancelled;
    job.cancellation = CancellationState::Completed;
    job.progress = Some(JobProgress {
        percent: job
            .progress
            .as_ref()
            .map_or(0.0, |progress| progress.percent),
        message: Some("Worker cancelled the job before output was committed.".to_string()),
    });
    job.log_tail.push("cancelled by user request".to_string());
}

fn status_event(status: &JobStatus) -> &'static str {
    match status {
        JobStatus::Queued => "queued",
        JobStatus::Running => "running",
        JobStatus::Succeeded => "succeeded",
        JobStatus::Failed => "failed",
        JobStatus::Cancelled => "cancelled",
    }
}

fn artifact(
    kind: RuntimeArtifactKind,
    path: &Path,
    mime_type: &str,
    summary: &str,
) -> io::Result<RuntimeJobArtifact> {
    Ok(RuntimeJobArtifact {
        kind,
        path: path.display().to_string(),
        mime_type: mime_type.to_string(),
        bytes: fs::metadata(path)?.len(),
        summary: summary.to_string(),
    })
}

fn write_json(path: impl AsRef<Path>, value: &impl Serialize) -> io::Result<()> {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let payload = serde_json::to_vec_pretty(value).map_err(io::Error::other)?;
    // F-005: durable write — temp file + fsync + atomic rename (mirrors project_library).
    let mut temp_path = path.as_os_str().to_os_string();
    temp_path.push(".tmp");
    let temp_path = PathBuf::from(temp_path);
    {
        let mut file = fs::File::create(&temp_path)?;
        file.write_all(&payload)?;
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

fn read_json<T: for<'de> Deserialize<'de>>(path: impl AsRef<Path>) -> io::Result<T> {
    let bytes = fs::read(path)?;
    serde_json::from_slice(&bytes).map_err(io::Error::other)
}

fn timestamp_millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_millis())
}

/// F-035: process-wide monotonic counter appended to job ids so two jobs enqueued
/// in the same millisecond (or under a faulted clock reporting 0) never collide and
/// overwrite each other's job directory.
fn next_job_sequence() -> u64 {
    static JOB_SEQUENCE: AtomicU64 = AtomicU64::new(0);
    JOB_SEQUENCE.fetch_add(1, Ordering::Relaxed)
}

fn timestamp_string() -> String {
    timestamp_millis().to_string()
}

fn synthesize_native_sfx(
    prompt: &str,
    workflow: CapabilityWorkflow,
    duration_ms: u64,
    sample_rate: u32,
    intensity: f32,
    realism: f32,
    loopable: bool,
) -> Vec<f32> {
    let frame_count = (sample_rate as u64 * duration_ms / 1000).max(1) as usize;
    let mut samples = Vec::with_capacity(frame_count * 2);
    let mut noise = hash_prompt(prompt);
    let prompt_lower = prompt.to_ascii_lowercase();
    let ambience = loopable
        || matches!(workflow, CapabilityWorkflow::Ambience)
        || prompt_lower.contains("ambience")
        || prompt_lower.contains("bed");
    let metallic = prompt_lower.contains("metal") || prompt_lower.contains("hatch");
    let creature = prompt_lower.contains("creature") || prompt_lower.contains("growl");
    let weather = prompt_lower.contains("wind") || prompt_lower.contains("rain");
    let base_frequency = if creature {
        74.0
    } else if metallic {
        330.0
    } else if weather {
        120.0
    } else {
        180.0 + (noise as f32 % 240.0)
    };
    let secondary_frequency = base_frequency * if metallic { 2.74 } else { 1.51 };
    let amplitude = 0.16 + intensity * 0.24;
    let texture = 0.12 + (1.0 - realism) * 0.2;

    for frame in 0..frame_count {
        let t = frame as f32 / sample_rate as f32;
        let progress = frame as f32 / frame_count as f32;
        noise = lcg(noise);
        let white = ((noise >> 8) as f32 / u32::MAX as f32) * 2.0 - 1.0;
        let envelope = if ambience {
            let fade = (progress * 12.0)
                .min((1.0 - progress) * 12.0)
                .clamp(0.0, 1.0);
            0.55 + 0.45 * fade
        } else {
            (-progress * 9.0).exp()
        };
        let impact = if ambience {
            0.0
        } else {
            let transient = (-progress * 42.0).exp();
            white * transient * (0.65 + intensity * 0.35)
        };
        let tonal = (std::f32::consts::TAU * base_frequency * t).sin() * 0.52
            + (std::f32::consts::TAU * secondary_frequency * t).sin() * 0.24
            + (std::f32::consts::TAU * (base_frequency * 0.49) * t).sin() * 0.18;
        let rumble = (std::f32::consts::TAU * (base_frequency * 0.25) * t).sin() * 0.22;
        let bed = if ambience {
            tonal * 0.38 + rumble + white * texture
        } else {
            tonal * 0.3 + impact + white * texture * envelope
        };
        let pan = (std::f32::consts::TAU * 0.07 * t).sin() * 0.18;
        let left = (bed * envelope * amplitude * (1.0 - pan)).clamp(-0.95, 0.95);
        let right = (bed * envelope * amplitude * (1.0 + pan)).clamp(-0.95, 0.95);
        samples.push(left);
        samples.push(right);
    }

    samples
}

fn synthesize_native_music(
    prompt: &str,
    workflow: CapabilityWorkflow,
    duration_ms: u64,
    sample_rate: u32,
    bpm: f32,
    bars: u16,
    beats: u16,
    musical_key: &str,
    instrument_family: &str,
    articulation: &str,
    velocity: f32,
) -> Vec<f32> {
    let frame_count = (sample_rate as u64 * duration_ms / 1000).max(1) as usize;
    let mut samples = Vec::with_capacity(frame_count * 2);
    let mut noise = hash_prompt(&format!(
        "{prompt}|{musical_key}|{instrument_family}|{articulation}"
    ));
    let root = key_frequency(musical_key);
    let minor = musical_key.to_ascii_lowercase().contains("minor");
    let scale = if minor {
        [0.0, 3.0, 5.0, 7.0, 10.0, 12.0, 15.0, 17.0]
    } else {
        [0.0, 4.0, 7.0, 9.0, 12.0, 16.0, 19.0, 21.0]
    };
    let beat_duration = 60.0 / bpm.max(1.0);
    let step_duration = (beat_duration / 2.0).max(0.05);
    let total_steps = (bars.max(1) as usize * beats.max(1) as usize * 2).max(1);
    let prompt_lower = prompt.to_ascii_lowercase();
    let bright = prompt_lower.contains("lead")
        || prompt_lower.contains("pluck")
        || instrument_family.to_ascii_lowercase().contains("keys");
    let bass =
        prompt_lower.contains("bass") || instrument_family.to_ascii_lowercase().contains("bass");
    let one_shot = matches!(workflow, CapabilityWorkflow::InstrumentSample);
    let amplitude = 0.16 + velocity.clamp(0.0, 1.0) * 0.24;

    for frame in 0..frame_count {
        let t = frame as f32 / sample_rate as f32;
        let progress = frame as f32 / frame_count as f32;
        let step = ((t / step_duration).floor() as usize).min(total_steps.saturating_sub(1));
        let degree = scale[(step + (noise as usize % scale.len())) % scale.len()];
        let octave = if bass {
            0.5
        } else if bright {
            2.0
        } else {
            1.0
        };
        let freq = root * octave * 2.0_f32.powf(degree / 12.0);
        let beat_phase = (t % step_duration) / step_duration;
        let gate = if one_shot {
            (-progress * 9.0).exp()
        } else {
            let attack = (beat_phase * 18.0).clamp(0.0, 1.0);
            let release = (1.0 - beat_phase).clamp(0.0, 1.0).powf(0.7);
            (attack * release).max(0.08)
        };
        noise = lcg(noise);
        let white = ((noise >> 8) as f32 / u32::MAX as f32) * 2.0 - 1.0;
        let fundamental = (std::f32::consts::TAU * freq * t).sin();
        let harmonic = (std::f32::consts::TAU * freq * 2.0 * t).sin() * 0.32;
        let sub = (std::f32::consts::TAU * freq * 0.5 * t).sin() * if bass { 0.42 } else { 0.16 };
        let click = if one_shot || step % 4 == 0 {
            white * (-beat_phase * 38.0).exp() * 0.18
        } else {
            0.0
        };
        let tone = (fundamental + harmonic + sub + click) * gate * amplitude;
        let shape = if one_shot {
            tone.tanh()
        } else {
            let bar_position = step as f32 / total_steps as f32;
            let phrase = 0.82 + 0.18 * (std::f32::consts::TAU * bar_position).sin();
            (tone * phrase).tanh()
        };
        let pan = (std::f32::consts::TAU * 0.11 * t).sin() * if one_shot { 0.04 } else { 0.16 };
        samples.push((shape * (1.0 - pan)).clamp(-0.95, 0.95));
        samples.push((shape * (1.0 + pan)).clamp(-0.95, 0.95));
    }

    samples
}

fn loop_duration_ms(bpm: f32, bars: u16, beats: u16) -> u64 {
    let beat_count = bars.max(1) as f32 * beats.max(1) as f32;
    ((60_000.0 / bpm.max(1.0)) * beat_count).round() as u64
}

fn key_frequency(musical_key: &str) -> f32 {
    let key = musical_key
        .split_whitespace()
        .next()
        .unwrap_or("A")
        .to_ascii_uppercase();
    match key.as_str() {
        "C" | "C1" => 32.70,
        "C#" | "DB" => 34.65,
        "D" | "D1" => 36.71,
        "D#" | "EB" => 38.89,
        "E" | "E1" => 41.20,
        "F" | "F1" => 43.65,
        "F#" | "GB" => 46.25,
        "G" | "G1" => 49.00,
        "G#" | "AB" => 51.91,
        "A" | "A1" => 55.00,
        "A#" | "BB" => 58.27,
        "B" | "B1" => 61.74,
        _ => 55.00,
    }
}

fn hash_prompt(prompt: &str) -> u32 {
    prompt.bytes().fold(0x811c_9dc5, |hash, byte| {
        hash.wrapping_mul(16_777_619) ^ u32::from(byte)
    })
}

fn lcg(seed: u32) -> u32 {
    seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223)
}

fn write_smoke_wav(path: &Path) -> io::Result<()> {
    let sample_rate = 16_000u32;
    let sample_count = sample_rate / 4;
    let samples: Vec<i16> = (0..sample_count)
        .map(|index| {
            let phase = (index as f32 / sample_rate as f32) * 440.0 * std::f32::consts::TAU;
            (phase.sin() * i16::MAX as f32 * 0.18) as i16
        })
        .collect();
    encode_pcm16_wav(path, sample_rate, 1, &samples)
}

fn write_pcm16_wav(path: &Path, samples: &[f32], sample_rate: u32) -> io::Result<()> {
    write_pcm16_wav_channels(path, samples, sample_rate, 1)
}

fn write_pcm16_wav_channels(
    path: &Path,
    samples: &[f32],
    sample_rate: u32,
    channels: u16,
) -> io::Result<()> {
    let pcm: Vec<i16> = samples
        .iter()
        .map(|sample| (sample.clamp(-1.0, 1.0) * i16::MAX as f32) as i16)
        .collect();
    encode_pcm16_wav(path, sample_rate, channels, &pcm)
}

/// Encode interleaved 16-bit PCM `samples` into a canonical RIFF/WAVE file
/// (format tag 1, 16-bit). Chunk sizes are computed in `u64` and validated to
/// fit the `u32` wire fields, so an oversized buffer returns an error instead of
/// silently truncating into a corrupt header; `samples` must also be a whole
/// number of frames for `channels`.
fn encode_pcm16_wav(
    path: &Path,
    sample_rate: u32,
    channels: u16,
    samples: &[i16],
) -> io::Result<()> {
    if channels == 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "wav channel count must be non-zero",
        ));
    }
    if !samples.len().is_multiple_of(usize::from(channels)) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "sample buffer is not a whole number of frames",
        ));
    }
    let data_bytes = (samples.len() as u64)
        .checked_mul(2)
        .filter(|&bytes| 36 + bytes <= u32::MAX as u64)
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "wav payload exceeds the u32 RIFF chunk size",
            )
        })?;
    let data_bytes = data_bytes as u32;
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

/// First powered-on device in the inventory, used to pick the runtime
/// accelerator and report available memory for native model state.
fn first_available_device(inventory: &DeviceInventory) -> Option<&DeviceReport> {
    inventory.devices.iter().find(|device| device.available)
}

#[cfg(test)]
mod tests {
    use super::{
        CacheStatus, CancellationState, DeviceInventory, ExecutionStrategy, LicenseAcceptanceState,
        ModelCacheState, NativeModel, ProviderAdapterKind, RuntimeArtifactKind,
        RuntimeAvailability, RuntimeCompatibility, RuntimeEngine, RuntimeHealth, RuntimeJobRequest,
        RuntimeJobStore, RuntimeOverview, RuntimePackagingPolicy, ValidationStatus, WarmupStatus,
    };
    use crate::domain::{JobKind, ModelRuntime};
    use crate::manifests::{CapabilityWorkflow, ModelInstallStatus, ProviderCatalog};
    use crate::CandidateInstallState;
    use std::collections::BTreeMap;
    use std::fs;
    use std::path::PathBuf;
    use std::sync::{Arc, Mutex};

    static CONSENT_ENV_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn reference_runtime_blocks_manifest_only_models_and_jobs() {
        let runtime = RuntimeOverview::reference();

        assert_eq!(runtime.schema_version, super::RUNTIME_SCHEMA_VERSION);
        assert_eq!(runtime.status_counts.installed, 0);
        assert_eq!(runtime.status_counts.available, 0);
        assert_eq!(runtime.status_counts.unavailable, 4);
        assert!(runtime
            .validation_checks
            .iter()
            .any(|check| check.id == "runtime.no_python" && check.recovery.is_none()));
        assert!(runtime.validation_checks.iter().any(|check| {
            check.id == "runtime.cache_evidence"
                && check.status == ValidationStatus::Failed
                && check.summary.contains("Manifest-only")
        }));
        assert!(runtime.jobs.is_empty());
        assert!(runtime.model_states.iter().all(|state| {
            state.cache.disk_usage_mb.is_none()
                && !state.cache.verified
                && state.cache.evidence.contains("manifest-only")
                && state.availability == RuntimeAvailability::Unavailable
        }));
    }

    #[test]
    fn shipped_runtime_policy_rejects_python_dependency() {
        let mut catalog = ProviderCatalog::reference();
        catalog.providers[0].models[0].requirements.dependencies =
            vec!["python>=3.11".to_string(), "torch".to_string()];

        let runtime = RuntimeOverview::from_catalog(
            &catalog,
            &DeviceInventory::reference_mac(),
            RuntimePackagingPolicy::shipped_desktop(),
        );

        let speech_state = runtime
            .model_states
            .iter()
            .find(|state| state.model_id == "reference-speech-suite")
            .expect("speech state");

        assert_eq!(speech_state.availability, RuntimeAvailability::Unavailable);
        assert!(runtime.validation_checks.iter().any(|check| {
            check.id == "runtime.no_python" && check.summary.contains("reference-speech-suite")
        }));
    }

    #[test]
    fn runtime_distinguishes_available_and_unavailable_models() {
        let mut catalog = ProviderCatalog::reference();
        catalog.providers[0].models[0].install.status = ModelInstallStatus::Installable;
        catalog.providers[0].models[1].runtime = ModelRuntime::ResearchOnly;

        let runtime = RuntimeOverview::from_catalog(
            &catalog,
            &DeviceInventory::reference_mac(),
            RuntimePackagingPolicy::shipped_desktop(),
        );

        assert_eq!(
            runtime.model_states[0].availability,
            RuntimeAvailability::Available
        );
        assert_eq!(
            runtime.model_states[1].availability,
            RuntimeAvailability::Unavailable
        );
        assert_eq!(runtime.status_counts.available, 1);
        assert_eq!(runtime.status_counts.unavailable, 3);
    }

    #[test]
    fn runtime_ignores_verified_cache_for_non_product_candidates() {
        let cache_root = temp_runtime_root("non-product-cache");
        let stable = cache_root.join("stable-audio-3");
        fs::create_dir_all(&stable).expect("create stable cache");
        fs::write(stable.join("README.md"), "stable audio").expect("write readme");
        fs::write(stable.join("model.safetensors"), "weights").expect("write weights");
        let manager = crate::ModelManagerOverview::from_catalog(
            &crate::ModelEvaluationCatalog::reference(),
            cache_root,
        );
        let candidate = manager
            .candidates
            .iter()
            .find(|candidate| candidate.candidate_id == "stable-audio-3")
            .expect("stable audio candidate");
        assert!(candidate.cache.verified);
        assert_ne!(candidate.install_state, CandidateInstallState::Installed);

        let store = RuntimeJobStore::new(temp_runtime_root("non-product-runtime"));
        let runtime = RuntimeOverview::from_model_manager(
            &manager,
            &DeviceInventory::reference_mac(),
            RuntimePackagingPolicy::shipped_desktop(),
            &store,
        );

        assert!(runtime
            .model_states
            .iter()
            .all(|state| state.model_id != "stable-audio-3"));
    }

    #[test]
    fn runtime_blocks_job_admission_for_unavailable_model() {
        let mut catalog = ProviderCatalog::reference();
        catalog.providers[0].models[0].runtime = ModelRuntime::ResearchOnly;
        let runtime = RuntimeOverview::from_catalog(
            &catalog,
            &DeviceInventory::reference_mac(),
            RuntimePackagingPolicy::shipped_desktop(),
        );

        let admission = runtime.admit_job(
            "soundworks-reference",
            "reference-speech-suite",
            JobKind::GenerateAudio,
        );

        assert!(!admission.accepted);
        assert!(admission.actionable_error.is_some());
    }

    #[test]
    fn runtime_does_not_fabricate_reference_jobs_without_verified_cache() {
        let runtime = RuntimeOverview::reference();

        assert!(runtime
            .cancel_job("job-runtime-reference-generate")
            .is_none());
    }

    #[test]
    fn runtime_job_store_executes_and_persists_artifacts() {
        let store = RuntimeJobStore::new(temp_runtime_root("success"));
        let overview = installed_overview(&store);

        let job = store
            .enqueue_and_run(
                &engine(),
                &overview,
                RuntimeJobRequest {
                    provider_id: "hexgrad".to_string(),
                    model_id: "native-smoke".to_string(),
                    kind: JobKind::GenerateAudio,
                    workflow: CapabilityWorkflow::Tts,
                    prompt: "Runtime smoke read".to_string(),
                    source_surface: "TTS Studio".to_string(),
                    parameters: BTreeMap::new(),
                },
            )
            .expect("enqueue succeeds");

        assert_eq!(job.status, crate::domain::JobStatus::Succeeded);
        assert_eq!(job.adapter, ProviderAdapterKind::NativeRust);
        let job_record_root = store.root().join(&job.record_root);
        assert!(job_record_root.join("recipe.json").is_file());
        assert!(job_record_root.join("model.json").is_file());
        assert!(job_record_root.join("events.jsonl").is_file());
        assert!(job.artifacts.iter().any(|artifact| {
            artifact.kind == RuntimeArtifactKind::AudioPreview
                && artifact.bytes > 44
                && PathBuf::from(&artifact.path).is_file()
        }));
        assert!(store
            .read_jobs()
            .expect("read jobs")
            .iter()
            .any(|stored| stored.id == job.id));
    }

    #[test]
    fn composition_render_job_writes_mixdown_wav() {
        // UX-NB1: a render-composition job bypasses model admission and runs the
        // offline mixer. With no resolvable clips in an isolated runtime root it
        // still renders a correctly-sized silent WAV, proving the admission
        // bypass + adapter branch + WAV write (mixing math is unit tested in
        // composition_mixdown).
        let store = RuntimeJobStore::new(temp_runtime_root("mixdown"));
        let overview = installed_overview(&store);
        let mut parameters = BTreeMap::new();
        parameters.insert("durationMs".to_string(), serde_json::json!(500));
        parameters.insert("sampleRateHz".to_string(), serde_json::json!(48_000));
        parameters.insert("channels".to_string(), serde_json::json!(2));
        parameters.insert("masterGainDb".to_string(), serde_json::json!(0.0));
        parameters.insert("clips".to_string(), serde_json::json!([]));

        let job = store
            .enqueue_and_run(
                &engine(),
                &overview,
                RuntimeJobRequest {
                    provider_id: "soundworks-native".to_string(),
                    model_id: "composition-mixdown".to_string(),
                    kind: JobKind::RenderComposition,
                    workflow: CapabilityWorkflow::CompositionRender,
                    prompt: "Render demo timeline".to_string(),
                    source_surface: "Multitrack".to_string(),
                    parameters,
                },
            )
            .expect("enqueue succeeds");

        assert_eq!(job.status, crate::domain::JobStatus::Succeeded);
        assert!(job.artifacts.iter().any(|artifact| {
            artifact.kind == RuntimeArtifactKind::AudioPreview
                && artifact.bytes > 44
                && PathBuf::from(&artifact.path).is_file()
        }));
    }

    #[test]
    fn composition_render_fails_when_all_requested_clips_are_unresolved() {
        let store = RuntimeJobStore::new(temp_runtime_root("mixdown-unresolved"));
        let overview = installed_overview(&store);
        let mut parameters = BTreeMap::new();
        parameters.insert("durationMs".to_string(), serde_json::json!(500));
        parameters.insert("sampleRateHz".to_string(), serde_json::json!(48_000));
        parameters.insert("channels".to_string(), serde_json::json!(2));
        parameters.insert("masterGainDb".to_string(), serde_json::json!(0.0));
        parameters.insert(
            "clips".to_string(),
            serde_json::json!([
                {
                    "assetId": "missing-library-clip",
                    "timelineStartMs": 0,
                    "sourceStartMs": 0,
                    "sourceEndMs": 500,
                    "gainDb": 0.0,
                    "pan": 0.0
                }
            ]),
        );

        let job = store
            .enqueue_and_run(
                &engine(),
                &overview,
                RuntimeJobRequest {
                    provider_id: "soundworks-native".to_string(),
                    model_id: "composition-mixdown".to_string(),
                    kind: JobKind::RenderComposition,
                    workflow: CapabilityWorkflow::CompositionRender,
                    prompt: "Render missing clip timeline".to_string(),
                    source_surface: "Multitrack".to_string(),
                    parameters,
                },
            )
            .expect("enqueue succeeds");

        assert_eq!(job.status, crate::domain::JobStatus::Failed);
        assert_eq!(
            job.actionable_error
                .as_ref()
                .map(|error| error.code.as_str()),
            Some("composition.unresolved_clips")
        );
        assert!(job.artifacts.iter().any(|artifact| {
            artifact.kind == RuntimeArtifactKind::ErrorReport
                && PathBuf::from(&artifact.path).is_file()
        }));
        assert!(job
            .log_tail
            .iter()
            .any(|line| line.contains("missing-library-clip")));
    }

    #[test]
    fn runtime_job_store_rejects_traversal_job_id() {
        // F-004: cancel/retry/artifacts all funnel through read_job, which now runs
        // the caller-supplied job_id through sanitized_join. A traversal id must be
        // rejected (Err), while a valid-but-missing id resolves to Ok(None).
        let store = RuntimeJobStore::new(temp_runtime_root("traversal"));
        let overview = installed_overview(&store);
        for malicious in ["../../etc/passwd", "..", "jobs/../escape", "with space", ""] {
            assert!(
                store.cancel(malicious).is_err(),
                "cancel must reject job_id {malicious:?}"
            );
            assert!(
                store.artifacts(malicious).is_err(),
                "artifacts must reject job_id {malicious:?}"
            );
            assert!(
                store.retry(&overview, malicious).is_err(),
                "retry must reject job_id {malicious:?}"
            );
        }
        // A clean id that does not exist is not an error, just absent.
        assert!(store
            .cancel("job-tts-0-0")
            .expect("clean id resolves")
            .is_none());
    }

    #[test]
    fn runtime_job_store_persists_actionable_failure() {
        let store = RuntimeJobStore::new(temp_runtime_root("failure"));
        let overview = RuntimeOverview::reference();

        let job = store
            .enqueue(
                &overview,
                RuntimeJobRequest {
                    provider_id: "missing-provider".to_string(),
                    model_id: "missing-model".to_string(),
                    kind: JobKind::GenerateAudio,
                    workflow: CapabilityWorkflow::Sfx,
                    prompt: "Missing runtime".to_string(),
                    source_surface: "SFX Studio".to_string(),
                    parameters: BTreeMap::new(),
                },
            )
            .expect("failed job persists");

        assert_eq!(job.status, crate::domain::JobStatus::Failed);
        assert!(job.actionable_error.is_some());
        assert!(job.artifacts.iter().any(|artifact| {
            artifact.kind == RuntimeArtifactKind::ErrorReport
                && PathBuf::from(&artifact.path).is_file()
        }));
    }

    #[test]
    fn kokoro_tts_adapter_requires_real_cache_files() {
        let store = RuntimeJobStore::new(temp_runtime_root("kokoro-missing"));
        let overview = kokoro_overview(&store, temp_runtime_root("missing-cache"));
        let mut parameters = BTreeMap::new();
        parameters.insert(
            "cachePath".to_string(),
            serde_json::json!(temp_runtime_root("missing-cache")),
        );
        parameters.insert("voice".to_string(), serde_json::json!("af_heart"));

        let job = store
            .enqueue_and_run(
                &engine(),
                &overview,
                RuntimeJobRequest {
                    provider_id: "hexgrad".to_string(),
                    model_id: "kokoro-82m".to_string(),
                    kind: JobKind::GenerateAudio,
                    workflow: CapabilityWorkflow::Tts,
                    prompt: "Real speech requires cache files.".to_string(),
                    source_surface: "TTS Studio".to_string(),
                    parameters,
                },
            )
            .expect("failed Kokoro job persists");

        assert_eq!(job.status, crate::domain::JobStatus::Failed);
        assert_eq!(
            job.actionable_error
                .as_ref()
                .map(|error| error.code.as_str()),
            Some("tts.kokoro_cache_missing")
        );
    }

    #[test]
    #[ignore = "requires local Kokoro ONNX model and voice files in SOUNDWORKS_MODEL_CACHE or the default SoundWorks model cache"]
    fn kokoro_tts_adapter_generates_real_speech_from_local_cache() {
        let store = RuntimeJobStore::new(temp_runtime_root("kokoro-real"));
        let cache_root = super::kokoro_cache_root();
        let overview = kokoro_overview(&store, cache_root.clone());
        let mut parameters = BTreeMap::new();
        parameters.insert(
            "cachePath".to_string(),
            serde_json::json!(cache_root.display().to_string()),
        );
        parameters.insert("voice".to_string(), serde_json::json!("af_heart"));
        parameters.insert("language".to_string(), serde_json::json!("en-US"));
        parameters.insert("speakerLabels".to_string(), serde_json::json!(["Narrator"]));
        parameters.insert(
            "voiceProfileIds".to_string(),
            serde_json::json!(["voice-profile-narrator"]),
        );
        parameters.insert("voiceConsentRecorded".to_string(), serde_json::json!(true));

        let job = store
            .enqueue_and_run(
                &engine(),
                &overview,
                RuntimeJobRequest {
                    provider_id: "hexgrad".to_string(),
                    model_id: "kokoro-82m".to_string(),
                    kind: JobKind::GenerateAudio,
                    workflow: CapabilityWorkflow::Tts,
                    prompt: "SoundWorks generated this speech from Kokoro.".to_string(),
                    source_surface: "TTS Studio".to_string(),
                    parameters,
                },
            )
            .expect("Kokoro job runs");

        assert_eq!(job.status, crate::domain::JobStatus::Succeeded);
        assert!(job.artifacts.iter().any(|artifact| {
            artifact.kind == RuntimeArtifactKind::AudioPreview
                && artifact.summary.contains("Generated Kokoro speech")
                && artifact.bytes > 44
                && PathBuf::from(&artifact.path).is_file()
        }));

        let library = crate::ProjectLibraryStore::new(temp_runtime_root("kokoro-library"));
        let saved = library
            .import_runtime_artifact_from_store(
                crate::ImportRuntimeArtifactRequest {
                    job_id: job.id.clone(),
                    project_id: None,
                    name: Some("Kokoro real speech smoke".to_string()),
                    scope: None,
                    tags: vec!["tts".to_string(), "generated-speech".to_string()],
                },
                &store,
            )
            .expect("generated speech imports into the project library");
        assert_eq!(
            saved
                .asset_library
                .selected_item
                .as_ref()
                .unwrap()
                .item
                .item_type,
            crate::asset_library::LibraryItemType::VoiceClip
        );
        let playback = library
            .playback_for_item(&saved.asset_library.selected_item.as_ref().unwrap().item.id)
            .expect("playback check works");
        assert!(playback.playable);
    }

    #[test]
    fn native_sfx_adapter_generates_playable_loopable_asset() {
        let store = RuntimeJobStore::new(temp_runtime_root("native-sfx"));
        let overview = native_sfx_overview(&store);
        let mut parameters = BTreeMap::new();
        parameters.insert("durationMs".to_string(), serde_json::json!(1_500));
        parameters.insert("loopable".to_string(), serde_json::json!(true));
        parameters.insert("category".to_string(), serde_json::json!("ambience-bed"));
        parameters.insert("intensity".to_string(), serde_json::json!(72));
        parameters.insert("realism".to_string(), serde_json::json!(64));
        parameters.insert(
            "tags".to_string(),
            serde_json::json!(["foley", "engine-room", "loopable"]),
        );

        let job = store
            .enqueue_and_run(
                &engine(),
                &overview,
                RuntimeJobRequest {
                    provider_id: "soundworks-native".to_string(),
                    model_id: "native-procedural-sfx".to_string(),
                    kind: JobKind::GenerateAudio,
                    workflow: CapabilityWorkflow::Sfx,
                    prompt: "Close metallic hatch impact into engine room ambience.".to_string(),
                    source_surface: "SFX Studio".to_string(),
                    parameters,
                },
            )
            .expect("native SFX job runs");

        assert_eq!(job.status, crate::domain::JobStatus::Succeeded);
        assert!(job.artifacts.iter().any(|artifact| {
            artifact.kind == RuntimeArtifactKind::AudioPreview
                && artifact.summary.contains("Generated native SFX")
                && artifact.bytes > 44
                && PathBuf::from(&artifact.path).is_file()
        }));

        let library = crate::ProjectLibraryStore::new(temp_runtime_root("native-sfx-library"));
        let saved = library
            .import_runtime_artifact_from_store(
                crate::ImportRuntimeArtifactRequest {
                    job_id: job.id.clone(),
                    project_id: None,
                    name: Some("Native generated hatch ambience".to_string()),
                    scope: None,
                    tags: vec!["sfx".to_string(), "loopable".to_string()],
                },
                &store,
            )
            .expect("generated SFX imports into the project library");
        assert_eq!(
            saved
                .asset_library
                .selected_item
                .as_ref()
                .unwrap()
                .item
                .item_type,
            crate::asset_library::LibraryItemType::Sfx
        );
        let technical = &saved
            .asset_library
            .selected_item
            .as_ref()
            .unwrap()
            .item
            .current_version
            .as_ref()
            .expect("current version")
            .technical;
        assert_eq!(technical.sample_rate_hz, 48_000);
        assert_eq!(technical.channels, 2);
        assert!(technical.duration_ms >= 1_500);
        assert!(technical.loop_points.is_some());
        assert!(technical.loudness_lufs.is_some());

        let playback = library
            .playback_for_item(&saved.asset_library.selected_item.as_ref().unwrap().item.id)
            .expect("playback check works");
        assert!(playback.playable);
    }

    #[test]
    fn native_music_adapter_generates_playable_loop_with_bpm_key_metadata() {
        let store = RuntimeJobStore::new(temp_runtime_root("native-music-loop"));
        let overview = native_music_overview(&store);
        let mut parameters = BTreeMap::new();
        parameters.insert("bpm".to_string(), serde_json::json!(120.0));
        parameters.insert("bars".to_string(), serde_json::json!(4));
        parameters.insert("beats".to_string(), serde_json::json!(4));
        parameters.insert("loopable".to_string(), serde_json::json!(true));
        parameters.insert("musicalKey".to_string(), serde_json::json!("A minor"));
        parameters.insert(
            "instrumentFamily".to_string(),
            serde_json::json!("synth-bass"),
        );
        parameters.insert(
            "articulation".to_string(),
            serde_json::json!("pulsed sequence"),
        );
        parameters.insert("velocityEnergy".to_string(), serde_json::json!(76));
        parameters.insert(
            "tags".to_string(),
            serde_json::json!(["loop", "synthwave", "bass"]),
        );

        let job = store
            .enqueue_and_run(
                &engine(),
                &overview,
                RuntimeJobRequest {
                    provider_id: "soundworks-native".to_string(),
                    model_id: "native-procedural-music".to_string(),
                    kind: JobKind::GenerateAudio,
                    workflow: CapabilityWorkflow::Loop,
                    prompt: "Tight analog synth bass four-bar loop for a neon chase cue."
                        .to_string(),
                    source_surface: "Samples + Loops".to_string(),
                    parameters,
                },
            )
            .expect("native music loop job runs");

        assert_eq!(job.status, crate::domain::JobStatus::Succeeded);
        assert!(job.artifacts.iter().any(|artifact| {
            artifact.kind == RuntimeArtifactKind::AudioPreview
                && artifact.summary.contains("Generated native sample/loop")
                && artifact.bytes > 44
                && PathBuf::from(&artifact.path).is_file()
        }));

        let library = crate::ProjectLibraryStore::new(temp_runtime_root("native-music-library"));
        let saved = library
            .import_runtime_artifact_from_store(
                crate::ImportRuntimeArtifactRequest {
                    job_id: job.id.clone(),
                    project_id: None,
                    name: Some("Native generated synth bass loop".to_string()),
                    scope: None,
                    tags: vec![
                        "loop".to_string(),
                        "generated-audio".to_string(),
                        "synthwave".to_string(),
                    ],
                },
                &store,
            )
            .expect("generated loop imports into the project library");
        assert_eq!(
            saved
                .asset_library
                .selected_item
                .as_ref()
                .unwrap()
                .item
                .item_type,
            crate::asset_library::LibraryItemType::Loop
        );
        let technical = &saved
            .asset_library
            .selected_item
            .as_ref()
            .unwrap()
            .item
            .current_version
            .as_ref()
            .expect("current version")
            .technical;
        assert_eq!(technical.sample_rate_hz, 48_000);
        assert_eq!(technical.channels, 2);
        assert_eq!(technical.bpm, Some(120.0));
        assert_eq!(technical.musical_key.as_deref(), Some("A minor"));
        assert!(technical.loop_points.is_some());
        assert!(technical.loudness_lufs.is_some());

        let playback = library
            .playback_for_item(&saved.asset_library.selected_item.as_ref().unwrap().item.id)
            .expect("playback check works");
        assert!(playback.playable);
    }

    #[test]
    fn native_music_adapter_generates_playable_instrument_sample() {
        let store = RuntimeJobStore::new(temp_runtime_root("native-music-sample"));
        let overview = native_music_overview(&store);
        let mut parameters = BTreeMap::new();
        parameters.insert("durationMs".to_string(), serde_json::json!(700));
        parameters.insert("loopable".to_string(), serde_json::json!(false));
        parameters.insert("musicalKey".to_string(), serde_json::json!("C2"));
        parameters.insert(
            "instrumentFamily".to_string(),
            serde_json::json!("synth-bass"),
        );
        parameters.insert("articulation".to_string(), serde_json::json!("short stab"));
        parameters.insert("velocityEnergy".to_string(), serde_json::json!(84));

        let job = store
            .enqueue_and_run(
                &engine(),
                &overview,
                RuntimeJobRequest {
                    provider_id: "soundworks-native".to_string(),
                    model_id: "native-procedural-music".to_string(),
                    kind: JobKind::GenerateAudio,
                    workflow: CapabilityWorkflow::InstrumentSample,
                    prompt: "Short synth bass stab one-shot with a clean transient.".to_string(),
                    source_surface: "Samples + Loops".to_string(),
                    parameters,
                },
            )
            .expect("native music sample job runs");

        assert_eq!(job.status, crate::domain::JobStatus::Succeeded);
        assert!(job.artifacts.iter().any(|artifact| {
            artifact.kind == RuntimeArtifactKind::AudioPreview
                && artifact.bytes > 44
                && PathBuf::from(&artifact.path).is_file()
        }));

        let library = crate::ProjectLibraryStore::new(temp_runtime_root("native-sample-library"));
        let saved = library
            .import_runtime_artifact_from_store(
                crate::ImportRuntimeArtifactRequest {
                    job_id: job.id.clone(),
                    project_id: None,
                    name: Some("Native generated synth bass stab".to_string()),
                    scope: None,
                    tags: vec![
                        "instrument-sample".to_string(),
                        "generated-audio".to_string(),
                    ],
                },
                &store,
            )
            .expect("generated sample imports into the project library");
        assert_eq!(
            saved
                .asset_library
                .selected_item
                .as_ref()
                .unwrap()
                .item
                .item_type,
            crate::asset_library::LibraryItemType::InstrumentSample
        );
        assert!(saved
            .asset_library
            .selected_item
            .as_ref()
            .unwrap()
            .item
            .current_version
            .as_ref()
            .expect("current version")
            .technical
            .loop_points
            .is_none());
    }

    #[test]
    fn voice_conversion_runtime_request_requires_explicit_consent() {
        let store = RuntimeJobStore::new(temp_runtime_root("voice-consent"));
        let overview = installed_overview(&store);

        let job = store
            .enqueue(
                &overview,
                RuntimeJobRequest {
                    provider_id: "hexgrad".to_string(),
                    model_id: "native-smoke".to_string(),
                    kind: JobKind::GenerateAudio,
                    workflow: CapabilityWorkflow::VoiceConversion,
                    prompt: "Convert this read into a target voice.".to_string(),
                    source_surface: "Voice Lab".to_string(),
                    parameters: BTreeMap::new(),
                },
            )
            .expect("consent-blocked job persists");

        assert_eq!(job.status, crate::domain::JobStatus::Failed);
        assert_eq!(
            job.actionable_error
                .as_ref()
                .map(|error| error.code.as_str()),
            Some("voice.consent_required")
        );
    }

    #[test]
    fn consent_boolean_no_longer_bypasses_the_gate() {
        // F-003: even when the caller self-asserts consent, a voice workflow with no
        // consented profile is rejected — the boolean is ignored entirely.
        let store = RuntimeJobStore::new(temp_runtime_root("consent-boolean"));
        let overview = installed_overview(&store);
        let mut parameters = BTreeMap::new();
        parameters.insert("voiceConsentRecorded".to_string(), serde_json::json!(true));
        let job = store
            .enqueue(
                &overview,
                RuntimeJobRequest {
                    provider_id: "hexgrad".to_string(),
                    model_id: "native-smoke".to_string(),
                    kind: JobKind::GenerateAudio,
                    workflow: CapabilityWorkflow::VoiceConversion,
                    prompt: "Convert this read into a target voice.".to_string(),
                    source_surface: "Voice Lab".to_string(),
                    parameters,
                },
            )
            .expect("job persists");
        assert_eq!(job.status, crate::domain::JobStatus::Failed);
        assert_eq!(
            job.actionable_error
                .as_ref()
                .map(|error| error.code.as_str()),
            Some("voice.consent_required")
        );
    }

    #[test]
    fn profile_referencing_job_is_blocked_without_recorded_consent() {
        // A profile in review (voice-profile-archival = RequiresReview) blocks even a
        // TTS job that references it.
        with_isolated_consent_root("consent-archival-root", || {
            let store = RuntimeJobStore::new(temp_runtime_root("consent-archival"));
            let overview = installed_overview(&store);
            let mut parameters = BTreeMap::new();
            parameters.insert(
                "voiceProfileId".to_string(),
                serde_json::json!("voice-profile-archival"),
            );
            let job = store
                .enqueue(
                    &overview,
                    RuntimeJobRequest {
                        provider_id: "hexgrad".to_string(),
                        model_id: "native-smoke".to_string(),
                        kind: JobKind::GenerateAudio,
                        workflow: CapabilityWorkflow::Tts,
                        prompt: "Read in the archival voice.".to_string(),
                        source_surface: "TTS Studio".to_string(),
                        parameters,
                    },
                )
                .expect("job persists");
            assert_eq!(job.status, crate::domain::JobStatus::Failed);
            assert_eq!(
                job.actionable_error
                    .as_ref()
                    .map(|error| error.code.as_str()),
                Some("voice.consent_required")
            );
        });
    }

    #[test]
    fn profile_referencing_job_runs_with_explicit_consent() {
        // The consented narrator profile (ExplicitConsentRecorded) passes the gate,
        // so the job runs to completion.
        with_isolated_consent_root("consent-narrator-root", || {
            let store = RuntimeJobStore::new(temp_runtime_root("consent-narrator"));
            let overview = installed_overview(&store);
            let mut parameters = BTreeMap::new();
            parameters.insert(
                "voiceProfileId".to_string(),
                serde_json::json!("voice-profile-narrator"),
            );
            let job = store
                .enqueue_and_run(
                    &engine(),
                    &overview,
                    RuntimeJobRequest {
                        provider_id: "hexgrad".to_string(),
                        model_id: "native-smoke".to_string(),
                        kind: JobKind::GenerateAudio,
                        workflow: CapabilityWorkflow::Tts,
                        prompt: "Read in the consented narrator voice.".to_string(),
                        source_surface: "TTS Studio".to_string(),
                        parameters,
                    },
                )
                .expect("job persists");
            assert_eq!(job.status, crate::domain::JobStatus::Succeeded);
            assert!(job.actionable_error.is_none());
        });
    }

    #[test]
    fn default_tts_speaker_profiles_pass_consent_gate() {
        // CR2-001: TTS owns both Narrator and Producer reference profiles. Runtime
        // consent resolution must see both, not only the Voice Lab fixture catalog.
        with_isolated_consent_root("consent-tts-speakers-root", || {
            let store = RuntimeJobStore::new(temp_runtime_root("consent-tts-speakers"));
            let overview = installed_overview(&store);
            let mut parameters = BTreeMap::new();
            parameters.insert(
                "voiceProfileIds".to_string(),
                serde_json::json!(["voice-profile-narrator", "voice-profile-producer"]),
            );
            let job = store
                .enqueue_and_run(
                    &engine(),
                    &overview,
                    RuntimeJobRequest {
                        provider_id: "hexgrad".to_string(),
                        model_id: "native-smoke".to_string(),
                        kind: JobKind::GenerateAudio,
                        workflow: CapabilityWorkflow::Tts,
                        prompt: "Read this with the default multi-speaker TTS profile set."
                            .to_string(),
                        source_surface: "TTS Studio".to_string(),
                        parameters,
                    },
                )
                .expect("job persists");
            assert_eq!(job.status, crate::domain::JobStatus::Succeeded);
            assert!(job.actionable_error.is_none());
        });
    }

    #[test]
    fn unknown_profile_override_still_blocks_runtime_admission() {
        // CR2-002: a persisted explicit-consent record is only an override for a
        // known catalog profile; it cannot invent a valid profile id.
        with_isolated_consent_root("unknown-profile-consent-root", || {
            crate::VoiceConsentStore::default()
                .record(
                    "voice-profile-made-up",
                    crate::domain::VoiceConsentStatus::ExplicitConsentRecorded,
                )
                .expect("record unknown override");

            let store = RuntimeJobStore::new(temp_runtime_root("unknown-profile-override"));
            let overview = installed_overview(&store);
            let mut parameters = BTreeMap::new();
            parameters.insert(
                "voiceProfileId".to_string(),
                serde_json::json!("voice-profile-made-up"),
            );
            let job = store
                .enqueue(
                    &overview,
                    RuntimeJobRequest {
                        provider_id: "hexgrad".to_string(),
                        model_id: "native-smoke".to_string(),
                        kind: JobKind::GenerateAudio,
                        workflow: CapabilityWorkflow::Tts,
                        prompt: "Read in a fabricated voice.".to_string(),
                        source_surface: "TTS Studio".to_string(),
                        parameters,
                    },
                )
                .expect("job persists");

            assert_eq!(job.status, crate::domain::JobStatus::Failed);
            assert_eq!(
                job.actionable_error
                    .as_ref()
                    .map(|error| error.code.as_str()),
                Some("voice.consent_required")
            );
        });
    }

    #[test]
    fn blocked_model_is_rejected_before_execution() {
        // F-003: a model whose use policy is Blocked (ChatTTS = noncommercial /
        // research-only) can never run, regardless of workflow.
        let store = RuntimeJobStore::new(temp_runtime_root("blocked-model"));
        let overview = installed_overview(&store);
        let job = store
            .enqueue(
                &overview,
                RuntimeJobRequest {
                    provider_id: "2noise".to_string(),
                    model_id: "chattts".to_string(),
                    kind: JobKind::GenerateAudio,
                    workflow: CapabilityWorkflow::Tts,
                    prompt: "Should never run.".to_string(),
                    source_surface: "TTS Studio".to_string(),
                    parameters: BTreeMap::new(),
                },
            )
            .expect("job persists");
        assert_eq!(job.status, crate::domain::JobStatus::Failed);
        assert_eq!(
            job.actionable_error
                .as_ref()
                .map(|error| error.code.as_str()),
            Some("model.use_blocked")
        );
    }

    #[test]
    fn unreviewed_model_policy_fails_closed_before_execution() {
        let store = RuntimeJobStore::new(temp_runtime_root("unreviewed-model"));
        let overview = installed_overview(&store);
        let job = store
            .enqueue(
                &overview,
                RuntimeJobRequest {
                    provider_id: "unknown".to_string(),
                    model_id: "not-a-reviewed-model".to_string(),
                    kind: JobKind::GenerateAudio,
                    workflow: CapabilityWorkflow::Tts,
                    prompt: "Should never run.".to_string(),
                    source_surface: "TTS Studio".to_string(),
                    parameters: BTreeMap::new(),
                },
            )
            .expect("job persists");

        assert_eq!(job.status, crate::domain::JobStatus::Failed);
        assert_eq!(
            job.actionable_error
                .as_ref()
                .map(|error| error.code.as_str()),
            Some("model.use_unreviewed")
        );
    }

    #[test]
    fn enqueue_returns_queued_without_running_synthesis() {
        // F-006: enqueue must persist a Queued job and return immediately, leaving
        // synthesis to the worker (run_job). The job is genuinely cancellable while
        // it waits, and no artifact has been produced yet.
        let store = RuntimeJobStore::new(temp_runtime_root("queued-only"));
        let overview = installed_overview(&store);

        let queued = store
            .enqueue(
                &overview,
                RuntimeJobRequest {
                    provider_id: "hexgrad".to_string(),
                    model_id: "native-smoke".to_string(),
                    kind: JobKind::GenerateAudio,
                    workflow: CapabilityWorkflow::Tts,
                    prompt: "Queue me".to_string(),
                    source_surface: "TTS Studio".to_string(),
                    parameters: BTreeMap::new(),
                },
            )
            .expect("queued job persists");

        assert_eq!(queued.status, crate::domain::JobStatus::Queued);
        assert_eq!(queued.cancellation, CancellationState::Cancellable);
        assert!(queued.artifacts.is_empty());
        // The persisted record is readable as Queued before any worker claims it.
        assert!(store
            .read_jobs()
            .expect("read jobs")
            .iter()
            .any(|stored| stored.id == queued.id
                && stored.status == crate::domain::JobStatus::Queued));
    }

    #[test]
    fn worker_observes_cancellation_and_writes_no_output() {
        // F-006: a real cooperative cancellation. Requesting cancel on the worker
        // boundary before the worker runs makes run_job stop at its first checkpoint
        // with a Cancelled terminal and no committed artifact.
        let store = RuntimeJobStore::new(temp_runtime_root("worker-cancel"));
        let overview = installed_overview(&store);
        let engine = engine();

        let queued = store
            .enqueue(
                &overview,
                RuntimeJobRequest {
                    provider_id: "hexgrad".to_string(),
                    model_id: "native-smoke".to_string(),
                    kind: JobKind::GenerateAudio,
                    workflow: CapabilityWorkflow::Tts,
                    prompt: "Cancel me before synthesis".to_string(),
                    source_surface: "TTS Studio".to_string(),
                    parameters: BTreeMap::new(),
                },
            )
            .expect("queued job persists");
        assert_eq!(queued.status, crate::domain::JobStatus::Queued);

        let ctx = engine.context_for(&queued.id);
        assert!(engine.request_cancel(&queued.id));
        let cancelled = store
            .run_job(&queued.id, &ctx)
            .expect("worker runs")
            .expect("job exists");
        assert_eq!(cancelled.status, crate::domain::JobStatus::Cancelled);
        assert_eq!(cancelled.cancellation, CancellationState::Completed);
        assert!(cancelled.artifacts.is_empty());
    }

    #[test]
    fn spawned_worker_drives_queued_job_to_success() {
        // F-006: the shared engine + store can run a queued job on a background
        // thread (proves the worker payload is Send and reaches a terminal state
        // off the enqueuing thread). The desktop uses exactly this handoff.
        let store = RuntimeJobStore::new(temp_runtime_root("worker-async"));
        let overview = installed_overview(&store);
        let engine = Arc::new(engine());

        let queued = store
            .enqueue(
                &overview,
                RuntimeJobRequest {
                    provider_id: "hexgrad".to_string(),
                    model_id: "native-smoke".to_string(),
                    kind: JobKind::GenerateAudio,
                    workflow: CapabilityWorkflow::Tts,
                    prompt: "Async worker run".to_string(),
                    source_surface: "TTS Studio".to_string(),
                    parameters: BTreeMap::new(),
                },
            )
            .expect("queued job persists");
        assert_eq!(queued.status, crate::domain::JobStatus::Queued);

        let job_id = queued.id.clone();
        let root = store.root().to_path_buf();
        let worker_engine = engine.clone();
        std::thread::spawn(move || {
            let store = RuntimeJobStore::new(root);
            let ctx = worker_engine.context_for(&job_id);
            let _ = store.run_job(&job_id, &ctx);
            worker_engine.forget(&job_id);
        })
        .join()
        .expect("worker thread joins");

        let done = store
            .read_jobs()
            .expect("read jobs")
            .into_iter()
            .find(|stored| stored.id == queued.id)
            .expect("job persisted");
        assert_eq!(done.status, crate::domain::JobStatus::Succeeded);
        assert!(done
            .artifacts
            .iter()
            .any(|artifact| artifact.kind == RuntimeArtifactKind::AudioPreview));
    }

    #[test]
    fn runtime_job_store_cancels_queued_job_and_retries_failed_job() {
        let store = RuntimeJobStore::new(temp_runtime_root("cancel-retry"));
        let overview = installed_overview(&store);

        let queued = store
            .enqueue(
                &overview,
                RuntimeJobRequest {
                    provider_id: "hexgrad".to_string(),
                    model_id: "native-smoke".to_string(),
                    kind: JobKind::GenerateAudio,
                    workflow: CapabilityWorkflow::Tts,
                    prompt: "Queue then cancel".to_string(),
                    source_surface: "TTS Studio".to_string(),
                    parameters: BTreeMap::new(),
                },
            )
            .expect("queued job persists");
        assert_eq!(queued.status, crate::domain::JobStatus::Queued);

        let cancelled = store
            .cancel(&queued.id)
            .expect("cancel command succeeds")
            .expect("job exists");
        assert_eq!(cancelled.status, crate::domain::JobStatus::Cancelled);
        assert_eq!(cancelled.cancellation, CancellationState::Completed);

        let failed = store
            .enqueue(
                &RuntimeOverview::reference(),
                RuntimeJobRequest {
                    provider_id: "missing-provider".to_string(),
                    model_id: "missing-model".to_string(),
                    kind: JobKind::GenerateAudio,
                    workflow: CapabilityWorkflow::Sfx,
                    prompt: "Missing runtime".to_string(),
                    source_surface: "SFX Studio".to_string(),
                    parameters: BTreeMap::new(),
                },
            )
            .expect("failed job persists");
        let retried = store
            .retry(&RuntimeOverview::reference(), &failed.id)
            .expect("retry command succeeds")
            .expect("retry job exists");
        assert_ne!(retried.id, failed.id);
        assert_eq!(retried.retry_count, 1);
    }

    fn engine() -> RuntimeEngine {
        RuntimeEngine::new().expect("runtime engine builds")
    }

    #[test]
    fn native_registry_is_single_source_of_truth() {
        // F-024: every native model round-trips through the registry, and an
        // unknown id resolves to None (so dispatch falls back to smoke, not a
        // silent mis-route).
        for native in [
            NativeModel::KokoroTts,
            NativeModel::ProceduralSfx,
            NativeModel::ProceduralMusic,
            NativeModel::Smoke,
        ] {
            assert_eq!(NativeModel::from_model_id(native.model_id()), Some(native));
            // F-007: native built-ins are always the NativeRust strategy,
            // regardless of how their catalog runtime is labelled.
            assert_eq!(
                ExecutionStrategy::for_model(native.model_id(), ModelRuntime::ResearchOnly),
                ExecutionStrategy::NativeRust
            );
        }
        assert_eq!(NativeModel::from_model_id("not-a-real-model"), None);
        assert_eq!(
            ExecutionStrategy::for_model("not-a-real-model", ModelRuntime::ExternalApi),
            ExecutionStrategy::ManagedApi
        );
        assert_eq!(
            ExecutionStrategy::for_model("not-a-real-model", ModelRuntime::ResearchOnly),
            ExecutionStrategy::ResearchOnly
        );
    }

    #[test]
    fn adapter_dispatch_reads_execution_strategy_not_model_id() {
        // F-007: adapter_for_model resolves from the catalog's declared strategy.
        // Flipping the strategy (without touching the model id) changes dispatch.
        let store = RuntimeJobStore::new(temp_runtime_root("adapter-strategy"));
        let mut overview = installed_overview(&store);
        let request = RuntimeJobRequest {
            provider_id: "hexgrad".to_string(),
            model_id: "native-smoke".to_string(),
            kind: JobKind::GenerateAudio,
            workflow: CapabilityWorkflow::Tts,
            prompt: "dispatch".to_string(),
            source_surface: "TTS Studio".to_string(),
            parameters: BTreeMap::new(),
        };

        assert_eq!(
            super::adapter_for_model(&overview, &request),
            ProviderAdapterKind::NativeRust
        );

        overview.model_states[0].execution_strategy = ExecutionStrategy::ManagedApi;
        assert_eq!(
            super::adapter_for_model(&overview, &request),
            ProviderAdapterKind::ManagedApi
        );
    }

    fn installed_overview(store: &RuntimeJobStore) -> RuntimeOverview {
        RuntimeOverview {
            schema_version: super::RUNTIME_SCHEMA_VERSION,
            packaging_policy: RuntimePackagingPolicy::shipped_desktop(),
            devices: DeviceInventory::reference_mac().devices,
            status_counts: super::RuntimeStatusCounts {
                installed: 1,
                available: 0,
                unavailable: 0,
            },
            model_states: vec![super::ModelRuntimeState {
                provider_id: "hexgrad".to_string(),
                model_id: "native-smoke".to_string(),
                model_name: "Native smoke adapter".to_string(),
                runtime: ModelRuntime::Local,
                execution_strategy: super::ExecutionStrategy::NativeRust,
                workflows: vec![CapabilityWorkflow::Tts],
                availability: RuntimeAvailability::Installed,
                install_status: ModelInstallStatus::Installed,
                cache: ModelCacheState {
                    cache_path: None,
                    package_id: Some("soundworks-native-smoke".to_string()),
                    status: CacheStatus::Ready,
                    expected_size_mb: Some(1450),
                    disk_usage_mb: Some(1450),
                    verified: true,
                    evidence: "verified test cache".to_string(),
                    license: LicenseAcceptanceState::Accepted,
                    warmup: WarmupStatus::Cold,
                },
                compatibility: RuntimeCompatibility {
                    supported: true,
                    selected_accelerator: Some(crate::manifests::DeviceAccelerator::Cpu),
                    min_memory_mb: None,
                    available_memory_mb: Some(32768),
                    requires_network: false,
                    reasons: vec![],
                },
                health: RuntimeHealth::Ready,
                reasons: vec![],
            }],
            jobs: store.read_jobs().unwrap_or_default(),
            validation_checks: vec![],
        }
    }

    fn kokoro_overview(store: &RuntimeJobStore, cache_root: PathBuf) -> RuntimeOverview {
        let mut overview = installed_overview(store);
        overview.model_states[0].model_id = "kokoro-82m".to_string();
        overview.model_states[0].model_name = "Kokoro 82M".to_string();
        overview.model_states[0].cache.cache_path = Some(cache_root.display().to_string());
        overview.model_states[0].cache.package_id =
            Some("onnx-community/Kokoro-82M-v1.0-ONNX".to_string());
        overview
    }

    fn native_sfx_overview(store: &RuntimeJobStore) -> RuntimeOverview {
        RuntimeOverview {
            schema_version: super::RUNTIME_SCHEMA_VERSION,
            packaging_policy: RuntimePackagingPolicy::shipped_desktop(),
            devices: DeviceInventory::reference_mac().devices,
            status_counts: super::RuntimeStatusCounts {
                installed: 1,
                available: 0,
                unavailable: 0,
            },
            model_states: vec![super::ModelRuntimeState::native_procedural_sfx(
                &DeviceInventory::reference_mac(),
            )],
            jobs: store.read_jobs().unwrap_or_default(),
            validation_checks: vec![],
        }
    }

    fn native_music_overview(store: &RuntimeJobStore) -> RuntimeOverview {
        RuntimeOverview {
            schema_version: super::RUNTIME_SCHEMA_VERSION,
            packaging_policy: RuntimePackagingPolicy::shipped_desktop(),
            devices: DeviceInventory::reference_mac().devices,
            status_counts: super::RuntimeStatusCounts {
                installed: 1,
                available: 0,
                unavailable: 0,
            },
            model_states: vec![super::ModelRuntimeState::native_procedural_music(
                &DeviceInventory::reference_mac(),
            )],
            jobs: store.read_jobs().unwrap_or_default(),
            validation_checks: vec![],
        }
    }

    fn temp_runtime_root(label: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!(
            "soundworks-runtime-{label}-{}",
            super::timestamp_millis()
        ));
        let _ = fs::remove_dir_all(&root);
        root
    }

    fn with_isolated_consent_root<T>(label: &str, f: impl FnOnce() -> T) -> T {
        let _guard = CONSENT_ENV_LOCK
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        let root = temp_runtime_root(label);
        std::env::set_var("SOUNDWORKS_VOICE_CONSENT_ROOT", &root);
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(f));
        std::env::remove_var("SOUNDWORKS_VOICE_CONSENT_ROOT");
        let _ = fs::remove_dir_all(&root);
        match result {
            Ok(value) => value,
            Err(payload) => std::panic::resume_unwind(payload),
        }
    }

    #[test]
    fn encode_pcm16_wav_writes_canonical_header_and_rejects_partial_frames() {
        let root = temp_runtime_root("encode-pcm16");
        fs::create_dir_all(&root).expect("create temp root");
        let path = root.join("frames.wav");

        // Two stereo frames -> 44-byte header + 4 samples * 2 bytes of payload.
        super::encode_pcm16_wav(&path, 48_000, 2, &[1, -1, 32_767, -32_768])
            .expect("encode stereo pcm16");
        let bytes = fs::read(&path).expect("read encoded wav");
        assert_eq!(bytes.len(), 44 + 4 * 2);
        assert_eq!(&bytes[0..4], b"RIFF");
        assert_eq!(u32::from_le_bytes(bytes[4..8].try_into().unwrap()), 36 + 8);
        assert_eq!(&bytes[8..12], b"WAVE");
        assert_eq!(&bytes[12..16], b"fmt ");
        assert_eq!(u32::from_le_bytes(bytes[16..20].try_into().unwrap()), 16);
        assert_eq!(u16::from_le_bytes(bytes[20..22].try_into().unwrap()), 1); // PCM tag
        assert_eq!(u16::from_le_bytes(bytes[22..24].try_into().unwrap()), 2); // channels
        assert_eq!(
            u32::from_le_bytes(bytes[24..28].try_into().unwrap()),
            48_000
        );
        assert_eq!(
            u32::from_le_bytes(bytes[28..32].try_into().unwrap()),
            48_000 * 2 * 2 // byte rate = rate * channels * bytes-per-sample
        );
        assert_eq!(u16::from_le_bytes(bytes[32..34].try_into().unwrap()), 4); // block align
        assert_eq!(u16::from_le_bytes(bytes[34..36].try_into().unwrap()), 16); // bits per sample
        assert_eq!(&bytes[36..40], b"data");
        assert_eq!(u32::from_le_bytes(bytes[40..44].try_into().unwrap()), 8); // data size

        // A buffer that is not a whole number of frames is rejected, not truncated.
        let partial =
            super::write_pcm16_wav_channels(&root.join("partial.wav"), &[0.0, 0.0, 0.0], 48_000, 2);
        assert!(partial.is_err());

        let _ = fs::remove_dir_all(&root);
    }
}
