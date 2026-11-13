import { randomBytes } from 'node:crypto';
import { writeFileSync, existsSync } from 'node:fs';
const file=new URL('../deploy/.env',import.meta.url);
if(existsSync(file))throw new Error('deploy/.env already exists; refusing to overwrite credentials');
writeFileSync(file,`LIVEKIT_API_KEY=${randomBytes(12).toString('hex')}\nLIVEKIT_API_SECRET=${randomBytes(32).toString('hex')}\nPOC_JOIN_CODE=${randomBytes(24).toString('hex')}\nLIVEKIT_URL=ws://127.0.0.1:7880\nUSE_EXTERNAL_IP=false\nAPI_DOMAIN=api.example.com\nMEDIA_DOMAIN=voice.example.com\nACME_EMAIL=admin@example.com\n`,{mode:0o600});
console.log('Created deploy/.env (server only). Edit endpoints before remote use.');
