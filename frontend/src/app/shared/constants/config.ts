import {
  FeaturesConfig,
  FeatureType,
  MinaEnv,
  MinaNode,
} from '@shared/types/core/environment/mina-env.type';
import { environment } from '@environment/environment';
import {
  getOrigin,
  hasValue,
  isBrowser,
  safelyExecuteInBrowser,
} from '@mina-rust/shared';

// NOTE: When modifying CONFIG or related functions, update the documentation at:
// website/docs/developers/frontend/environment-configuration.mdx
export const CONFIG: Readonly<MinaEnv> = {
  ...environment,
  globalConfig: {
    ...environment.globalConfig,
    graphQL: getURL(environment.globalConfig?.graphQL),
  },
  configs: !isBrowser()
    ? []
    : environment.configs.map(config => ({
        ...config,
        url: getURL(config.url),
        memoryProfiler: getURL(config.memoryProfiler),
        debugger: getURL(config.debugger),
      })),
};

safelyExecuteInBrowser(() => {
  (window as any).config = CONFIG;
});

export function getAvailableFeatures(config: MinaNode): FeatureType[] {
  const featuresConfig = getFeaturesConfig(config);
  if (!featuresConfig) {
    console.warn('getAvailableFeatures: No features configuration found', {
      config,
      CONFIG,
    });
    return [];
  }
  return Object.keys(featuresConfig) as FeatureType[];
}

export function getFirstFeature(
  config: MinaNode = CONFIG.configs[0],
): FeatureType {
  if (!isBrowser()) {
    return '' as FeatureType;
  }

  console.log('getFirstFeature called with config:', {
    config,
    'CONFIG.configs': CONFIG.configs,
    globalConfig: CONFIG.globalConfig,
  });

  if (Array.isArray(config?.features)) {
    return config.features[0];
  }

  const featuresConfig = getFeaturesConfig(config);
  if (!featuresConfig) {
    console.warn('getFirstFeature: No features configuration found', {
      config,
      CONFIG,
    });
    return '' as FeatureType;
  }
  return Object.keys(featuresConfig)[0] as FeatureType;
}

export function isFeatureEnabled(
  config: MinaNode,
  feature: FeatureType,
): boolean {
  if (Array.isArray(config?.features)) {
    return hasValue(config.features[0]);
  }

  const featuresConfig = getFeaturesConfig(config);
  if (!featuresConfig) {
    console.warn('isFeatureEnabled: No features configuration found', {
      config,
      feature,
    });
    return false;
  }
  return hasValue(featuresConfig[feature]);
}

export function getFeaturesConfig(
  config: MinaNode,
): FeaturesConfig | undefined {
  if (CONFIG.configs.length === 0) {
    return CONFIG.globalConfig?.features;
  }
  return config?.features || CONFIG.globalConfig?.features;
}

export function isSubFeatureEnabled(
  config: MinaNode,
  feature: FeatureType,
  subFeature: string,
): boolean {
  const features = getFeaturesConfig(config);
  if (!features) {
    console.warn('isSubFeatureEnabled: No features configuration found', {
      config,
      feature,
      subFeature,
    });
    return false;
  }
  return hasValue(features[feature]) && features[feature].includes(subFeature);
}

export function getURL(pathOrUrl: string): string {
  if (isBrowser()) {
    if (pathOrUrl) {
      let href = new URL(pathOrUrl, getOrigin()).href;
      if (href.endsWith('/')) {
        href = href.slice(0, -1);
      }
      return href;
    }
    return pathOrUrl;
  }
  return pathOrUrl;
}
