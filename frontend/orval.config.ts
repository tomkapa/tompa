// orval.config.ts — TypeScript version of the config for editor tooling.
// The actual config used at runtime is orval.config.json (avoids esbuild native binary requirement).
import { defineConfig } from "orval";

export default defineConfig({
  pipeline: {
    input: "./src/api/openapi.json",
    output: {
      target: "./src/api/generated",
      client: "react-query",
      httpClient: "fetch",
      mode: "tags-split",
    },
  },
});
