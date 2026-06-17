export type ArchitectureLayer = {
  id: string;
  name: string;
  responsibility: string;
  status: "planned" | "scaffolded";
};

export type StudioSurface = {
  id: string;
  name: string;
  route: string;
  status: "planned" | "scaffolded";
};

export type CommandBoundary = {
  name: string;
  direction: "ui-to-backend" | "backend-to-ui";
  purpose: string;
};

export type AppOverview = {
  productName: string;
  architecture: {
    layers: ArchitectureLayer[];
  };
  studios: StudioSurface[];
  commands: CommandBoundary[];
  providerCatalog: ProviderCatalogOverview;
};

export type RuntimeAvailability = "installed" | "available" | "unavailable";

export type RuntimeModelState = {
  providerId: string;
  modelId: string;
  modelName: string;
  availability: RuntimeAvailability;
  installStatus: string;
  health: string;
  workflows: CapabilityWorkflow[];
  reasons: string[];
  cache: {
    status: string;
    expectedSizeMb?: number | null;
    diskUsageMb?: number | null;
    warmup: string;
  };
  compatibility: {
    supported: boolean;
    selectedAccelerator?: string | null;
    minMemoryMb?: number | null;
    availableMemoryMb?: number | null;
    requiresNetwork: boolean;
    reasons: string[];
  };
};

export type RuntimeJobSnapshot = {
  id: string;
  kind: string;
  status: string;
  providerId: string;
  modelId: string;
  progress?: {
    percent: number;
    message?: string | null;
  } | null;
  cancellation: string;
  retryCount: number;
  logTail: string[];
  actionableError?: {
    code: string;
    summary: string;
    recovery: string;
  } | null;
};

export type RuntimeValidationCheck = {
  id: string;
  status: "passed" | "warning" | "failed";
  summary: string;
  recovery?: string | null;
};

export type RuntimeOverview = {
  schemaVersion: number;
  packagingPolicy: {
    name: string;
    productRuntimeAllowsPython: boolean;
    shippedPlatforms: string[];
    workerProcess: string;
    modelCacheRoots: Array<{
      platform: string;
      pathHint: string;
      purpose: string;
    }>;
  };
  devices: Array<{
    accelerator: string;
    name: string;
    memoryMb?: number | null;
    available: boolean;
    driver?: string | null;
  }>;
  statusCounts: {
    installed: number;
    available: number;
    unavailable: number;
  };
  modelStates: RuntimeModelState[];
  jobs: RuntimeJobSnapshot[];
  validationChecks: RuntimeValidationCheck[];
};

export type CapabilityWorkflow =
  | "tts"
  | "voice-clone"
  | "voice-conversion"
  | "sfx"
  | "ambience"
  | "instrument-sample"
  | "loop"
  | "song"
  | "stem-separation"
  | "video-to-audio"
  | "edit"
  | "composition-render";

export type CapabilityWorkflowSummary = {
  workflow: CapabilityWorkflow;
  defaultProviderId: string;
  defaultModelId: string;
};

export type ProviderCatalogOverview = {
  schemaVersion: number;
  providerCount: number;
  modelCount: number;
  capabilityCount: number;
  workflows: CapabilityWorkflowSummary[];
};
