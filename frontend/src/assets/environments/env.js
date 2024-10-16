/**
 * This file is imported in the environment.prod.ts file in frontend/src/index.html
 * The content of this file is replaced from a docker command (frontend/docker/startup.sh)
 * => 1 Docker image = multiple environments
 */
export default {
  production: true,
  configs: [],
};
