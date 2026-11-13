import React, { useEffect, useRef, useState } from 'react';
import { createRoot } from 'react-dom/client';
import { invoke, isTauri } from '@tauri-apps/api/core';
import { getCurrentWindow, LogicalSize } from '@tauri-apps/api/window';
import './style.css';
type Device = { id: string; name: string };
type Snapshot = { phase: string; channelId: string | null; members: {id:string;name:string;speaking:boolean}[]; muted:boolean; subscribedAudio:number };
const empty: Snapshot = { phase:'disconnected',channelId:null,members:[],muted:true,subscribedAudio:0 };
function App() {
 const [state,setState]=useState(empty), [devices,setDevices]=useState<Device[]>([]), [mic,setMic]=useState('');
 const [endpoint,setEndpoint]=useState('http://127.0.0.1:3001'), [nickname,setNickname]=useState(''), [code,setCode]=useState('');
 const [settings,setSettings]=useState(false), [busy,setBusy]=useState(false), [error,setError]=useState(''), [selected,setSelected]=useState(false);
 const operation=useRef(0);
 const desktop=isTauri();
 const refresh=async()=>{try{const d=await invoke<Device[]>('microphones');setDevices(d);setMic(old=>d.some(x=>x.id===old)?old:d[0]?.id??'');}catch(e){setError(String(e));}};
 useEffect(()=>{if(!desktop)return; void refresh();let live=true;let timer:ReturnType<typeof setTimeout>;async function poll(){try{const s=await invoke<Snapshot>('snapshot');if(live)setState(s);}catch(e){if(live)setError(String(e));}finally{if(live)timer=setTimeout(()=>void poll(),750);}}void poll();return()=>{live=false;clearTimeout(timer);};},[]);
 async function connect(){const op=++operation.current;setBusy(true);setError('');try{await invoke('connect',{endpoint,nickname,joinCode:code,microphone:mic});if(op===operation.current){setState(await invoke<Snapshot>('snapshot'));setCode('');}}catch(e){if(op===operation.current)setError(String(e));}finally{if(op===operation.current)setBusy(false);}}
 async function disconnect(){++operation.current;setBusy(true);try{await invoke('disconnect');setState(empty);}catch(e){setError(String(e));}finally{setBusy(false);}}
 async function toggleSettings(){const next=!settings;setSettings(next);if(desktop)try{const w=getCurrentWindow();const size=await w.innerSize();const scale=await w.scaleFactor();await w.setSize(new LogicalSize(next?680:340,size.height/scale));}catch(e){setError(`窗口调整失败：${String(e)}`);}}
 async function changeMic(id:string){if(state.phase==='connected')try{await invoke('select_microphone',{id});setState(await invoke<Snapshot>('snapshot'));setMic(id);}catch(e){setError(String(e));setState(await invoke<Snapshot>('snapshot'));}else setMic(id);}
 const connected=state.phase==='connected'||state.phase==='reconnecting';
 return <div className={`shell ${settings?'expanded':''}`}><main>
  <header><strong>xxspeak<span> M0</span></strong><small>核心语音 PoC</small></header>
  {!desktop&&<p role="alert" className="error">请用 Tauri 桌面启动。网页预览不具备原生音频能力。</p>}
  {!connected?<form onSubmit={e=>{e.preventDefault();void connect();}}>
   <label>书签<select disabled><option>书签将在 M2 提供</option></select></label>
   <label>服务器 ID<input placeholder="8 位 ID · M2 开放" disabled /></label>
   <label>你的昵称<input maxLength={32} value={nickname} onChange={e=>setNickname(e.target.value)} required disabled={busy}/></label>
   <details open><summary>M0 内测连接</summary><label>Token API 地址<input value={endpoint} onChange={e=>setEndpoint(e.target.value)} disabled={busy} required/></label><label>内测连接码<input type="password" autoComplete="off" value={code} onChange={e=>setCode(e.target.value)} disabled={busy} required/></label></details>
   <div className="actions"><button disabled={!desktop||busy||!mic} type="submit">{busy?'正在连接…':'仅本次连接'}</button><button disabled title="M2 接入书签">连接并收藏</button></div>
   {busy&&<button type="button" onClick={()=>void disconnect()}>取消连接</button>}
  </form>:<section className="tree" aria-label="服务器与频道"><h2>内测语音服务器</h2><button className={`channel ${selected?'selected':''}`} onClick={()=>setSelected(true)} onDoubleClick={()=>setSelected(true)} aria-pressed={selected}>▾ 语音大厅 <small>当前频道</small></button><ul>{state.members.map(m=><li key={m.id}><span className={m.speaking?'speaking':''}>●</span> {m.name}</li>)}</ul><p className="hint">M0 仅提供一个测试频道。多频道加入将在 M1 接入双击入口。</p></section>}
  {error&&<p role="alert" className="error">{error}</p>}
  <footer><div className="status"><i className={connected?'online':''}/>{busy?'连接操作中':state.phase==='reconnecting'?'媒体重连中 · 已静音':connected?'媒体会话已连接':'未连接'}<button onClick={()=>void toggleSettings()} aria-expanded={settings}>设置 {settings?'‹':'›'}</button></div>
  {connected&&<><p className="hint">已订阅远端音轨：{state.subscribedAudio} · 听感需双方确认</p><div className="actions"><button disabled={state.phase!=='connected'} onClick={async()=>{try{await invoke('set_muted',{muted:!state.muted});setState(await invoke<Snapshot>('snapshot'));}catch(e){setError(String(e));}}}>{state.muted?'开启麦克风':'静音麦克风'}</button><button onClick={()=>void disconnect()}>断开</button></div></>}
  </footer>
 </main>{settings&&<aside><h2>设置</h2><h3>麦克风</h3><label>输入设备<select disabled={!desktop||busy} value={mic} onChange={e=>void changeMic(e.target.value)}>{devices.length?devices.map(d=><option key={d.id} value={d.id}>{d.name||d.id}</option>):<option value="">没有可用麦克风</option>}</select></label><button disabled={busy||!desktop} onClick={()=>void refresh()}>刷新设备</button><p className="hint">连接后默认静音。切换设备或媒体重连后，请手动开麦。请佩戴耳机。</p><h3>说话方式</h3><select disabled aria-label="说话方式"><option>自由麦 · M0 手动开关</option><option>PTT · M1</option></select><details><summary>高级设置</summary><p className="hint">VAD、全局 PTT、输入电平、降噪与音频诊断将在后续里程碑提供。</p></details><h3>声音输出</h3><p className="hint">M0 使用原生音频设备列表中的第一个播放设备。输出选择将在 M3 提供。</p><h3>偏好</h3><p className="hint">身份、上次频道恢复及托盘将在 M1 实现。关闭窗口将结束进程与语音。</p></aside>}</div>;
}
createRoot(document.getElementById('root')!).render(<App/>);
