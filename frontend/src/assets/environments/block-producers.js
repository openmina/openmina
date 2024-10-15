/**
 * This configuration is used with the demo block producers inside a private network.
 * https://github.com/openmina/openmina/blob/main/docs/producer-demo.md#launch
 */

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
    graphQL: 'http://localhost:11010/graphql',
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
