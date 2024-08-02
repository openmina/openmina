export interface MinaEnv {
  production: boolean;
  configs: MinaNode[];
  identifier?: string;
  globalConfig?: {
    features?: FeaturesConfig;
    canAddNodes?: boolean;
  };
}

export interface MinaNode {
  name: string;
  url: string;
  memoryProfiler?: string;
  debugger?: string;
  features?: FeaturesConfig;
  minaExplorerNetwork?: 'mainnet' | 'devnet' | 'berkeley';
  isCustom?: boolean;
}

export type FeaturesConfig = Partial<{
  'dashboard': string[];
  'nodes': string[];
  'state': string[];
  'network': string[];
  'snarks': string[];
  'resources': string[];
  'testing-tool': string[];
  'block-production': string[];
  'mempool': string[];
  'benchmarks': string[];
}>;

export type FeatureType = keyof FeaturesConfig;
