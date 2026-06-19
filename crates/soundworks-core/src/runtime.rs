use crate::domain::{JobKind, JobProgress, JobStatus, ModelRuntime};
use crate::evaluation::{EvaluationLane, ProductEligibility, ProductRuntimePath};
use crate::manifests::{
    CapabilityWorkflow, DeviceAccelerator, ModelInstallStatus, ModelManifest, ProviderCatalog,
};
use crate::model_manager::{
    CandidateInstallState, ModelCandidateInstallState, ModelManagerOverview,
};
use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
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

        let model_states: Vec<ModelRuntimeState> = manager
            .candidates
            .iter()
            .filter(|candidate| candidate.install_state == CandidateInstallState::Installed)
            .map(|candidate| ModelRuntimeState::from_candidate(candidate, inventory))
            .collect();
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
        let availability = if candidate.cache.verified {
            RuntimeAvailability::Installed
        } else {
            RuntimeAvailability::Unavailable
        };
        let health = if availability == RuntimeAvailability::Installed {
            RuntimeHealth::Ready
        } else {
            RuntimeHealth::Blocked
        };
        let selected = inventory
            .devices
            .iter()
            .find(|device| device.available)
            .map(|device| device.accelerator);
        let mut reasons = candidate.blockers.clone();
        if candidate.product_eligibility == ProductEligibility::ResearchOnly {
            reasons.push("Research-only model cannot be product-enabled.".to_string());
        }

        Self {
            provider_id: provider_id_for_candidate(candidate),
            model_id: candidate.candidate_id.clone(),
            model_name: candidate.name.clone(),
            runtime,
            workflows,
            availability,
            install_status: ModelInstallStatus::Installed,
            cache: ModelCacheState {
                package_id: candidate.download_plan.repository_id.clone(),
                status: CacheStatus::Ready,
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
                available_memory_mb: inventory
                    .devices
                    .iter()
                    .find(|device| device.available)
                    .and_then(|device| device.memory_mb),
                requires_network: false,
                reasons: vec![],
            },
            health,
            reasons,
        }
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
    pub record_root: String,
    pub recipe_path: String,
    pub model_metadata_path: String,
    pub events_path: String,
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
            "job-{}-{}",
            request.workflow_id_fragment(),
            timestamp_millis()
        );
        let record_root = self.root.join("jobs").join(&job_id);
        fs::create_dir_all(record_root.join("artifacts"))?;
        let created_at = timestamp_string();
        let adapter = adapter_for_model(overview, &request);
        let admission = overview.admit_job(&request.provider_id, &request.model_id, request.kind);

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
            id: job_id,
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
            record_root: record_root.display().to_string(),
            recipe_path: record_root.join("recipe.json").display().to_string(),
            model_metadata_path: record_root.join("model.json").display().to_string(),
            events_path: record_root.join("events.jsonl").display().to_string(),
            log_tail: vec!["persisted job record".to_string()],
            artifacts: vec![],
            actionable_error: None,
        };
        self.append_event(&job, "queued", "persisted job record")?;

        if !admission.accepted {
            job.status = JobStatus::Failed;
            job.progress = Some(JobProgress {
                percent: 100.0,
                message: Some("Runtime rejected the job before adapter execution.".to_string()),
            });
            job.cancellation = CancellationState::NotCancellable;
            job.updated_at = timestamp_string();
            job.log_tail.push(admission.reason.clone());
            job.actionable_error = admission.actionable_error;
            self.write_error_report(&mut job)?;
            self.append_event(&job, "failed", "runtime admission rejected job")?;
            self.write_job(&job)?;
            return Ok(job);
        }

        self.run_adapter(job, &request)
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
        let request: RuntimeJobRequest = read_json(Path::new(&job.recipe_path))?;
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
        let path = self.root.join("jobs").join(job_id).join("job.json");
        if path.is_file() {
            Ok(Some(read_json(path)?))
        } else {
            Ok(None)
        }
    }

    fn run_adapter(
        &self,
        mut job: RuntimeJobSnapshot,
        request: &RuntimeJobRequest,
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
        if request
            .prompt
            .to_ascii_lowercase()
            .contains("hold-for-cancel")
        {
            job.progress = Some(JobProgress {
                percent: 50.0,
                message: Some(
                    "Provider adapter is holding the job for cancellation QA.".to_string(),
                ),
            });
            job.log_tail
                .push("holding job for cancellation QA".to_string());
            self.write_job(&job)?;
            return Ok(job);
        }

        match job.adapter {
            ProviderAdapterKind::NativeRust => self.write_smoke_audio(&mut job, request)?,
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
        let record_root = PathBuf::from(&job.record_root);
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

    fn write_error_report(&self, job: &mut RuntimeJobSnapshot) -> io::Result<()> {
        let path = PathBuf::from(&job.record_root).join("error.json");
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
        write_json(PathBuf::from(&job.record_root).join("job.json"), job)
    }

    fn append_event(&self, job: &RuntimeJobSnapshot, event: &str, message: &str) -> io::Result<()> {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&job.events_path)?;
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
    overview
        .model_states
        .iter()
        .find(|state| {
            state.provider_id == request.provider_id && state.model_id == request.model_id
        })
        .map(|state| match state.runtime {
            ModelRuntime::Local if state.model_id == "kokoro-82m" => {
                ProviderAdapterKind::NativeRust
            }
            ModelRuntime::Local => ProviderAdapterKind::LocalExecutable,
            ModelRuntime::ExternalApi => ProviderAdapterKind::ManagedApi,
            ModelRuntime::ResearchOnly => ProviderAdapterKind::ResearchOnly,
        })
        .unwrap_or(ProviderAdapterKind::ResearchOnly)
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
    if let Some(parent) = path.as_ref().parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(
        path,
        serde_json::to_vec_pretty(value).map_err(io::Error::other)?,
    )
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

fn timestamp_string() -> String {
    timestamp_millis().to_string()
}

fn write_smoke_wav(path: &Path) -> io::Result<()> {
    let sample_rate = 16_000u32;
    let samples = sample_rate / 4;
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
        let phase = (index as f32 / sample_rate as f32) * 440.0 * std::f32::consts::TAU;
        let sample = (phase.sin() * i16::MAX as f32 * 0.18) as i16;
        bytes.extend_from_slice(&sample.to_le_bytes());
    }
    fs::write(path, bytes)
}

#[cfg(test)]
mod tests {
    use super::{
        CacheStatus, CancellationState, DeviceInventory, LicenseAcceptanceState, ModelCacheState,
        ProviderAdapterKind, RuntimeArtifactKind, RuntimeAvailability, RuntimeCompatibility,
        RuntimeHealth, RuntimeJobRequest, RuntimeJobStore, RuntimeOverview, RuntimePackagingPolicy,
        ValidationStatus, WarmupStatus,
    };
    use crate::domain::{JobKind, ModelRuntime};
    use crate::manifests::{CapabilityWorkflow, ModelInstallStatus, ProviderCatalog};
    use std::fs;
    use std::path::PathBuf;

    #[test]
    fn reference_runtime_blocks_manifest_only_models_and_jobs() {
        let runtime = RuntimeOverview::reference();

        assert_eq!(runtime.schema_version, super::RUNTIME_SCHEMA_VERSION);
        assert_eq!(runtime.status_counts.installed, 0);
        assert_eq!(runtime.status_counts.available, 0);
        assert_eq!(runtime.status_counts.unavailable, 3);
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
        assert_eq!(runtime.status_counts.unavailable, 2);
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
            .enqueue(
                &overview,
                RuntimeJobRequest {
                    provider_id: "hexgrad".to_string(),
                    model_id: "kokoro-82m".to_string(),
                    kind: JobKind::GenerateAudio,
                    workflow: CapabilityWorkflow::Tts,
                    prompt: "Runtime smoke read".to_string(),
                    source_surface: "TTS Studio".to_string(),
                },
            )
            .expect("enqueue succeeds");

        assert_eq!(job.status, crate::domain::JobStatus::Succeeded);
        assert_eq!(job.adapter, ProviderAdapterKind::NativeRust);
        assert!(PathBuf::from(&job.recipe_path).is_file());
        assert!(PathBuf::from(&job.model_metadata_path).is_file());
        assert!(PathBuf::from(&job.events_path).is_file());
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
    fn runtime_job_store_cancels_running_job_and_retries_failed_job() {
        let store = RuntimeJobStore::new(temp_runtime_root("cancel-retry"));
        let overview = installed_overview(&store);

        let running = store
            .enqueue(
                &overview,
                RuntimeJobRequest {
                    provider_id: "hexgrad".to_string(),
                    model_id: "kokoro-82m".to_string(),
                    kind: JobKind::GenerateAudio,
                    workflow: CapabilityWorkflow::Tts,
                    prompt: "hold-for-cancel".to_string(),
                    source_surface: "TTS Studio".to_string(),
                },
            )
            .expect("running job persists");
        assert_eq!(running.status, crate::domain::JobStatus::Running);

        let cancelled = store
            .cancel(&running.id)
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
                model_id: "kokoro-82m".to_string(),
                model_name: "Kokoro 82M".to_string(),
                runtime: ModelRuntime::Local,
                workflows: vec![CapabilityWorkflow::Tts],
                availability: RuntimeAvailability::Installed,
                install_status: ModelInstallStatus::Installed,
                cache: ModelCacheState {
                    package_id: Some("onnx-community/Kokoro-82M-v1.0-ONNX".to_string()),
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

    fn temp_runtime_root(label: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!(
            "soundworks-runtime-{label}-{}",
            super::timestamp_millis()
        ));
        let _ = fs::remove_dir_all(&root);
        root
    }
}
