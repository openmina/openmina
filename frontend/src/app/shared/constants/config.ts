import { FeaturesConfig, FeatureType, MinaEnv, MinaNode } from '@shared/types/core/environment/mina-env.type';
import { environment } from '@environment/environment';
import { getOrigin, hasValue, isBrowser, safelyExecuteInBrowser } from '@openmina/shared';

console.log(environment)

export const CONFIG: Readonly<MinaEnv> = {
  ...environment,
  globalConfig: {
    ...environment.globalConfig,
    graphQL: getURL(environment.globalConfig?.graphQL),
  },
  configs: !isBrowser() ? [] : environment.configs.map((config) => ({
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
  return Object.keys(getFeaturesConfig(config)) as FeatureType[];
}

export function getFirstFeature(config: MinaNode = CONFIG.configs[0]): FeatureType {
  if (!isBrowser()) {
    return '' as FeatureType;
  }
  if (Array.isArray(config?.features)) {
    return config.features[0];
  }

  return Object.keys(getFeaturesConfig(config))[0] as FeatureType;
}

export function isFeatureEnabled(config: MinaNode, feature: FeatureType): boolean {
  if (Array.isArray(config.features)) {
    return hasValue(config.features[0]);
  }

  return hasValue(getFeaturesConfig(config)[feature]);
}

export function getFeaturesConfig(config: MinaNode): FeaturesConfig {
  if (CONFIG.configs.length === 0) {
    return CONFIG.globalConfig?.features;
  }
  return config?.features || CONFIG.globalConfig?.features;
}

export function isSubFeatureEnabled(config: MinaNode, feature: FeatureType, subFeature: string): boolean {
  const features = getFeaturesConfig(config);
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
