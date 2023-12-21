import { MinaEnv } from '@shared/types/core/environment/mina-env.type';

export const environment: Readonly<MinaEnv> = {
  production: false,
  identifier: 'local',
  globalConfig: {
    features: {
      dashboard: [],
      nodes: ['overview', 'live', 'bootstrap'],
      state: ['actions'],
      snarks: ['scan-state'],
    },
  },
  configs: [
    {
      name: 'Local rust node',
      url: 'http://0.0.0.0:3000',
    }
  ],
};

