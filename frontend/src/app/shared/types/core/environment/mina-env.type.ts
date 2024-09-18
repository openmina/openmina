import { MinaNetwork } from '@shared/types/core/mina/mina.type';

export interface MinaEnv {
  production: boolean;
  configs: MinaNode[];
  identifier?: string;
  hideToolbar?: boolean;
  hideNodeStats?: boolean;
  globalConfig?: {
    features?: FeaturesConfig;
    canAddNodes?: boolean;
    minaExplorerNetwork?: MinaNetwork;
  };
}

export interface MinaNode {
  name: string;
  url?: string;
  memoryProfiler?: string;
  debugger?: string;
  features?: FeaturesConfig;
  minaExplorerNetwork?: MinaNetwork;
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
  'testing-tool': string[];
  'block-production': string[];
  'mempool': string[];
  'benchmarks': string[];
  'zk': string[];
}>;

export type FeatureType = keyof FeaturesConfig;
