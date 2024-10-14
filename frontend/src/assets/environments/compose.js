export default {
  production: true,
  identifier: 'Running in Docker',
  globalConfig: {
    features: {
      dashboard: [],
      nodes: ['overview', 'live', 'bootstrap'],
      'block-production': ['won-slots'],
      state: ['actions'],
      snarks: ['scan-state'],
      benchmarks: ['wallets'],
    },
    canAddNodes: true,
    graphQL: '/openmina-node/graphql',
  },
  configs: [
    {
      name: 'Compose rust node',
      url: '/openmina-node',
    },
  ],
};
