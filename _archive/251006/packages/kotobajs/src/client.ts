// MERKLE: e8a9b0c1 (Kotoba API Client)

import axios, { AxiosInstance } from 'axios';

export interface KotobaClientOptions {
  /** The URL of the kotoba-server's GraphQL endpoint. */
  serverUrl: string;
  /** Optional authentication token. */
  authToken?: string;
}

export class KotobaClient {
  private axiosInstance: AxiosInstance;

  constructor(options: KotobaClientOptions) {
    if (!options.serverUrl) {
      throw new Error('serverUrl is required to initialize the KotobaClient.');
    }

    this.axiosInstance = axios.create({
      baseURL: options.serverUrl,
      headers: {
        'Content-Type': 'application/ld+json',
        'Accept': 'application/ld+json',
        ...(options.authToken && { 'Authorization': `Bearer ${options.authToken}` }),
      },
    });
  }

  /**
   * Sends a GraphQL query to the kotoba-server.
   * @param query The GraphQL query string.
   * @param variables Optional variables for the query.
   * @returns The data returned from the server.
   */
  public async query<T = any>(query: string, variables?: Record<string, any>): Promise<T> {
    try {
      console.log(`[KotobaClient] Sending query:`, { query, variables });
      // Convert to JSON-LD format
      const jsonldBody = {
        '@context': 'https://github.com/com-junkawasaki/kotoba/blob/22712d997449ec6229800adf42698936aa24b386/schemas/kotoba-context.jsonld',
        '@type': 'kotoba:GraphQLQuery',
        query,
        variables: variables || {},
      };
      const response = await this.axiosInstance.post('', jsonldBody);

      if (response.data.errors) {
        throw new Error(`GraphQL Error: ${JSON.stringify(response.data.errors)}`);
      }

      // Extract data from JSON-LD response
      const responseData = response.data.data || response.data;
      if (responseData['@context']) {
        delete responseData['@context'];
      }
      if (responseData['@id']) {
        delete responseData['@id'];
      }
      if (responseData['@type']) {
        delete responseData['@type'];
      }
      return responseData as T;
    } catch (error) {
      console.error('[KotobaClient] Query failed:', error);
      // Re-throw the error to be handled by the caller
      throw error;
    }
  }
}
