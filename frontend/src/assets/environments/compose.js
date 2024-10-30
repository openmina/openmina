/**
 * This configuration is used for lunching devnet rust nodes inside a docker container.
 * https://github.com/openmina/openmina?tab=readme-ov-file#how-to-launch-the-node-with-docker-compose
 */

export default {
  production: true,
  identifier: 'Running in Docker',
  canAddNodes: true,
  globalConfig: {
    features: {
      dashboard: [],
      nodes: ['overview', 'live', 'bootstrap'],
      state: ['actions'],
      snarks: ['scan-state'],
    },
    graphQL: '/openmina-node/graphql',
  },
  configs: [
    {
      name: 'Compose rust node',
      url: '/openmina-node',
    },
  ],
};
