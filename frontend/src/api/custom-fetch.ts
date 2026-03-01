/**
 * Credentials-aware fetch wrapper for the generated orval API client.
 *
 * Usage with generated hooks:
 *   useListStories({ query: {} }, undefined, { fetch: credentialsFetch })
 *
 * Or configure globally via TanStack Query's default options.
 */
export const credentialsFetch = (url: string, options?: RequestInit): Promise<Response> =>
  fetch(url, { ...options, credentials: "include" });
