use crate::domain::{JobKind, JobProgress, JobStatus, ModelRuntime};
use crate::manifests::{
    CapabilityWorkflow, DeviceAccelerator, ModelInstallStatus, ModelManifest, ProviderCatalog,
};
use serde::{Deserialize, Serialize};

pub const RUNTIME_SCHEMA_VERSION: u32 = 1;

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
            jobs: reference_jobs(&model_states),
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
    pub progress: Option<JobProgress>,
    pub cancellation: CancellationState,
    pub retry_count: u8,
    pub log_tail: Vec<String>,
    pub actionable_error: Option<ActionableRuntimeError>,
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

fn reference_jobs(model_states: &[ModelRuntimeState]) -> Vec<RuntimeJobSnapshot> {
    let ready_model = model_states
        .iter()
        .find(|state| state.availability == RuntimeAvailability::Installed);
    let Some(ready_model) = ready_model else {
        return vec![];
    };

    let mut jobs = vec![RuntimeJobSnapshot {
        id: "job-runtime-reference-generate".to_string(),
        kind: JobKind::GenerateAudio,
        status: JobStatus::Running,
        provider_id: ready_model.provider_id.clone(),
        model_id: ready_model.model_id.clone(),
        progress: Some(JobProgress {
            percent: 42.0,
            message: Some("Generating preview audio from queued worker contract.".to_string()),
        }),
        cancellation: CancellationState::Cancellable,
        retry_count: 0,
        log_tail: vec![
            "claimed job from local queue".to_string(),
            "loaded model package from cache".to_string(),
            "streamed progress event 42%".to_string(),
        ],
        actionable_error: None,
    }];

    jobs.push(RuntimeJobSnapshot {
        id: "job-runtime-reference-cache-repair".to_string(),
        kind: JobKind::EvaluateModel,
        status: JobStatus::Failed,
        provider_id: ready_model.provider_id.clone(),
        model_id: ready_model.model_id.clone(),
        progress: Some(JobProgress {
            percent: 0.0,
            message: Some("Runtime validation detected a repairable package issue.".to_string()),
        }),
        cancellation: CancellationState::NotCancellable,
        retry_count: 0,
        log_tail: vec![
            "verified package manifest".to_string(),
            "detected cache checksum mismatch".to_string(),
        ],
        actionable_error: Some(ActionableRuntimeError {
            code: "runtime.cache_mismatch".to_string(),
            summary: "Model package cache needs repair".to_string(),
            recovery:
                "Reinstall the provider package or clear the model cache entry before retrying."
                    .to_string(),
        }),
    });

    jobs
}

#[cfg(test)]
mod tests {
    use super::{
        DeviceInventory, RuntimeAvailability, RuntimeOverview, RuntimePackagingPolicy,
        ValidationStatus,
    };
    use crate::domain::{JobKind, ModelRuntime};
    use crate::manifests::{ModelInstallStatus, ProviderCatalog};

    #[test]
    fn reference_runtime_blocks_manifest_only_models_and_jobs() {
        let runtime = RuntimeOverview::reference();

        assert_eq!(runtime.schema_version, 1);
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
}
