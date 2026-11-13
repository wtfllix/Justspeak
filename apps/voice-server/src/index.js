import { buildApp, readConfig } from './app.js';
const app = await buildApp(readConfig(), true);
await app.listen({ host: process.env.HOST || '127.0.0.1', port: Number(process.env.PORT || 3001) });
for (const signal of ['SIGINT', 'SIGTERM']) process.on(signal, async () => { await app.close(); process.exit(0); });
