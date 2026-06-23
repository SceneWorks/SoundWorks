use crate::evaluation::REQUIRED_CANDIDATE_IDS;
use crate::export_workflow::ExportWorkflowOverview;
use crate::manifests::CapabilityWorkflow;
use crate::model_manager::ModelManagerOverview;
use crate::rights::{
    PolicyDecision, PolicyGateStatus, RightsSafetyOverview, RightsValidationStatus,
};
use crate::runtime::{
    DeviceInventory, ExecutionStrategy, RuntimeJobStore, RuntimeOverview, RuntimePackagingPolicy,
    ValidationStatus,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

pub const MVP_VALIDATION_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MvpValidationOverview {
    pub schema_version: u32,
    pub release_gate: MvpReleaseGate,
    pub runtime_evidence: Vec<RuntimeEvidenceRequirement>,
    pub demo_workflows: Vec<DemoWorkflow>,
    pub regression_fixtures: Vec<RegressionFixture>,
    pub automated_checks: Vec<ValidationCheck>,
    pub manual_scorecards: Vec<ManualQaScorecard>,
    pub stress_cases: Vec<StressCase>,
    pub known_limitations: Vec<KnownLimitation>,
    pub requirement_coverage: Vec<RequirementCoverage>,
}

impl MvpValidationOverview {
    pub fn reference() -> Self {
        let gate_inputs = DerivedValidationInputs::reference();
        let demo_workflows = demo_workflows();
        let regression_fixtures = regression_fixtures();
        let automated_checks = automated_checks(&gate_inputs);
        let manual_scorecards = manual_scorecards();
        let stress_cases = stress_cases();
        let known_limitations = known_limitations();
        let runtime_evidence = runtime_evidence(&gate_inputs);
        let requirement_coverage = requirement_coverage();
        let release_gate = MvpReleaseGate::from_sections(
            &demo_workflows,
            &regression_fixtures,
            &automated_checks,
            &manual_scorecards,
            &stress_cases,
            &known_limitations,
            &runtime_evidence,
        );

        Self {
            schema_version: MVP_VALIDATION_SCHEMA_VERSION,
            release_gate,
            runtime_evidence,
            demo_workflows,
            regression_fixtures,
            automated_checks,
            manual_scorecards,
            stress_cases,
            known_limitations,
            requirement_coverage,
        }
    }

    pub fn workflow_coverage(&self) -> Vec<CapabilityWorkflow> {
        let workflows: BTreeSet<CapabilityWorkflow> = self
            .demo_workflows
            .iter()
            .map(|workflow| workflow.workflow)
            .chain(
                self.regression_fixtures
                    .iter()
                    .map(|fixture| fixture.workflow),
            )
            .collect();

        workflows.into_iter().collect()
    }
}

#[derive(Debug, Clone)]
struct DerivedValidationInputs {
    model_manager: ModelManagerOverview,
    rights: RightsSafetyOverview,
    runtime: RuntimeOverview,
    export_workflow: ExportWorkflowOverview,
}

impl DerivedValidationInputs {
    fn reference() -> Self {
        let model_manager = ModelManagerOverview::reference();
        let runtime = RuntimeOverview::from_model_manager(
            &model_manager,
            &DeviceInventory::reference_mac(),
            RuntimePackagingPolicy::shipped_desktop(),
            &RuntimeJobStore::new(std::env::temp_dir().join("soundworks-mvp-validation-reference")),
        );

        Self {
            model_manager,
            rights: RightsSafetyOverview::reference(),
            runtime,
            export_workflow: ExportWorkflowOverview::reference(),
        }
    }

    fn model_manager_check_passed(&self, check_id: &str) -> bool {
        self.model_manager
            .validation_checks
            .iter()
            .any(|check| check.id == check_id && check.passed)
    }

    fn model_cache_counts_are_derived(&self) -> bool {
        self.model_manager_check_passed("model-manager.candidate-coverage")
            && self.model_manager_check_passed("model-manager.no-metadata-installs")
    }

    fn full_model_use_coverage(&self) -> bool {
        REQUIRED_CANDIDATE_IDS.iter().all(|candidate_id| {
            self.rights
                .model_use_decisions
                .iter()
                .any(|decision| decision.candidate_id == *candidate_id)
        })
    }

    fn safety_gates_passed(&self) -> bool {
        let required_rights_checks_passed = [
            "validation.voice-consent",
            "validation.model-license",
            "validation.provenance-sidecar",
        ]
        .iter()
        .all(|check_id| {
            self.rights.validation_checks.iter().any(|check| {
                check.id == *check_id && check.status == RightsValidationStatus::Passed
            })
        });
        let has_blocking_consent_decision = self
            .rights
            .consent_checks
            .iter()
            .any(|check| check.decision == PolicyDecision::Blocked);
        let has_allowed_model = self
            .rights
            .model_use_decisions
            .iter()
            .any(|decision| decision.decision == PolicyDecision::Allowed);
        let has_blocked_model = self
            .rights
            .model_use_decisions
            .iter()
            .any(|decision| decision.decision == PolicyDecision::Blocked);
        let blocked_models_are_not_export_candidates =
            self.rights.model_use_decisions.iter().all(|decision| {
                decision.decision != PolicyDecision::Blocked || !decision.export_candidate
            });

        required_rights_checks_passed
            && self.full_model_use_coverage()
            && has_blocking_consent_decision
            && has_allowed_model
            && has_blocked_model
            && blocked_models_are_not_export_candidates
    }

    fn native_runtime_audio_lanes_available(&self) -> bool {
        let native_workflows: BTreeSet<CapabilityWorkflow> = self
            .runtime
            .model_states
            .iter()
            .filter(|state| state.execution_strategy == ExecutionStrategy::NativeRust)
            .flat_map(|state| state.workflows.iter().copied())
            .collect();

        [
            CapabilityWorkflow::Sfx,
            CapabilityWorkflow::Ambience,
            CapabilityWorkflow::InstrumentSample,
            CapabilityWorkflow::Loop,
        ]
        .iter()
        .all(|workflow| native_workflows.contains(workflow))
    }

    fn runtime_jobs_are_supported(&self) -> bool {
        self.runtime
            .validation_checks
            .iter()
            .all(|check| check.status != ValidationStatus::Failed)
            && self.runtime.status_counts.available > 0
            && self.native_runtime_audio_lanes_available()
    }

    fn export_contract_passed(&self) -> bool {
        self.export_workflow
            .validation_checks
            .iter()
            .all(|check| check.passed)
    }

    fn commercial_or_safety_blockers_remain(&self) -> bool {
        self.rights
            .content_policy_gates
            .iter()
            .any(|gate| gate.status == PolicyGateStatus::Blocked)
            || self
                .rights
                .model_use_decisions
                .iter()
                .any(|decision| decision.decision == PolicyDecision::Blocked)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MvpValidationSummary {
    pub schema_version: u32,
    pub ready_for_mvp: bool,
    pub blocking_item_count: usize,
    pub runtime_evidence_count: usize,
    pub satisfied_runtime_evidence_count: usize,
    pub fixture_only_evidence_count: usize,
    pub demo_workflow_count: usize,
    pub regression_fixture_count: usize,
    pub automated_check_count: usize,
    pub manual_scorecard_count: usize,
    pub stress_case_count: usize,
    pub known_limitation_count: usize,
    pub requirement_count: usize,
    pub workflow_count: usize,
}

impl MvpValidationSummary {
    pub fn from_overview(overview: &MvpValidationOverview) -> Self {
        Self {
            schema_version: overview.schema_version,
            ready_for_mvp: overview.release_gate.ready_for_mvp,
            blocking_item_count: overview.release_gate.blocking_items.len(),
            runtime_evidence_count: overview.runtime_evidence.len(),
            satisfied_runtime_evidence_count: overview
                .runtime_evidence
                .iter()
                .filter(|evidence| evidence.status == MvpValidationStatus::Passed)
                .count(),
            fixture_only_evidence_count: overview
                .runtime_evidence
                .iter()
                .filter(|evidence| evidence.fixture_only)
                .count(),
            demo_workflow_count: overview.demo_workflows.len(),
            regression_fixture_count: overview.regression_fixtures.len(),
            automated_check_count: overview.automated_checks.len(),
            manual_scorecard_count: overview.manual_scorecards.len(),
            stress_case_count: overview.stress_cases.len(),
            known_limitation_count: overview.known_limitations.len(),
            requirement_count: overview.requirement_coverage.len(),
            workflow_count: overview.workflow_coverage().len(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MvpReleaseGate {
    pub ready_for_mvp: bool,
    pub required_workflow_count: usize,
    pub covered_workflow_count: usize,
    pub required_runtime_evidence_count: usize,
    pub satisfied_runtime_evidence_count: usize,
    pub fixture_only_evidence_count: usize,
    pub required_automated_check_count: usize,
    pub passed_automated_check_count: usize,
    pub required_manual_scorecard_count: usize,
    pub passed_manual_scorecard_count: usize,
    pub required_stress_case_count: usize,
    pub passed_stress_case_count: usize,
    pub blocking_items: Vec<String>,
}

impl MvpReleaseGate {
    fn from_sections(
        demo_workflows: &[DemoWorkflow],
        regression_fixtures: &[RegressionFixture],
        automated_checks: &[ValidationCheck],
        manual_scorecards: &[ManualQaScorecard],
        stress_cases: &[StressCase],
        known_limitations: &[KnownLimitation],
        runtime_evidence: &[RuntimeEvidenceRequirement],
    ) -> Self {
        let required_workflows = CapabilityWorkflow::all();
        let demo_workflow_set: BTreeSet<CapabilityWorkflow> = demo_workflows
            .iter()
            .map(|workflow| workflow.workflow)
            .collect();
        let fixture_workflow_set: BTreeSet<CapabilityWorkflow> = regression_fixtures
            .iter()
            .map(|fixture| fixture.workflow)
            .collect();
        let covered_workflows: BTreeSet<CapabilityWorkflow> = required_workflows
            .iter()
            .copied()
            .filter(|workflow| {
                demo_workflow_set.contains(workflow) && fixture_workflow_set.contains(workflow)
            })
            .collect();

        let required_automated_check_count = automated_checks
            .iter()
            .filter(|check| check.required_for_mvp)
            .count();
        let passed_automated_check_count = automated_checks
            .iter()
            .filter(|check| check.required_for_mvp && check.status == MvpValidationStatus::Passed)
            .count();
        let required_manual_scorecard_count = manual_scorecards
            .iter()
            .filter(|scorecard| scorecard.required_for_mvp)
            .count();
        let passed_manual_scorecard_count = manual_scorecards
            .iter()
            .filter(|scorecard| {
                scorecard.required_for_mvp && scorecard.status == MvpValidationStatus::Passed
            })
            .count();
        let required_stress_case_count = stress_cases
            .iter()
            .filter(|stress_case| stress_case.required_for_mvp)
            .count();
        let passed_stress_case_count = stress_cases
            .iter()
            .filter(|stress_case| {
                stress_case.required_for_mvp && stress_case.status == MvpValidationStatus::Passed
            })
            .count();
        let required_runtime_evidence_count = runtime_evidence
            .iter()
            .filter(|evidence| evidence.required_for_mvp)
            .count();
        let satisfied_runtime_evidence_count = runtime_evidence
            .iter()
            .filter(|evidence| {
                evidence.required_for_mvp && evidence.status == MvpValidationStatus::Passed
            })
            .count();
        let fixture_only_evidence_count = runtime_evidence
            .iter()
            .filter(|evidence| evidence.required_for_mvp && evidence.fixture_only)
            .count();

        let mut blocking_items = Vec::new();
        if covered_workflows.len() != required_workflows.len() {
            blocking_items.push(
                "Every capability workflow needs both a golden demo and a regression fixture."
                    .to_string(),
            );
        }
        if passed_automated_check_count != required_automated_check_count {
            blocking_items
                .push("Required automated validation checks have not all passed.".to_string());
        }
        if passed_manual_scorecard_count != required_manual_scorecard_count {
            blocking_items
                .push("Required manual audio-quality scorecards are not all passed.".to_string());
        }
        if passed_stress_case_count != required_stress_case_count {
            blocking_items
                .push("Required stress cases are not all passed on release hardware.".to_string());
        }
        if satisfied_runtime_evidence_count != required_runtime_evidence_count {
            blocking_items.push(
                "Runtime evidence is missing; fixture/demo data cannot satisfy generated audio, playback, edit, or export criteria."
                    .to_string(),
            );
        }
        if fixture_only_evidence_count > 0 {
            blocking_items.push(
                "Fixture-only evidence is still present in MVP-critical runtime criteria."
                    .to_string(),
            );
        }
        if known_limitations
            .iter()
            .any(|limitation| limitation.blocks_mvp)
        {
            blocking_items.push("Known MVP-blocking limitations remain documented.".to_string());
        }

        Self {
            ready_for_mvp: blocking_items.is_empty(),
            required_workflow_count: required_workflows.len(),
            covered_workflow_count: covered_workflows.len(),
            required_runtime_evidence_count,
            satisfied_runtime_evidence_count,
            fixture_only_evidence_count,
            required_automated_check_count,
            passed_automated_check_count,
            required_manual_scorecard_count,
            passed_manual_scorecard_count,
            required_stress_case_count,
            passed_stress_case_count,
            blocking_items,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DemoWorkflow {
    pub id: String,
    pub workflow: CapabilityWorkflow,
    pub title: String,
    pub goal: String,
    pub required_artifacts: Vec<String>,
    pub acceptance: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegressionFixture {
    pub id: String,
    pub workflow: CapabilityWorkflow,
    pub name: String,
    pub input_contract: String,
    pub expected_outputs: Vec<String>,
    pub automated_check_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidationCheck {
    pub id: String,
    pub category: ValidationCategory,
    pub status: MvpValidationStatus,
    pub required_for_mvp: bool,
    pub summary: String,
    pub evidence: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeEvidenceRequirement {
    pub id: String,
    pub workflow: CapabilityWorkflow,
    pub required_for_mvp: bool,
    pub status: MvpValidationStatus,
    pub fixture_only: bool,
    pub requirement: String,
    pub evidence: String,
    pub blocker: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ValidationCategory {
    JobContracts,
    RecipePersistence,
    MetadataExtraction,
    ProviderManifest,
    AssetLifecycle,
    ExportSidecars,
    SafetyGates,
    AudioQuality,
    RuntimeEvidence,
    Stress,
    Documentation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum MvpValidationStatus {
    Passed,
    Pending,
    ManualRequired,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManualQaScorecard {
    pub id: String,
    pub workflow: CapabilityWorkflow,
    pub status: MvpValidationStatus,
    pub required_for_mvp: bool,
    pub scoring_axes: Vec<String>,
    pub pass_threshold: String,
    pub reviewer_notes: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StressCase {
    pub id: String,
    pub title: String,
    pub workflow: CapabilityWorkflow,
    pub status: MvpValidationStatus,
    pub required_for_mvp: bool,
    pub scenario: String,
    pub expected_behavior: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KnownLimitation {
    pub id: String,
    pub area: String,
    pub summary: String,
    pub mitigation: String,
    pub blocks_mvp: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RequirementCoverage {
    pub requirement_id: String,
    pub epic_requirement: String,
    pub demo_workflow_ids: Vec<String>,
    pub fixture_ids: Vec<String>,
    pub check_ids: Vec<String>,
    pub status: MvpValidationStatus,
}

fn demo_workflows() -> Vec<DemoWorkflow> {
    vec![
        demo(
            "demo-short-narration",
            CapabilityWorkflow::Tts,
            "Narrate short script",
            "Generate a clean 30-60 second voice clip from a script with pronunciation metadata.",
            &["voice clip WAV", "recipe JSON", "voice consent record", "provenance sidecar"],
            &["speech is intelligible", "script segments are preserved", "saved output is reusable from the library"],
        ),
        demo(
            "demo-podcast-dialogue",
            CapabilityWorkflow::VoiceClone,
            "Multi-speaker podcast segment",
            "Generate a short two-speaker segment with explicit voice-profile consent gates.",
            &["multi-speaker voice clips", "speaker map", "consent audit", "export sidecar"],
            &["each speaker maps to a consented profile", "blocked voices cannot submit", "dialogue can export with provenance"],
        ),
        demo(
            "demo-voice-conversion",
            CapabilityWorkflow::VoiceConversion,
            "Consented voice conversion",
            "Convert a source read into an approved target voice profile without treating it as TTS.",
            &["source clip", "target voice profile", "converted voice clip", "conversion recipe"],
            &["source and target IDs are preserved", "voice conversion scorecard applies", "output remains a voice clip asset"],
        ),
        demo(
            "demo-game-ui-sfx",
            CapabilityWorkflow::Sfx,
            "Generate game UI SFX",
            "Create short selectable impacts, buttons, and confirmations with tags and one-shot metadata.",
            &["SFX variants", "tags", "loudness metrics", "saved output assets"],
            &["variants are auditionable", "selected outputs save to library", "game export preset accepts the assets"],
        ),
        demo(
            "demo-loopable-ambience",
            CapabilityWorkflow::Ambience,
            "Create loopable ambience",
            "Generate a seamless ambience bed with loop markers and crossfade QA.",
            &["ambience asset", "loop points", "waveform preview", "loopability score"],
            &["loop points are inspectable", "loudness stays within target", "loop export preserves metadata"],
        ),
        demo(
            "demo-instrument-sample-pack",
            CapabilityWorkflow::InstrumentSample,
            "Generate instrument sample pack",
            "Create one-shots with family, articulation, tags, and pack membership.",
            &["sample variants", "pack collection", "BPM/key metadata", "sample-pack export"],
            &["pack contains selected variants", "metadata survives export", "duplicates can be identified"],
        ),
        demo(
            "demo-loop-pack",
            CapabilityWorkflow::Loop,
            "Generate musical loop pack",
            "Create tempo-aligned loops with key, bar count, and loop marker metadata.",
            &["loop variants", "loop points", "pack collection", "DAW handoff"],
            &["BPM/key fields are populated", "loop points are valid", "DAW export preserves loop metadata"],
        ),
        demo(
            "demo-complete-song",
            CapabilityWorkflow::Song,
            "Generate complete song from lyrics and structure",
            "Generate a song draft from lyrics, sections, style tags, and requested stems.",
            &["song variants", "section map", "lyrics sidecar", "stem request metadata"],
            &["structure matches requested sections", "lyrics alignment is scored", "song master export includes disclosure"],
        ),
        demo(
            "demo-stem-separation",
            CapabilityWorkflow::StemSeparation,
            "Prepare stem bundle",
            "Validate that song or composition outputs can carry stem metadata for DAW handoff.",
            &["stem asset records", "stem kinds", "bundle manifest", "sidecar"],
            &["stem kinds are explicit", "bundle references source asset", "sidecar links model and recipe"],
        ),
        demo(
            "demo-video-foley",
            CapabilityWorkflow::VideoToAudio,
            "Prototype silent video Foley",
            "Track the video-to-audio demo path so MVP cannot forget multimodal SFX coverage.",
            &["source video ID", "time range map", "generated sync points", "provenance sidecar"],
            &["source media rights are checked", "time ranges survive export", "provider capability is flagged until sc-6183 ships"],
        ),
        demo(
            "demo-edit-trim-normalize",
            CapabilityWorkflow::Edit,
            "Edit, trim, and normalize",
            "Round-trip a generated asset through non-destructive trim, fade, loop crossfade, and normalize actions.",
            &["edited asset version", "edit recipe", "version comparison", "preview cache"],
            &["original version remains intact", "edit chain is inspectable", "saved version can be exported"],
        ),
        demo(
            "demo-composition-export",
            CapabilityWorkflow::CompositionRender,
            "Export composition with provenance",
            "Render a composition mixdown with optional stems, DAW handoff, and SceneWorks package metadata.",
            &["mixdown", "optional stems", "DAW bundle", "SceneWorks handoff", "provenance manifest"],
            &["export preset is selectable", "sidecar contains recipe/model/source/rights", "SceneWorks constraints are carried as warnings"],
        ),
    ]
}

fn regression_fixtures() -> Vec<RegressionFixture> {
    vec![
        fixture(
            "fixture-short-narration",
            CapabilityWorkflow::Tts,
            "Short narration script",
            "three-segment script with two consented speakers",
            &["voice clip asset", "generation job", "recipe summary"],
            &[
                "check-job-contracts",
                "check-recipe-persistence",
                "check-safety-gates",
            ],
        ),
        fixture(
            "fixture-podcast-dialogue",
            CapabilityWorkflow::VoiceClone,
            "Podcast voice clone gate",
            "voice-clone request with approved and rejected profile variants",
            &["allowed submission", "blocked submission", "consent audit"],
            &["check-job-contracts", "check-safety-gates"],
        ),
        fixture(
            "fixture-voice-conversion",
            CapabilityWorkflow::VoiceConversion,
            "Voice conversion source-target pair",
            "source audio plus consented target voice profile",
            &[
                "conversion job",
                "voice clip output",
                "source-target provenance",
            ],
            &["check-job-contracts", "check-recipe-persistence"],
        ),
        fixture(
            "fixture-game-sfx",
            CapabilityWorkflow::Sfx,
            "Game UI SFX batch",
            "short SFX prompt with duration, category, negative prompt, and tags",
            &["SFX variants", "loudness metrics", "saved output"],
            &[
                "check-job-contracts",
                "check-metadata-extraction",
                "check-asset-lifecycle",
            ],
        ),
        fixture(
            "fixture-ambience-loop",
            CapabilityWorkflow::Ambience,
            "Loopable ambience bed",
            "ambience prompt with loopable control and crossfade target",
            &["ambience asset", "loop points", "export warning set"],
            &["check-metadata-extraction", "check-export-sidecars"],
        ),
        fixture(
            "fixture-sample-pack",
            CapabilityWorkflow::InstrumentSample,
            "Instrument sample pack",
            "sample generation request with instrument family and articulation",
            &[
                "sample variants",
                "pack collection",
                "sample export metadata",
            ],
            &["check-metadata-extraction", "check-asset-lifecycle"],
        ),
        fixture(
            "fixture-loop-pack",
            CapabilityWorkflow::Loop,
            "Musical loop pack",
            "four-bar loop request with BPM, key, and loop points",
            &["loop variants", "BPM/key metadata", "DAW handoff"],
            &["check-metadata-extraction", "check-export-sidecars"],
        ),
        fixture(
            "fixture-song-structure",
            CapabilityWorkflow::Song,
            "Complete song structure",
            "lyrics, section map, style tags, stems requested",
            &["song variants", "section scores", "export targets"],
            &[
                "check-job-contracts",
                "check-metadata-extraction",
                "check-export-sidecars",
            ],
        ),
        fixture(
            "fixture-stem-bundle",
            CapabilityWorkflow::StemSeparation,
            "Stem bundle handoff",
            "song output with requested stem kinds",
            &["stem records", "stem bundle sidecar", "DAW package"],
            &["check-export-sidecars", "check-provider-manifests"],
        ),
        fixture(
            "fixture-video-foley",
            CapabilityWorkflow::VideoToAudio,
            "Silent video Foley map",
            "video source ID, target ranges, object notes, and text direction",
            &[
                "sync points",
                "source media provenance",
                "blocked provider gate",
            ],
            &["check-provider-manifests", "check-safety-gates"],
        ),
        fixture(
            "fixture-edit-normalize",
            CapabilityWorkflow::Edit,
            "Non-destructive edit chain",
            "trim, fade, normalize, and loop crossfade operations on a generated loop",
            &["edited version", "comparison metrics", "edit provenance"],
            &["check-recipe-persistence", "check-asset-lifecycle"],
        ),
        fixture(
            "fixture-composition-export",
            CapabilityWorkflow::CompositionRender,
            "Composition render and handoff",
            "two-track composition with voice clip and loop assets",
            &["mixdown", "optional stems", "SceneWorks package metadata"],
            &["check-export-sidecars", "check-recipe-persistence"],
        ),
    ]
}

fn automated_checks(inputs: &DerivedValidationInputs) -> Vec<ValidationCheck> {
    vec![
        check("check-job-contracts", ValidationCategory::JobContracts, passed_if(inputs.runtime_jobs_are_supported()), true, "Generation job contracts serialize status, progress, outputs, cancellation, and actionable errors from the runtime job store.", if inputs.runtime_jobs_are_supported() { "Derived from RuntimeOverview: runtime validation checks pass, native runtime lanes are available, and queued job execution is covered by runtime tests." } else { "Runtime job contracts are implemented and tested, but the reference validation store has no release-run job artifacts attached yet." }),
        check("check-recipe-persistence", ValidationCategory::RecipePersistence, MvpValidationStatus::Passed, true, "Recipes preserve provider/model, seed, references, post-processing, outputs, and replayability.", "Fixture and review tests assert serializable inspectable recipes and edit chains."),
        check("check-metadata-extraction", ValidationCategory::MetadataExtraction, MvpValidationStatus::Pending, true, "Audio metadata contracts include duration, sample rate, channels, loudness, true peak, BPM, key, and loop points, but generated-file extraction is not proven.", "SFX, samples, songs, review, library, and export reference data expose fields. Real generated audio must replace fixture values."),
        check("check-provider-manifests", ValidationCategory::ProviderManifest, passed_if(inputs.model_cache_counts_are_derived() && inputs.runtime_jobs_are_supported()), true, "Provider manifests distinguish workflows, inputs, outputs, limits, hardware, license, runnable defaults, and runtime execution strategy.", if inputs.runtime_jobs_are_supported() { "Derived from ModelManagerOverview candidate coverage plus RuntimeOverview execution/admission validation." } else { "ModelManagerOverview covers candidate/cache state, but runtime job-store release evidence is not attached yet." }),
        check("check-asset-lifecycle", ValidationCategory::AssetLifecycle, passed_if(inputs.runtime_jobs_are_supported()), true, "Asset lifecycle contracts cover project/global library, tags, collections, saved outputs, version history, reuse targets, and persisted runtime artifacts.", if inputs.runtime_jobs_are_supported() { "Derived from runtime job artifact support and project-library persistence/export tests instead of fixture cards alone." } else { "Project-library persistence and export tests exist, but release-run runtime artifacts are not attached yet." }),
        check("check-export-sidecars", ValidationCategory::ExportSidecars, passed_if(inputs.export_contract_passed()), true, "Export sidecar contracts include preset, target, formats, DAW bundle, SceneWorks handoff, and metadata.", "Derived from ExportWorkflowOverview validation checks; real project exports write WAV plus provenance and SceneWorks handoff sidecars."),
        check("check-safety-gates", ValidationCategory::SafetyGates, passed_if(inputs.safety_gates_passed()), true, "Voice consent and commercial model-use gates are enforced in runtime/export policy and cover every evaluated catalog candidate.", "Derived from RightsSafetyOverview validation checks, full model-use decision coverage, persisted voice-profile consent gates, and blocked model decisions being excluded from export candidates."),
        check("check-runtime-evidence", ValidationCategory::RuntimeEvidence, runtime_evidence_status(inputs), true, "Runtime installed counts, queued jobs, generated audio, playback, edits, and export claims are derived from current runtime/model/library/export gates.", "Model cache and job-contract evidence are live-gate derived; full release evidence remains pending until all provider audio, playback/edit, export-format, and release-run artifacts are attached."),
        check("check-release-docs", ValidationCategory::Documentation, MvpValidationStatus::Passed, true, "Validation matrix maps back to epic requirements and states what remains unverified.", "docs/mvp-validation.md is the human-readable release matrix."),
        check("check-release-run-artifacts", ValidationCategory::Stress, MvpValidationStatus::Pending, true, "Release run artifacts must capture current Mac and Windows validation evidence before MVP signoff.", "This story defines the evidence contract; release artifacts remain pending until real provider runs exist."),
    ]
}

fn runtime_evidence(inputs: &DerivedValidationInputs) -> Vec<RuntimeEvidenceRequirement> {
    vec![
        evidence(
            "evidence-model-cache",
            CapabilityWorkflow::Tts,
            passed_if(inputs.model_cache_counts_are_derived()),
            !inputs.model_cache_counts_are_derived(),
            "Installed model counts must come from verified cache/package files, not static provider manifests.",
            if inputs.model_cache_counts_are_derived() {
                "ModelManagerOverview derives installed counts from verified cache/package evidence and keeps missing-cache candidates visible."
            } else {
                "Model manager validation has not proven cache-derived installed counts."
            },
            "Keep model installation counts blocked unless cache/package verification passes for the selected candidate.",
        ),
        evidence(
            "evidence-generation-jobs",
            CapabilityWorkflow::Tts,
            passed_if(inputs.runtime_jobs_are_supported()),
            !inputs.runtime_jobs_are_supported(),
            "Generation controls must enqueue persisted runtime jobs with progress, errors, logs, and artifacts.",
            if inputs.runtime_jobs_are_supported() {
                "RuntimeOverview exposes executable native lanes and the job store persists queued/running/terminal jobs with artifacts and actionable errors."
            } else {
                "RuntimeOverview does not yet expose passing job-store/runtime validation."
            },
            "Attach current release-run job artifacts for each MVP workflow before final signoff.",
        ),
        evidence(
            "evidence-generated-audio",
            CapabilityWorkflow::Sfx,
            MvpValidationStatus::Pending,
            !inputs.native_runtime_audio_lanes_available(),
            "TTS, SFX, samples, loops, and song criteria require generated audio files from selected providers.",
            if inputs.native_runtime_audio_lanes_available() {
                "Native Rust SFX, ambience, sample, and loop lanes generate real WAV artifacts; TTS cache-backed speech and full-song provider evidence remain pending."
            } else {
                "Generated-audio evidence is still fixture-only."
            },
            "Attach generated audio artifacts or source-backed blockers for every MVP workflow, including cache-backed TTS and song lanes.",
        ),
        evidence(
            "evidence-playback-edit",
            CapabilityWorkflow::Edit,
            MvpValidationStatus::Pending,
            !inputs.runtime_jobs_are_supported(),
            "Playback, trim, fade, normalize, loop inspection, and version comparison must run against real audio files.",
            if inputs.runtime_jobs_are_supported() {
                "Runtime artifacts and project-library review edits persist real files and non-destructive versions; release playback audition evidence remains pending."
            } else {
                "Review/edit fixtures describe the workflow but do not prove persisted runtime files."
            },
            "Attach playback/audition evidence and edited-version artifacts from a release validation run.",
        ),
        evidence(
            "evidence-export-files",
            CapabilityWorkflow::CompositionRender,
            MvpValidationStatus::Pending,
            !inputs.export_contract_passed(),
            "Export criteria require actual WAV/FLAC/MP3/OGG files plus provenance sidecars on disk.",
            if inputs.export_contract_passed() {
                "Project export writes real WAV output plus SoundWorks provenance and SceneWorks handoff sidecars; additional encoded formats remain blocked until encoders are validated."
            } else {
                "Export contract data exists, but file-writing evidence is incomplete."
            },
            "Validate all required release export formats and attach the generated sidecars before final MVP signoff.",
        ),
    ]
}

fn passed_if(condition: bool) -> MvpValidationStatus {
    if condition {
        MvpValidationStatus::Passed
    } else {
        MvpValidationStatus::Pending
    }
}

fn runtime_evidence_status(inputs: &DerivedValidationInputs) -> MvpValidationStatus {
    let passed = runtime_evidence(inputs)
        .iter()
        .filter(|evidence| evidence.required_for_mvp)
        .all(|evidence| evidence.status == MvpValidationStatus::Passed);
    passed_if(passed && !inputs.commercial_or_safety_blockers_remain())
}

fn manual_scorecards() -> Vec<ManualQaScorecard> {
    vec![
        scorecard(
            "score-tts-quality",
            CapabilityWorkflow::Tts,
            MvpValidationStatus::ManualRequired,
            "intelligibility, pronunciation, prosody, noise floor",
            "mean 4/5 with no blocker on intelligibility",
            "Needs real generated audio from the selected first TTS provider.",
        ),
        scorecard(
            "score-dialogue-quality",
            CapabilityWorkflow::VoiceClone,
            MvpValidationStatus::ManualRequired,
            "speaker consistency, consent fit, turn-taking, artifact rate",
            "all consent gates pass and mean 4/5 on speaker consistency",
            "Requires approved reference voices and generated dialogue artifacts.",
        ),
        scorecard(
            "score-voice-conversion-quality",
            CapabilityWorkflow::VoiceConversion,
            MvpValidationStatus::ManualRequired,
            "source preservation, target timbre, intelligibility, artifact rate",
            "mean 4/5 with explicit source-target provenance",
            "Requires RVC-style conversion smoke evidence.",
        ),
        scorecard(
            "score-sfx-quality",
            CapabilityWorkflow::Sfx,
            MvpValidationStatus::ManualRequired,
            "prompt adherence, transient quality, loudness, game usability",
            "at least two variants accepted by a reviewer",
            "Requires generated game UI SFX artifacts.",
        ),
        scorecard(
            "score-ambience-loop-quality",
            CapabilityWorkflow::Ambience,
            MvpValidationStatus::ManualRequired,
            "loop seam, tonal stability, noise, loudness drift",
            "seam is not distracting across three loop passes",
            "Requires loop audition evidence.",
        ),
        scorecard(
            "score-sample-pack-quality",
            CapabilityWorkflow::InstrumentSample,
            MvpValidationStatus::ManualRequired,
            "transient cleanliness, pitch usefulness, tag accuracy, pack consistency",
            "sample pack receives reviewer acceptance for reuse",
            "Requires sample pack preview artifacts.",
        ),
        scorecard(
            "score-loop-pack-quality",
            CapabilityWorkflow::Loop,
            MvpValidationStatus::ManualRequired,
            "BPM fit, key fit, loop seam, musical usefulness",
            "loop aligns to grid and repeats cleanly",
            "Requires DAW or timeline audition.",
        ),
        scorecard(
            "score-song-quality",
            CapabilityWorkflow::Song,
            MvpValidationStatus::ManualRequired,
            "lyric alignment, section structure, mix balance, originality disclosure",
            "song passes structure and disclosure review",
            "Requires complete song artifacts.",
        ),
        scorecard(
            "score-video-foley-quality",
            CapabilityWorkflow::VideoToAudio,
            MvpValidationStatus::ManualRequired,
            "sync accuracy, event coverage, ambience fit, rights clarity",
            "generated audio syncs to target ranges",
            "Blocked on sc-6183 implementation evidence.",
        ),
    ]
}

fn stress_cases() -> Vec<StressCase> {
    vec![
        stress(
            "stress-long-script",
            "Long script chunking",
            CapabilityWorkflow::Tts,
            MvpValidationStatus::Pending,
            "Generate a long-form narration with many segments and speakers.",
            "chunks resume cleanly, output order is stable, partial failures are recoverable",
        ),
        stress(
            "stress-long-song",
            "Long song generation",
            CapabilityWorkflow::Song,
            MvpValidationStatus::Pending,
            "Generate a multi-minute lyrics-to-song draft with stems requested.",
            "duration stays within provider limits and section metadata survives",
        ),
        stress(
            "stress-cancellation",
            "Cancellation during generation",
            CapabilityWorkflow::CompositionRender,
            MvpValidationStatus::Passed,
            "Cancel a running provider job and preserve actionable state.",
            "job enters canceling/canceled state without orphaned outputs",
        ),
        stress(
            "stress-failed-download",
            "Failed model download",
            CapabilityWorkflow::Tts,
            MvpValidationStatus::Pending,
            "Simulate a provider package download failure.",
            "runtime exposes recovery guidance and generation remains blocked",
        ),
        stress(
            "stress-missing-gpu",
            "Missing GPU or accelerator",
            CapabilityWorkflow::Song,
            MvpValidationStatus::Pending,
            "Run provider preflight on unsupported hardware.",
            "manifest compatibility reports unavailable rather than queueing",
        ),
        stress(
            "stress-unsupported-language",
            "Unsupported language",
            CapabilityWorkflow::Tts,
            MvpValidationStatus::Pending,
            "Submit a script language outside provider support.",
            "provider matcher blocks or warns before generation",
        ),
        stress(
            "stress-rejected-voice-consent",
            "Rejected voice consent",
            CapabilityWorkflow::VoiceClone,
            MvpValidationStatus::Passed,
            "Attempt cloning or conversion with rejected consent.",
            "submission remains blocked and audit reason is visible",
        ),
        stress(
            "stress-noncommercial-commercial-project",
            "Noncommercial model in commercial project",
            CapabilityWorkflow::Song,
            MvpValidationStatus::Passed,
            "Request commercial export from a noncommercial or unknown model.",
            "commercial export is blocked with license reasons",
        ),
    ]
}

fn known_limitations() -> Vec<KnownLimitation> {
    vec![
        limitation("limit-no-real-provider-audio", "Provider evidence", "Reference fixtures define contracts but do not yet prove generated audio quality from real selected providers.", "Run first-provider smoke tests and attach artifacts to this matrix.", true),
        limitation("limit-video-to-audio-prototype", "Multimodal SFX", "Video-to-audio has a fixture and gate, but the product workflow remains tracked by sc-6183.", "Keep the workflow blocked until sc-6183 supplies prototype evidence.", true),
        limitation("limit-sceneworks-import", "SceneWorks handoff", "SoundWorks export package metadata, target video identity, compatibility checks, and provenance manifest are defined; direct runtime attachment still needs a SceneWorks-side importer.", "Do not claim in-app SceneWorks attachment until the target importer endpoint is implemented and tested in SceneWorks.", false),
        limitation("limit-release-hardware", "Runtime validation", "Mac and Windows release hardware runs are not captured by this static reference matrix.", "Attach release-run artifacts before MVP signoff.", true),
    ]
}

fn requirement_coverage() -> Vec<RequirementCoverage> {
    vec![
        coverage("epic-req-1", "Text-to-speech with many voices and consent-aware voice profiles.", &["demo-short-narration", "demo-podcast-dialogue"], &["fixture-short-narration", "fixture-podcast-dialogue"], &["check-job-contracts", "check-safety-gates"], MvpValidationStatus::ManualRequired),
        coverage("epic-req-2", "Generated sound effects, Foley, ambience, and loopable background beds.", &["demo-game-ui-sfx", "demo-loopable-ambience", "demo-video-foley"], &["fixture-game-sfx", "fixture-ambience-loop", "fixture-video-foley"], &["check-metadata-extraction", "check-provider-manifests"], MvpValidationStatus::ManualRequired),
        coverage("epic-req-3", "Instrument samples and loops with BPM, key, loop points, tags, and provenance.", &["demo-instrument-sample-pack", "demo-loop-pack"], &["fixture-sample-pack", "fixture-loop-pack"], &["check-metadata-extraction", "check-asset-lifecycle"], MvpValidationStatus::ManualRequired),
        coverage("epic-req-4", "Complete song generation with lyrics, structure, stems, and exportable masters.", &["demo-complete-song", "demo-stem-separation"], &["fixture-song-structure", "fixture-stem-bundle"], &["check-job-contracts", "check-export-sidecars"], MvpValidationStatus::ManualRequired),
        coverage("epic-req-5", "Recipe, model, seed, reference, license, provenance, and post-processing persistence.", &["demo-edit-trim-normalize", "demo-composition-export"], &["fixture-edit-normalize", "fixture-composition-export"], &["check-recipe-persistence", "check-export-sidecars"], MvpValidationStatus::Passed),
        coverage("epic-req-6", "Voice cloning, style imitation, copyrighted music similarity, and disclosure safety gates.", &["demo-podcast-dialogue", "demo-voice-conversion", "demo-complete-song"], &["fixture-podcast-dialogue", "fixture-voice-conversion", "fixture-song-structure"], &["check-safety-gates"], MvpValidationStatus::Passed),
        coverage("epic-req-7", "Capability-based provider manifests rather than one-off model assumptions.", &["demo-video-foley", "demo-stem-separation"], &["fixture-video-foley", "fixture-stem-bundle"], &["check-provider-manifests"], MvpValidationStatus::Pending),
        coverage("epic-req-8", "Audio-native review tools, version comparison, edits, and production exports.", &["demo-edit-trim-normalize", "demo-composition-export"], &["fixture-edit-normalize", "fixture-composition-export"], &["check-asset-lifecycle", "check-export-sidecars"], MvpValidationStatus::Passed),
    ]
}

fn demo(
    id: &str,
    workflow: CapabilityWorkflow,
    title: &str,
    goal: &str,
    required_artifacts: &[&str],
    acceptance: &[&str],
) -> DemoWorkflow {
    DemoWorkflow {
        id: id.to_string(),
        workflow,
        title: title.to_string(),
        goal: goal.to_string(),
        required_artifacts: strings(required_artifacts),
        acceptance: strings(acceptance),
    }
}

fn fixture(
    id: &str,
    workflow: CapabilityWorkflow,
    name: &str,
    input_contract: &str,
    expected_outputs: &[&str],
    automated_check_ids: &[&str],
) -> RegressionFixture {
    RegressionFixture {
        id: id.to_string(),
        workflow,
        name: name.to_string(),
        input_contract: input_contract.to_string(),
        expected_outputs: strings(expected_outputs),
        automated_check_ids: strings(automated_check_ids),
    }
}

fn check(
    id: &str,
    category: ValidationCategory,
    status: MvpValidationStatus,
    required_for_mvp: bool,
    summary: &str,
    evidence: &str,
) -> ValidationCheck {
    ValidationCheck {
        id: id.to_string(),
        category,
        status,
        required_for_mvp,
        summary: summary.to_string(),
        evidence: evidence.to_string(),
    }
}

fn evidence(
    id: &str,
    workflow: CapabilityWorkflow,
    status: MvpValidationStatus,
    fixture_only: bool,
    requirement: &str,
    current_evidence: &str,
    blocker: &str,
) -> RuntimeEvidenceRequirement {
    RuntimeEvidenceRequirement {
        id: id.to_string(),
        workflow,
        required_for_mvp: true,
        status,
        fixture_only,
        requirement: requirement.to_string(),
        evidence: current_evidence.to_string(),
        blocker: blocker.to_string(),
    }
}

fn scorecard(
    id: &str,
    workflow: CapabilityWorkflow,
    status: MvpValidationStatus,
    scoring_axes: &str,
    pass_threshold: &str,
    reviewer_notes: &str,
) -> ManualQaScorecard {
    ManualQaScorecard {
        id: id.to_string(),
        workflow,
        status,
        required_for_mvp: true,
        scoring_axes: scoring_axes
            .split(", ")
            .map(|axis| axis.to_string())
            .collect(),
        pass_threshold: pass_threshold.to_string(),
        reviewer_notes: reviewer_notes.to_string(),
    }
}

fn stress(
    id: &str,
    title: &str,
    workflow: CapabilityWorkflow,
    status: MvpValidationStatus,
    scenario: &str,
    expected_behavior: &str,
) -> StressCase {
    StressCase {
        id: id.to_string(),
        title: title.to_string(),
        workflow,
        status,
        required_for_mvp: true,
        scenario: scenario.to_string(),
        expected_behavior: expected_behavior.to_string(),
    }
}

fn limitation(
    id: &str,
    area: &str,
    summary: &str,
    mitigation: &str,
    blocks_mvp: bool,
) -> KnownLimitation {
    KnownLimitation {
        id: id.to_string(),
        area: area.to_string(),
        summary: summary.to_string(),
        mitigation: mitigation.to_string(),
        blocks_mvp,
    }
}

fn coverage(
    requirement_id: &str,
    epic_requirement: &str,
    demo_workflow_ids: &[&str],
    fixture_ids: &[&str],
    check_ids: &[&str],
    status: MvpValidationStatus,
) -> RequirementCoverage {
    RequirementCoverage {
        requirement_id: requirement_id.to_string(),
        epic_requirement: epic_requirement.to_string(),
        demo_workflow_ids: strings(demo_workflow_ids),
        fixture_ids: strings(fixture_ids),
        check_ids: strings(check_ids),
        status,
    }
}

fn strings(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| value.to_string()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    #[test]
    fn matrix_covers_every_capability_with_demo_and_fixture() {
        let matrix = MvpValidationOverview::reference();
        let demo_workflows: BTreeSet<CapabilityWorkflow> = matrix
            .demo_workflows
            .iter()
            .map(|workflow| workflow.workflow)
            .collect();
        let fixture_workflows: BTreeSet<CapabilityWorkflow> = matrix
            .regression_fixtures
            .iter()
            .map(|fixture| fixture.workflow)
            .collect();

        assert_eq!(
            demo_workflows,
            CapabilityWorkflow::all().into_iter().collect()
        );
        assert_eq!(
            fixture_workflows,
            CapabilityWorkflow::all().into_iter().collect()
        );
        assert_eq!(matrix.release_gate.covered_workflow_count, 12);
    }

    #[test]
    fn automated_checks_cover_story_categories() {
        let matrix = MvpValidationOverview::reference();
        let categories = matrix
            .automated_checks
            .iter()
            .map(|check| check.category)
            .collect::<BTreeSet<_>>();

        for required in [
            ValidationCategory::JobContracts,
            ValidationCategory::RecipePersistence,
            ValidationCategory::MetadataExtraction,
            ValidationCategory::ProviderManifest,
            ValidationCategory::AssetLifecycle,
            ValidationCategory::ExportSidecars,
            ValidationCategory::SafetyGates,
            ValidationCategory::RuntimeEvidence,
        ] {
            assert!(categories.contains(&required), "missing {required:?}");
        }
    }

    #[test]
    fn safety_check_status_tracks_rights_gate_coverage() {
        let inputs = DerivedValidationInputs::reference();
        assert!(inputs.full_model_use_coverage());
        assert!(inputs.safety_gates_passed());

        let matrix = MvpValidationOverview::reference();
        let safety = matrix
            .automated_checks
            .iter()
            .find(|check| check.id == "check-safety-gates")
            .expect("safety check");

        assert_eq!(safety.status, MvpValidationStatus::Passed);
        assert!(safety.evidence.contains("full model-use decision coverage"));
    }

    #[test]
    fn stress_cases_include_required_failure_modes() {
        let matrix = MvpValidationOverview::reference();
        let stress_ids = matrix
            .stress_cases
            .iter()
            .map(|stress| stress.id.as_str())
            .collect::<BTreeSet<_>>();

        for required in [
            "stress-long-script",
            "stress-long-song",
            "stress-cancellation",
            "stress-failed-download",
            "stress-missing-gpu",
            "stress-unsupported-language",
            "stress-rejected-voice-consent",
            "stress-noncommercial-commercial-project",
        ] {
            assert!(stress_ids.contains(required), "missing {required}");
        }
    }

    #[test]
    fn release_gate_stays_blocked_until_real_release_evidence_passes() {
        let matrix = MvpValidationOverview::reference();

        assert!(!matrix.release_gate.ready_for_mvp);
        assert_eq!(matrix.release_gate.required_runtime_evidence_count, 5);
        assert_eq!(matrix.release_gate.satisfied_runtime_evidence_count, 1);
        assert_eq!(matrix.release_gate.fixture_only_evidence_count, 2);
        assert!(matrix
            .release_gate
            .blocking_items
            .iter()
            .any(|item| { item.contains("manual audio-quality scorecards") }));
        assert!(matrix
            .release_gate
            .blocking_items
            .iter()
            .any(|item| { item.contains("fixture/demo data cannot satisfy") }));
        assert!(matrix.known_limitations.iter().any(|limitation| {
            limitation.blocks_mvp && limitation.id == "limit-no-real-provider-audio"
        }));
    }

    #[test]
    fn runtime_evidence_status_is_derived_from_live_gates() {
        let matrix = MvpValidationOverview::reference();
        let evidence_by_id = matrix
            .runtime_evidence
            .iter()
            .map(|evidence| (evidence.id.as_str(), evidence))
            .collect::<BTreeMap<_, _>>();

        let model_cache = evidence_by_id
            .get("evidence-model-cache")
            .expect("model cache evidence");
        assert_eq!(model_cache.status, MvpValidationStatus::Passed);
        assert!(!model_cache.fixture_only);
        assert!(model_cache.evidence.contains("ModelManagerOverview"));

        let generated_audio = evidence_by_id
            .get("evidence-generated-audio")
            .expect("generated audio evidence");
        assert_eq!(generated_audio.status, MvpValidationStatus::Pending);
        assert!(!generated_audio.fixture_only);
        assert!(generated_audio.evidence.contains("Native Rust"));

        let generation_jobs = evidence_by_id
            .get("evidence-generation-jobs")
            .expect("generation job evidence");
        assert_eq!(generation_jobs.status, MvpValidationStatus::Pending);
        assert!(generation_jobs.fixture_only);
        assert!(generation_jobs.evidence.contains("RuntimeOverview"));
    }

    #[test]
    fn requirement_coverage_maps_back_to_epic_requirements() {
        let matrix = MvpValidationOverview::reference();
        let statuses = matrix
            .requirement_coverage
            .iter()
            .map(|coverage| (coverage.requirement_id.as_str(), coverage.status))
            .collect::<BTreeMap<_, _>>();

        assert_eq!(matrix.requirement_coverage.len(), 8);
        assert_eq!(statuses["epic-req-1"], MvpValidationStatus::ManualRequired);
        assert!(matrix.requirement_coverage.iter().all(|coverage| !coverage
            .demo_workflow_ids
            .is_empty()
            && !coverage.fixture_ids.is_empty()
            && !coverage.check_ids.is_empty()));
    }
}
