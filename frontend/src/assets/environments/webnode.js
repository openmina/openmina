export default {
  production: true,
  globalConfig: {
    features: {
      'dashboard': [],
      'block-production': ['won-slots'],
      'nodes': ['overview', 'live', 'bootstrap'],
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
