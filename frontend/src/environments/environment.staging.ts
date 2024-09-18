import { MinaEnv } from '@shared/types/core/environment/mina-env.type';

export const environment: Readonly<MinaEnv> = {
  production: true,
  globalConfig: {
    features: {
      'dashboard': [],
      'block-production': ['won-slots'],
      'nodes': ['overview', 'live', 'bootstrap'],
      'mempool': [],
      'state': ['actions'],
      'snarks': ['scan-state', 'work-pool'],
      'benchmarks': ['wallets'],
    },
    canAddNodes: false,
    minaExplorerNetwork: 'devnet',
  },
  configs: [
    {
      name: 'staging-devnet-bp-0',
      url: 'https://staging-devnet-openmina-bp-0.minaprotocol.network',
    },
    {
      name: 'staging-devnet-bp-1',
      url: 'https://staging-devnet-openmina-bp-1.minaprotocol.network',
    },
    {
      name: 'staging-devnet-bp-2',
      url: 'https://staging-devnet-openmina-bp-2.minaprotocol.network',
    },
    {
      name: 'staging-devnet-bp-3',
      url: 'https://staging-devnet-openmina-bp-3.minaprotocol.network',
    },
  ],
};

