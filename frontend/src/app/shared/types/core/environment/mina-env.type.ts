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
  debugger?: string;
  features?: FeaturesConfig;
}

export type FeaturesConfig = {
  [key in FeatureType]?: string[];
};

export type FeatureType =
  | 'dashboard'
  | 'nodes'
  | 'network'
  | 'resources'
  | 'state'
  | 'snarks'
  | 'testing-tool'
  ;
