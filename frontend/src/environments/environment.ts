import { MinaEnv } from '@shared/types/core/environment/mina-env.type';

export const environment: Readonly<MinaEnv> = {
  production: false,
  identifier: 'local',
  globalConfig: {
    features: {
      dashboard: [],
      nodes: ['overview', 'live', 'bootstrap'],
      state: ['actions'],
      network: ['messages', 'connections', 'blocks'],
      snarks: ['scan-state', /*'work-pool'*/],
      // 'testing-tool': ['scenarios'],
    },
  },
  configs: [
    {
      name: 'feat/frontend-api-peers',
      url: 'http://176.9.147.28:3000',
      features: {
        dashboard: [],
        nodes: ['overview', 'live', 'bootstrap'],
        state: ['actions'],
        snarks: ['scan-state', /*'work-pool'*/],
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
        resources: ['memory'],
      },
    },
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
    // }
  ],
};

