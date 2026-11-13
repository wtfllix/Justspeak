import test from 'node:test';
import assert from 'node:assert/strict';
import { TokenVerifier } from 'livekit-server-sdk';
import { buildApp, readConfig } from './app.js';
const config = { apiKey:'test-key',secret:'test-only-secret-32-characters-minimum',joinCode:'test-only-join-code-24-characters',url:'ws://127.0.0.1:7880',room:'xxspeak-m0-lobby' };
test('configuration fails closed', () => { assert.throws(()=>readConfig({})); assert.throws(()=>readConfig({LIVEKIT_API_KEY:'x',LIVEKIT_API_SECRET:'short',POC_JOIN_CODE:'short',LIVEKIT_URL:'ws://localhost'})); });
test('restricted signed token, unique identity, no secret in response', async t => {
 const app = await buildApp(config); t.after(()=>app.close());
 const request = () => app.inject({method:'POST',url:'/v1/poc/token',payload:{nickname:'玩家',joinCode:config.joinCode}});
 const a=await request(), b=await request(); assert.equal(a.statusCode,200);assert.match(a.headers['cache-control'],/no-store/);
 const body=a.json();assert.notEqual(body.identity,b.json().identity); assert.ok(!a.body.includes(config.secret));assert.ok(!a.body.includes(config.joinCode));
 const claims=await new TokenVerifier(config.apiKey,config.secret).verify(body.token);
 assert.equal(claims.sub,body.identity);assert.equal(claims.video.room,config.room);assert.equal(claims.video.roomAdmin,false);assert.equal(claims.video.canPublishData,false);assert.deepEqual(claims.video.canPublishSources,['microphone']);assert.ok(claims.exp-Math.floor(Date.now()/1000)<=300);
});
test('reject missing credentials, bad code and invalid nickname',async t=>{const app=await buildApp(config);t.after(()=>app.close());
 for(const payload of [{},{nickname:'x',joinCode:'bad'},{nickname:' ',joinCode:config.joinCode},{nickname:'x'.repeat(33),joinCode:config.joinCode}]){const r=await app.inject({method:'POST',url:'/v1/poc/token',payload});assert.ok([400,401].includes(r.statusCode));assert.ok(!r.body.includes(config.secret));}
});
test('room cannot be selected by client and issuance is rate limited',async t=>{const app=await buildApp(config);t.after(()=>app.close());
 const r=await app.inject({method:'POST',url:'/v1/poc/token',payload:{nickname:'x',joinCode:config.joinCode,room:'admin'}}); assert.equal(r.statusCode,400);
 let last;for(let i=0;i<21;i++)last=await app.inject({method:'POST',url:'/v1/poc/token',payload:{nickname:'x',joinCode:'bad'}});assert.equal(last.statusCode,429);
});
