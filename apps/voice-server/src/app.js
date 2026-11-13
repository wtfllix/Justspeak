import Fastify, { LogController } from 'fastify';
import rateLimit from '@fastify/rate-limit';
import { AccessToken, TrackSource } from 'livekit-server-sdk';
import { randomUUID, timingSafeEqual, createHash } from 'node:crypto';

export function readConfig(env = process.env) {
  for (const key of ['LIVEKIT_API_KEY', 'LIVEKIT_API_SECRET', 'POC_JOIN_CODE', 'LIVEKIT_URL']) {
    if (!env[key]) throw new Error(`Missing server configuration: ${key}`);
  }
  if (env.LIVEKIT_API_SECRET.length < 32 || env.POC_JOIN_CODE.length < 24) throw new Error('Server credentials are too short');
  const url = new URL(env.LIVEKIT_URL);
  if (!['ws:', 'wss:'].includes(url.protocol) || url.username || url.password) throw new Error('Invalid media URL');
  return { apiKey: env.LIVEKIT_API_KEY, secret: env.LIVEKIT_API_SECRET, joinCode: env.POC_JOIN_CODE, url: url.href, room: 'xxspeak-m0-lobby' };
}
export async function buildApp(config, logger = false) {
  const app = Fastify({ logger, bodyLimit: 4096, ajv: { customOptions: { removeAdditional: false } }, logController: new LogController({ disableRequestLogging: true }) });
  await app.register(rateLimit, { max: 20, timeWindow: '1 minute' });
  app.setErrorHandler((error, req, reply) => {
    const status = error.statusCode || 500;
    reply.code(status).send({ error: { code: status === 429 ? 'RATE_LIMITED' : 'REQUEST_FAILED', message: status >= 500 ? '服务暂不可用' : '请求无效或过于频繁' }, requestId: req.id });
  });
  app.get('/healthz', async () => ({ status: 'ok', scope: 'token-api-only' }));
  app.post('/v1/poc/token', {
    schema: {
      body: { type: 'object', additionalProperties: false, required: ['nickname', 'joinCode'], properties: { nickname: { type: 'string', minLength: 1, maxLength: 32, pattern: '\\S' }, joinCode: { type: 'string', minLength: 1, maxLength: 256 } } },
      response: { 200: { type: 'object', additionalProperties: false, properties: { token: { type: 'string' }, livekitUrl: { type: 'string' }, channelId: { type: 'string' }, channelName: { type: 'string' }, identity: { type: 'string' }, expiresAt: { type: 'string' } } } }
    }
  }, async (req, reply) => {
    const digest = value => createHash('sha256').update(value).digest();
    if (!timingSafeEqual(digest(req.body.joinCode), digest(config.joinCode))) return reply.code(401).send({ error: { code: 'INVALID_JOIN_CODE', message: '内测连接码不正确' }, requestId: req.id });
    const identity = randomUUID();
    const token = new AccessToken(config.apiKey, config.secret, { identity, name: req.body.nickname.trim(), ttl: 300 });
    token.addGrant({ roomJoin: true, room: config.room, canPublish: true, canSubscribe: true, canPublishData: false, canPublishSources: [TrackSource.MICROPHONE], roomAdmin: false, canUpdateOwnMetadata: false });
    reply.header('Cache-Control', 'no-store');
    return { token: await token.toJwt(), livekitUrl: config.url, channelId: config.room, channelName: '语音大厅', identity, expiresAt: new Date(Date.now() + 300000).toISOString() };
  });
  return app;
}
