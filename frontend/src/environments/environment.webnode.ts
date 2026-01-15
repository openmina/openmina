import { MinaEnv } from '@shared/types/core/environment/mina-env.type';

/** Substituted on launch of docker container from MINA_WEBNODE_SEED_URLS by `ng build` */
declare const MINA_WEBNODE_SEED_URLS: string;

/** Default list of seed URLs if MINA_WEBNODE_SEED_URLS isn't specified */
const defaultWebNodeSeedUrls: Readonly<string[]> = [
  'https://bootnodes.minaprotocol.com/networks/devnet-webrtc.txt',
  '/webnode/pkg/devnet-webrtc.txt',
];

/** Substituted on launch of docker container from MINA_WEBNODE_BOOTNODES by `ng build` */
declare const MINA_WEBNODE_BOOTNODES: string;

/** Default list of bootnodes if MINA_WEBNODE_BOOTNODES is unspecified */
const defaultWebNodeBootNodes: Readonly<string[]> = [
  // example:
  // "/2az589QvS6i3EJiVKUfVHCkyqf4khGy9PjQF7nSQuveF27wp7xX/https/mina-rust-seed-1.gcp.o1test.net/443",
];

function commaSeparatedEnv(
  envVal: string,
  fallback: Readonly<string[]>,
): Readonly<string[]> {
  const envTrimmed = envVal.trim();
  if (envTrimmed.length !== 0) {
    const envValue = envTrimmed.split(',').map(s => s.trim());
    return envValue;
  }

  return fallback;
}

export const environment: Readonly<MinaEnv> = {
  production: true,
  identifier: 'Web Node FE',
  canAddNodes: true,
  showWebNodeLandingPage: false,
  hidePeersPill: true,
  hideTxPill: true,
  globalConfig: {
    features: {
      dashboard: [],
      state: ['actions'],
      'block-production': ['won-slots'],
      mempool: [],
      benchmarks: ['wallets'],
    },
    webNodeSeedUrls: commaSeparatedEnv(
      MINA_WEBNODE_SEED_URLS ?? '',
      defaultWebNodeSeedUrls,
    ),
    webNodeBootNodes: commaSeparatedEnv(
      MINA_WEBNODE_BOOTNODES ?? '',
      defaultWebNodeBootNodes,
    ),
  },
  configs: [
    {
      name: 'Web Node',
      isWebNode: true,
    },
  ],
};
