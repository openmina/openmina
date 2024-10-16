/**
 * This configuration is used for the staging-webnode environment.
 */

export default {
  production: true,
  globalConfig: {
    features: {
      'dashboard': [],
      'block-production': ['won-slots'],
      'mempool': [],
      'benchmarks': ['wallets'],
    },
    canAddNodes: false,
  },
  configs: [
    {
      name: 'Web Node',
      isWebNode: true,
    },
  ],
};
