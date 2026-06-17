use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppOverview {
    pub product_name: String,
    pub architecture: ArchitectureOverview,
    pub studios: Vec<StudioSurface>,
    pub commands: Vec<CommandBoundary>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchitectureOverview {
    pub layers: Vec<ArchitectureLayer>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchitectureLayer {
    pub id: String,
    pub name: String,
    pub responsibility: String,
    pub status: ScaffoldStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StudioSurface {
    pub id: String,
    pub name: String,
    pub route: String,
    pub status: ScaffoldStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandBoundary {
    pub name: String,
    pub direction: CommandDirection,
    pub purpose: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CommandDirection {
    UiToBackend,
    BackendToUi,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ScaffoldStatus {
    Planned,
    Scaffolded,
}

impl AppOverview {
    pub fn baseline() -> Self {
        Self {
            product_name: "SoundWorks".to_string(),
            architecture: ArchitectureOverview {
                layers: vec![
                    ArchitectureLayer {
                        id: "react-ui".to_string(),
                        name: "React UI".to_string(),
                        responsibility:
                            "Workflow surfaces, library navigation, waveform review, and composition controls."
                                .to_string(),
                        status: ScaffoldStatus::Scaffolded,
                    },
                    ArchitectureLayer {
                        id: "tauri-commands".to_string(),
                        name: "Tauri Commands".to_string(),
                        responsibility:
                            "Narrow command bridge between the UI and local Rust services.".to_string(),
                        status: ScaffoldStatus::Scaffolded,
                    },
                    ArchitectureLayer {
                        id: "soundworks-core".to_string(),
                        name: "Rust Core".to_string(),
                        responsibility:
                            "Shared domain contracts for assets, recipes, jobs, providers, and exports."
                                .to_string(),
                        status: ScaffoldStatus::Scaffolded,
                    },
                    ArchitectureLayer {
                        id: "worker-runtime".to_string(),
                        name: "Worker Runtime".to_string(),
                        responsibility:
                            "Model execution, installation, device capabilities, progress, and cancellation."
                                .to_string(),
                        status: ScaffoldStatus::Planned,
                    },
                ],
            },
            studios: vec![
                StudioSurface::planned("tts", "TTS Studio", "/studios/tts"),
                StudioSurface::planned("voice-lab", "Voice Lab", "/studios/voice-lab"),
                StudioSurface::planned("sfx", "SFX + Ambience", "/studios/sfx"),
                StudioSurface::planned("loops", "Samples + Loops", "/studios/loops"),
                StudioSurface::planned("songs", "Song Studio", "/studios/songs"),
                StudioSurface::planned(
                    "video-to-audio",
                    "Video to Audio",
                    "/studios/video-to-audio",
                ),
            ],
            commands: vec![CommandBoundary {
                name: "get_app_overview".to_string(),
                direction: CommandDirection::UiToBackend,
                purpose:
                    "Load scaffolded architecture and workflow metadata from the Rust backend."
                        .to_string(),
            }],
        }
    }
}

impl StudioSurface {
    fn planned(id: &str, name: &str, route: &str) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            route: route.to_string(),
            status: ScaffoldStatus::Planned,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{AppOverview, ScaffoldStatus};

    #[test]
    fn baseline_contains_all_initial_studio_surfaces() {
        let overview = AppOverview::baseline();
        let studio_ids: Vec<&str> = overview
            .studios
            .iter()
            .map(|studio| studio.id.as_str())
            .collect();

        assert_eq!(
            studio_ids,
            vec![
                "tts",
                "voice-lab",
                "sfx",
                "loops",
                "songs",
                "video-to-audio"
            ]
        );
    }

    #[test]
    fn baseline_marks_current_architecture_layers() {
        let overview = AppOverview::baseline();

        assert!(overview
            .architecture
            .layers
            .iter()
            .any(|layer| layer.id == "react-ui" && layer.status == ScaffoldStatus::Scaffolded));
        assert!(overview
            .architecture
            .layers
            .iter()
            .any(|layer| layer.id == "worker-runtime" && layer.status == ScaffoldStatus::Planned));
    }

    #[test]
    fn baseline_serializes_for_tauri_boundary() {
        let payload = serde_json::to_value(AppOverview::baseline()).expect("baseline serializes");

        assert_eq!(payload["productName"], "SoundWorks");
        assert_eq!(payload["commands"][0]["name"], "get_app_overview");
    }
}
