import { MinaEnv } from '@shared/types/core/environment/mina-env.type';

export const environment: Readonly<MinaEnv> = {
  production: true,
  identifier: 'Running In Docker',
  globalConfig: {
    features: {
      dashboard: [],
      nodes: ['overview', 'live', 'bootstrap'],
      'block-production': ['won-slots'],
      state: ['actions'],
      snarks: ['scan-state'],
      benchmarks: ['wallets'],
    },
    canAddNodes: true,
  },
  configs: [
    {
      name: 'Compose rust node',
      url: '/openmina-node',
    },
  ],
};

