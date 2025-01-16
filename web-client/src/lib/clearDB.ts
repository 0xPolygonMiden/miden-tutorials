export async function clearDatabase(): Promise<void> {
  try {
    const dbs = await indexedDB.databases();
    for (const db of dbs) {
      if (db.name) {
        const dbName = db.name; // Assign to a constant after checking it's not undefined
        await new Promise<void>((resolve, reject) => {
          const request = indexedDB.deleteDatabase(dbName);
          request.onsuccess = () => resolve();
          request.onerror = () => reject(request.error);
          request.onblocked = () => console.warn('Delete blocked for', dbName);
        });
        console.log(`Deleted database: ${dbName}`);
      }
    }
    console.log('All databases deleted.');
  } catch (error) {
    console.error('Error clearing databases:', error);
    throw error;
  }
}
