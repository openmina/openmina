import { MinaEnv } from '@shared/types/core/environment/mina-env.type';

export const environment: Readonly<MinaEnv> = {
  production: true,
  globalConfig: {
    features: {
      'block-production': ['overview'],
    },
  },
  configs: [
    {
      name: 'Producer-0',
      url: 'https://staging-devnet-openmina-bp-0-dashboard.minaprotocol.network',
    },
    {
      name: 'Producer-1',
      url: 'https://staging-devnet-openmina-bp-1-dashboard.minaprotocol.network',
    },
    {
      name: 'Producer-2',
      url: 'https://staging-devnet-openmina-bp-2-dashboard.minaprotocol.network',
    },
  ],
};
