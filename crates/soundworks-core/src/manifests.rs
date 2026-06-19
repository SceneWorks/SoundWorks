use crate::domain::{
    AudioAssetKind, AudioFileFormat, ModelDescriptor, ModelRuntime, StemKind, WatermarkStatus,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderCatalog {
    pub schema_version: u32,
    pub providers: Vec<ProviderManifest>,
}

impl ProviderCatalog {
    pub fn reference() -> Self {
        Self {
            schema_version: 1,
            providers: vec![reference_provider(), native_recovery_provider()],
        }
    }

    pub fn capability_count(&self) -> usize {
        self.providers
            .iter()
            .flat_map(|provider| provider.models.iter())
            .map(|model| model.capabilities.len())
            .sum()
    }

    pub fn model_count(&self) -> usize {
        self.providers
            .iter()
            .map(|provider| provider.models.len())
            .sum()
    }

    pub fn workflow_coverage(&self) -> Vec<CapabilityWorkflow> {
        let workflows: BTreeSet<CapabilityWorkflow> = self
            .providers
            .iter()
            .flat_map(|provider| provider.models.iter())
            .flat_map(|model| model.capabilities.iter())
            .map(|capability| capability.workflow)
            .collect();

        workflows.into_iter().collect()
    }

    pub fn find_matches(&self, query: &CapabilityQuery) -> Vec<CapabilityMatch> {
        let mut matches: Vec<CapabilityMatch> = self
            .providers
            .iter()
            .flat_map(|provider| {
                provider.models.iter().filter_map(move |model| {
                    model
                        .capabilities
                        .iter()
                        .find(|capability| capability.matches(query, model))
                        .map(|capability| CapabilityMatch {
                            provider_id: provider.id.clone(),
                            model_id: model.id.clone(),
                            model_version: model.version.clone(),
                            workflow: capability.workflow,
                            priority: capability.priority,
                            defaults: capability.defaults.clone(),
                            descriptor: model.descriptor(provider.id.clone()),
                        })
                })
            })
            .collect();

        matches.sort_by(|left, right| right.priority.cmp(&left.priority));
        matches
    }

    pub fn default_for(&self, workflow: CapabilityWorkflow) -> Option<CapabilityMatch> {
        let query = CapabilityQuery {
            workflow,
            require_runnable: true,
            ..CapabilityQuery::default()
        };

        self.find_matches(&query).into_iter().next()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderManifest {
    pub id: String,
    pub name: String,
    pub manifest_version: String,
    pub source: ManifestSource,
    pub models: Vec<ModelManifest>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManifestSource {
    pub origin: ManifestOrigin,
    pub uri: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ManifestOrigin {
    BuiltInReference,
    LocalFile,
    ProviderPackage,
    RemoteRegistry,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelManifest {
    pub id: String,
    pub name: String,
    pub version: Option<String>,
    pub model_hash: Option<String>,
    pub runtime: ModelRuntime,
    pub install: ModelInstall,
    pub requirements: ModelRequirements,
    pub capabilities: Vec<ModelCapability>,
}

impl ModelManifest {
    pub fn descriptor(&self, provider_id: String) -> ModelDescriptor {
        ModelDescriptor {
            provider_id,
            model_id: self.id.clone(),
            model_version: self.version.clone(),
            model_hash: self.model_hash.clone(),
            runtime: self.runtime,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelInstall {
    pub status: ModelInstallStatus,
    pub package_id: Option<String>,
    pub installed_size_mb: Option<u32>,
}

impl ModelInstall {
    fn is_runnable(&self) -> bool {
        matches!(
            self.status,
            ModelInstallStatus::Installed
                | ModelInstallStatus::Packaged
                | ModelInstallStatus::External
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ModelInstallStatus {
    Installed,
    Packaged,
    Installable,
    External,
    Unavailable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelRequirements {
    pub accelerators: Vec<DeviceAccelerator>,
    pub min_memory_mb: Option<u32>,
    pub dependencies: Vec<String>,
    pub requires_network: bool,
    pub license: ModelLicense,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DeviceAccelerator {
    Cpu,
    Mps,
    Cuda,
    Metal,
    CoreMl,
    ExternalService,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ModelLicense {
    Open,
    CommercialAllowed,
    NonCommercial,
    ProviderTerms,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelCapability {
    pub workflow: CapabilityWorkflow,
    pub display_name: String,
    pub priority: u16,
    pub inputs: Vec<CapabilityInput>,
    pub outputs: CapabilityOutput,
    pub limits: CapabilityLimits,
    pub defaults: CapabilityDefaults,
    pub safety: CapabilitySafety,
}

impl ModelCapability {
    fn matches(&self, query: &CapabilityQuery, model: &ModelManifest) -> bool {
        self.workflow == query.workflow
            && query
                .required_inputs
                .iter()
                .all(|required| self.inputs.contains(required))
            && query
                .preferred_runtime
                .map_or(true, |runtime| runtime == model.runtime)
            && query
                .output_asset_kind
                .map_or(true, |kind| self.outputs.asset_kinds.contains(&kind))
            && query.min_sample_rate_hz.map_or(true, |sample_rate| {
                self.outputs
                    .sample_rates_hz
                    .iter()
                    .any(|supported| *supported >= sample_rate)
            })
            && query.channel_layout.map_or(true, |layout| {
                self.outputs.channel_layouts.contains(&layout)
            })
            && query
                .requested_stems
                .iter()
                .all(|stem| self.outputs.stems.contains(stem))
            && query.language.as_ref().map_or(true, |language| {
                self.limits.supported_languages.is_empty()
                    || self
                        .limits
                        .supported_languages
                        .iter()
                        .any(|supported| supported == language)
            })
            && query.duration_ms.map_or(true, |duration| {
                self.limits
                    .min_duration_ms
                    .map_or(true, |minimum| duration >= minimum)
                    && self
                        .limits
                        .max_duration_ms
                        .map_or(true, |maximum| duration <= maximum)
            })
            && (!query.commercial_use_required || self.safety.commercial_use_allowed)
            && (!query.require_runnable || model.install.is_runnable())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CapabilityWorkflow {
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
    Edit,
    CompositionRender,
}

impl CapabilityWorkflow {
    pub fn all() -> Vec<Self> {
        vec![
            Self::Tts,
            Self::VoiceClone,
            Self::VoiceConversion,
            Self::Sfx,
            Self::Ambience,
            Self::InstrumentSample,
            Self::Loop,
            Self::Song,
            Self::StemSeparation,
            Self::VideoToAudio,
            Self::Edit,
            Self::CompositionRender,
        ]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CapabilityInput {
    TextPrompt,
    Script,
    SourceVoice,
    SourceAudio,
    SourceVideo,
    ReferenceAudio,
    Midi,
    Tempo,
    MusicalKey,
    Duration,
    Lyrics,
    StyleTags,
    Stems,
    Composition,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CapabilityOutput {
    pub asset_kinds: Vec<AudioAssetKind>,
    pub formats: Vec<AudioFileFormat>,
    pub channel_layouts: Vec<ChannelLayout>,
    pub sample_rates_hz: Vec<u32>,
    pub stems: Vec<StemKind>,
    pub metadata_sidecar: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ChannelLayout {
    Mono,
    Stereo,
    Surround51,
    Stems,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CapabilityLimits {
    pub min_duration_ms: Option<u64>,
    pub max_duration_ms: Option<u64>,
    pub supported_languages: Vec<String>,
    pub max_speakers: Option<u16>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CapabilityDefaults {
    pub format: AudioFileFormat,
    pub sample_rate_hz: u32,
    pub channel_layout: ChannelLayout,
    pub parameters: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CapabilitySafety {
    pub commercial_use_allowed: bool,
    pub requires_voice_consent: bool,
    pub watermark: WatermarkStatus,
    pub provenance_sidecar: bool,
    pub disallowed_uses: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CapabilityQuery {
    pub workflow: CapabilityWorkflow,
    pub required_inputs: Vec<CapabilityInput>,
    pub preferred_runtime: Option<ModelRuntime>,
    pub output_asset_kind: Option<AudioAssetKind>,
    pub min_sample_rate_hz: Option<u32>,
    pub channel_layout: Option<ChannelLayout>,
    pub requested_stems: Vec<StemKind>,
    pub language: Option<String>,
    pub duration_ms: Option<u64>,
    pub commercial_use_required: bool,
    pub require_runnable: bool,
}

impl Default for CapabilityQuery {
    fn default() -> Self {
        Self {
            workflow: CapabilityWorkflow::Tts,
            required_inputs: vec![],
            preferred_runtime: None,
            output_asset_kind: None,
            min_sample_rate_hz: None,
            channel_layout: None,
            requested_stems: vec![],
            language: None,
            duration_ms: None,
            commercial_use_required: false,
            require_runnable: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CapabilityMatch {
    pub provider_id: String,
    pub model_id: String,
    pub model_version: Option<String>,
    pub workflow: CapabilityWorkflow,
    pub priority: u16,
    pub defaults: CapabilityDefaults,
    pub descriptor: ModelDescriptor,
}

fn reference_provider() -> ProviderManifest {
    ProviderManifest {
        id: "soundworks-reference".to_string(),
        name: "SoundWorks Reference Capability Registry".to_string(),
        manifest_version: "1.0.0".to_string(),
        source: ManifestSource {
            origin: ManifestOrigin::BuiltInReference,
            uri: None,
            notes: Some(
                "Fixture provider used to validate manifest contracts before source-backed model evaluation."
                    .to_string(),
            ),
        },
        models: vec![
            speech_reference_model(),
            generation_reference_model(),
            utility_reference_model(),
        ],
    }
}

fn speech_reference_model() -> ModelManifest {
    ModelManifest {
        id: "reference-speech-suite".to_string(),
        name: "Reference Speech Suite".to_string(),
        version: Some("0.1.0".to_string()),
        model_hash: Some("sha256:reference-speech".to_string()),
        runtime: ModelRuntime::Local,
        install: packaged_install("soundworks-reference-speech", 2048),
        requirements: local_requirements(4096),
        capabilities: vec![
            capability(
                CapabilityWorkflow::Tts,
                "Text-to-speech",
                90,
                vec![
                    CapabilityInput::Script,
                    CapabilityInput::SourceVoice,
                    CapabilityInput::Duration,
                ],
                vec![AudioAssetKind::VoiceClip],
                vec![ChannelLayout::Mono, ChannelLayout::Stereo],
                vec![],
                speech_defaults(),
                safety(true, true),
            ),
            capability(
                CapabilityWorkflow::VoiceClone,
                "Voice clone profile creation",
                80,
                vec![CapabilityInput::SourceVoice, CapabilityInput::SourceAudio],
                vec![AudioAssetKind::ReferenceAudio],
                vec![ChannelLayout::Mono],
                vec![],
                speech_defaults(),
                safety(true, true),
            ),
            capability(
                CapabilityWorkflow::VoiceConversion,
                "Voice conversion",
                85,
                vec![CapabilityInput::SourceAudio, CapabilityInput::SourceVoice],
                vec![AudioAssetKind::VoiceClip],
                vec![ChannelLayout::Mono, ChannelLayout::Stereo],
                vec![],
                speech_defaults(),
                safety(true, true),
            ),
        ],
    }
}

fn generation_reference_model() -> ModelManifest {
    ModelManifest {
        id: "reference-generation-suite".to_string(),
        name: "Reference Audio Generation Suite".to_string(),
        version: Some("0.1.0".to_string()),
        model_hash: Some("sha256:reference-generation".to_string()),
        runtime: ModelRuntime::Local,
        install: packaged_install("soundworks-reference-generation", 8192),
        requirements: local_requirements(12_288),
        capabilities: vec![
            capability(
                CapabilityWorkflow::Sfx,
                "Sound effects",
                90,
                vec![CapabilityInput::TextPrompt, CapabilityInput::Duration],
                vec![AudioAssetKind::Sfx],
                vec![ChannelLayout::Mono, ChannelLayout::Stereo],
                vec![],
                music_defaults(),
                safety(true, false),
            ),
            capability(
                CapabilityWorkflow::Ambience,
                "Ambience beds",
                85,
                vec![CapabilityInput::TextPrompt, CapabilityInput::Duration],
                vec![AudioAssetKind::Ambience],
                vec![ChannelLayout::Stereo],
                vec![],
                music_defaults(),
                safety(true, false),
            ),
            capability(
                CapabilityWorkflow::InstrumentSample,
                "Instrument samples",
                85,
                vec![
                    CapabilityInput::TextPrompt,
                    CapabilityInput::MusicalKey,
                    CapabilityInput::Duration,
                ],
                vec![AudioAssetKind::InstrumentSample],
                vec![ChannelLayout::Mono, ChannelLayout::Stereo],
                vec![],
                music_defaults(),
                safety(true, false),
            ),
            capability(
                CapabilityWorkflow::Loop,
                "Loop generation",
                88,
                vec![
                    CapabilityInput::TextPrompt,
                    CapabilityInput::Tempo,
                    CapabilityInput::MusicalKey,
                    CapabilityInput::Duration,
                ],
                vec![AudioAssetKind::Loop],
                vec![ChannelLayout::Stereo],
                vec![],
                music_defaults(),
                safety(true, false),
            ),
            capability(
                CapabilityWorkflow::Song,
                "Complete song generation",
                82,
                vec![
                    CapabilityInput::TextPrompt,
                    CapabilityInput::Lyrics,
                    CapabilityInput::StyleTags,
                    CapabilityInput::Stems,
                ],
                vec![AudioAssetKind::Song, AudioAssetKind::Stem],
                vec![ChannelLayout::Stereo, ChannelLayout::Stems],
                vec![
                    StemKind::Vocals,
                    StemKind::Drums,
                    StemKind::Bass,
                    StemKind::Instruments,
                ],
                music_defaults(),
                safety(true, false),
            ),
            capability(
                CapabilityWorkflow::VideoToAudio,
                "Video-to-audio",
                78,
                vec![
                    CapabilityInput::SourceVideo,
                    CapabilityInput::TextPrompt,
                    CapabilityInput::Duration,
                ],
                vec![AudioAssetKind::Sfx, AudioAssetKind::Ambience],
                vec![ChannelLayout::Stereo],
                vec![],
                music_defaults(),
                safety(true, false),
            ),
        ],
    }
}

fn native_recovery_provider() -> ProviderManifest {
    ProviderManifest {
        id: "soundworks-native".to_string(),
        name: "SoundWorks native recovery adapters".to_string(),
        manifest_version: "0.1.0".to_string(),
        source: ManifestSource {
            origin: ManifestOrigin::BuiltInReference,
            uri: None,
            notes: Some(
                "Built-in Rust recovery adapters provide real generated artifacts while ML provider ports remain gated."
                    .to_string(),
            ),
        },
        models: vec![ModelManifest {
            id: "native-procedural-music".to_string(),
            name: "SoundWorks native procedural samples and loops".to_string(),
            version: Some("0.1.0".to_string()),
            model_hash: Some("sha256:soundworks-native-procedural-music".to_string()),
            runtime: ModelRuntime::Local,
            install: packaged_install("soundworks-native-procedural-music", 1),
            requirements: local_requirements(256),
            capabilities: vec![
                capability(
                    CapabilityWorkflow::InstrumentSample,
                    "Procedural instrument samples",
                    91,
                    vec![
                        CapabilityInput::TextPrompt,
                        CapabilityInput::MusicalKey,
                        CapabilityInput::Duration,
                    ],
                    vec![AudioAssetKind::InstrumentSample],
                    vec![ChannelLayout::Mono, ChannelLayout::Stereo],
                    vec![],
                    music_defaults(),
                    safety(true, false),
                ),
                capability(
                    CapabilityWorkflow::Loop,
                    "Procedural tempo-aligned loops",
                    92,
                    vec![
                        CapabilityInput::TextPrompt,
                        CapabilityInput::Tempo,
                        CapabilityInput::MusicalKey,
                        CapabilityInput::Duration,
                    ],
                    vec![AudioAssetKind::Loop],
                    vec![ChannelLayout::Stereo],
                    vec![],
                    music_defaults(),
                    safety(true, false),
                ),
            ],
        }],
    }
}

fn utility_reference_model() -> ModelManifest {
    ModelManifest {
        id: "reference-utility-suite".to_string(),
        name: "Reference Utility Suite".to_string(),
        version: Some("0.1.0".to_string()),
        model_hash: Some("sha256:reference-utility".to_string()),
        runtime: ModelRuntime::Local,
        install: packaged_install("soundworks-reference-utility", 1024),
        requirements: local_requirements(2048),
        capabilities: vec![
            capability(
                CapabilityWorkflow::StemSeparation,
                "Stem separation",
                80,
                vec![CapabilityInput::SourceAudio, CapabilityInput::Stems],
                vec![AudioAssetKind::Stem],
                vec![ChannelLayout::Stems],
                vec![
                    StemKind::Vocals,
                    StemKind::Drums,
                    StemKind::Bass,
                    StemKind::Instruments,
                ],
                music_defaults(),
                safety(true, false),
            ),
            capability(
                CapabilityWorkflow::Edit,
                "Audio edit rendering",
                75,
                vec![CapabilityInput::SourceAudio, CapabilityInput::TextPrompt],
                vec![
                    AudioAssetKind::VoiceClip,
                    AudioAssetKind::Sfx,
                    AudioAssetKind::MusicClip,
                    AudioAssetKind::Loop,
                ],
                vec![ChannelLayout::Mono, ChannelLayout::Stereo],
                vec![],
                music_defaults(),
                safety(true, false),
            ),
            capability(
                CapabilityWorkflow::CompositionRender,
                "Composition render",
                70,
                vec![CapabilityInput::Composition, CapabilityInput::Stems],
                vec![AudioAssetKind::Composition, AudioAssetKind::MixdownExport],
                vec![ChannelLayout::Stereo, ChannelLayout::Stems],
                vec![
                    StemKind::Vocals,
                    StemKind::Drums,
                    StemKind::Bass,
                    StemKind::Instruments,
                ],
                music_defaults(),
                safety(true, false),
            ),
        ],
    }
}

fn capability(
    workflow: CapabilityWorkflow,
    display_name: &str,
    priority: u16,
    inputs: Vec<CapabilityInput>,
    asset_kinds: Vec<AudioAssetKind>,
    channel_layouts: Vec<ChannelLayout>,
    stems: Vec<StemKind>,
    defaults: CapabilityDefaults,
    safety: CapabilitySafety,
) -> ModelCapability {
    ModelCapability {
        workflow,
        display_name: display_name.to_string(),
        priority,
        inputs,
        outputs: CapabilityOutput {
            asset_kinds,
            formats: vec![AudioFileFormat::Wav, AudioFileFormat::Flac],
            channel_layouts,
            sample_rates_hz: vec![44_100, 48_000],
            stems,
            metadata_sidecar: true,
        },
        limits: CapabilityLimits {
            min_duration_ms: Some(250),
            max_duration_ms: Some(300_000),
            supported_languages: vec!["en-US".to_string()],
            max_speakers: Some(4),
        },
        defaults,
        safety,
    }
}

fn packaged_install(package_id: &str, installed_size_mb: u32) -> ModelInstall {
    ModelInstall {
        status: ModelInstallStatus::Packaged,
        package_id: Some(package_id.to_string()),
        installed_size_mb: Some(installed_size_mb),
    }
}

fn local_requirements(min_memory_mb: u32) -> ModelRequirements {
    ModelRequirements {
        accelerators: vec![DeviceAccelerator::Cpu, DeviceAccelerator::Mps],
        min_memory_mb: Some(min_memory_mb),
        dependencies: vec![],
        requires_network: false,
        license: ModelLicense::CommercialAllowed,
    }
}

fn speech_defaults() -> CapabilityDefaults {
    CapabilityDefaults {
        format: AudioFileFormat::Wav,
        sample_rate_hz: 48_000,
        channel_layout: ChannelLayout::Mono,
        parameters: BTreeMap::from([
            ("temperature".to_string(), json!(0.7)),
            ("normalizeLoudnessLufs".to_string(), json!(-18.0)),
        ]),
    }
}

fn music_defaults() -> CapabilityDefaults {
    CapabilityDefaults {
        format: AudioFileFormat::Wav,
        sample_rate_hz: 48_000,
        channel_layout: ChannelLayout::Stereo,
        parameters: BTreeMap::from([
            ("temperature".to_string(), json!(0.8)),
            ("normalizeLoudnessLufs".to_string(), json!(-14.0)),
        ]),
    }
}

fn safety(commercial_use_allowed: bool, requires_voice_consent: bool) -> CapabilitySafety {
    CapabilitySafety {
        commercial_use_allowed,
        requires_voice_consent,
        watermark: WatermarkStatus::SidecarOnly,
        provenance_sidecar: true,
        disallowed_uses: vec!["impersonation without consent".to_string()],
    }
}

#[cfg(test)]
mod tests {
    use super::{
        CapabilityInput, CapabilityQuery, CapabilityWorkflow, ChannelLayout, ProviderCatalog,
    };
    use crate::domain::{AudioAssetKind, ModelRuntime, StemKind};

    #[test]
    fn reference_catalog_covers_all_initial_capability_workflows() {
        let catalog = ProviderCatalog::reference();

        assert_eq!(catalog.workflow_coverage(), CapabilityWorkflow::all());
        assert_eq!(catalog.model_count(), 4);
        assert_eq!(catalog.capability_count(), 14);
    }

    #[test]
    fn matcher_filters_by_workflow_inputs_runtime_outputs_and_defaults() {
        let catalog = ProviderCatalog::reference();
        let matches = catalog.find_matches(&CapabilityQuery {
            workflow: CapabilityWorkflow::Song,
            required_inputs: vec![CapabilityInput::Lyrics, CapabilityInput::Stems],
            preferred_runtime: Some(ModelRuntime::Local),
            output_asset_kind: Some(AudioAssetKind::Song),
            min_sample_rate_hz: Some(48_000),
            channel_layout: Some(ChannelLayout::Stereo),
            requested_stems: vec![StemKind::Vocals, StemKind::Drums],
            language: Some("en-US".to_string()),
            duration_ms: Some(180_000),
            commercial_use_required: true,
            require_runnable: true,
        });

        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].provider_id, "soundworks-reference");
        assert_eq!(matches[0].model_id, "reference-generation-suite");
        assert_eq!(matches[0].defaults.sample_rate_hz, 48_000);
        assert_eq!(matches[0].descriptor.runtime, ModelRuntime::Local);
    }

    #[test]
    fn matcher_rejects_unsupported_requirements() {
        let catalog = ProviderCatalog::reference();
        let matches = catalog.find_matches(&CapabilityQuery {
            workflow: CapabilityWorkflow::Tts,
            required_inputs: vec![CapabilityInput::SourceVideo],
            ..CapabilityQuery::default()
        });

        assert!(matches.is_empty());
    }

    #[test]
    fn default_selection_uses_highest_priority_runnable_capability() {
        let catalog = ProviderCatalog::reference();
        let default = catalog
            .default_for(CapabilityWorkflow::VoiceConversion)
            .expect("voice conversion default");

        assert_eq!(default.model_id, "reference-speech-suite");
        assert_eq!(default.priority, 85);
    }

    #[test]
    fn reference_catalog_serializes_for_tauri_and_storage_boundaries() {
        let payload = serde_json::to_value(ProviderCatalog::reference()).expect("catalog json");

        assert_eq!(payload["schemaVersion"], 1);
        assert_eq!(
            payload["providers"][0]["models"][0]["capabilities"][0]["workflow"],
            "tts"
        );
        assert_eq!(
            payload["providers"][0]["models"][1]["capabilities"][4]["workflow"],
            "song"
        );
    }
}
