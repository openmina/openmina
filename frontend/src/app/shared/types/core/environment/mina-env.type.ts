export interface MinaEnv {
  production: boolean;
  configs: MinaNode[];
  identifier?: string;
  hideToolbar?: boolean;
  hideNodeStats?: boolean;
  canAddNodes?: boolean;
  showWebNodeLandingPage?: boolean;
  showLeaderboard?: boolean;
  hidePeersPill?: boolean;
  hideTxPill?: boolean;
  sentry?: {
    dsn: string;
    tracingOrigins: string[];
  };
  globalConfig?: {
    features?: FeaturesConfig;
    graphQL?: string;
    firebase?: any;
    heartbeats?: boolean;
  };
}

export interface MinaNode {
  name: string;
  url?: string;
  memoryProfiler?: string;
  debugger?: string;
  features?: FeaturesConfig;
  isCustom?: boolean;
  isWebNode?: boolean;
}

export type FeaturesConfig = Partial<{
  'dashboard': string[];
  'nodes': string[];
  'state': string[];
  'network': string[];
  'snarks': string[];
  'resources': string[];
  'block-production': string[];
  'mempool': string[];
  'benchmarks': string[];
  'fuzzing': string[];
}>;

export type FeatureType = keyof FeaturesConfig;
