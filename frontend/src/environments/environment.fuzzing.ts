import { MinaEnv } from '@shared/types/core/environment/mina-env.type';

export const environment: Readonly<MinaEnv> = {
  production: true,
  identifier: 'Fuzzing FE',
  canAddNodes: false,
  hideToolbar: true,
  hideNodeStats: true,
  showWebNodeLandingPage: false,
  globalConfig: {
    features: {
      fuzzing: [],
    },
  },
  configs: [],
};

