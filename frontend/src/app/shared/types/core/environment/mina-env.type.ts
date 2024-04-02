export interface MinaEnv {
  production: boolean;
  configs: MinaNode[];
  identifier?: string;
  globalConfig?: {
    features?: FeaturesConfig;
  };
}

export interface MinaNode {
  name: string;
  url: string;
  memoryProfiler?: string;
  debugger?: string;
  features?: FeaturesConfig;
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
}>;

export type FeatureType = keyof FeaturesConfig;
