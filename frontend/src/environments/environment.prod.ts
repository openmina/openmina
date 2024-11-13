import { MinaEnv } from '@shared/types/core/environment/mina-env.type';

const env = typeof window !== 'undefined' ? (window as any).env : {};
export const environment: Readonly<MinaEnv> = {
  production: true,
  configs: env.configs,
  globalConfig: env.globalConfig,
  hideNodeStats: env.hideNodeStats,
  identifier: env.identifier,
  hideToolbar: env.hideToolbar,
  canAddNodes: env.canAddNodes,
  showWebNodeLandingPage: env.showWebNodeLandingPage,
  sentry: env.sentry,
};
