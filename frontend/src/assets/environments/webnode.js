/**
 * This configuration is used for the staging-webnode environment.
 */

export default {
  production: true,
  canAddNodes: false,
  showWebNodeLandingPage: true,
  globalConfig: {
    features: {
      'dashboard': [],
      'block-production': ['won-slots'],
      'mempool': [],
      'benchmarks': ['wallets'],
    },
  },
  configs: [
    {
      name: 'Web Node',
      isWebNode: true,
    },
  ],
};
