use crate::domain::ModelRuntime;
use crate::manifests::CapabilityWorkflow;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const MODEL_EVALUATION_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelEvaluationCatalog {
    pub schema_version: u32,
    pub candidates: Vec<ModelEvaluationCandidate>,
    pub fixtures: Vec<EvaluationFixture>,
    pub lane_scorecards: Vec<LaneScorecard>,
    pub recommendations: Vec<ModelRecommendation>,
}

impl ModelEvaluationCatalog {
    pub fn reference() -> Self {
        let candidates = reference_candidates();
        let fixtures = reference_fixtures();
        let lane_scorecards = reference_lane_scorecards();
        let recommendations = reference_recommendations();

        Self {
            schema_version: MODEL_EVALUATION_SCHEMA_VERSION,
            candidates,
            fixtures,
            lane_scorecards,
            recommendations,
        }
    }

    pub fn overview(&self) -> ModelEvaluationOverview {
        ModelEvaluationOverview::from_catalog(self)
    }

    pub fn candidates_for_lane(&self, lane: EvaluationLane) -> Vec<&ModelEvaluationCandidate> {
        self.candidates
            .iter()
            .filter(|candidate| candidate.lanes.contains(&lane))
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelEvaluationOverview {
    pub schema_version: u32,
    pub candidate_count: usize,
    pub source_count: usize,
    pub fixture_count: usize,
    pub lane_count: usize,
    pub status_counts: BTreeMap<EvaluationStatus, usize>,
    pub product_eligibility_counts: BTreeMap<ProductEligibility, usize>,
    pub recommended_candidate_ids: Vec<String>,
}

impl ModelEvaluationOverview {
    pub fn from_catalog(catalog: &ModelEvaluationCatalog) -> Self {
        let mut status_counts = BTreeMap::new();
        let mut product_eligibility_counts = BTreeMap::new();

        for candidate in &catalog.candidates {
            *status_counts.entry(candidate.status).or_insert(0) += 1;
            *product_eligibility_counts
                .entry(candidate.product_eligibility)
                .or_insert(0) += 1;
        }

        Self {
            schema_version: catalog.schema_version,
            candidate_count: catalog.candidates.len(),
            source_count: catalog
                .candidates
                .iter()
                .map(|candidate| candidate.sources.len())
                .sum(),
            fixture_count: catalog.fixtures.len(),
            lane_count: catalog.lane_scorecards.len(),
            status_counts,
            product_eligibility_counts,
            recommended_candidate_ids: catalog
                .recommendations
                .iter()
                .map(|recommendation| recommendation.candidate_id.clone())
                .collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelEvaluationCandidate {
    pub id: String,
    pub name: String,
    pub provider: String,
    pub lanes: Vec<EvaluationLane>,
    pub sources: Vec<EvidenceSource>,
    pub license: CandidateLicense,
    pub runtime: EvaluationRuntimeProfile,
    pub status: EvaluationStatus,
    pub product_eligibility: ProductEligibility,
    pub evidence_level: EvidenceLevel,
    pub smoke_tests: Vec<SmokeTestPlan>,
    pub score_focus: Vec<ScoreFocus>,
    pub blockers: Vec<String>,
    pub notes: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EvidenceSource {
    pub label: String,
    pub url: String,
    pub source_type: EvidenceSourceType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum EvidenceSourceType {
    OfficialRepo,
    ModelCard,
    Paper,
    ProjectPage,
    Docs,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CandidateLicense {
    pub label: String,
    pub commercial_use: CommercialUseEvaluation,
    pub safety_notes: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CommercialUseEvaluation {
    Allowed,
    ProviderTerms,
    NonCommercial,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EvaluationRuntimeProfile {
    pub runtime: ModelRuntime,
    pub product_path: ProductRuntimePath,
    pub requires_python_runtime: bool,
    pub mac_packaging: PackagingAssessment,
    pub windows_packaging: PackagingAssessment,
    pub dependency_notes: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProductRuntimePath {
    RustNative,
    NativeLibraryBinding,
    ExternalExecutable,
    ManagedApi,
    PythonPocOnly,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PackagingAssessment {
    Supported,
    NeedsValidation,
    Blocked,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum EvaluationStatus {
    Ready,
    PromisingSpike,
    Blocked,
    Unsuitable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProductEligibility {
    ProductCandidate,
    NeedsRuntimePort,
    ApiOnlyCandidate,
    ResearchOnly,
    Blocked,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum EvidenceLevel {
    SourceMetadata,
    InstallDocumented,
    SmokeTestPlanned,
    RunnableEvidence,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum EvaluationLane {
    Tts,
    VoiceClone,
    VoiceConversion,
    Sfx,
    Ambience,
    InstrumentSample,
    Loop,
    Song,
    StemSeparation,
    VideoToAudio,
}

impl EvaluationLane {
    pub fn workflow(self) -> CapabilityWorkflow {
        match self {
            Self::Tts => CapabilityWorkflow::Tts,
            Self::VoiceClone => CapabilityWorkflow::VoiceClone,
            Self::VoiceConversion => CapabilityWorkflow::VoiceConversion,
            Self::Sfx => CapabilityWorkflow::Sfx,
            Self::Ambience => CapabilityWorkflow::Ambience,
            Self::InstrumentSample => CapabilityWorkflow::InstrumentSample,
            Self::Loop => CapabilityWorkflow::Loop,
            Self::Song => CapabilityWorkflow::Song,
            Self::StemSeparation => CapabilityWorkflow::StemSeparation,
            Self::VideoToAudio => CapabilityWorkflow::VideoToAudio,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SmokeTestPlan {
    Install,
    Load,
    FirstGeneration,
    Cancellation,
    ErrorRecovery,
    Repeatability,
    LicenseGate,
    PackagingPreflight,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ScoreFocus {
    Intelligibility,
    SpeakerConsistency,
    ProsodyControl,
    PromptAdherence,
    Loopability,
    Isolation,
    BpmKeyUsefulness,
    LyricAlignment,
    SongStructure,
    AudioVisualSync,
    DurationStability,
    LoudnessClipping,
    RuntimeFootprint,
    CommercialSafety,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EvaluationFixture {
    pub id: String,
    pub lane: EvaluationLane,
    pub name: String,
    pub prompt_or_script: String,
    pub expected_outputs: Vec<String>,
    pub measurements: Vec<EvaluationMeasurement>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum EvaluationMeasurement {
    LatencyMs,
    WarmupMs,
    PeakMemoryMb,
    OutputDurationMs,
    SampleRateHz,
    LoudnessLufs,
    TruePeakDbfs,
    Clipping,
    ArtifactFrequency,
    RepeatabilityHash,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LaneScorecard {
    pub lane: EvaluationLane,
    pub axes: Vec<ScoreAxis>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScoreAxis {
    pub id: String,
    pub label: String,
    pub weight: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelRecommendation {
    pub lane: EvaluationLane,
    pub candidate_id: String,
    pub status: EvaluationStatus,
    pub rationale: String,
    pub required_next_evidence: Vec<SmokeTestPlan>,
}

fn reference_candidates() -> Vec<ModelEvaluationCandidate> {
    vec![
        candidate(
            "stable-audio-3",
            "Stable Audio 3",
            "Stability AI",
            vec![EvaluationLane::Song, EvaluationLane::Sfx, EvaluationLane::Ambience],
            vec![
                source("Stability AI product page", "https://stability.ai/stable-audio", EvidenceSourceType::ProjectPage),
                source("Stable Audio 3 GitHub", "https://github.com/Stability-AI/stable-audio-3", EvidenceSourceType::OfficialRepo),
                source("Stable Audio 3 Medium model card", "https://huggingface.co/stabilityai/stable-audio-3-medium", EvidenceSourceType::ModelCard),
            ],
            license("Stability AI Community License / Enterprise terms", CommercialUseEvaluation::ProviderTerms),
            runtime(ProductRuntimePath::ExternalExecutable, false, PackagingAssessment::NeedsValidation, PackagingAssessment::NeedsValidation, &["open weights and inference pipeline; product packaging still needs local dependency audit"]),
            EvaluationStatus::PromisingSpike,
            ProductEligibility::NeedsRuntimePort,
            EvidenceLevel::SourceMetadata,
            vec![ScoreFocus::PromptAdherence, ScoreFocus::SongStructure, ScoreFocus::DurationStability, ScoreFocus::CommercialSafety],
            vec!["Community license and enterprise threshold must be enforced before product enablement"],
            "Strong broad audio candidate, but SoundWorks needs runnable Mac/Windows packaging proof and license gating.",
        ),
        candidate(
            "ace-step-1-5",
            "ACE-Step 1.5",
            "ACE-Step",
            vec![EvaluationLane::Song, EvaluationLane::Loop],
            vec![
                source("ACE-Step 1.5 GitHub", "https://github.com/ace-step/ACE-Step-1.5", EvidenceSourceType::OfficialRepo),
                source("ACE-Step project page", "https://ace-step.github.io/", EvidenceSourceType::ProjectPage),
            ],
            license("MIT", CommercialUseEvaluation::Allowed),
            runtime(ProductRuntimePath::ExternalExecutable, false, PackagingAssessment::NeedsValidation, PackagingAssessment::NeedsValidation, &["local music generation repo; device coverage claims need packaged proof"]),
            EvaluationStatus::PromisingSpike,
            ProductEligibility::NeedsRuntimePort,
            EvidenceLevel::SourceMetadata,
            vec![ScoreFocus::LyricAlignment, ScoreFocus::SongStructure, ScoreFocus::RuntimeFootprint],
            vec!["Local runtime must be isolated as a provider package rather than bundled as Python product dependency"],
            "High-priority music spike because license is permissive and local execution is documented.",
        ),
        candidate(
            "levo-2",
            "LeVo 2 / SongGeneration 2",
            "Tencent AI Lab",
            vec![EvaluationLane::Song],
            vec![
                source("SongGeneration GitHub", "https://github.com/tencent-ailab/songgeneration", EvidenceSourceType::OfficialRepo),
                source("Tencent SongGeneration model card", "https://huggingface.co/tencent/SongGeneration", EvidenceSourceType::ModelCard),
            ],
            license("Source-backed license review required", CommercialUseEvaluation::Unknown),
            runtime(ProductRuntimePath::PythonPocOnly, true, PackagingAssessment::Blocked, PackagingAssessment::Blocked, &["Python research stack expected until a product-safe execution wrapper is defined"]),
            EvaluationStatus::PromisingSpike,
            ProductEligibility::ResearchOnly,
            EvidenceLevel::SourceMetadata,
            vec![ScoreFocus::LyricAlignment, ScoreFocus::SongStructure, ScoreFocus::DurationStability],
            vec!["Commercial rights and no-Python product path are unresolved"],
            "Useful comparison target for complete-song quality; not product-eligible without license/runtime work.",
        ),
        candidate(
            "yue",
            "YuE",
            "M-A-P",
            vec![EvaluationLane::Song],
            vec![
                source("YuE GitHub", "https://github.com/multimodal-art-projection/YuE", EvidenceSourceType::OfficialRepo),
                source("YuE project page", "https://map-yue.github.io/", EvidenceSourceType::ProjectPage),
                source("YuE OpenReview", "https://openreview.net/forum?id=hZy6YG2Ij8", EvidenceSourceType::Paper),
            ],
            license("Open-source model license requires product review", CommercialUseEvaluation::Unknown),
            runtime(ProductRuntimePath::PythonPocOnly, true, PackagingAssessment::Blocked, PackagingAssessment::Blocked, &["long-form model expected to be compute-heavy; no Rust/native product path yet"]),
            EvaluationStatus::PromisingSpike,
            ProductEligibility::ResearchOnly,
            EvidenceLevel::SourceMetadata,
            vec![ScoreFocus::LyricAlignment, ScoreFocus::SongStructure, ScoreFocus::DurationStability],
            vec!["Compute footprint and product license must be resolved"],
            "Long-form lyrics-to-song baseline; compare against faster candidates before product work.",
        ),
        candidate(
            "diffrhythm-2",
            "DiffRhythm 2",
            "ASLP Lab / Xiaomi Research",
            vec![EvaluationLane::Song],
            vec![
                source("DiffRhythm GitHub", "https://github.com/ASLP-lab/DiffRhythm", EvidenceSourceType::OfficialRepo),
                source("DiffRhythm 2 model card", "https://huggingface.co/ASLP-lab/DiffRhythm2", EvidenceSourceType::ModelCard),
            ],
            license("Source-backed license review required", CommercialUseEvaluation::Unknown),
            runtime(ProductRuntimePath::PythonPocOnly, true, PackagingAssessment::Blocked, PackagingAssessment::Blocked, &["research inference code and checkpoints; product runtime path unknown"]),
            EvaluationStatus::PromisingSpike,
            ProductEligibility::ResearchOnly,
            EvidenceLevel::SourceMetadata,
            vec![ScoreFocus::LyricAlignment, ScoreFocus::DurationStability, ScoreFocus::RuntimeFootprint],
            vec!["No no-Python runtime path yet"],
            "Candidate for fast complete-song smoke tests, but currently research-only for SoundWorks.",
        ),
        candidate(
            "khala",
            "Khala",
            "Khala Music AI",
            vec![EvaluationLane::Song],
            vec![
                source("Khala GitHub", "https://github.com/Khala-Music-AI/Khala", EvidenceSourceType::OfficialRepo),
                source("Khala arXiv", "https://arxiv.org/pdf/2605.01790", EvidenceSourceType::Paper),
            ],
            license("Source-backed license review required", CommercialUseEvaluation::Unknown),
            runtime(ProductRuntimePath::PythonPocOnly, true, PackagingAssessment::Blocked, PackagingAssessment::Blocked, &["full-song research stack needs packaging audit"]),
            EvaluationStatus::PromisingSpike,
            ProductEligibility::ResearchOnly,
            EvidenceLevel::SourceMetadata,
            vec![ScoreFocus::LyricAlignment, ScoreFocus::SongStructure, ScoreFocus::CommercialSafety],
            vec!["License and runtime isolation are unresolved"],
            "Promising full-song system for comparison, not a product candidate yet.",
        ),
        candidate(
            "heartmula",
            "HeartMuLa",
            "HeartMuLa",
            vec![EvaluationLane::Song, EvaluationLane::Loop],
            vec![
                source("HeartMuLa GitHub", "https://github.com/HeartMuLa/heartlib", EvidenceSourceType::OfficialRepo),
                source("HeartMuLa arXiv", "https://arxiv.org/html/2601.10547v1", EvidenceSourceType::Paper),
            ],
            license("Source-backed license review required", CommercialUseEvaluation::Unknown),
            runtime(ProductRuntimePath::PythonPocOnly, true, PackagingAssessment::Blocked, PackagingAssessment::Blocked, &["open model family; packaged runtime and dependency footprint unknown"]),
            EvaluationStatus::PromisingSpike,
            ProductEligibility::ResearchOnly,
            EvidenceLevel::SourceMetadata,
            vec![ScoreFocus::SongStructure, ScoreFocus::BpmKeyUsefulness, ScoreFocus::RuntimeFootprint],
            vec!["Mac/Windows packaging evidence missing"],
            "Model-family candidate for song and loop comparisons.",
        ),
        candidate(
            "muse-song",
            "Muse",
            "Muse authors",
            vec![EvaluationLane::Song],
            vec![
                source("Muse GitHub", "https://github.com/yuhui1038/Muse", EvidenceSourceType::OfficialRepo),
                source("Muse arXiv", "https://arxiv.org/pdf/2601.03973", EvidenceSourceType::Paper),
            ],
            license("Source-backed license review required", CommercialUseEvaluation::Unknown),
            runtime(ProductRuntimePath::PythonPocOnly, true, PackagingAssessment::Blocked, PackagingAssessment::Blocked, &["research repo with evaluation pipeline; product runtime path unknown"]),
            EvaluationStatus::PromisingSpike,
            ProductEligibility::ResearchOnly,
            EvidenceLevel::SourceMetadata,
            vec![ScoreFocus::SongStructure, ScoreFocus::PromptAdherence, ScoreFocus::LyricAlignment],
            vec!["License, runtime, and artifact packaging need validation"],
            "Useful long-form style-control benchmark.",
        ),
        candidate(
            "kokoro-82m",
            "Kokoro 82M",
            "hexgrad",
            vec![EvaluationLane::Tts],
            vec![
                source("Kokoro model card", "https://huggingface.co/hexgrad/Kokoro-82M", EvidenceSourceType::ModelCard),
                source("Kokoro GitHub", "https://github.com/hexgrad/kokoro", EvidenceSourceType::OfficialRepo),
                source("Kokoro ONNX model card", "https://huggingface.co/onnx-community/Kokoro-82M-v1.0-ONNX", EvidenceSourceType::ModelCard),
            ],
            license("Apache-licensed weights", CommercialUseEvaluation::Allowed),
            runtime(ProductRuntimePath::NativeLibraryBinding, false, PackagingAssessment::NeedsValidation, PackagingAssessment::NeedsValidation, &["ONNX and Rust community paths make this a strong no-Python candidate"]),
            EvaluationStatus::PromisingSpike,
            ProductEligibility::ProductCandidate,
            EvidenceLevel::InstallDocumented,
            vec![ScoreFocus::Intelligibility, ScoreFocus::RuntimeFootprint, ScoreFocus::CommercialSafety],
            vec!["Need SoundWorks-owned smoke output and latency/memory measurements"],
            "Best first TTS spike because it is small, permissively licensed, and has plausible native runtime paths.",
        ),
        candidate(
            "vibevoice",
            "VibeVoice",
            "Microsoft",
            vec![EvaluationLane::Tts],
            vec![
                source("VibeVoice GitHub", "https://github.com/microsoft/VibeVoice", EvidenceSourceType::OfficialRepo),
                source("VibeVoice project page", "https://microsoft.github.io/VibeVoice/", EvidenceSourceType::ProjectPage),
            ],
            license("Microsoft repository license requires review", CommercialUseEvaluation::Unknown),
            runtime(ProductRuntimePath::PythonPocOnly, true, PackagingAssessment::Blocked, PackagingAssessment::Blocked, &["long-form TTS stack currently Python/PyTorch-oriented"]),
            EvaluationStatus::PromisingSpike,
            ProductEligibility::ResearchOnly,
            EvidenceLevel::SourceMetadata,
            vec![ScoreFocus::Intelligibility, ScoreFocus::SpeakerConsistency, ScoreFocus::DurationStability],
            vec!["License and no-Python product runtime path unresolved"],
            "Strong long-form multi-speaker benchmark; not first product provider.",
        ),
        candidate(
            "xtts-v2",
            "XTTS-v2",
            "Coqui",
            vec![EvaluationLane::Tts, EvaluationLane::VoiceClone],
            vec![
                source("XTTS-v2 model card", "https://huggingface.co/coqui/XTTS-v2", EvidenceSourceType::ModelCard),
                source("Coqui XTTS docs", "https://github.com/coqui-ai/TTS/blob/dev/docs/source/models/xtts.md", EvidenceSourceType::Docs),
            ],
            license("Coqui Public Model License", CommercialUseEvaluation::NonCommercial),
            runtime(ProductRuntimePath::PythonPocOnly, true, PackagingAssessment::Blocked, PackagingAssessment::Blocked, &["Python TTS stack; model card says 6-second voice reference while docs mention 3 seconds"]),
            EvaluationStatus::Blocked,
            ProductEligibility::Blocked,
            EvidenceLevel::SourceMetadata,
            vec![ScoreFocus::SpeakerConsistency, ScoreFocus::CommercialSafety],
            vec!["Non-commercial model license blocks shipped product use", "Reference clip requirement must be normalized as 6 seconds from the model card unless docs are reconciled"],
            "Keep as a comparison/reference only unless licensing changes.",
        ),
        candidate(
            "chattts",
            "ChatTTS",
            "2Noise",
            vec![EvaluationLane::Tts],
            vec![
                source("ChatTTS GitHub", "https://github.com/2noise/ChatTTS", EvidenceSourceType::OfficialRepo),
                source("ChatTTS model card", "https://huggingface.co/2Noise/ChatTTS", EvidenceSourceType::ModelCard),
            ],
            license("AGPLv3+ code / CC BY-NC 4.0 model", CommercialUseEvaluation::NonCommercial),
            runtime(ProductRuntimePath::PythonPocOnly, true, PackagingAssessment::Blocked, PackagingAssessment::Blocked, &["Python/Torch stack and non-commercial model"]),
            EvaluationStatus::Blocked,
            ProductEligibility::Blocked,
            EvidenceLevel::SourceMetadata,
            vec![ScoreFocus::Intelligibility, ScoreFocus::CommercialSafety],
            vec!["Non-commercial model license blocks product use"],
            "Research comparison only.",
        ),
        candidate(
            "fish-speech",
            "Fish Speech",
            "Fish Audio",
            vec![EvaluationLane::Tts, EvaluationLane::VoiceClone],
            vec![
                source("Fish Speech GitHub", "https://github.com/fishaudio/fish-speech", EvidenceSourceType::OfficialRepo),
                source("Fish Speech docs", "https://github.com/fishaudio/fish-speech/blob/main/docs/en/index.md", EvidenceSourceType::Docs),
            ],
            license("Apache-2.0 code; model terms require version-specific review", CommercialUseEvaluation::Unknown),
            runtime(ProductRuntimePath::ExternalExecutable, false, PackagingAssessment::NeedsValidation, PackagingAssessment::NeedsValidation, &["Rust/Candle community path exists for older Fish Speech; current model path must be verified"]),
            EvaluationStatus::PromisingSpike,
            ProductEligibility::NeedsRuntimePort,
            EvidenceLevel::SourceMetadata,
            vec![ScoreFocus::Intelligibility, ScoreFocus::SpeakerConsistency, ScoreFocus::ProsodyControl],
            vec!["Need current model license and no-Python runtime proof"],
            "Good quality candidate, but product eligibility depends on version-specific licensing and runtime path.",
        ),
        candidate(
            "chatterbox",
            "Chatterbox",
            "Resemble AI",
            vec![EvaluationLane::Tts, EvaluationLane::VoiceClone, EvaluationLane::VoiceConversion],
            vec![
                source("Chatterbox GitHub", "https://github.com/resemble-ai/chatterbox", EvidenceSourceType::OfficialRepo),
                source("Resemble Chatterbox page", "https://www.resemble.ai/learn/models/chatterbox", EvidenceSourceType::ProjectPage),
                source("Chatterbox model card", "https://huggingface.co/ResembleAI/chatterbox", EvidenceSourceType::ModelCard),
            ],
            license("MIT", CommercialUseEvaluation::Allowed),
            runtime(ProductRuntimePath::ExternalExecutable, false, PackagingAssessment::NeedsValidation, PackagingAssessment::NeedsValidation, &["Python package today, but permissive license allows isolated executable/provider packaging"]),
            EvaluationStatus::PromisingSpike,
            ProductEligibility::NeedsRuntimePort,
            EvidenceLevel::InstallDocumented,
            vec![ScoreFocus::SpeakerConsistency, ScoreFocus::ProsodyControl, ScoreFocus::CommercialSafety],
            vec!["Need no-Python product path and watermark/provenance validation"],
            "Strong voice candidate for a packaged provider spike.",
        ),
        candidate(
            "chatterbox-turbo",
            "Chatterbox Turbo",
            "Resemble AI",
            vec![EvaluationLane::Tts],
            vec![
                source("Chatterbox GitHub", "https://github.com/resemble-ai/chatterbox", EvidenceSourceType::OfficialRepo),
                source("Resemble Chatterbox page", "https://www.resemble.ai/learn/models/chatterbox", EvidenceSourceType::ProjectPage),
            ],
            license("MIT", CommercialUseEvaluation::Allowed),
            runtime(ProductRuntimePath::ExternalExecutable, false, PackagingAssessment::NeedsValidation, PackagingAssessment::NeedsValidation, &["turbo variant needs separate latency and quality measurements"]),
            EvaluationStatus::PromisingSpike,
            ProductEligibility::NeedsRuntimePort,
            EvidenceLevel::SourceMetadata,
            vec![ScoreFocus::Intelligibility, ScoreFocus::RuntimeFootprint],
            vec!["Need SoundWorks-owned realtime latency measurements"],
            "Track separately from base Chatterbox because latency is the product question.",
        ),
        candidate(
            "gpt-sovits",
            "GPT-SoVITS",
            "RVC-Boss",
            vec![EvaluationLane::Tts, EvaluationLane::VoiceClone],
            vec![
                source("GPT-SoVITS GitHub", "https://github.com/RVC-Boss/GPT-SoVITS", EvidenceSourceType::OfficialRepo),
                source("GPT-SoVITS README", "https://github.com/RVC-Boss/GPT-SoVITS/blob/main/README.md?plain=1", EvidenceSourceType::Docs),
            ],
            license("Source-backed license review required", CommercialUseEvaluation::Unknown),
            runtime(ProductRuntimePath::PythonPocOnly, true, PackagingAssessment::Blocked, PackagingAssessment::Blocked, &["Python web UI/research stack"]),
            EvaluationStatus::PromisingSpike,
            ProductEligibility::ResearchOnly,
            EvidenceLevel::SourceMetadata,
            vec![ScoreFocus::SpeakerConsistency, ScoreFocus::ProsodyControl],
            vec!["No product-safe no-Python runtime path yet"],
            "Useful zero/few-shot voice baseline; research-only until isolated runtime strategy exists.",
        ),
        candidate(
            "f5-tts",
            "F5-TTS",
            "SWivid",
            vec![EvaluationLane::Tts, EvaluationLane::VoiceClone],
            vec![
                source("F5-TTS GitHub", "https://github.com/SWivid/F5-TTS", EvidenceSourceType::OfficialRepo),
                source("F5-TTS model card", "https://huggingface.co/SWivid/F5-TTS", EvidenceSourceType::ModelCard),
            ],
            license("MIT code / CC-BY-NC pretrained models", CommercialUseEvaluation::NonCommercial),
            runtime(ProductRuntimePath::PythonPocOnly, true, PackagingAssessment::Blocked, PackagingAssessment::Blocked, &["pretrained model license is non-commercial; Python stack"]),
            EvaluationStatus::Blocked,
            ProductEligibility::Blocked,
            EvidenceLevel::SourceMetadata,
            vec![ScoreFocus::Intelligibility, ScoreFocus::SpeakerConsistency, ScoreFocus::CommercialSafety],
            vec!["Pretrained model non-commercial license blocks product use"],
            "Use only as a research comparison unless commercially trainable weights are produced.",
        ),
        candidate(
            "cosyvoice-2",
            "CosyVoice 2",
            "FunAudioLLM / Alibaba",
            vec![EvaluationLane::Tts, EvaluationLane::VoiceClone],
            vec![
                source("CosyVoice GitHub", "https://github.com/FunAudioLLM/CosyVoice", EvidenceSourceType::OfficialRepo),
                source("CosyVoice 2 project page", "https://funaudiollm.github.io/cosyvoice2/", EvidenceSourceType::ProjectPage),
                source("CosyVoice 2 arXiv", "https://arxiv.org/html/2412.10117v2", EvidenceSourceType::Paper),
            ],
            license("Source-backed license review required", CommercialUseEvaluation::Unknown),
            runtime(ProductRuntimePath::PythonPocOnly, true, PackagingAssessment::Blocked, PackagingAssessment::Blocked, &["streaming TTS research stack; product runtime path unknown"]),
            EvaluationStatus::PromisingSpike,
            ProductEligibility::ResearchOnly,
            EvidenceLevel::SourceMetadata,
            vec![ScoreFocus::Intelligibility, ScoreFocus::SpeakerConsistency, ScoreFocus::RuntimeFootprint],
            vec!["License and packaged runtime path unresolved"],
            "Evaluate for quality and streaming behavior, not as first product provider.",
        ),
        candidate(
            "openvoice-v2",
            "OpenVoice V2",
            "MyShell AI",
            vec![EvaluationLane::VoiceClone, EvaluationLane::VoiceConversion, EvaluationLane::Tts],
            vec![
                source("OpenVoice GitHub", "https://github.com/myshell-ai/OpenVoice", EvidenceSourceType::OfficialRepo),
                source("OpenVoice project page", "https://research.myshell.ai/open-voice", EvidenceSourceType::ProjectPage),
                source("OpenVoiceV2 model card", "https://huggingface.co/myshell-ai/OpenVoiceV2", EvidenceSourceType::ModelCard),
            ],
            license("MIT", CommercialUseEvaluation::Allowed),
            runtime(ProductRuntimePath::ExternalExecutable, false, PackagingAssessment::NeedsValidation, PackagingAssessment::NeedsValidation, &["Python implementation today; permissive license allows provider isolation work"]),
            EvaluationStatus::PromisingSpike,
            ProductEligibility::NeedsRuntimePort,
            EvidenceLevel::SourceMetadata,
            vec![ScoreFocus::SpeakerConsistency, ScoreFocus::ProsodyControl, ScoreFocus::CommercialSafety],
            vec!["Need consent UX and executable packaging proof"],
            "Good voice-clone/conversion candidate once consent and packaging are in place.",
        ),
        candidate(
            "rvc",
            "RVC",
            "RVC Project",
            vec![EvaluationLane::VoiceConversion],
            vec![
                source("RVC WebUI GitHub", "https://github.com/RVC-Project/Retrieval-based-Voice-Conversion-WebUI", EvidenceSourceType::OfficialRepo),
                source("RVC library GitHub", "https://github.com/RVC-Project/Retrieval-based-Voice-Conversion", EvidenceSourceType::OfficialRepo),
            ],
            license("MIT", CommercialUseEvaluation::Allowed),
            runtime(ProductRuntimePath::ExternalExecutable, false, PackagingAssessment::NeedsValidation, PackagingAssessment::NeedsValidation, &["speech-to-speech voice conversion, not TTS; Python stack can be isolated as external executable"]),
            EvaluationStatus::PromisingSpike,
            ProductEligibility::NeedsRuntimePort,
            EvidenceLevel::SourceMetadata,
            vec![ScoreFocus::SpeakerConsistency, ScoreFocus::RuntimeFootprint, ScoreFocus::CommercialSafety],
            vec!["Must be routed only as voice conversion and gated by consent"],
            "Score as speech-to-speech voice conversion, not text-to-speech.",
        ),
        candidate(
            "stable-audio-open-1",
            "Stable Audio Open 1.0",
            "Stability AI",
            vec![EvaluationLane::Sfx, EvaluationLane::Ambience, EvaluationLane::Loop],
            vec![
                source("Stable Audio Open model card", "https://huggingface.co/stabilityai/stable-audio-open-1.0", EvidenceSourceType::ModelCard),
                source("Stable Audio Open GitHub", "https://github.com/Stability-AI/stable-audio-tools", EvidenceSourceType::OfficialRepo),
            ],
            license("Stability AI Community License", CommercialUseEvaluation::ProviderTerms),
            runtime(ProductRuntimePath::ExternalExecutable, false, PackagingAssessment::NeedsValidation, PackagingAssessment::NeedsValidation, &["source-backed open model; license and local packaging need verification"]),
            EvaluationStatus::PromisingSpike,
            ProductEligibility::NeedsRuntimePort,
            EvidenceLevel::SourceMetadata,
            vec![ScoreFocus::PromptAdherence, ScoreFocus::Loopability, ScoreFocus::CommercialSafety],
            vec!["Community license terms must be enforced"],
            "SFX/music-bed candidate, but license terms make it less clean than Apache/MIT options.",
        ),
        candidate(
            "audiocraft-audiogen",
            "AudioCraft / AudioGen",
            "Meta",
            vec![EvaluationLane::Sfx],
            vec![
                source("AudioCraft GitHub", "https://github.com/facebookresearch/audiocraft", EvidenceSourceType::OfficialRepo),
                source("AudioGen docs", "https://audiocraft.metademolab.com/audiogen.html", EvidenceSourceType::Docs),
                source("AudioGen model card", "https://github.com/facebookresearch/audiocraft/blob/main/model_cards/AUDIOGEN_MODEL_CARD.md", EvidenceSourceType::ModelCard),
            ],
            license("MIT code; model card warns research use", CommercialUseEvaluation::Unknown),
            runtime(ProductRuntimePath::PythonPocOnly, true, PackagingAssessment::Blocked, PackagingAssessment::Blocked, &["PyTorch research library"]),
            EvaluationStatus::PromisingSpike,
            ProductEligibility::ResearchOnly,
            EvidenceLevel::SourceMetadata,
            vec![ScoreFocus::PromptAdherence, ScoreFocus::DurationStability],
            vec!["Model card requires downstream risk investigation before product use"],
            "Baseline SFX comparator; not first product provider.",
        ),
        candidate(
            "audioldm",
            "AudioLDM",
            "AudioLDM authors",
            vec![EvaluationLane::Sfx, EvaluationLane::Ambience],
            vec![
                source("AudioLDM GitHub", "https://github.com/haoheliu/AudioLDM", EvidenceSourceType::OfficialRepo),
                source("AudioLDM project page", "https://audioldm.github.io/", EvidenceSourceType::ProjectPage),
            ],
            license("CC BY-NC checkpoints", CommercialUseEvaluation::NonCommercial),
            runtime(ProductRuntimePath::PythonPocOnly, true, PackagingAssessment::Blocked, PackagingAssessment::Blocked, &["Python diffusion stack; non-commercial pretrained checkpoints"]),
            EvaluationStatus::Blocked,
            ProductEligibility::Blocked,
            EvidenceLevel::SourceMetadata,
            vec![ScoreFocus::PromptAdherence, ScoreFocus::CommercialSafety],
            vec!["Non-commercial checkpoint license blocks product use"],
            "Research comparison only.",
        ),
        candidate(
            "audioldm-2",
            "AudioLDM 2",
            "AudioLDM authors",
            vec![EvaluationLane::Sfx, EvaluationLane::Ambience],
            vec![
                source("AudioLDM 2 GitHub", "https://github.com/haoheliu/AudioLDM2", EvidenceSourceType::OfficialRepo),
                source("AudioLDM 2 project page", "https://audioldm.github.io/audioldm2/", EvidenceSourceType::ProjectPage),
                source("Diffusers AudioLDM2 docs", "https://huggingface.co/docs/diffusers/en/api/pipelines/audioldm2", EvidenceSourceType::Docs),
            ],
            license("Source-backed license review required; likely research/non-commercial checkpoint constraints", CommercialUseEvaluation::Unknown),
            runtime(ProductRuntimePath::PythonPocOnly, true, PackagingAssessment::Blocked, PackagingAssessment::Blocked, &["Diffusers/PyTorch path; product runtime and license unresolved"]),
            EvaluationStatus::PromisingSpike,
            ProductEligibility::ResearchOnly,
            EvidenceLevel::SourceMetadata,
            vec![ScoreFocus::PromptAdherence, ScoreFocus::DurationStability, ScoreFocus::CommercialSafety],
            vec!["License and no-Python runtime path unresolved"],
            "Evaluate as SFX baseline only.",
        ),
        candidate(
            "audiox",
            "AudioX",
            "AudioX authors",
            vec![EvaluationLane::Sfx, EvaluationLane::VideoToAudio, EvaluationLane::Ambience],
            vec![
                source("AudioX GitHub", "https://github.com/zeyuet/AudioX", EvidenceSourceType::OfficialRepo),
                source("AudioX project page", "https://zeyuet.github.io/AudioX/", EvidenceSourceType::ProjectPage),
            ],
            license("Source-backed license review required", CommercialUseEvaluation::Unknown),
            runtime(ProductRuntimePath::PythonPocOnly, true, PackagingAssessment::Blocked, PackagingAssessment::Blocked, &["multimodal diffusion transformer; product runtime path unknown"]),
            EvaluationStatus::PromisingSpike,
            ProductEligibility::ResearchOnly,
            EvidenceLevel::SourceMetadata,
            vec![ScoreFocus::PromptAdherence, ScoreFocus::AudioVisualSync, ScoreFocus::DurationStability],
            vec!["License, weights, and runtime packaging need validation"],
            "Good multimodal benchmark; not product-ready.",
        ),
        candidate(
            "mmaudio",
            "MMAudio",
            "MMAudio authors",
            vec![EvaluationLane::VideoToAudio, EvaluationLane::Sfx],
            vec![
                source("MMAudio GitHub", "https://github.com/hkchengrex/MMAudio", EvidenceSourceType::OfficialRepo),
                source("MMAudio arXiv", "https://arxiv.org/html/2412.15322v2", EvidenceSourceType::Paper),
            ],
            license("Source-backed license review required", CommercialUseEvaluation::Unknown),
            runtime(ProductRuntimePath::PythonPocOnly, true, PackagingAssessment::Blocked, PackagingAssessment::Blocked, &["PyTorch video-to-audio stack"]),
            EvaluationStatus::PromisingSpike,
            ProductEligibility::ResearchOnly,
            EvidenceLevel::SourceMetadata,
            vec![ScoreFocus::AudioVisualSync, ScoreFocus::PromptAdherence, ScoreFocus::RuntimeFootprint],
            vec!["Commercial rights and no-Python product path unresolved"],
            "Primary video-to-audio benchmark candidate.",
        ),
        candidate(
            "thinksound",
            "ThinkSound",
            "FunAudioLLM",
            vec![EvaluationLane::VideoToAudio, EvaluationLane::Sfx, EvaluationLane::Ambience],
            vec![
                source("ThinkSound GitHub", "https://github.com/FunAudioLLM/ThinkSound", EvidenceSourceType::OfficialRepo),
                source("ThinkSound arXiv", "https://arxiv.org/html/2506.21448v1", EvidenceSourceType::Paper),
                source("ThinkSound model card", "https://huggingface.co/FunAudioLLM/ThinkSound", EvidenceSourceType::ModelCard),
            ],
            license("Source-backed license review required", CommercialUseEvaluation::Unknown),
            runtime(ProductRuntimePath::PythonPocOnly, true, PackagingAssessment::Blocked, PackagingAssessment::Blocked, &["MLLM reasoning plus audio generation stack; no product packaging path yet"]),
            EvaluationStatus::PromisingSpike,
            ProductEligibility::ResearchOnly,
            EvidenceLevel::SourceMetadata,
            vec![ScoreFocus::AudioVisualSync, ScoreFocus::PromptAdherence, ScoreFocus::DurationStability],
            vec!["License and dependency footprint unresolved"],
            "Useful for reasoning-heavy video-to-audio comparisons.",
        ),
        candidate(
            "moss-soundeffect",
            "MOSS-SoundEffect",
            "OpenMOSS",
            vec![EvaluationLane::Sfx, EvaluationLane::Ambience],
            vec![
                source("MOSS-TTS GitHub", "https://github.com/OpenMOSS/MOSS-TTS", EvidenceSourceType::OfficialRepo),
                source("MOSS-SoundEffect model card", "https://github.com/OpenMOSS/MOSS-TTS/blob/main/docs/moss_sound_effect_model_card.md", EvidenceSourceType::ModelCard),
                source("MOSS-SoundEffect v2 README", "https://github.com/OpenMOSS/MOSS-TTS/blob/main/moss_soundeffect_v2/README.md", EvidenceSourceType::Docs),
                source("MOSS-SoundEffect MLX conversion", "https://huggingface.co/mlx-community/MOSS-SoundEffect-v2.0-4bit", EvidenceSourceType::ModelCard),
            ],
            license("Apache-2.0 for v2 MLX/upstream path per model card evidence", CommercialUseEvaluation::Allowed),
            runtime(ProductRuntimePath::NativeLibraryBinding, false, PackagingAssessment::NeedsValidation, PackagingAssessment::NeedsValidation, &["MLX conversion provides promising Apple Silicon path; Windows path still needs validation"]),
            EvaluationStatus::PromisingSpike,
            ProductEligibility::ProductCandidate,
            EvidenceLevel::InstallDocumented,
            vec![ScoreFocus::PromptAdherence, ScoreFocus::DurationStability, ScoreFocus::RuntimeFootprint, ScoreFocus::CommercialSafety],
            vec!["Need SoundWorks-owned prompt adherence smoke test and Windows packaging answer"],
            "Best first SFX spike because an Apache-licensed MLX path exists for local Mac validation.",
        ),
    ]
}

fn reference_fixtures() -> Vec<EvaluationFixture> {
    vec![
        fixture(
            "eval-tts-dialogue",
            EvaluationLane::Tts,
            "TTS dialogue",
            "Read two short production lines with clear punctuation, pauses, and emotional contrast.",
            vec!["wav 48 kHz", "metadata sidecar", "latency/memory log"],
        ),
        fixture(
            "eval-voice-clone-consent",
            EvaluationLane::VoiceClone,
            "Consented voice clone",
            "Clone a consented ten-second reference and synthesize a short disclosure line.",
            vec!["voice clip", "consent provenance", "similarity notes"],
        ),
        fixture(
            "eval-voice-conversion",
            EvaluationLane::VoiceConversion,
            "Speech-to-speech conversion",
            "Convert a neutral source clip into a target consented voice while preserving timing.",
            vec!["converted voice clip", "timing drift measurement", "consent provenance"],
        ),
        fixture(
            "eval-sfx-foley",
            EvaluationLane::Sfx,
            "Foley prompt adherence",
            "A heavy wooden door creaks open slowly in a small concrete room.",
            vec!["wav 48 kHz", "prompt adherence notes", "artifact count"],
        ),
        fixture(
            "eval-ambience-layer",
            EvaluationLane::Ambience,
            "Layered ambience",
            "Distant city rain under a covered bus stop with occasional passing cars.",
            vec!["loopable ambience bed", "loudness report", "seam notes"],
        ),
        fixture(
            "eval-loop-120bpm",
            EvaluationLane::Loop,
            "Tempo/key loop",
            "A 120 BPM four-bar synthwave bass loop in A minor, clean loop points.",
            vec!["loop asset", "BPM/key estimate", "loop point report"],
        ),
        fixture(
            "eval-sample-isolation",
            EvaluationLane::InstrumentSample,
            "Instrument sample isolation",
            "A single dry pizzicato cello note at C3 with no room reverb.",
            vec!["sample asset", "isolation notes", "pitch estimate"],
        ),
        fixture(
            "eval-song-verse-chorus",
            EvaluationLane::Song,
            "Verse/chorus song",
            "Generate a 90-second song with verse, chorus, and supplied lyrics.",
            vec!["full mix", "stem request result", "lyric alignment notes"],
        ),
        fixture(
            "eval-video-foley-sync",
            EvaluationLane::VideoToAudio,
            "Video-to-audio sync",
            "Generate synchronized audio for a short clip with footsteps and a dropped metal key.",
            vec!["synced audio", "event timing report", "visual alignment notes"],
        ),
    ]
}

fn reference_lane_scorecards() -> Vec<LaneScorecard> {
    vec![
        lane(
            EvaluationLane::Tts,
            &[
                ("intelligibility", "Intelligibility", 30),
                ("speaker-consistency", "Speaker consistency", 25),
                ("prosody", "Prosody and emotion control", 20),
                ("runtime", "Latency and memory", 15),
                ("safety", "Consent and commercial safety", 10),
            ],
        ),
        lane(
            EvaluationLane::VoiceClone,
            &[
                ("similarity", "Reference similarity", 30),
                ("consent", "Consent enforcement", 25),
                ("cross-language", "Cross-language stability", 15),
                ("artifact", "Artifacts", 15),
                ("runtime", "Runtime footprint", 15),
            ],
        ),
        lane(
            EvaluationLane::VoiceConversion,
            &[
                ("timing", "Source timing preservation", 25),
                ("target", "Target voice consistency", 25),
                ("consent", "Consent enforcement", 20),
                ("latency", "Latency", 15),
                ("artifact", "Artifacts", 15),
            ],
        ),
        lane(
            EvaluationLane::Sfx,
            &[
                ("prompt", "Prompt adherence", 30),
                ("texture", "Texture realism", 20),
                ("duration", "Duration control", 15),
                ("artifact", "Artifact frequency", 15),
                ("runtime", "Runtime footprint", 20),
            ],
        ),
        lane(
            EvaluationLane::Ambience,
            &[
                ("layering", "Layering and continuity", 30),
                ("loopability", "Loopability", 25),
                ("loudness", "Loudness stability", 15),
                ("prompt", "Prompt adherence", 15),
                ("runtime", "Runtime footprint", 15),
            ],
        ),
        lane(
            EvaluationLane::InstrumentSample,
            &[
                ("isolation", "Sample isolation", 30),
                ("pitch", "Pitch usefulness", 25),
                ("tail", "Tail and noise floor", 15),
                ("format", "Export format", 10),
                ("runtime", "Runtime footprint", 20),
            ],
        ),
        lane(
            EvaluationLane::Loop,
            &[
                ("loopability", "Loopability", 30),
                ("tempo", "BPM/key usefulness", 25),
                ("prompt", "Prompt adherence", 15),
                ("loudness", "Loudness stability", 10),
                ("runtime", "Runtime footprint", 20),
            ],
        ),
        lane(
            EvaluationLane::Song,
            &[
                ("lyrics", "Lyric alignment", 25),
                ("structure", "Structure coherence", 25),
                ("mix", "Mix quality", 15),
                ("duration", "Duration stability", 15),
                ("runtime", "Runtime footprint", 20),
            ],
        ),
        lane(
            EvaluationLane::StemSeparation,
            &[
                ("separation", "Stem separation quality", 40),
                ("bleed", "Bleed/artifacts", 25),
                ("timing", "Timing preservation", 15),
                ("format", "Stem export format", 10),
                ("runtime", "Runtime footprint", 10),
            ],
        ),
        lane(
            EvaluationLane::VideoToAudio,
            &[
                ("sync", "Audio-visual synchronization", 35),
                ("semantic", "Semantic event match", 25),
                ("prompt", "Prompt adherence", 15),
                ("duration", "Duration stability", 10),
                ("runtime", "Runtime footprint", 15),
            ],
        ),
    ]
}

fn reference_recommendations() -> Vec<ModelRecommendation> {
    vec![
        recommendation(
            EvaluationLane::Tts,
            "kokoro-82m",
            "Small permissive TTS model with plausible native/ONNX/Rust paths; needs SoundWorks-owned smoke evidence before becoming Ready.",
        ),
        recommendation(
            EvaluationLane::VoiceClone,
            "chatterbox",
            "Permissive license and strong voice feature surface make it the first voice-clone spike, but it still needs packaged no-Python execution.",
        ),
        recommendation(
            EvaluationLane::VoiceConversion,
            "rvc",
            "RVC is the correct lane-specific speech-to-speech baseline; it must stay consent-gated and external-executable until ported.",
        ),
        recommendation(
            EvaluationLane::Sfx,
            "moss-soundeffect",
            "Apache-licensed SoundEffect v2 evidence and MLX conversion make this the best first local SFX spike.",
        ),
        recommendation(
            EvaluationLane::Song,
            "ace-step-1-5",
            "Permissive local music model with documented install path; compare against Stable Audio 3 and LeVo/YuE before product selection.",
        ),
        recommendation(
            EvaluationLane::VideoToAudio,
            "mmaudio",
            "MMAudio is the primary public video-to-audio benchmark, but remains research-only until license and runtime path are resolved.",
        ),
    ]
}

fn candidate(
    id: &str,
    name: &str,
    provider: &str,
    lanes: Vec<EvaluationLane>,
    sources: Vec<EvidenceSource>,
    license: CandidateLicense,
    runtime: EvaluationRuntimeProfile,
    status: EvaluationStatus,
    product_eligibility: ProductEligibility,
    evidence_level: EvidenceLevel,
    score_focus: Vec<ScoreFocus>,
    blockers: Vec<&str>,
    notes: &str,
) -> ModelEvaluationCandidate {
    ModelEvaluationCandidate {
        id: id.to_string(),
        name: name.to_string(),
        provider: provider.to_string(),
        lanes,
        sources,
        license,
        runtime,
        status,
        product_eligibility,
        evidence_level,
        smoke_tests: vec![
            SmokeTestPlan::Install,
            SmokeTestPlan::Load,
            SmokeTestPlan::FirstGeneration,
            SmokeTestPlan::Cancellation,
            SmokeTestPlan::ErrorRecovery,
            SmokeTestPlan::Repeatability,
            SmokeTestPlan::LicenseGate,
            SmokeTestPlan::PackagingPreflight,
        ],
        score_focus,
        blockers: blockers.into_iter().map(str::to_string).collect(),
        notes: notes.to_string(),
    }
}

fn source(label: &str, url: &str, source_type: EvidenceSourceType) -> EvidenceSource {
    EvidenceSource {
        label: label.to_string(),
        url: url.to_string(),
        source_type,
    }
}

fn license(label: &str, commercial_use: CommercialUseEvaluation) -> CandidateLicense {
    CandidateLicense {
        label: label.to_string(),
        commercial_use,
        safety_notes: vec![
            "Consent must be captured for voice cloning and voice conversion.".to_string(),
            "Generated outputs need provenance sidecars before product enablement.".to_string(),
        ],
    }
}

fn runtime(
    product_path: ProductRuntimePath,
    requires_python_runtime: bool,
    mac_packaging: PackagingAssessment,
    windows_packaging: PackagingAssessment,
    dependency_notes: &[&str],
) -> EvaluationRuntimeProfile {
    EvaluationRuntimeProfile {
        runtime: if requires_python_runtime {
            ModelRuntime::ResearchOnly
        } else {
            ModelRuntime::Local
        },
        product_path,
        requires_python_runtime,
        mac_packaging,
        windows_packaging,
        dependency_notes: dependency_notes
            .iter()
            .map(|note| note.to_string())
            .collect(),
    }
}

fn fixture(
    id: &str,
    lane: EvaluationLane,
    name: &str,
    prompt_or_script: &str,
    expected_outputs: Vec<&str>,
) -> EvaluationFixture {
    EvaluationFixture {
        id: id.to_string(),
        lane,
        name: name.to_string(),
        prompt_or_script: prompt_or_script.to_string(),
        expected_outputs: expected_outputs.into_iter().map(str::to_string).collect(),
        measurements: vec![
            EvaluationMeasurement::LatencyMs,
            EvaluationMeasurement::WarmupMs,
            EvaluationMeasurement::PeakMemoryMb,
            EvaluationMeasurement::OutputDurationMs,
            EvaluationMeasurement::SampleRateHz,
            EvaluationMeasurement::LoudnessLufs,
            EvaluationMeasurement::TruePeakDbfs,
            EvaluationMeasurement::Clipping,
            EvaluationMeasurement::ArtifactFrequency,
            EvaluationMeasurement::RepeatabilityHash,
        ],
    }
}

fn lane(lane: EvaluationLane, axes: &[(&str, &str, u8)]) -> LaneScorecard {
    LaneScorecard {
        lane,
        axes: axes
            .iter()
            .map(|(id, label, weight)| ScoreAxis {
                id: id.to_string(),
                label: label.to_string(),
                weight: *weight,
            })
            .collect(),
    }
}

fn recommendation(
    lane: EvaluationLane,
    candidate_id: &str,
    rationale: &str,
) -> ModelRecommendation {
    ModelRecommendation {
        lane,
        candidate_id: candidate_id.to_string(),
        status: EvaluationStatus::PromisingSpike,
        rationale: rationale.to_string(),
        required_next_evidence: vec![
            SmokeTestPlan::Install,
            SmokeTestPlan::FirstGeneration,
            SmokeTestPlan::Cancellation,
            SmokeTestPlan::PackagingPreflight,
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::{
        CommercialUseEvaluation, EvaluationLane, EvaluationStatus, ModelEvaluationCatalog,
        PackagingAssessment, ProductEligibility, ProductRuntimePath, SmokeTestPlan,
    };

    #[test]
    fn reference_catalog_covers_every_story_candidate() {
        let catalog = ModelEvaluationCatalog::reference();
        let ids: Vec<&str> = catalog
            .candidates
            .iter()
            .map(|candidate| candidate.id.as_str())
            .collect();

        assert_eq!(ids.len(), 28);
        for expected in [
            "stable-audio-3",
            "ace-step-1-5",
            "levo-2",
            "yue",
            "diffrhythm-2",
            "khala",
            "heartmula",
            "muse-song",
            "kokoro-82m",
            "vibevoice",
            "xtts-v2",
            "chattts",
            "fish-speech",
            "chatterbox",
            "chatterbox-turbo",
            "gpt-sovits",
            "f5-tts",
            "cosyvoice-2",
            "openvoice-v2",
            "rvc",
            "stable-audio-open-1",
            "audiocraft-audiogen",
            "audioldm",
            "audioldm-2",
            "audiox",
            "mmaudio",
            "thinksound",
            "moss-soundeffect",
        ] {
            assert!(ids.contains(&expected), "missing {expected}");
        }
    }

    #[test]
    fn every_candidate_has_source_status_license_and_smoke_plan() {
        let catalog = ModelEvaluationCatalog::reference();

        for candidate in catalog.candidates {
            assert!(
                !candidate.sources.is_empty(),
                "missing source for {}",
                candidate.id
            );
            assert!(!candidate.license.label.is_empty());
            assert_ne!(candidate.status, EvaluationStatus::Ready);
            assert!(candidate.smoke_tests.contains(&SmokeTestPlan::Install));
            assert!(candidate
                .smoke_tests
                .contains(&SmokeTestPlan::PackagingPreflight));
            assert!(!candidate.score_focus.is_empty());
        }
    }

    #[test]
    fn product_candidates_do_not_require_python_runtime() {
        let catalog = ModelEvaluationCatalog::reference();

        for candidate in catalog.candidates.iter().filter(|candidate| {
            candidate.product_eligibility == ProductEligibility::ProductCandidate
        }) {
            assert!(!candidate.runtime.requires_python_runtime);
            assert_ne!(
                candidate.runtime.product_path,
                ProductRuntimePath::PythonPocOnly
            );
        }
    }

    #[test]
    fn blocked_candidates_capture_license_or_runtime_blockers() {
        let catalog = ModelEvaluationCatalog::reference();
        let blocked = catalog
            .candidates
            .iter()
            .filter(|candidate| candidate.status == EvaluationStatus::Blocked);

        for candidate in blocked {
            assert!(
                candidate.license.commercial_use == CommercialUseEvaluation::NonCommercial
                    || candidate.runtime.mac_packaging == PackagingAssessment::Blocked
                    || candidate.runtime.windows_packaging == PackagingAssessment::Blocked
            );
            assert!(!candidate.blockers.is_empty());
        }
    }

    #[test]
    fn fixtures_and_scorecards_cover_required_lanes() {
        let catalog = ModelEvaluationCatalog::reference();

        for lane in [
            EvaluationLane::Tts,
            EvaluationLane::VoiceClone,
            EvaluationLane::VoiceConversion,
            EvaluationLane::Sfx,
            EvaluationLane::Ambience,
            EvaluationLane::InstrumentSample,
            EvaluationLane::Loop,
            EvaluationLane::Song,
            EvaluationLane::VideoToAudio,
        ] {
            assert!(
                catalog.fixtures.iter().any(|fixture| fixture.lane == lane),
                "missing fixture for {lane:?}"
            );
        }

        for lane in [
            EvaluationLane::Tts,
            EvaluationLane::VoiceClone,
            EvaluationLane::VoiceConversion,
            EvaluationLane::Sfx,
            EvaluationLane::Ambience,
            EvaluationLane::InstrumentSample,
            EvaluationLane::Loop,
            EvaluationLane::Song,
            EvaluationLane::StemSeparation,
            EvaluationLane::VideoToAudio,
        ] {
            let scorecard = catalog
                .lane_scorecards
                .iter()
                .find(|scorecard| scorecard.lane == lane)
                .expect("lane scorecard");
            assert_eq!(
                scorecard
                    .axes
                    .iter()
                    .map(|axis| axis.weight as u16)
                    .sum::<u16>(),
                100
            );
        }
    }

    #[test]
    fn recommendations_point_to_existing_non_ready_candidates() {
        let catalog = ModelEvaluationCatalog::reference();

        for recommendation in &catalog.recommendations {
            let candidate = catalog
                .candidates
                .iter()
                .find(|candidate| candidate.id == recommendation.candidate_id)
                .expect("recommended candidate exists");

            assert_eq!(candidate.status, EvaluationStatus::PromisingSpike);
            assert!(candidate.lanes.contains(&recommendation.lane));
        }
    }

    #[test]
    fn overview_summarizes_evaluation_surface() {
        let catalog = ModelEvaluationCatalog::reference();
        let overview = catalog.overview();

        assert_eq!(overview.candidate_count, 28);
        assert_eq!(overview.fixture_count, 9);
        assert_eq!(overview.lane_count, 10);
        assert!(overview.source_count >= 40);
        assert_eq!(
            overview
                .status_counts
                .get(&EvaluationStatus::PromisingSpike)
                .copied(),
            Some(24)
        );
        assert!(overview
            .recommended_candidate_ids
            .contains(&"kokoro-82m".to_string()));
    }

    #[test]
    fn catalog_serializes_for_tauri_and_storage_boundaries() {
        let payload =
            serde_json::to_value(ModelEvaluationCatalog::reference()).expect("catalog serializes");

        assert_eq!(payload["schemaVersion"], 1);
        assert_eq!(payload["candidates"][0]["id"], "stable-audio-3");
        assert_eq!(payload["recommendations"][0]["candidateId"], "kokoro-82m");
    }
}
