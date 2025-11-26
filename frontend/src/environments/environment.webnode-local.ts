import { MinaEnv } from '@shared/types/core/environment/mina-env.type';

export const environment: Readonly<MinaEnv> = {
  production: true,
  identifier: 'Web Node FE',
  canAddNodes: true,
  showWebNodeLandingPage: false,
  hidePeersPill: true,
  hideTxPill: true,
  globalConfig: {
    features: {
      dashboard: [],
      state: ['actions'],
      'block-production': ['won-slots'],
      mempool: [],
      benchmarks: ['wallets'],
    },
  },
  configs: [
    {
      name: 'Web Node',
      isWebNode: true,
    },
  ],
};
