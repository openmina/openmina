import { MinaEnv } from '@shared/types/core/environment/mina-env.type';

export const environment: Readonly<MinaEnv> = {
  production: false,
  identifier: 'Local FE',
  globalConfig: {
    features: {
      dashboard: [],
      nodes: ['overview', 'live', 'bootstrap'],
      state: ['actions'],
      network: ['node-dht', 'graph-overview', 'bootstrap-stats'],
      snarks: ['scan-state'],
    },
  },
  configs: [
    {
      name: 'Compose rust node',
      url: '/openmina-node',
    },
  ],
};

