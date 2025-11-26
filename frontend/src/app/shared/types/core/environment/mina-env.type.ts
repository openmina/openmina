/**
 * Main environment configuration interface for the Mina Rust frontend.
 * This interface defines all possible configuration options that can be set
 * per environment (development, production, local, etc.).
 *
 * To configure a frontend instance, modify the appropriate environment file:
 * - Development: src/environments/environment.ts
 * - Production: src/environments/environment.prod.ts
 * - Local: src/environments/environment.local.ts
 * - WebNode: src/environments/environment.webnode-local.ts
 * - Producer: src/environments/environment.producer.ts
 * - Fuzzing: src/environments/environment.fuzzing.ts
 *
 * @see {@link https://github.com/o1-labs/mina-rust/tree/develop/frontend/src/environments}
 */
export interface MinaEnv {
  /** Whether this is a production build */
  production: boolean;

  /** Array of Mina node configurations to connect to */
  configs: MinaNode[];

  /** Human-readable identifier for this environment (e.g., "Dev FE") */
  identifier?: string;

  /** Hide the top toolbar in the UI */
  hideToolbar?: boolean;

  /** Hide node statistics display */
  hideNodeStats?: boolean;

  /** Allow adding custom nodes through the UI */
  canAddNodes?: boolean;

  /** Show the WebNode landing page */
  showWebNodeLandingPage?: boolean;

  /** Hide the peers pill in the status bar */
  hidePeersPill?: boolean;

  /** Hide the transactions pill in the status bar */
  hideTxPill?: boolean;

  /** Sentry error tracking configuration */
  sentry?: {
    /** Sentry Data Source Name for error reporting */
    dsn: string;
    /** Origins to trace for performance monitoring */
    tracingOrigins: string[];
  };

  /** Global configuration shared across all nodes */
  globalConfig?: {
    /** Feature flags configuration defining which UI sections are available */
    features?: FeaturesConfig;
    /** GraphQL endpoint URL for blockchain queries */
    graphQL?: string;
  };
}

/**
 * Configuration for a single Mina node connection.
 * Each node can have different endpoints and feature sets enabled.
 */
export interface MinaNode {
  /** Display name for this node (e.g., "Local rust node", "Producer-0") */
  name: string;

  /** Base URL for the node's API endpoint (e.g., "http://127.0.0.1:3000") */
  url?: string;

  /** URL for memory profiling endpoint */
  memoryProfiler?: string;

  /** URL for debugger interface */
  debugger?: string;

  /** Node-specific feature configuration (overrides globalConfig.features) */
  features?: FeaturesConfig;

  /** Whether this is a user-added custom node */
  isCustom?: boolean;

  /** Whether this node runs in the browser as a WebNode */
  isWebNode?: boolean;
}

/**
 * Feature flags configuration that controls which UI sections and sub-features
 * are available. Each feature can have multiple sub-features enabled.
 *
 * @example
 * ```typescript
 * features: {
 *   'dashboard': [],                    // Dashboard tab (no sub-features)
 *   'nodes': ['overview', 'live'],      // Nodes tab with overview and live sub-tabs
 *   'network': ['messages', 'blocks']   // Network tab with specific sub-sections
 * }
 * ```
 */
export type FeaturesConfig = Partial<{
  /** Main dashboard view */
  dashboard: string[];

  /** Node management and monitoring features */
  nodes: string[];

  /** State machine and action tracking */
  state: string[];

  /** Network topology, messages, and peer connections */
  network: string[];

  /** SNARK proof generation and verification */
  snarks: string[];

  /** System resource monitoring (memory, CPU) */
  resources: string[];

  /** Block production and slot tracking */
  'block-production': string[];

  /** Transaction mempool monitoring */
  mempool: string[];

  /** Performance benchmarking tools */
  benchmarks: string[];

  /** Fuzzing and testing tools */
  fuzzing: string[];
}>;

/**
 * Union type of all available feature names.
 * Used for type safety when referencing features in code.
 */
export type FeatureType = keyof FeaturesConfig;
