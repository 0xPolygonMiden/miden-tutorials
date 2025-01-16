// src/lib/createClient.ts
import { WebClient } from '@demox-labs/miden-sdk';

const nodeEndpoint = 'http://localhost:57291';
// const delegatedProver = "http://localhost:8080";

class ClientSingleton {
  private static instance: WebClient | null = null;
  private constructor() {}

  public static async getInstance(): Promise<WebClient> {
    if (this.instance === null) {
      this.instance = new WebClient();
      await this.instance.create_client(nodeEndpoint);

      console.log('WebClient initialized.');
    }

    let accounts = this.instance.get_accounts();
    console.log('accounts', (await accounts).length);

    return this.instance;
  }
}

export default ClientSingleton;
