export default {
  production: true,
  identifier: 'Running in Docker',
  globalConfig: {
    features: {
      dashboard: [],
      nodes: ['overview', 'live', 'bootstrap'],
      state: ['actions'],
      network: ['node-dht', 'graph-overview', 'bootstrap-stats'],
      snarks: ['scan-state'],
      benchmarks: ['wallets'],
      mempool: [],
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
