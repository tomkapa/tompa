/**
 * Credentials-aware fetch wrapper for the generated orval API client.
 *
 * Usage with generated hooks:
 *   useListStories({ query: {} }, undefined, { fetch: credentialsFetch })
 *
 * Or configure globally via TanStack Query's default options.
 */
export const credentialsFetch = async (url: string, options?: RequestInit): Promise<Response> => {
  const response = await fetch(url, { ...options, credentials: "include" });

  if (response.status === 401) {
    window.location.href = '/login';
  }

  return response;
};
