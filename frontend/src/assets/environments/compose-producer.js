/**
 * This configuration is used for lunching devnet rust nodes and user's own node to produce block. All inside a docker container.
 * Todo: github documentation link
 */

export default {
  production: true,
  identifier: 'Running in Docker',
  globalConfig: {
    features: {
      dashboard: [],
      'block-production': ['won-slots'],
      nodes: ['overview', 'live', 'bootstrap'],
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
