import type {SidebarsConfig} from '@docusaurus/plugin-content-docs';

// This runs in Node.js - Don't use client-side code here (browser APIs, JSX...)

/**
 * Creating a sidebar enables you to:
 - create an ordered group of docs
 - render a sidebar for each doc of that group
 - provide next/previous navigation

 The sidebars can be generated from the filesystem, or explicitly defined here.

 Create as many sidebars as you want.
 */
const sidebars: SidebarsConfig = {
  // Sidebar for node operators - focus on operational guides
  nodeRunnersSidebar: [
    {
      type: 'category',
      label: 'Getting Started',
      items: [
        'node-operators/getting-started',
        'node-operators/join-devnet',
      ],
    },
    {
      type: 'category',
      label: 'Installation',
      items: [
        'node-operators/docker-usage',
        'node-operators/building-from-source',
      ],
    },
    {
      type: 'category',
      label: 'Node Operations',
      items: [
        'node-operators/block-producer',
        'node-operators/archive-node',
        'node-operators/network-configuration',
        'node-operators/node-management',
      ],
    },
    {
      type: 'category',
      label: 'o1Labs Infrastructure',
      items: [
        'node-operators/infrastructure/seed-nodes',
        'node-operators/infrastructure/plain-nodes',
        'node-operators/infrastructure/archive-nodes',
        'node-operators/infrastructure/block-producers',
        'node-operators/infrastructure/replayer-nodes',
        'node-operators/infrastructure/memory-profiler',
        'node-operators/infrastructure/network-debugger',
        'node-operators/infrastructure/frontend',
      ],
    },
    {
      type: 'category',
      label: 'Advanced Topics',
      items: [
        'node-operators/alpha-testing',
        'node-operators/webnode/local-webnode',
        'node-operators/testing/overview',
      ],
    },
  ],

  // Sidebar for developers - focus on codebase and development
  developersSidebar: [
    {
      type: 'category',
      label: 'Introduction',
      items: [
        'developers/getting-started',
        'developers/updating-ocaml-node',
      ],
    },
    {
      type: 'category',
      label: 'Architecture',
      items: [
        'developers/why-rust',
        'developers/architecture',
        'developers/circuits',
        'developers/ledger-crate',
      ],
    },
    {
      type: 'category',
      label: 'APIs and Data',
      items: [
        'developers/graphql-api',
        'developers/archive-database-queries',
      ],
    },
    {
      type: 'category',
      label: 'Wallet operations',
      items: [
        'developers/wallet/index',
        'developers/wallet/address',
        'developers/wallet/balance',
        'developers/wallet/generate',
        'developers/wallet/send',
        'developers/wallet/status',
      ],
    },
    {
      type: 'category',
      label: 'Frontend',
      items: [
        'developers/frontend/index',
        'developers/frontend/node-dashboard',
        'developers/frontend/webnode',
        'developers/frontend/environment-configuration',
        'developers/frontend/api-endpoints',
      ],
    },
    {
      type: 'category',
      label: 'Docker',
      items: [
        'developers/docker-images',
      ],
    },
    {
      type: 'category',
      label: 'P2P Networking',
      items: [
        'developers/p2p-networking',
        'developers/webrtc',
        'developers/libp2p',
      ],
    },
    {
      type: 'category',
      label: 'Testing',
      items: [
        'developers/testing/testing-framework',
        'developers/testing/unit-tests',
        'developers/testing/scenario-tests',
        'developers/testing/ledger-tests',
        'developers/testing/p2p-tests',
        'developers/testing/network-connectivity',
        'developers/testing/ocaml-node-tests',
        'developers/testing/replayer',
      ],
    },
    {
      type: 'category',
      label: 'Performance',
      items: [
        'developers/benchmarks',
      ],
    },
    {
      type: 'category',
      label: 'Documentation',
      items: [
        'developers/referencing-code-in-documentation',
        'developers/ocaml-reference-tracking',
      ],
    },
    {
      type: 'category',
      label: 'Future Work',
      items: [
        'developers/future-work',
        'developers/mainnet-readiness',
        'developers/p2p-evolution',
        'developers/persistence-design',
      ],
    },
  ],

  // Sidebar for researchers - focus on protocol and cryptography
  researchersSidebar: [
    {
      type: 'category',
      label: 'Protocol',
      items: [
        'researchers/protocol',
        'researchers/scan-state',
      ],
    },
    {
      type: 'category',
      label: 'Cryptography',
      items: [
        'researchers/snark-work',
      ],
    },
  ],

  // Appendix sidebar - general reference material
  appendixSidebar: [
    {
      type: 'category',
      label: 'Installation References',
      items: [
        'appendix/docker-installation',
      ],
    },
    {
      type: 'category',
      label: 'Development Processes',
      items: [
        'appendix/release-process',
      ],
    },
  ],
};

export default sidebars;
