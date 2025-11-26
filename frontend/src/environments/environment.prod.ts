import { MinaEnv } from '@shared/types/core/environment/mina-env.type';

// This environment provides build-time configuration for production
// The actual runtime configuration gets loaded from /environments/env.js
export const environment: Readonly<MinaEnv> = {
  production: true,
  identifier: 'Production',
  canAddNodes: false,
  hideNodeStats: false,
  hideToolbar: false,
  showWebNodeLandingPage: false,
  hidePeersPill: false,
  hideTxPill: false,
  globalConfig: {
    features: {
      dashboard: [],
      nodes: ['overview', 'live', 'bootstrap'],
      state: ['actions'],
      snarks: ['scan-state', 'work-pool'],
      mempool: [],
      'block-production': ['won-slots'],
    },
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
  ],
  sentry: undefined,
};
