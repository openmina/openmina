import { MinaEnv } from '@shared/types/core/environment/mina-env.type';

export const environment: Readonly<MinaEnv> = {
  production: false,
  identifier: 'Dev FE',
  canAddNodes: true,
  showWebNodeLandingPage: false,
  hidePeersPill: true,
  hideTxPill: true,
  globalConfig: {
    features: {
      dashboard: [],
      nodes: ['overview', 'live', 'bootstrap'],
      state: ['actions'],
      network: [
        'messages',
        'connections',
        'blocks',
        'topology',
        'node-dht',
        'graph-overview',
        'bootstrap-stats',
      ],
      snarks: ['scan-state', 'work-pool'],
      resources: ['memory'],
      'block-production': ['won-slots'],
      mempool: [],
      benchmarks: ['wallets'],
      fuzzing: [],
    },
    // graphQL: 'https://api.minascan.io/node/devnet/v1/graphql',
    // graphQL: 'http://65.109.105.40:5000/graphql',
  },
  configs: [
    // {
    //   name: 'http://localhost:8070',
    //   url: 'http://127.0.0.1:8070/openmina-node',
    // },
    // {
    //   name: 'Producer-0',
    //   url: 'https://staging-devnet-openmina-bp-0-dashboard.minaprotocol.network',
    // },
    // {
    //   name: 'Producer-1',
    //   url: 'https://staging-devnet-openmina-bp-1-dashboard.minaprotocol.network',
    // },
    // {
    //   name: 'Producer-2',
    //   url: 'https://staging-devnet-openmina-bp-2-dashboard.minaprotocol.network',
    // },
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
    // {
    //   name: 'Web Node 1',
    //   isWebNode: true,
    // },
    // {
    //   name: 'http://65.109.105.40:3000',
    //   url: 'http://65.109.105.40:3000',
    // },
    // {
    //   name: 'Local rust node',
    //   url: 'http://127.0.0.1:3000',
    // },
    // {
    //   name: 'feat/frontend-api-peers',
    //   url: 'http://176.9.147.28:3000',
    //   features: {
    //     dashboard: [],
    //     nodes: ['overview', 'live', 'bootstrap'],
    //     state: ['actions'],
    //     snarks: ['scan-state', /*'work-pool'*/],
    //     resources: ['memory'],
    //   },
    // },
    // {
    //   name: 'Docker 11010',
    //   url: 'http://localhost:11010',
    // },
    // {
    //   name: 'Docker 11012',
    //   url: 'http://localhost:11012',
    // },
    // {
    //   name: 'Docker 11014',
    //   url: 'http://localhost:11014',
    // },
    // {
    //   name: 'Producer',
    //   url: 'http://65.109.105.40:3000',
    //   memoryProfiler: 'http://1.k8.openmina.com:31164',
    // },
    // {
    //   name: 'http://65.109.110.75:3000',
    //   url: 'http://65.109.110.75:3000',
    //   memoryProfiler: 'http://1.k8.openmina.com:31164',
    // },
    // {
    //   name: 'http://65.109.110.75:11010',
    //   url: 'http://65.109.110.75:11010',
    //   memoryProfiler: 'http://1.k8.openmina.com:31164',
    // },
    // {
    //   name: 'http://65.109.110.75:11012',
    //   url: 'http://65.109.110.75:11012',
    //   memoryProfiler: 'http://1.k8.openmina.com:31164',
    // },
    // {
    //   name: 'http://65.109.110.75:11014',
    //   url: 'http://65.109.110.75:11014',
    //   memoryProfiler: 'http://1.k8.openmina.com:31164',
    // },
    // {
    //   name: 'Node with mem profiler',
    //   url: 'http://1.k8.openmina.com:30252',
    //   memoryProfiler: 'http://1.k8.openmina.com:31164',
    //   features: {
    //     dashboard: [],
    //     nodes: ['overview', 'live', 'bootstrap'],
    //     state: ['actions'],
    //     snarks: ['scan-state'],
    //     resources: ['memory'],
    //     network: ['topology', 'node-dht', 'graph-overview'],
    //   },
    // },
    // {
    //   name: 'Node with debugger',
    //   url: 'http://1.k8.openmina.com:31688',
    //   debugger: 'http://1.k8.openmina.com:31072',
    //   features: {
    //     nodes: ['overview', 'live', 'bootstrap'],
    //     state: ['actions'],
    //     network: ['messages', 'connections', 'blocks'],
    //     snarks: ['scan-state'],
    //     resources: ['memory'],
    //   },
    // },
    // {
    //   name: 'Snarker 1',
    //   url: 'http://webrtc2.webnode.openmina.com:10000',
    // },
    // {
    //   name: 'Snarker 2',
    //   url: 'http://webrtc3.webnode.openmina.com:10000',
    // },
    // {
    //   name: 'Snarker 3',
    //   url: 'http://webrtc4.webnode.openmina.com:10000',
    // },
  ],
};
