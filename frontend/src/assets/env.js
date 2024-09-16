export default {
  production: false,
  globalConfig: {
    features: {
      'dashboard': [],
      'nodes': ['overview', 'live', 'bootstrap'],
      'state': ['actions'],
      'network': ['messages', 'connections', 'blocks', 'topology', 'node-dht', 'graph-overview', 'bootstrap-stats'],
      'snarks': ['scan-state', 'work-pool'],
      'testing-tool': ['scenarios'],
      'resources': ['memory'],
      'block-production': ['overview', 'won-slots'],
      'mempool': [],
      'benchmarks': ['wallets'],
    },
    canAddNodes: false,
    minaExplorerNetwork: 'devnet',
  },
  configs: [
    {
      name: 'staging-devnet-bp-0',
      url: 'https://staging-devnet-openmina-bp-0.minaprotocol.network',
    },
    {
      name: 'staging-devnet-bp-1',
      url: 'https://staging-devnet-openmina-bp-1.minaprotocol.network',
    },
    {
      name: 'staging-devnet-bp-2',
      url: 'https://staging-devnet-openmina-bp-2.minaprotocol.network',
    },
    {
      name: 'staging-devnet-bp-3',
      url: 'https://staging-devnet-openmina-bp-3.minaprotocol.network',
    },
  ],
};
