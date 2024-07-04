import { MinaEnv } from '@shared/types/core/environment/mina-env.type';

export const environment: Readonly<MinaEnv> = {
  production: false,
  identifier: 'Local FE',
  globalConfig: {
    features: {
      dashboard: [],
      nodes: ['overview', 'live', 'bootstrap'],
      state: ['actions'],
      snarks: ['scan-state'],
      'block-production': ['won-slots'],
    },
    canAddNodes: true,
  },
  configs: [
    {
      name: 'Local rust node',
      url: 'http://127.0.0.1:3000',
    },
  ],
};

