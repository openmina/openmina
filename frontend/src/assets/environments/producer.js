/**
 * This configuration is used for the producer dashboard that are based on archive node.
 * Currently, there is no pipeline that this file is linked to, therefore it is not used anywhere right now.
 * Check frontend/src/environments/environment.producer.ts file.
 */

export default {
  production: true,
  hideNodeStats: true,
  globalConfig: {
    features: {
      'block-production': ['overview'],
    },
  },
  configs: [
    {
      name: 'Producer-0',
      url: 'https://staging-devnet-openmina-bp-0-dashboard.minaprotocol.network',
    },
    {
      name: 'Producer-1',
      url: 'https://staging-devnet-openmina-bp-1-dashboard.minaprotocol.network',
    },
    {
      name: 'Producer-2',
      url: 'https://staging-devnet-openmina-bp-2-dashboard.minaprotocol.network',
    },
  ],
}
