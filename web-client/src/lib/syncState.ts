import ClientSingleton from './createClient';

export async function syncState(): Promise<boolean> {
  try {
    const webClient = await ClientSingleton.getInstance();
    await webClient.sync_state();
    return true;
  } catch (error) {
    console.error('Error syncing state:', error);
    throw error;
  }
}
