use crate::manifests::CapabilityWorkflow;
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
        let demo_workflows = demo_workflows();
        let regression_fixtures = regression_fixtures();
        let automated_checks = automated_checks();
        let manual_scorecards = manual_scorecards();
        let stress_cases = stress_cases();
        let known_limitations = known_limitations();
        let runtime_evidence = runtime_evidence();
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

fn automated_checks() -> Vec<ValidationCheck> {
    vec![
        check("check-job-contracts", ValidationCategory::JobContracts, MvpValidationStatus::Pending, true, "Generation job contracts serialize status, progress, outputs, cancellation, and actionable errors, but product-runtime execution is not proven.", "Current tests cover fixture snapshots only. SC-6468 must attach real queued job records before this can pass."),
        check("check-recipe-persistence", ValidationCategory::RecipePersistence, MvpValidationStatus::Passed, true, "Recipes preserve provider/model, seed, references, post-processing, outputs, and replayability.", "Fixture and review tests assert serializable inspectable recipes and edit chains."),
        check("check-metadata-extraction", ValidationCategory::MetadataExtraction, MvpValidationStatus::Pending, true, "Audio metadata contracts include duration, sample rate, channels, loudness, true peak, BPM, key, and loop points, but generated-file extraction is not proven.", "SFX, samples, songs, review, library, and export reference data expose fields. Real generated audio must replace fixture values."),
        check("check-provider-manifests", ValidationCategory::ProviderManifest, MvpValidationStatus::Pending, true, "Provider manifests distinguish workflows, inputs, outputs, limits, hardware, license, and runnable defaults, but a single capability catalog is not yet proven end to end at runtime.", "Runtime dispatch is now data-driven (F-007: each model carries an execution strategy resolved from the native registry, not a literal id match). Remaining: collapse the reference and runtime catalogs into one and capture runtime evidence before this can pass."),
        check("check-asset-lifecycle", ValidationCategory::AssetLifecycle, MvpValidationStatus::Pending, true, "Asset lifecycle contracts cover project/global library, tags, collections, saved outputs, version history, and reuse targets, but persisted runtime assets are not proven.", "Asset library fixtures cover scopes, tags, collections, lifecycle actions, and provenance links. SC-6469 must prove persisted assets."),
        check("check-export-sidecars", ValidationCategory::ExportSidecars, MvpValidationStatus::Pending, true, "Export sidecar contracts include preset, target, formats, DAW bundle, SceneWorks handoff, and metadata, but file-writing evidence is not attached.", "Export workflow sidecars include recipe, model, source media, rights, disclosure, and edit-chain fields. SC-6473 must prove real files."),
        check("check-safety-gates", ValidationCategory::SafetyGates, MvpValidationStatus::Passed, true, "Voice consent and commercial model-use gates are enforced in the runtime, not just described.", "F-003: generation admission resolves consent from the persisted voice profile (a caller-supplied boolean cannot bypass it) and rejects Blocked model-use; export is blocked per-asset on unconsented voices and disallowed commercial use. Runtime tests assert each gate (consent_boolean_no_longer_bypasses_the_gate, blocked_model_is_rejected_before_execution, profile_referencing_job_is_blocked_without_recorded_consent)."),
        check("check-runtime-evidence", ValidationCategory::RuntimeEvidence, MvpValidationStatus::Pending, true, "Runtime installed counts, queued jobs, generated audio, playback, edits, and export claims require real artifact evidence.", "SC-6466 blocks fixture-only completion. Follow-on recovery stories must attach cache, job, media, playback, edit, and export artifacts."),
        check("check-release-docs", ValidationCategory::Documentation, MvpValidationStatus::Passed, true, "Validation matrix maps back to epic requirements and states what remains unverified.", "docs/mvp-validation.md is the human-readable release matrix."),
        check("check-release-run-artifacts", ValidationCategory::Stress, MvpValidationStatus::Pending, true, "Release run artifacts must capture current Mac and Windows validation evidence before MVP signoff.", "This story defines the evidence contract; release artifacts remain pending until real provider runs exist."),
    ]
}

fn runtime_evidence() -> Vec<RuntimeEvidenceRequirement> {
    vec![
        evidence(
            "evidence-model-cache",
            CapabilityWorkflow::Tts,
            "Installed model counts must come from verified cache/package files, not static provider manifests.",
            "Reference manifests currently describe packaged models, but no cache/package verification is attached.",
            "SC-6467 must implement model download/cache verification before any model can be counted as installed.",
        ),
        evidence(
            "evidence-generation-jobs",
            CapabilityWorkflow::Tts,
            "Generation controls must enqueue persisted runtime jobs with progress, errors, logs, and artifacts.",
            "Current UI and Rust snapshots are contract/demo data only.",
            "SC-6468 must replace snapshots with runtime job execution.",
        ),
        evidence(
            "evidence-generated-audio",
            CapabilityWorkflow::Sfx,
            "TTS, SFX, samples, loops, and song criteria require generated audio files from selected providers.",
            "Fixture media paths are representative and do not prove generated bytes exist.",
            "SC-6470, SC-6471, and SC-6472 must attach generated audio artifacts or source-backed blockers.",
        ),
        evidence(
            "evidence-playback-edit",
            CapabilityWorkflow::Edit,
            "Playback, trim, fade, normalize, loop inspection, and version comparison must run against real audio files.",
            "Review/edit fixtures describe the workflow but do not prove audible playback or file edits.",
            "SC-6473 must validate real playback and non-destructive edited versions.",
        ),
        evidence(
            "evidence-export-files",
            CapabilityWorkflow::CompositionRender,
            "Export criteria require actual WAV/FLAC/MP3/OGG files plus provenance sidecars on disk.",
            "Export contract data exists, but no runtime file-writing evidence is attached.",
            "SC-6473 must write and validate real export files before this gate can pass.",
        ),
    ]
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
    requirement: &str,
    current_evidence: &str,
    blocker: &str,
) -> RuntimeEvidenceRequirement {
    RuntimeEvidenceRequirement {
        id: id.to_string(),
        workflow,
        required_for_mvp: true,
        status: MvpValidationStatus::Pending,
        fixture_only: true,
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
        assert_eq!(matrix.release_gate.satisfied_runtime_evidence_count, 0);
        assert_eq!(matrix.release_gate.fixture_only_evidence_count, 5);
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
    fn fixture_only_evidence_cannot_satisfy_runtime_mvp_criteria() {
        let matrix = MvpValidationOverview::reference();

        assert!(matrix
            .runtime_evidence
            .iter()
            .all(|evidence| evidence.required_for_mvp
                && evidence.fixture_only
                && evidence.status != MvpValidationStatus::Passed));
        assert!(matrix.runtime_evidence.iter().any(|evidence| {
            evidence.id == "evidence-model-cache"
                && evidence
                    .requirement
                    .contains("verified cache/package files")
        }));
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
