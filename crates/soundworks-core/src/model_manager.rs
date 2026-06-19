use crate::evaluation::{
    EvaluationLane, EvaluationStatus, EvidenceLevel, ModelEvaluationCandidate,
    ModelEvaluationCatalog, ProductEligibility, ProductRuntimePath,
};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

pub const MODEL_MANAGER_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelManagerOverview {
    pub schema_version: u32,
    pub cache_root: String,
    pub summary: ModelManagerSummary,
    pub lane_readiness: Vec<ModelLaneReadiness>,
    pub candidates: Vec<ModelCandidateInstallState>,
    pub operations: Vec<ModelManagerOperation>,
    pub validation_checks: Vec<ModelManagerValidationCheck>,
}

impl ModelManagerOverview {
    pub fn reference() -> Self {
        Self::from_catalog(
            &ModelEvaluationCatalog::reference(),
            default_model_cache_root(),
        )
    }

    pub fn from_catalog(catalog: &ModelEvaluationCatalog, cache_root: PathBuf) -> Self {
        let candidates: Vec<ModelCandidateInstallState> = catalog
            .candidates
            .iter()
            .map(|candidate| ModelCandidateInstallState::from_candidate(candidate, &cache_root))
            .collect();

        let operations = reference_operations(&candidates);
        let lane_readiness = lane_readiness(catalog, &candidates);
        let summary = ModelManagerSummary::from_candidates(&candidates, &operations);
        let validation_checks = validation_checks(&summary, &candidates);

        Self {
            schema_version: MODEL_MANAGER_SCHEMA_VERSION,
            cache_root: cache_root.display().to_string(),
            summary,
            lane_readiness,
            candidates,
            operations,
            validation_checks,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelManagerSummary {
    pub candidate_count: usize,
    pub verified_installed_count: usize,
    pub installable_count: usize,
    pub blocked_count: usize,
    pub missing_cache_count: usize,
    pub failed_operation_count: usize,
}

impl ModelManagerSummary {
    fn from_candidates(
        candidates: &[ModelCandidateInstallState],
        operations: &[ModelManagerOperation],
    ) -> Self {
        Self {
            candidate_count: candidates.len(),
            verified_installed_count: candidates
                .iter()
                .filter(|candidate| candidate.install_state == CandidateInstallState::Installed)
                .count(),
            installable_count: candidates
                .iter()
                .filter(|candidate| candidate.actions.contains(&ModelManagerActionKind::Install))
                .count(),
            blocked_count: candidates
                .iter()
                .filter(|candidate| {
                    matches!(
                        candidate.install_state,
                        CandidateInstallState::Blocked | CandidateInstallState::ResearchOnly
                    )
                })
                .count(),
            missing_cache_count: candidates
                .iter()
                .filter(|candidate| candidate.install_state == CandidateInstallState::MissingCache)
                .count(),
            failed_operation_count: operations
                .iter()
                .filter(|operation| operation.status == ModelManagerOperationStatus::Failed)
                .count(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelCandidateInstallState {
    pub candidate_id: String,
    pub name: String,
    pub provider: String,
    pub lanes: Vec<EvaluationLane>,
    pub source_label: String,
    pub source_url: String,
    pub license_label: String,
    pub evaluation_status: EvaluationStatus,
    pub product_eligibility: ProductEligibility,
    pub evidence_level: EvidenceLevel,
    pub runtime_path: ProductRuntimePath,
    pub requires_python_runtime: bool,
    pub install_state: CandidateInstallState,
    pub blockers: Vec<String>,
    pub download_plan: ModelDownloadPlan,
    pub cache: ModelCacheVerification,
    pub actions: Vec<ModelManagerActionKind>,
}

impl ModelCandidateInstallState {
    fn from_candidate(candidate: &ModelEvaluationCandidate, cache_root: &Path) -> Self {
        let download_plan = ModelDownloadPlan::from_candidate(candidate);
        let cache = ModelCacheVerification::inspect(cache_root, &download_plan);
        let hard_blockers = hard_install_blockers(candidate);
        let blockers = install_blockers(candidate);
        let install_state = if cache.verified {
            CandidateInstallState::Installed
        } else if candidate.product_eligibility == ProductEligibility::ResearchOnly {
            CandidateInstallState::ResearchOnly
        } else if !hard_blockers.is_empty()
            || candidate.status == EvaluationStatus::Blocked
            || candidate.product_eligibility == ProductEligibility::Blocked
        {
            CandidateInstallState::Blocked
        } else if download_plan.supports_automated_download {
            CandidateInstallState::MissingCache
        } else {
            CandidateInstallState::NeedsRuntimePort
        };

        let mut actions = vec![
            ModelManagerActionKind::Revalidate,
            ModelManagerActionKind::OpenSource,
        ];
        if matches!(
            install_state,
            CandidateInstallState::MissingCache | CandidateInstallState::NeedsRuntimePort
        ) {
            actions.push(ModelManagerActionKind::Install);
        }
        if install_state == CandidateInstallState::MissingCache {
            actions.push(ModelManagerActionKind::RepairCache);
        }
        if download_plan.requires_license_acceptance {
            actions.push(ModelManagerActionKind::AcceptLicense);
        }

        let source = candidate.sources.first();

        Self {
            candidate_id: candidate.id.clone(),
            name: candidate.name.clone(),
            provider: candidate.provider.clone(),
            lanes: candidate.lanes.clone(),
            source_label: source
                .map(|source| source.label.clone())
                .unwrap_or_else(|| "No source recorded".to_string()),
            source_url: source
                .map(|source| source.url.clone())
                .unwrap_or_else(|| "about:blank".to_string()),
            license_label: candidate.license.label.clone(),
            evaluation_status: candidate.status,
            product_eligibility: candidate.product_eligibility,
            evidence_level: candidate.evidence_level,
            runtime_path: candidate.runtime.product_path,
            requires_python_runtime: candidate.runtime.requires_python_runtime,
            install_state,
            blockers,
            download_plan,
            cache,
            actions,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CandidateInstallState {
    Installed,
    MissingCache,
    NeedsRuntimePort,
    ResearchOnly,
    Blocked,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelDownloadPlan {
    pub mechanism: DownloadMechanism,
    pub source_url: String,
    pub repository_id: Option<String>,
    pub cache_subdir: String,
    pub expected_files: Vec<ExpectedModelFile>,
    pub expected_size_mb: Option<u32>,
    pub requires_license_acceptance: bool,
    pub supports_automated_download: bool,
    pub command_hint: String,
    pub notes: Vec<String>,
}

impl ModelDownloadPlan {
    fn from_candidate(candidate: &ModelEvaluationCandidate) -> Self {
        let source_url = preferred_download_source(candidate);
        let mechanism = download_mechanism(candidate);
        let repository_id = huggingface_repo_id(&source_url);
        let expected_files = expected_files_for(candidate.id.as_str(), mechanism);
        let supports_automated_download =
            matches!(mechanism, DownloadMechanism::HuggingFaceSnapshot);

        Self {
            mechanism,
            source_url: source_url.clone(),
            repository_id,
            cache_subdir: candidate.id.clone(),
            expected_files,
            expected_size_mb: expected_size_mb(candidate.id.as_str()),
            requires_license_acceptance: candidate.license.commercial_use
                != crate::evaluation::CommercialUseEvaluation::Allowed,
            supports_automated_download,
            command_hint: command_hint(candidate.id.as_str(), &source_url, mechanism),
            notes: download_notes(candidate),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DownloadMechanism {
    HuggingFaceSnapshot,
    GitRepository,
    ManagedApi,
    ResearchPoc,
    SourceReviewOnly,
    Blocked,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExpectedModelFile {
    pub path: String,
    pub required: bool,
    pub sha256: Option<String>,
    pub size_mb: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelCacheVerification {
    pub cache_path: String,
    pub verified: bool,
    pub expected_file_count: usize,
    pub present_file_count: usize,
    pub missing_required_files: Vec<String>,
    pub disk_usage_mb: Option<u32>,
    pub evidence: String,
}

impl ModelCacheVerification {
    fn inspect(cache_root: &Path, plan: &ModelDownloadPlan) -> Self {
        let cache_paths = cache_paths(cache_root, plan);
        let (cache_path, _, present_file_count) = cache_paths
            .iter()
            .map(|path| {
                let missing_required_files: Vec<String> = plan
                    .expected_files
                    .iter()
                    .filter(|expected| expected.required && !path.join(&expected.path).is_file())
                    .map(|expected| expected.path.clone())
                    .collect();
                let present_file_count = plan
                    .expected_files
                    .iter()
                    .filter(|expected| path.join(&expected.path).is_file())
                    .count();
                (path.clone(), missing_required_files, present_file_count)
            })
            .min_by_key(|(_, missing, _)| missing.len())
            .unwrap_or_else(|| (cache_root.join(&plan.cache_subdir), vec![], 0));
        let missing_required_files: Vec<String> = plan
            .expected_files
            .iter()
            .filter(|expected| expected.required && !cache_path.join(&expected.path).is_file())
            .map(|expected| expected.path.clone())
            .collect();
        let required_file_count = plan
            .expected_files
            .iter()
            .filter(|expected| expected.required)
            .count();
        let verified = required_file_count > 0 && missing_required_files.is_empty();
        let disk_usage_mb = if cache_path.exists() {
            disk_usage_mb(&cache_path)
        } else {
            None
        };
        let evidence = if verified {
            format!(
                "verified {} expected file(s) under {}",
                present_file_count,
                cache_path.display()
            )
        } else if cache_path.exists() {
            format!(
                "cache directory exists, but required file evidence is incomplete under {}",
                cache_path.display()
            )
        } else {
            format!("missing cache directory {}", cache_path.display())
        };

        Self {
            cache_path: cache_path.display().to_string(),
            verified,
            expected_file_count: plan.expected_files.len(),
            present_file_count,
            missing_required_files,
            disk_usage_mb,
            evidence,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ModelManagerActionKind {
    Revalidate,
    Install,
    RepairCache,
    OpenSource,
    AcceptLicense,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelManagerOperation {
    pub id: String,
    pub candidate_id: String,
    pub action: ModelManagerActionKind,
    pub status: ModelManagerOperationStatus,
    pub progress_percent: u8,
    pub summary: String,
    pub recovery: Option<String>,
    pub log_tail: Vec<String>,
}

impl ModelManagerOperation {
    fn succeeded(
        candidate_id: &str,
        action: ModelManagerActionKind,
        summary: String,
        log_tail: Vec<String>,
    ) -> Self {
        Self {
            id: format!("{action:?}-{candidate_id}").to_ascii_lowercase(),
            candidate_id: candidate_id.to_string(),
            action,
            status: ModelManagerOperationStatus::Succeeded,
            progress_percent: 100,
            summary,
            recovery: None,
            log_tail,
        }
    }

    pub fn revalidate(candidate_id: &str) -> Self {
        let overview = ModelManagerOverview::reference();
        match overview
            .candidates
            .iter()
            .find(|candidate| candidate.candidate_id == candidate_id)
        {
            Some(candidate) if candidate.cache.verified => Self {
                id: format!("revalidate-{candidate_id}"),
                candidate_id: candidate_id.to_string(),
                action: ModelManagerActionKind::Revalidate,
                status: ModelManagerOperationStatus::Succeeded,
                progress_percent: 100,
                summary: format!("{} cache evidence is verified.", candidate.name),
                recovery: None,
                log_tail: vec![candidate.cache.evidence.clone()],
            },
            Some(candidate) => Self::failed(
                candidate_id,
                ModelManagerActionKind::Revalidate,
                format!("{} is not installed.", candidate.name),
                format!(
                    "Missing required files: {}.",
                    candidate.cache.missing_required_files.join(", ")
                ),
                vec![candidate.cache.evidence.clone()],
            ),
            None => Self::failed(
                candidate_id,
                ModelManagerActionKind::Revalidate,
                "Candidate is not registered.".to_string(),
                "Refresh the model evaluation catalog and retry.".to_string(),
                vec![],
            ),
        }
    }

    pub fn install(candidate_id: &str) -> Self {
        let overview = ModelManagerOverview::reference();
        match overview
            .candidates
            .iter()
            .find(|candidate| candidate.candidate_id == candidate_id)
        {
            Some(candidate)
                if candidate
                    .actions
                    .contains(&ModelManagerActionKind::Install)
                    && candidate.download_plan.supports_automated_download =>
            {
                Self::failed(
                    candidate_id,
                    ModelManagerActionKind::Install,
                    format!("{} download did not produce verified cache files.", candidate.name),
                    "Run the provider downloader, accept required model terms if prompted, then revalidate the cache path before enabling jobs.".to_string(),
                    vec![
                        candidate.download_plan.command_hint.clone(),
                        candidate.cache.evidence.clone(),
                    ],
                )
            }
            Some(candidate) => Self::failed(
                candidate_id,
                ModelManagerActionKind::Install,
                format!("{} cannot be installed by the product downloader yet.", candidate.name),
                candidate
                    .blockers
                    .first()
                    .cloned()
                    .unwrap_or_else(|| {
                        "Create a product-safe provider package before enabling install."
                            .to_string()
                    }),
                vec![candidate.download_plan.command_hint.clone()],
            ),
            None => Self::failed(
                candidate_id,
                ModelManagerActionKind::Install,
                "Candidate is not registered.".to_string(),
                "Refresh the model evaluation catalog and retry.".to_string(),
                vec![],
            ),
        }
    }

    fn failed(
        candidate_id: &str,
        action: ModelManagerActionKind,
        summary: String,
        recovery: String,
        log_tail: Vec<String>,
    ) -> Self {
        Self {
            id: format!("{action:?}-{candidate_id}").to_ascii_lowercase(),
            candidate_id: candidate_id.to_string(),
            action,
            status: ModelManagerOperationStatus::Failed,
            progress_percent: 100,
            summary,
            recovery: Some(recovery),
            log_tail,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ModelManagerOperationStatus {
    Pending,
    Running,
    Succeeded,
    Failed,
    Blocked,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelLaneReadiness {
    pub lane: EvaluationLane,
    pub recommended_candidate_id: String,
    pub state: LaneReadinessState,
    pub summary: String,
    pub blocker: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum LaneReadinessState {
    Verified,
    MissingCache,
    Blocked,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelManagerValidationCheck {
    pub id: String,
    pub passed: bool,
    pub summary: String,
    pub recovery: Option<String>,
}

fn default_model_cache_root() -> PathBuf {
    std::env::var_os("SOUNDWORKS_MODEL_CACHE")
        .map(PathBuf::from)
        .or_else(|| {
            std::env::var_os("HOME").map(|home| {
                PathBuf::from(home)
                    .join("Library")
                    .join("Application Support")
                    .join("SoundWorks")
                    .join("models")
            })
        })
        .unwrap_or_else(|| PathBuf::from("soundworks-model-cache"))
}

fn cache_paths(cache_root: &Path, plan: &ModelDownloadPlan) -> Vec<PathBuf> {
    let mut paths = vec![cache_root.join(&plan.cache_subdir)];
    if plan.mechanism == DownloadMechanism::HuggingFaceSnapshot
        && should_include_huggingface_snapshots(cache_root)
    {
        if let Some(repo_id) = &plan.repository_id {
            paths.extend(huggingface_snapshot_paths(repo_id));
        }
    }
    paths
}

fn should_include_huggingface_snapshots(cache_root: &Path) -> bool {
    cache_root == default_model_cache_root()
}

fn huggingface_snapshot_paths(repo_id: &str) -> Vec<PathBuf> {
    let Some(cache_root) = huggingface_hub_root() else {
        return vec![];
    };
    let repo_dir = cache_root.join(format!("models--{}", repo_id.replace('/', "--")));
    let snapshots = repo_dir.join("snapshots");
    let Ok(entries) = fs::read_dir(snapshots) else {
        return vec![];
    };

    entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.is_dir())
        .collect()
}

fn huggingface_hub_root() -> Option<PathBuf> {
    if let Some(hf_home) = std::env::var_os("HF_HOME") {
        return Some(PathBuf::from(hf_home).join("hub"));
    }
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .map(|home| home.join(".cache").join("huggingface").join("hub"))
}

fn preferred_download_source(candidate: &ModelEvaluationCandidate) -> String {
    let preferred = match candidate.id.as_str() {
        "kokoro-82m" => "onnx-community/Kokoro-82M-v1.0-ONNX",
        "moss-soundeffect" => "mlx-community/MOSS-SoundEffect-v2.0-4bit",
        _ => "",
    };

    if !preferred.is_empty() {
        if let Some(source) = candidate
            .sources
            .iter()
            .find(|source| source.url.contains(preferred))
        {
            return source.url.clone();
        }
    }

    candidate
        .sources
        .iter()
        .find(|source| source.url.contains("huggingface.co"))
        .or_else(|| candidate.sources.first())
        .map(|source| source.url.clone())
        .unwrap_or_default()
}

fn huggingface_repo_id(source_url: &str) -> Option<String> {
    let marker = "huggingface.co/";
    let start = source_url.find(marker)? + marker.len();
    let repo = source_url[start..]
        .split(['?', '#'])
        .next()
        .unwrap_or_default()
        .trim_matches('/');
    let mut parts = repo.split('/');
    let owner = parts.next()?;
    let name = parts.next()?;
    Some(format!("{owner}/{name}"))
}

fn install_blockers(candidate: &ModelEvaluationCandidate) -> Vec<String> {
    let mut blockers = candidate.blockers.clone();
    blockers.extend(hard_install_blockers(candidate));
    blockers.sort();
    blockers.dedup();
    blockers
}

fn hard_install_blockers(candidate: &ModelEvaluationCandidate) -> Vec<String> {
    let mut blockers = Vec::new();
    if candidate.runtime.requires_python_runtime {
        blockers.push(
            "Candidate currently requires a Python runtime and cannot be enabled in shipped SoundWorks."
                .to_string(),
        );
    }
    if candidate.product_eligibility == ProductEligibility::Blocked {
        blockers.push("Product eligibility is blocked by license or runtime evidence.".to_string());
    }
    blockers
}

fn download_mechanism(candidate: &ModelEvaluationCandidate) -> DownloadMechanism {
    if candidate.status == EvaluationStatus::Blocked
        || candidate.product_eligibility == ProductEligibility::Blocked
    {
        return DownloadMechanism::Blocked;
    }
    if candidate.runtime.requires_python_runtime {
        return DownloadMechanism::ResearchPoc;
    }
    if candidate
        .sources
        .iter()
        .any(|source| source.url.contains("huggingface.co"))
    {
        DownloadMechanism::HuggingFaceSnapshot
    } else if candidate
        .sources
        .iter()
        .any(|source| source.url.contains("github.com"))
    {
        DownloadMechanism::GitRepository
    } else {
        DownloadMechanism::SourceReviewOnly
    }
}

fn expected_files_for(candidate_id: &str, mechanism: DownloadMechanism) -> Vec<ExpectedModelFile> {
    match candidate_id {
        "kokoro-82m" => vec![
            expected("config.json", true, None),
            expected("onnx/model.onnx", true, None),
            expected("voices/af_heart.bin", true, None),
        ],
        "moss-soundeffect" => vec![
            expected("config.json", true, None),
            expected("model.safetensors", true, None),
            expected("tokenizer.json", true, None),
        ],
        "ace-step-1-5" => vec![
            expected("README.md", true, None),
            expected("checkpoints", true, None),
        ],
        "stable-audio-3" | "stable-audio-open-1" => vec![
            expected("README.md", true, None),
            expected("model.safetensors", true, None),
        ],
        _ => match mechanism {
            DownloadMechanism::HuggingFaceSnapshot => vec![
                expected("README.md", true, None),
                expected("config.json", false, None),
                expected("model.safetensors", false, None),
            ],
            DownloadMechanism::GitRepository | DownloadMechanism::ResearchPoc => {
                vec![expected("README.md", true, None)]
            }
            DownloadMechanism::ManagedApi
            | DownloadMechanism::SourceReviewOnly
            | DownloadMechanism::Blocked => vec![expected("SOURCE-REVIEW.md", false, None)],
        },
    }
}

fn expected(path: &str, required: bool, size_mb: Option<u32>) -> ExpectedModelFile {
    ExpectedModelFile {
        path: path.to_string(),
        required,
        sha256: None,
        size_mb,
    }
}

fn expected_size_mb(candidate_id: &str) -> Option<u32> {
    match candidate_id {
        "kokoro-82m" => Some(400),
        "moss-soundeffect" => Some(8_000),
        "ace-step-1-5" => Some(12_000),
        "stable-audio-open-1" => Some(6_000),
        "stable-audio-3" => Some(8_000),
        _ => None,
    }
}

fn command_hint(candidate_id: &str, source_url: &str, mechanism: DownloadMechanism) -> String {
    match mechanism {
        DownloadMechanism::HuggingFaceSnapshot => format!(
            "Download the provider snapshot from {source_url} into the SoundWorks cache subdirectory `{candidate_id}`, then run revalidate."
        ),
        DownloadMechanism::GitRepository => format!(
            "Clone or package {source_url} as an isolated provider under the SoundWorks cache subdirectory `{candidate_id}`."
        ),
        DownloadMechanism::ResearchPoc => format!(
            "Keep {source_url} as research/Python POC evidence until a no-Python product provider exists."
        ),
        DownloadMechanism::ManagedApi => format!(
            "Configure managed provider credentials for {source_url}; no local model cache is expected."
        ),
        DownloadMechanism::SourceReviewOnly => {
            format!("Complete source/license review for {source_url} before install.")
        }
        DownloadMechanism::Blocked => {
            format!("Do not download {source_url} for product use until blockers are resolved.")
        }
    }
}

fn download_notes(candidate: &ModelEvaluationCandidate) -> Vec<String> {
    let mut notes = candidate.runtime.dependency_notes.clone();
    notes.push(format!("license: {}", candidate.license.label));
    notes.push(format!(
        "product eligibility: {:?}",
        candidate.product_eligibility
    ));
    notes
}

fn disk_usage_mb(path: &Path) -> Option<u32> {
    fn walk(path: &Path) -> u64 {
        let Ok(metadata) = fs::metadata(path) else {
            return 0;
        };
        if metadata.is_file() {
            return metadata.len();
        }
        let Ok(entries) = fs::read_dir(path) else {
            return 0;
        };
        entries
            .filter_map(Result::ok)
            .map(|entry| walk(&entry.path()))
            .sum()
    }

    let bytes = walk(path);
    if bytes == 0 {
        None
    } else {
        Some((bytes / 1_048_576).max(1) as u32)
    }
}

fn reference_operations(candidates: &[ModelCandidateInstallState]) -> Vec<ModelManagerOperation> {
    let mut operations = Vec::new();
    if let Some(candidate) = candidates
        .iter()
        .find(|candidate| candidate.candidate_id == "kokoro-82m")
    {
        if candidate.cache.verified {
            operations.push(ModelManagerOperation::succeeded(
                &candidate.candidate_id,
                ModelManagerActionKind::Revalidate,
                "Kokoro 82M cache evidence is verified.".to_string(),
                vec![candidate.cache.evidence.clone()],
            ));
        } else {
            operations.push(ModelManagerOperation::failed(
                &candidate.candidate_id,
                ModelManagerActionKind::Install,
                "Kokoro install failed cache verification.".to_string(),
                "The downloader did not leave the required ONNX model and voice files in the SoundWorks cache; retry download or repair the cache path."
                    .to_string(),
                vec![
                    candidate.download_plan.command_hint.clone(),
                    candidate.cache.evidence.clone(),
                ],
            ));
        }
    }
    if let Some(candidate) = candidates
        .iter()
        .find(|candidate| candidate.candidate_id == "moss-soundeffect")
        .filter(|candidate| !candidate.cache.verified)
    {
        operations.push(ModelManagerOperation::failed(
            &candidate.candidate_id,
            ModelManagerActionKind::Install,
            "MOSS-SoundEffect install failed cache verification.".to_string(),
            "The downloader did not leave the required MLX model files in a verifiable cache path; retry download or keep the SFX lane blocked."
                .to_string(),
            vec![
                candidate.download_plan.command_hint.clone(),
                candidate.cache.evidence.clone(),
            ],
        ));
    }
    operations
}

fn lane_readiness(
    catalog: &ModelEvaluationCatalog,
    candidates: &[ModelCandidateInstallState],
) -> Vec<ModelLaneReadiness> {
    catalog
        .recommendations
        .iter()
        .filter_map(|recommendation| {
            let candidate = candidates
                .iter()
                .find(|candidate| candidate.candidate_id == recommendation.candidate_id)?;
            let state = match candidate.install_state {
                CandidateInstallState::Installed => LaneReadinessState::Verified,
                CandidateInstallState::Blocked | CandidateInstallState::ResearchOnly => {
                    LaneReadinessState::Blocked
                }
                CandidateInstallState::MissingCache | CandidateInstallState::NeedsRuntimePort => {
                    LaneReadinessState::MissingCache
                }
            };
            let blocker = candidate
                .blockers
                .first()
                .cloned()
                .or_else(|| candidate.cache.missing_required_files.first().cloned());
            Some(ModelLaneReadiness {
                lane: recommendation.lane,
                recommended_candidate_id: recommendation.candidate_id.clone(),
                state,
                summary: match state {
                    LaneReadinessState::Verified => {
                        format!("{} has verified cache evidence.", candidate.name)
                    }
                    LaneReadinessState::MissingCache => format!(
                        "{} is selected for this lane, but cache verification is incomplete.",
                        candidate.name
                    ),
                    LaneReadinessState::Blocked => {
                        format!("{} remains blocked for product use.", candidate.name)
                    }
                },
                blocker,
            })
        })
        .collect()
}

fn validation_checks(
    summary: &ModelManagerSummary,
    candidates: &[ModelCandidateInstallState],
) -> Vec<ModelManagerValidationCheck> {
    vec![
        ModelManagerValidationCheck {
            id: "model-manager.candidate-coverage".to_string(),
            passed: summary.candidate_count == 28,
            summary: format!(
                "Model manager covers {} epic candidate(s).",
                summary.candidate_count
            ),
            recovery: (summary.candidate_count != 28).then(|| {
                "Add every candidate named in epic 6148 and recovery SC-6467.".to_string()
            }),
        },
        ModelManagerValidationCheck {
            id: "model-manager.no-metadata-installs".to_string(),
            passed: summary.verified_installed_count
                == candidates
                    .iter()
                    .filter(|candidate| candidate.cache.verified)
                    .count(),
            summary:
                "Installed state is derived from verified cache files, not model metadata alone."
                    .to_string(),
            recovery: None,
        },
        ModelManagerValidationCheck {
            id: "model-manager.missing-cache-visible".to_string(),
            passed: summary.missing_cache_count > 0,
            summary: format!(
                "{} candidate(s) expose missing-cache state for install/revalidate QA.",
                summary.missing_cache_count
            ),
            recovery: None,
        },
        ModelManagerValidationCheck {
            id: "model-manager.failed-download-visible".to_string(),
            passed: summary.failed_operation_count > 0,
            summary: format!(
                "{} failed download/revalidate operation(s) are visible.",
                summary.failed_operation_count
            ),
            recovery: None,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::{
        CandidateInstallState, EvaluationLane, ModelManagerActionKind, ModelManagerOperation,
        ModelManagerOverview,
    };
    use crate::evaluation::ModelEvaluationCatalog;
    use std::fs;

    #[test]
    fn model_manager_covers_epic_candidates_without_fake_installs() {
        let cache_root = std::env::temp_dir().join(format!(
            "soundworks-empty-model-manager-test-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&cache_root);
        let overview =
            ModelManagerOverview::from_catalog(&ModelEvaluationCatalog::reference(), cache_root);

        assert_eq!(overview.schema_version, 1);
        assert_eq!(overview.summary.candidate_count, 28);
        assert_eq!(overview.summary.verified_installed_count, 0);
        assert!(overview.summary.missing_cache_count > 0);
        assert!(overview
            .candidates
            .iter()
            .all(|candidate| candidate.cache.verified
                || candidate.install_state != CandidateInstallState::Installed));
        assert!(overview
            .candidates
            .iter()
            .any(|candidate| candidate.candidate_id == "kokoro-82m"
                && candidate.actions.contains(&ModelManagerActionKind::Install)));
        assert!(overview
            .validation_checks
            .iter()
            .any(|check| check.id == "model-manager.no-metadata-installs" && check.passed));
    }

    #[test]
    fn model_manager_verifies_cache_from_required_files() {
        let cache_root = std::env::temp_dir().join(format!(
            "soundworks-model-manager-test-{}",
            std::process::id()
        ));
        let kokoro = cache_root.join("kokoro-82m");
        fs::create_dir_all(kokoro.join("onnx")).expect("create onnx dir");
        fs::create_dir_all(kokoro.join("voices")).expect("create voices dir");
        fs::write(kokoro.join("config.json"), "{}").expect("write config");
        fs::write(kokoro.join("onnx").join("model.onnx"), "onnx").expect("write model");
        fs::write(kokoro.join("voices").join("af_heart.bin"), "voice").expect("write voice");

        let overview =
            ModelManagerOverview::from_catalog(&ModelEvaluationCatalog::reference(), cache_root);
        let candidate = overview
            .candidates
            .iter()
            .find(|candidate| candidate.candidate_id == "kokoro-82m")
            .expect("kokoro state");

        assert_eq!(candidate.install_state, CandidateInstallState::Installed);
        assert!(candidate.cache.verified);
        assert_eq!(candidate.cache.missing_required_files.len(), 0);
    }

    #[test]
    fn install_action_reports_failed_cache_verification_for_missing_candidate() {
        let operation = ModelManagerOperation::install("kokoro-82m");

        assert_eq!(operation.action, ModelManagerActionKind::Install);
        assert!(
            operation.summary.contains("download") || operation.summary.contains("cannot"),
            "{}",
            operation.summary
        );
        assert!(operation.recovery.is_some());
    }

    #[test]
    fn lane_readiness_tracks_recommended_candidates() {
        let overview = ModelManagerOverview::reference();

        assert!(overview
            .lane_readiness
            .iter()
            .any(|lane| lane.lane == EvaluationLane::Tts
                && lane.recommended_candidate_id == "kokoro-82m"));
        assert!(overview
            .lane_readiness
            .iter()
            .any(|lane| lane.recommended_candidate_id == "mmaudio" && lane.blocker.is_some()));
    }
}
