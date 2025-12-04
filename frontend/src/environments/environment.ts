/**
 * Development environment configuration
 */

import { MinaEnv } from '@shared/types/core/environment/mina-env.type';

export const environment: Readonly<MinaEnv> = {
  production: false,
  identifier: 'Dev FE',
  canAddNodes: true,
  showWebNodeLandingPage: false,
  hidePeersPill: true,
  hideTxPill: true,
  globalConfig: {
    features: {
      dashboard: [],
      nodes: ['overview', 'live', 'bootstrap'],
      state: ['actions'],
      network: [
        'messages',
        'connections',
        'blocks',
        'topology',
        'node-dht',
        'graph-overview',
        'bootstrap-stats',
      ],
      snarks: ['scan-state', 'work-pool'],
      resources: ['memory'],
      'block-production': ['won-slots'],
      mempool: [],
      benchmarks: ['wallets'],
      fuzzing: [],
    },
    graphQL: 'https://devnet-plain-1.gcp.o1test.net/graphql',
  },
  configs: [
    {
      name: 'o1Labs Plain Node 1',
      url: 'https://mina-rust-plain-1.gcp.o1test.net/',
    },
    {
      name: 'o1Labs Plain Node 2',
      url: 'https://mina-rust-plain-2.gcp.o1test.net/',
    },
    {
      name: 'o1Labs Plain Node 3',
      url: 'https://mina-rust-plain-3.gcp.o1test.net/',
    },
    {
      name: 'o1Labs BP Node 1',
      url: 'https://mina-rust-bp-1.gcp.o1test.net/',
    },
    {
      name: 'o1Labs BP Node 2',
      url: 'https://mina-rust-bp-2.gcp.o1test.net/',
    },
    {
      name: 'o1Labs Seed Node 1',
      url: 'https://mina-rust-seed-1.gcp.o1test.net/',
    },
    {
      name: 'o1Labs Seed Node 2',
      url: 'https://mina-rust-seed-2.gcp.o1test.net/',
    },
    {
      name: 'o1Labs Seed Node 3',
      url: 'https://mina-rust-seed-3.gcp.o1test.net/',
    },
    {
      name: 'Local rust node',
      url: 'http://127.0.0.1:3000',
    },
  ],
};
