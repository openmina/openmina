import { MinaEnv } from '@shared/types/core/environment/mina-env.type';

export const environment: Readonly<MinaEnv> = {
  production: true,
  identifier: 'Rust based node',
  globalConfig: {
    features: {
      nodes: ['overview', 'live', 'bootstrap'],
      state: ['actions'],
      snarks: ['scan-state', 'work-pool'],
    },
  },
  configs: [
    {
      name: 'Node with mem profiler',
      url: 'http://1.k8.openmina.com:30252',
      memoryProfiler: 'http://1.k8.openmina.com:31164',
      features: {
        nodes: ['overview', 'live', 'bootstrap'],
        state: ['actions'],
        snarks: ['scan-state'],
        resources: ['memory'],
      },
    },
    {
      name: 'Node with debugger',
      url: 'http://1.k8.openmina.com:31688',
      debugger: 'http://1.k8.openmina.com:31072',
      features: {
        nodes: ['overview', 'live', 'bootstrap'],
        state: ['actions'],
        network: ['messages', 'connections', 'blocks'],
        snarks: ['scan-state'],
      },
    },
    {
      name: 'feat/frontend-api-peers',
      url: 'http://176.9.147.28:3000'
    },
    {
      name: 'Snarker 1',
      url: 'http://webrtc2.webnode.openmina.com:10000',
      features: {
        nodes: ['overview', 'live', 'bootstrap'],
        state: ['actions'],
        snarks: ['scan-state', 'work-pool'],
      },
    },
    {
      name: 'Snarker 2',
      url: 'http://webrtc3.webnode.openmina.com:10000',
      features: {
        nodes: ['overview', 'live', 'bootstrap'],
        state: ['actions'],
        snarks: ['scan-state', 'work-pool'],
      },
    },
    {
      name: 'Snarker 3',
      url: 'http://webrtc4.webnode.openmina.com:10000',
      features: {
        nodes: ['overview', 'live', 'bootstrap'],
        state: ['actions'],
        snarks: ['scan-state', 'work-pool'],
      },
    },
  ],
};

