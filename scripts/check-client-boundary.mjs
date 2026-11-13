import { readdirSync, readFileSync } from 'node:fs';
import { resolve } from 'node:path';
const roots=['apps/desktop/src','apps/desktop/dist','apps/desktop/src-tauri','crates/voice-core'];
const forbidden=['LIVEKIT_API_SECRET','LIVEKIT_API_KEY','process.env.POC_JOIN_CODE','VITE_POC_JOIN_CODE'];
function scan(path){for(const item of readdirSync(path,{withFileTypes:true})){const file=resolve(path,item.name);if(item.isDirectory()){if(!['target','gen'].includes(item.name))scan(file);}else if(/\.(rs|tsx?|js|json|html|css|toml)$/.test(item.name)){const body=readFileSync(file,'utf8');for(const key of forbidden)if(body.includes(key))throw new Error(`Server configuration reference in client: ${file}: ${key}`);}}}
for(const root of roots)scan(root);
console.log('Client source/bundle boundary scan passed. No server credential configuration references.');
