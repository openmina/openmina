export default {
  production: true,
  globalConfig: {
    features: {
      'block-production': ['won-slots'],
      'mempool': [],
      'benchmarks': ['wallets'],
      'snarks': ['scan-state', 'work-pool'],
    },
    canAddNodes: true,
  },
  configs: [
    {
      name: 'Producer 11010',
      url: 'http://localhost:11010',
    },
    {
      name: 'Producer 11012',
      url: 'http://localhost:11012',
    },
    {
      name: 'Producer 11014',
      url: 'http://localhost:11014',
    },
  ],
}
