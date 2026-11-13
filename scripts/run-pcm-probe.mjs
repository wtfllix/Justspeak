// Local developer verification only. Reads an independently running test API's join code.
import { spawn } from 'node:child_process';
if(!process.env.POC_API_URL||!process.env.POC_JOIN_CODE)throw new Error('Set POC_API_URL and POC_JOIN_CODE for your controlled test server');
const child=spawn('cargo',['run','--locked','-p','voice-core','--example','pcm_probe'],{stdio:'inherit',env:process.env});
child.on('exit',code=>process.exit(code??1));
