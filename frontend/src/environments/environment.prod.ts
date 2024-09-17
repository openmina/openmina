import { MinaEnv } from '@shared/types/core/environment/mina-env.type';

const env = (window as any).env;
export const environment: Readonly<MinaEnv> = {
  production: true,
  configs: env.configs,
  globalConfig: env.globalConfig,
  hideNodeStats: env.hideNodeStats,
  identifier: env.identifier,
  hideToolbar: env.hideToolbar,
};
