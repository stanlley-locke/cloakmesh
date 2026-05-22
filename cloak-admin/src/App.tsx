import React, { useState, useEffect, useRef } from 'react';
import * as ReactGridLayout from 'react-grid-layout';
import 'react-grid-layout/css/styles.css';
import 'react-resizable/css/styles.css';
import { listen } from '@tauri-apps/api/event';
import { 
  LayoutDashboard, Globe, Shield, Settings, Server, 
  Menu, Search, History, RefreshCcw, Plus, ChevronUp, ChevronDown,
  ExternalLink, Trash2, CheckCircle2, AlertCircle, Terminal as TerminalIcon, 
  Database, Zap, Lock, Eye, Users, Share2, Key, HardDrive,
  Network, MessageSquare, FileText, ArrowUpRight, ArrowDownLeft, Sliders,
  Filter, MoreVertical, Edit3, HelpCircle, UserPlus, Clock, Layout,
  ChevronLeft, ChevronRight, BarChart2, ShieldCheck, Radio, Box, List, LogOut,
  Bell, X, Move, Info, ShieldAlert as SecurityIcon, Monitor, DownloadCloud, UploadCloud, Palette,
  PaintBucket, Timer, Wifi, Layers, Fingerprint, Command, Send, FileCode,
  Cpu as HardwareIcon, Activity as ActivityIcon
} from 'lucide-react';
import { 
  BarChart, Bar, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer,
  PieChart, Pie, Cell, AreaChart, Area, LineChart, Line, Legend
} from 'recharts';

// Robust Vite/ESM Interop for react-grid-layout
const RGL: any = (ReactGridLayout as any).default || ReactGridLayout;
const ResponsiveGridLayout = RGL.Responsive || (ReactGridLayout as any).Responsive;

// ── Custom Hooks ─────────────────────────────────────────────────────────────

const useContainerWidth = (ref: React.RefObject<HTMLDivElement>) => {
  const [width, setWidth] = useState(0);
  useEffect(() => {
    const observer = new ResizeObserver((entries) => {
      for (let entry of entries) {
        setWidth(entry.contentRect.width);
      }
    });
    if (ref.current) observer.observe(ref.current);
    return () => observer.disconnect();
  }, [ref]);
  return width;
};

// ── Tauri IPC Guard ──────────────────────────────────────────────────────────

const tauriInvoke = async (cmd: string, args: any = {}) => {
  if (typeof window !== 'undefined' && (window as any).__TAURI_IPC__) {
    const { invoke } = await import('@tauri-apps/api/tauri');
    return await invoke(cmd, args);
  }
  
  if (cmd === 'get_node_status') return {
    status: "online",
    address: "ahqw6zrrljnem7gxqlducifffw2v7nhgyqujcy36jlfwr5xbmaxfg3iuxdoa.cloak",
    active_nodes: 142,
    throughput: "8.4 MB/s",
    latency: "42ms",
    reputation: 0.99,
    dht_entries: 1240,
    uptime: "4d 12h 04m",
    handling_time: "03:17",
    cpu_usage: 14.2,
    mem_usage: 2.1
  };
  if (cmd === 'get_relays') return [
    { id: '1', name: 'Relay Alpha-01', addr: 'ahqw6zr...', load: '12%', estab: '12ms', status: 'STABLE', conversion: '82%', target: '90%' },
    { id: '2', name: 'Relay Beta-02', addr: 'zmij1m7...', load: '45%', estab: '88ms', status: 'ACTIVE', conversion: '65%', target: '80%' },
    { id: '3', name: 'Relay Gamma-03', addr: 'node7p5...', load: '88%', estab: '210ms', status: 'LOADED', conversion: '42%', target: '85%' },
    { id: '4', name: 'Relay Delta-04', addr: 'cloak1qy...', load: '5%', estab: '5ms', status: 'STABLE', conversion: '94%', target: '95%' },
  ];
  return {};
};

// ── Shared UI Components ─────────────────────────────────────────────────────

const Card = ({ children, title, subTitle, onRemove, isEditMode, settings, updateSettings, className = "" }: any) => {
  const [showMenu, setShowMenu] = useState(false);
  const menuRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (menuRef.current && !menuRef.current.contains(event.target as Node)) {
        setShowMenu(false);
      }
    };
    document.addEventListener("mousedown", handleClickOutside);
    return () => document.removeEventListener("mousedown", handleClickOutside);
  }, []);

  const accentColor = settings?.color || '#0f172a';
  const bgColor = settings?.bg || '#ffffff';
  const isDarkBg = bgColor !== '#ffffff' && bgColor !== '#f8fafc';

  return (
    <div 
      className={`border border-slate-100 flex flex-col h-full rounded-2xl shadow-[0_4px_20px_rgba(0,0,0,0.03)] hover:shadow-[0_8px_30px_rgba(0,0,0,0.06)] transition-all duration-300 relative group overflow-hidden ${className}`}
      style={{ backgroundColor: bgColor }}
    >
      {isEditMode && (
        <div className="absolute inset-0 bg-slate-900/10 z-20 flex items-center justify-center cursor-move border-2 border-dashed border-slate-400 rounded-2xl backdrop-blur-[1px]">
          <div className="bg-white/90 p-4 rounded-2xl shadow-2xl flex flex-col items-center space-y-4 text-slate-900">
             <Move size={32} className="animate-bounce" />
             <button onClick={(e) => { e.stopPropagation(); onRemove(); }} className="p-2 bg-rose-500 text-white rounded-xl hover:bg-rose-600 transition-colors shadow-lg">
               <Trash2 size={16} />
             </button>
          </div>
        </div>
      )}
      <div className="flex justify-between items-start p-6 pb-2">
        <div style={{ borderLeft: `3px solid ${accentColor}`, paddingLeft: '12px' }}>
          <h3 className={`text-[10px] font-black uppercase tracking-[0.15em] leading-none ${isDarkBg ? 'text-white' : 'text-slate-900'}`}>{title}</h3>
          {subTitle && <p className={`text-[9px] mt-1.5 font-bold uppercase tracking-wider leading-none ${isDarkBg ? 'text-white/60' : 'text-slate-400'}`}>{subTitle}</p>}
        </div>
        <div className="relative" ref={menuRef}>
          <div 
            className={`p-1.5 rounded-lg cursor-pointer transition-colors ${isDarkBg ? 'hover:bg-white/10' : 'hover:bg-slate-50'}`} 
            onClick={() => setShowMenu(!showMenu)}
          >
            <MoreVertical size={14} className={isDarkBg ? 'text-white/40 group-hover:text-white' : 'text-slate-300 group-hover:text-slate-900'} />
          </div>
          {showMenu && (
            <div className="absolute top-8 right-0 w-56 bg-white border border-slate-100 rounded-xl shadow-2xl z-[100] p-4 animate-in fade-in zoom-in duration-200">
               <div className="text-[9px] font-black text-slate-400 uppercase tracking-widest mb-3 px-1 flex items-center"><Palette size={10} className="mr-2" /> Accent Color</div>
               <div className="grid grid-cols-4 gap-2 mb-4">
                  {['#0f172a', '#10b981', '#f59e0b', '#ef4444', '#3b82f6', '#8b5cf6', '#ec4899', '#64748b'].map(c => (
                    <div key={c} className={`h-6 rounded-md cursor-pointer border-2 transition-all ${settings?.color === c ? 'border-slate-900' : 'border-transparent'}`} style={{ backgroundColor: c }} onClick={() => updateSettings({ color: c })} />
                  ))}
               </div>
               
               <div className="text-[9px] font-black text-slate-400 uppercase tracking-widest mb-3 px-1 flex items-center"><PaintBucket size={10} className="mr-2" /> Background</div>
               <div className="grid grid-cols-4 gap-2 mb-4">
                  {['#ffffff', '#f8fafc', '#f1f5f9', '#0f172a', '#1e293b', '#334155', '#475569', '#000000'].map(bg => (
                    <div key={bg} className={`h-6 rounded-md cursor-pointer border-2 transition-all ${settings?.bg === bg ? 'border-slate-900' : 'border-slate-200'}`} style={{ backgroundColor: bg }} onClick={() => updateSettings({ bg })} />
                  ))}
               </div>

               <div className="border-t border-slate-100 pt-3 mt-1">
                  <button onClick={() => { onRemove(); setShowMenu(false); }} className="w-full text-left p-2 rounded-lg hover:bg-rose-50 text-[10px] font-bold text-rose-600 flex items-center"><Trash2 size={12} className="mr-2" /> Remove Widget</button>
               </div>
            </div>
          )}
        </div>
      </div>
      <div className="flex-1 relative flex flex-col justify-center px-6 pb-6 overflow-hidden">
        <div className={isDarkBg ? 'text-white' : ''}>
          {children}
        </div>
      </div>
    </div>
  );
};

// ── Dashboard Widgets Library ────────────────────────────────────────────────

const Widgets: Record<string, any> = {
  AvailableHops: ({ status, settings }: any) => (
    <div className="flex items-baseline space-x-2">
      <span className="text-3xl font-black font-mono tracking-tighter" style={{ color: settings?.color || (settings?.bg && settings.bg !== '#ffffff' && settings.bg !== '#f8fafc' ? '#fff' : '#0f172a') }}>{status?.active_nodes || "0"}</span>
      <span className="text-[10px] font-black flex items-center text-emerald-500"><ChevronUp size={12} strokeWidth={3} /> 4.2%</span>
    </div>
  ),
  AvgHSTime: ({ status, settings }: any) => (
    <div className="flex items-baseline space-x-2">
      <span className="text-3xl font-black font-mono tracking-tighter" style={{ color: settings?.color || (settings?.bg && settings.bg !== '#ffffff' && settings.bg !== '#f8fafc' ? '#fff' : '#0f172a') }}>{status?.latency || "0ms"}</span>
      <span className="text-[10px] font-black flex items-center text-slate-300"><ChevronDown size={12} strokeWidth={3} /> 1.8%</span>
    </div>
  ),
  MeanHandlingTime: ({ status, settings }: any) => (
    <div className="flex flex-col items-center justify-center">
      <span className="text-8xl font-black font-mono tracking-tighter leading-none" style={{ color: settings?.color || (settings?.bg && settings.bg !== '#ffffff' && settings.bg !== '#f8fafc' ? '#fff' : '#0f172a') }}>{status?.handling_time || "03:17"}</span>
      <span className={`text-[10px] font-black uppercase mt-4 tracking-[0.2em] ${settings?.bg && settings.bg !== '#ffffff' && settings.bg !== '#f8fafc' ? 'text-white/40' : 'text-slate-400'}`}>MASTER_NODE_UTILIZATION</span>
    </div>
  ),
  NetworkStability: ({ settings }: any) => {
    const chartData = Array.from({ length: 24 }, (_, i) => ({ time: `${i}:00`, val: 80 + Math.random() * 20 }));
    const isDark = settings?.bg && settings.bg !== '#ffffff' && settings.bg !== '#f8fafc';
    const color = settings?.color || (isDark ? '#fff' : '#0f172a');
    return (
      <div className="h-full w-full">
        <ResponsiveContainer width="100%" height="100%">
          <AreaChart data={chartData}>
            <CartesianGrid strokeDasharray="0" vertical={false} stroke={isDark ? '#ffffff10' : '#f1f5f9'} />
            <XAxis dataKey="time" hide />
            <YAxis hide domain={[60, 110]} />
            <Area type="step" dataKey="val" stroke={color} fill={color} fillOpacity={0.04} strokeWidth={2.5} />
            <Tooltip contentStyle={{ borderRadius: '12px', border: 'none', fontSize: '11px', backgroundColor: isDark ? '#000' : '#fff', color: isDark ? '#fff' : '#000' }} />
          </AreaChart>
        </ResponsiveContainer>
      </div>
    );
  },
  RelayAudit: ({ relays, searchQuery, settings, filterStatus }: any) => {
    const filteredRelays = relays?.filter((r: any) => {
      const matchesSearch = r.name.toLowerCase().includes(searchQuery.toLowerCase()) || r.addr.toLowerCase().includes(searchQuery.toLowerCase());
      const matchesStatus = filterStatus === 'ALL' || r.status === filterStatus;
      return matchesSearch && matchesStatus;
    });
    const isDark = settings?.bg && settings.bg !== '#ffffff' && settings.bg !== '#f8fafc';
    const accentColor = settings?.color || (isDark ? '#fff' : '#0f172a');
    return (
      <div className="overflow-x-auto h-full scrollbar-hide">
        <table className="w-full border-collapse">
          <thead>
            <tr className={`text-[9px] font-black uppercase tracking-widest border-b text-left ${isDark ? 'border-white/10 text-white/40' : 'border-slate-100 text-slate-400'}`}>
              <th className="py-4 px-2">PEER_ID</th>
              <th className="py-4 px-2 text-center">CONVERSION</th>
              <th className="py-4 px-2 text-right">STATUS</th>
            </tr>
          </thead>
          <tbody className={`text-[11px] font-medium ${isDark ? 'text-white' : 'text-slate-900'}`}>
            {filteredRelays?.map((r: any) => (
              <tr key={r.id} className={`border-b transition-colors ${isDark ? 'border-white/5 hover:bg-white/5' : 'border-slate-50 hover:bg-slate-50/50'}`}>
                <td className="py-3 px-2 font-bold italic">{r.name}</td>
                <td className="py-3 px-2 text-center"><div className="w-20 h-1.5 bg-slate-100/10 rounded-full mx-auto overflow-hidden"><div className="h-full transition-all duration-1000" style={{ width: r.conversion, backgroundColor: accentColor }} /></div></td>
                <td className={`py-3 px-2 text-right font-black uppercase tracking-widest text-[8px] ${isDark ? 'text-white/40' : 'text-slate-500'}`}>{r.status}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    );
  },
  HardwareLoad: ({ status, settings }: any) => {
    const isDark = settings?.bg && settings.bg !== '#ffffff' && settings.bg !== '#f8fafc';
    const color = settings?.color || (isDark ? '#fff' : '#0f172a');
    return (
      <div className="space-y-6">
        <div className="space-y-2">
           <div className={`flex justify-between text-[10px] font-black uppercase tracking-widest ${isDark ? 'text-white/80' : ''}`}><span>CPU Core Utilization</span><span className="font-mono">{status?.cpu_usage || '0'}%</span></div>
           <div className={`w-full h-1.5 rounded-full overflow-hidden ${isDark ? 'bg-white/10' : 'bg-slate-100'}`}><div className="h-full transition-all duration-1000" style={{ width: `${status?.cpu_usage || 0}%`, backgroundColor: color }} /></div>
        </div>
        <div className="space-y-2">
           <div className={`flex justify-between text-[10px] font-black uppercase tracking-widest ${isDark ? 'text-white/80' : ''}`}><span>RAM Kernel Buffer</span><span className="font-mono">{status?.mem_usage || '0'}GB</span></div>
           <div className={`w-full h-1.5 rounded-full overflow-hidden ${isDark ? 'bg-white/10' : 'bg-slate-100'}`}><div className="h-full transition-all duration-1000" style={{ width: `${((status?.mem_usage || 0) / 16) * 100}%`, backgroundColor: color }} /></div>
        </div>
      </div>
    );
  },
  MeshTraffic: ({ settings }: any) => {
    const data = Array.from({ length: 20 }, (_, i) => ({ time: i, in: 20 + Math.random() * 60, out: 15 + Math.random() * 40 }));
    const isDark = settings?.bg && settings.bg !== '#ffffff' && settings.bg !== '#f8fafc';
    const color = settings?.color || (isDark ? '#10b981' : '#0f172a');
    return (
      <div className="h-full w-full">
        <ResponsiveContainer width="100%" height="100%">
          <LineChart data={data}>
            <CartesianGrid strokeDasharray="3 3" stroke={isDark ? '#ffffff05' : '#00000005'} vertical={false} />
            <XAxis dataKey="time" hide />
            <YAxis hide />
            <Tooltip contentStyle={{ borderRadius: '12px', border: 'none', fontSize: '10px' }} />
            <Line type="monotone" dataKey="in" stroke={color} strokeWidth={3} dot={false} />
            <Line type="monotone" dataKey="out" stroke={isDark ? '#ffffff40' : '#cbd5e1'} strokeWidth={2} dot={false} strokeDasharray="5 5" />
          </LineChart>
        </ResponsiveContainer>
      </div>
    );
  },
  SecurityAudit: ({ settings }: any) => {
    const events = [
      { id: 1, type: 'VERIFY', msg: 'Identity ahqw6... verified', time: '1m ago' },
      { id: 2, type: 'SIGN', msg: 'Capability token created', time: '4m ago' },
      { id: 3, type: 'ALERT', msg: 'Unrecognized handshake attempt', time: '12m ago' },
    ];
    const isDark = settings?.bg && settings.bg !== '#ffffff' && settings.bg !== '#f8fafc';
    return (
      <div className="space-y-3">
        {events.map(e => (
          <div key={e.id} className={`flex items-center justify-between p-2 rounded-xl border ${isDark ? 'border-white/5 bg-white/5' : 'border-slate-50 bg-slate-50/50'}`}>
            <div className="flex items-center space-x-3">
              <div className={`w-1.5 h-1.5 rounded-full ${e.type === 'ALERT' ? 'bg-rose-500 animate-pulse' : 'bg-emerald-500'}`} />
              <span className={`text-[10px] font-black uppercase tracking-tighter ${isDark ? 'text-white/80' : 'text-slate-900'}`}>{e.msg}</span>
            </div>
            <span className="text-[8px] font-black opacity-30 uppercase">{e.time}</span>
          </div>
        ))}
      </div>
    );
  },
  UptimeMonitor: ({ status, settings }: any) => {
    const isDark = settings?.bg && settings.bg !== '#ffffff' && settings.bg !== '#f8fafc';
    return (
      <div className="flex flex-col items-center justify-center">
        <div className="relative">
           <span className={`text-6xl font-black font-mono tracking-tighter ${isDark ? 'text-white' : 'text-slate-900'}`}>99.9</span>
           <span className="text-xl font-black ml-1 text-emerald-500">%</span>
        </div>
        <div className="flex items-center space-x-2 mt-2">
           <Wifi size={12} className="text-emerald-500" />
           <span className={`text-[9px] font-black uppercase tracking-widest ${isDark ? 'text-white/40' : 'text-slate-400'}`}>Continuous Uptime</span>
        </div>
      </div>
    );
  }
};

const WIDGET_CATALOG = [
  { id: 'AvailableHops', title: 'AVAILABLE_HOPS', subTitle: 'ACTIVE_MESH_NODES', w: 2, h: 2 },
  { id: 'AvgHSTime', title: 'AVG_HS_TIME', subTitle: 'AVERAGE_HANDSHAKE', w: 2, h: 2 },
  { id: 'MeanHandlingTime', title: 'HANDLING_TIME', subTitle: 'MASTER_UTIL', w: 4, h: 4 },
  { id: 'NetworkStability', title: 'STABILITY', subTitle: 'STABILITY_INDEX', w: 4, h: 4 },
  { id: 'RelayAudit', title: 'RELAY_AUDIT', subTitle: 'MESH_PERFORMANCE', w: 8, h: 4 },
  { id: 'HardwareLoad', title: 'HARDWARE_LOAD', subTitle: 'CORE_RESOURCE', w: 4, h: 4 },
  { id: 'MeshTraffic', title: 'TRAFFIC_ANALYSIS', subTitle: 'REALTIME_IO_FLOW', w: 4, h: 3 },
  { id: 'SecurityAudit', title: 'SECURITY_AUDIT', subTitle: 'CRYPTOGRAPHIC_LOG', w: 4, h: 3 },
  { id: 'UptimeMonitor', title: 'UPTIME_INDEX', subTitle: 'NODE_RELIABILITY', w: 3, h: 3 },
];

// ── Sub-Page Views ───────────────────────────────────────────────────────────

const IdentityView = ({ status }: any) => (
  <div className="grid grid-cols-2 gap-8 h-full">
    <Card title="ACTIVE_IDENTITY" subTitle="ED25519_FINGERPRINT">
      <div className="bg-slate-50 p-8 border border-slate-200 font-mono text-sm break-all relative rounded-2xl mb-8">
        <div className="text-[9px] font-black text-black/30 uppercase mb-4 tracking-widest">Public Address</div>
        <span className="text-slate-900 font-black text-lg select-all leading-tight">{status?.address || "AWAITING_INITIALIZATION..."}</span>
        <button className="absolute bottom-4 right-4 p-2 bg-white border border-slate-200 rounded-xl hover:bg-slate-900 hover:text-white transition-all shadow-sm"><Share2 size={16} /></button>
      </div>
      <div className="grid grid-cols-2 gap-6 pt-6 border-t border-slate-100">
        <div className="space-y-1"><div className="text-[9px] font-black text-slate-400 uppercase">Generation Date</div><div className="text-xs font-black font-mono text-slate-900 italic underline">2026.05.22_12:04</div></div>
        <div className="space-y-1"><div className="text-[9px] font-black text-slate-400 uppercase">Identity Type</div><div className="text-xs font-black font-mono text-slate-900">CLOAK_v1_ED25519</div></div>
      </div>
    </Card>
    <Card title="GATED_PERMISSIONS" subTitle="ACTIVE_CAPABILITY_TOKENS">
      <div className="space-y-4">
        {[{ id: 1, scope: 'NETWORK_ADMIN', ttl: '04:12:11' }, { id: 2, scope: 'FILE_STREAM', ttl: '00:59:01' }].map((token) => (
          <div key={token.id} className="flex justify-between items-center p-4 border border-slate-100 bg-slate-50 rounded-2xl group hover:border-slate-300 transition-all">
            <div className="flex items-center space-x-4"><div className="p-2 bg-white rounded-xl shadow-sm border border-slate-200"><Lock size={14} className="text-slate-900" /></div><div><div className="text-[10px] font-black uppercase tracking-widest text-slate-900">{token.scope}</div></div></div>
            <div className="text-right"><div className="text-[9px] font-black text-slate-400 uppercase">TTL Remaining</div><div className="text-[11px] font-black font-mono text-slate-900 italic underline">{token.ttl}</div></div>
          </div>
        ))}
        <button className="w-full py-4 border-2 border-dashed border-slate-200 rounded-2xl text-[10px] font-black uppercase tracking-widest text-slate-400 hover:text-slate-900 hover:border-slate-900 transition-all active:scale-95">Generate Session Authorization</button>
      </div>
    </Card>
  </div>
);

const MessagingView = () => {
  const [target, setTarget] = useState('');
  const [msg, setMsg] = useState('');
  const [chatLogs, setChatLogs] = useState<any[]>([]);

  const handleSend = async () => {
    if (!target || !msg) return;
    try {
      await tauriInvoke('send_message', { target, content: msg });
      setChatLogs(prev => [...prev, { id: Date.now(), from: 'ME', to: target, body: msg, time: new Date().toLocaleTimeString() }]);
      setMsg('');
    } catch (e) { alert(e); }
  };

  return (
    <div className="grid grid-cols-12 gap-8 h-full">
      <div className="col-span-4">
        <Card title="NEW_ENCRYPTED_MESSAGE" subTitle="DOUBLE_RATCHET_PROTOCOL">
          <div className="space-y-6 mt-4">
             <div className="space-y-2">
                <label className="text-[9px] font-black uppercase text-slate-400">Target Address</label>
                <input value={target} onChange={e => setTarget(e.target.value)} className="w-full bg-slate-50 border border-slate-200 p-4 rounded-2xl text-xs font-mono outline-none focus:border-slate-900" placeholder="ahqw6...cloak" />
             </div>
             <div className="space-y-2">
                <label className="text-[9px] font-black uppercase text-slate-400">Content</label>
                <textarea value={msg} onChange={e => setMsg(e.target.value)} className="w-full bg-slate-50 border border-slate-200 p-4 rounded-2xl text-xs font-medium outline-none focus:border-slate-900 h-32" placeholder="Enter private message..." />
             </div>
             <button onClick={handleSend} className="w-full py-4 bg-slate-900 text-white rounded-2xl font-black uppercase tracking-[0.2em] text-[10px] flex items-center justify-center space-x-2 hover:bg-black transition-all">
                <Send size={14} /> <span>Deliver via Mesh</span>
             </button>
          </div>
        </Card>
      </div>
      <div className="col-span-8">
        <Card title="SECURE_CHAT_HISTORY" subTitle="FORWARD_SECRECY_AUDIT">
           <div className="space-y-4 max-h-[600px] overflow-y-auto pr-2 scrollbar-hide">
              {chatLogs.length === 0 && <div className="text-center py-20 text-slate-300 font-black uppercase italic opacity-40">Zero Encrypted Transmissions</div>}
              {chatLogs.map(log => (
                <div key={log.id} className="p-5 border border-slate-100 bg-white rounded-[24px] shadow-sm">
                   <div className="flex justify-between items-start mb-2">
                      <span className="px-2 py-1 bg-slate-900 text-white text-[8px] font-black rounded-lg">OUTGOING</span>
                      <span className="text-[9px] font-black text-slate-300">{log.time}</span>
                   </div>
                   <p className="text-xs font-medium text-slate-700 leading-relaxed">{log.body}</p>
                   <div className="mt-3 pt-3 border-t border-slate-50 flex items-center text-[9px] text-slate-400 font-mono italic">
                      <ArrowUpRight size={10} className="mr-1" /> To: {log.to.slice(0, 16)}...
                   </div>
                </div>
              ))}
           </div>
        </Card>
      </div>
    </div>
  );
};

const TerminalView = ({ logs }: { logs: any[] }) => {
  return (
    <Card title="KERNEL_STRUCTURED_LOGS" subTitle="LIVE_SYSTEM_AUDIT">
       <div className="bg-slate-900 text-slate-100 p-8 h-[600px] font-mono text-[10px] flex flex-col space-y-3 overflow-y-auto rounded-[32px] border border-slate-800 shadow-2xl scrollbar-hide">
          {logs.length === 0 && <div className="text-center py-40 text-slate-700 uppercase font-black italic">Awaiting Kernal Bridge...</div>}
          {logs.map((log, i) => (
            <div key={i} className="flex space-x-4 border-b border-slate-800 pb-2 last:border-0 group">
               <span className="text-slate-600 font-black whitespace-nowrap">[{log.timestamp}]</span>
               <span className={`font-black ${log.level === 'ERROR' ? 'text-rose-500' : log.level === 'WARN' ? 'text-amber-500' : 'text-emerald-500'}`}>[{log.level}]</span>
               <span className="text-slate-400 font-bold group-hover:text-white transition-colors">{log.message}</span>
               <span className="ml-auto text-[8px] text-slate-600 uppercase italic opacity-0 group-hover:opacity-100">{log.target}</span>
            </div>
          ))}
       </div>
    </Card>
  );
};

// ── Main App Component ───────────────────────────────────────────────────────

export default function App() {
  const [page, setPage] = useState('dashboard');
  const [isSidebarExpanded, setIsSidebarExpanded] = useState(true);
  const [status, setStatus] = useState<any>(null);
  const [relays, setRelays] = useState<any[]>([]);
  const [isEditMode, setIsEditMode] = useState(false);
  const [searchQuery, setSearchQuery] = useState('');
  const [filterStatus, setFilterStatus] = useState('ALL');
  const [showFilterMenu, setShowFilterMenu] = useState(false);
  const [showLibrary, setShowLibrary] = useState(false);
  const [showNotifications, setShowNotifications] = useState(false);
  const [isRefreshing, setIsRefreshing] = useState(false);
  const [expandedGroups, setExpandedGroups] = useState<string[]>(['core', 'network', 'security', 'ops', 'system']);
  const [notifications, setNotifications] = useState<any[]>([]);
  const [structuredLogs, setStructuredLogs] = useState<any[]>([]);

  // Dynamic Layout & Persistence
  const [layout, setLayout] = useState<any[]>(() => {
    const saved = localStorage.getItem('cloak_dashboard_layout_v18');
    return saved ? JSON.parse(saved) : [{ i: 'AvailableHops', x: 0, y: 0, w: 2, h: 2 }, { i: 'AvgHSTime', x: 0, y: 2, w: 2, h: 2 }, { i: 'MeanHandlingTime', x: 2, y: 0, w: 4, h: 4 }, { i: 'NetworkStability', x: 6, y: 0, w: 6, h: 4 }, { i: 'RelayAudit', x: 0, y: 4, w: 8, h: 4 }];
  });

  const [widgetSettings, setWidgetSettings] = useState<any>(() => {
    const saved = localStorage.getItem('cloak_widget_settings_v18');
    return saved ? JSON.parse(saved) : {};
  });

  const containerRef = useRef<HTMLDivElement>(null);
  const containerWidth = useContainerWidth(containerRef);

  useEffect(() => { localStorage.setItem('cloak_dashboard_layout_v18', JSON.stringify(layout)); }, [layout]);
  useEffect(() => { localStorage.setItem('cloak_widget_settings_v18', JSON.stringify(widgetSettings)); }, [widgetSettings]);

  useEffect(() => {
    const initLogListener = async () => {
       const unlisten = await listen('structured-log', (event: any) => {
         setStructuredLogs(prev => [event.payload, ...prev].slice(0, 100));
         if (event.payload.level === 'WARN' || event.payload.level === 'ERROR') {
            setNotifications(prev => [{ id: Date.now(), type: 'warning', msg: event.payload.message, time: 'Just now' }, ...prev]);
         }
       });
       return unlisten;
    };
    const logCleaner = initLogListener();
    return () => { logCleaner.then(f => f()); };
  }, []);

  const fetchData = async (manual = false) => {
    if (manual) setIsRefreshing(true);
    try {
      const s = await tauriInvoke('get_node_status'); setStatus(s);
      const r = await tauriInvoke('get_relays'); setRelays(r as any[]);
      if (manual) setNotifications(prev => [{ id: Date.now(), type: 'info', msg: 'System synchronization successful', time: 'Just now' }, ...prev]);
    } catch (e) { console.error(e); }
    finally { if (manual) setTimeout(() => setIsRefreshing(false), 800); }
  };

  useEffect(() => { fetchData(); const timer = setInterval(fetchData, 5000); return () => clearInterval(timer); }, []);

  const addWidget = (module: any) => {
    const instanceId = `${module.id}_${Date.now()}`;
    const newWidget = { 
      i: instanceId, 
      type: module.id,
      x: (layout.length * 2) % 12, 
      y: 100, 
      w: module.w, 
      h: module.h 
    };
    setLayout((prev: any[]) => [...prev, newWidget]);
    setShowLibrary(false);
    setNotifications((prev: any[]) => [{ id: Date.now(), type: 'success', msg: `Provisioned ${module.title}`, time: 'Just now' }, ...prev]);
  };

  const updateWidgetSettings = (id: string, newSettings: any) => {
    setWidgetSettings((prev: any) => ({ ...prev, [id]: { ...(prev[id] || {}), ...newSettings } }));
  };

  const toggleGroup = (id: string) => {
    setExpandedGroups(prev => prev.includes(id) ? prev.filter(g => g !== id) : [...prev, id]);
  };

  const navGroups = [
    { id: 'core', label: 'Main', items: [{ id: 'dashboard', label: 'Overview', icon: LayoutDashboard }, { id: 'messenger', label: 'Darknet Chat', icon: MessageSquare }] },
    { id: 'network', label: 'Network', items: [{ id: 'discovery', label: 'Peer Discovery', icon: Globe }, { id: 'circuits', label: 'Circuit Audit', icon: Radio }, { id: 'dht', label: 'DHT Explorer', icon: Database }] },
    { id: 'security', label: 'Security', items: [{ id: 'identity', label: 'ID Management', icon: Key }, { id: 'capabilities', label: 'Permissions', icon: ShieldCheck }, { id: 'encryption', label: 'Crypto Lab', icon: Lock }] },
    { id: 'ops', label: 'Operations', items: [{ id: 'hosting', label: 'Service Hosting', icon: Server }, { id: 'terminal', label: 'Kernel Logs', icon: TerminalIcon }] },
    { id: 'system', label: 'System', items: [{ id: 'settings', label: 'Preferences', icon: Settings }, { id: 'hardware', label: 'Hardware', icon: HardwareIcon }] }
  ];

  return (
    <div className="flex flex-col h-screen bg-[#F8FAFC] text-slate-900 font-sans select-none antialiased">
      <header className="h-16 bg-white border-b border-slate-100 flex items-center justify-between px-8 z-50 shadow-sm relative">
        <div className="flex items-center space-x-6">
          <div className="bg-slate-900 p-2 rounded-xl shadow-lg cursor-pointer active:scale-95 transition-transform" onClick={() => setIsSidebarExpanded(!isSidebarExpanded)}>
            <img src="/assets/images/realistic-icon-black-white-inside-vampire-cloak-with-golden-detail-vector-illustration.png" className="w-5 h-5 grayscale brightness-[5]" alt="" />
          </div>
          <div className="p-2 hover:bg-slate-50 rounded-lg cursor-pointer transition-colors" onClick={() => setIsSidebarExpanded(!isSidebarExpanded)}><Menu size={20} className="text-slate-400 hover:text-slate-900" /></div>
          <h1 className="text-sm font-black uppercase tracking-[0.2em] border-l border-slate-100 pl-6 ml-2">CloakMesh <span className="text-slate-300 font-light ml-2">Network Ops</span></h1>
        </div>

        <div className="flex-1 max-w-xl px-12">
          <div className="relative group">
            <Search className="absolute left-4 top-1/2 -translate-y-1/2 text-slate-300 group-focus-within:text-slate-900 transition-colors" size={16} />
            <input type="text" placeholder="Search Mesh Addresses, Peers, or Records..." className="w-full bg-slate-50 border border-slate-100 rounded-2xl py-2.5 pl-12 pr-4 text-xs font-medium focus:bg-white focus:border-slate-900 focus:outline-none transition-all placeholder:text-slate-300" value={searchQuery} onChange={(e) => setSearchQuery(e.target.value)} />
            {searchQuery && <X className="absolute right-4 top-1/2 -translate-y-1/2 text-slate-300 hover:text-rose-500 cursor-pointer transition-colors" size={14} onClick={() => setSearchQuery('')} />}
          </div>
        </div>
        
        <div className="flex items-center space-x-4">
          <div className="relative">
            <div className={`flex items-center bg-slate-50 border border-slate-200 px-3.5 py-2 rounded-xl space-x-3 cursor-pointer hover:bg-slate-100 transition-all active:scale-95 shadow-sm ${filterStatus !== 'ALL' ? 'border-slate-900 bg-slate-100' : ''}`} onClick={() => setShowFilterMenu(!showFilterMenu)}>
              <Filter size={14} className={filterStatus !== 'ALL' ? 'text-slate-900' : 'text-slate-500'} />
              <span className="text-[10px] font-black uppercase tracking-widest text-slate-700">{filterStatus === 'ALL' ? 'Filter' : filterStatus}</span>
            </div>
            {showFilterMenu && (
              <div className="absolute top-12 left-0 w-40 bg-white border border-slate-100 rounded-xl shadow-2xl z-[100] p-2 animate-in slide-in-from-top-2 duration-200">
                {['ALL', 'STABLE', 'ACTIVE', 'LOADED'].map(f => (
                  <button key={f} className={`w-full text-left px-3 py-2 rounded-lg text-[10px] font-black uppercase tracking-widest transition-colors ${filterStatus === f ? 'bg-slate-900 text-white' : 'hover:bg-slate-50 text-slate-500'}`} onClick={() => { setFilterStatus(f); setShowFilterMenu(false); }}>{f}</button>
                ))}
              </div>
            )}
          </div>

          <div className="flex items-center space-x-1.5 ml-2">
            <div className="relative">
              <div className="p-2.5 hover:bg-slate-50 rounded-xl cursor-pointer transition-all active:scale-90 group relative" onClick={() => setShowNotifications(!showNotifications)}><Bell size={18} className={`${showNotifications ? 'text-slate-900' : 'text-slate-400'} group-hover:text-slate-900`} />{notifications.length > 0 && <div className="absolute top-2 right-2 w-2 h-2 bg-rose-500 rounded-full border-2 border-white" />}</div>
              {showNotifications && (
                <div className="absolute top-14 right-0 w-80 bg-white border border-slate-100 rounded-3xl shadow-2xl z-[100] p-6 animate-in slide-in-from-top-2 duration-300">
                  <div className="flex justify-between items-center mb-6"><span className="text-[10px] font-black uppercase tracking-[0.2em] text-slate-400 font-mono italic underline">SYSTEM_ACTIVITY</span><button onClick={() => setNotifications([])} className="text-[9px] font-black text-slate-300 hover:text-rose-500 transition-colors uppercase">Flush</button></div>
                  <div className="space-y-4 max-h-80 overflow-y-auto pr-1 scrollbar-hide">
                    {notifications.length === 0 && <p className="text-center text-slate-300 py-12 text-[10px] uppercase italic opacity-40">Zero Events Found</p>}
                    {notifications.map(n => (<div key={n.id} className="p-4 bg-slate-50/50 rounded-2xl border border-slate-100 hover:bg-white transition-all group">
                        <p className="text-[10px] font-black text-slate-900 leading-relaxed uppercase italic">{n.msg}</p>
                        <p className="text-[8px] text-slate-400 font-black tracking-tighter opacity-40 mt-2">{n.time}</p>
                    </div>))}
                  </div>
                </div>
              )}
            </div>
            <div className="p-2.5 hover:bg-slate-50 rounded-xl cursor-pointer transition-all active:scale-90 group" onClick={() => fetchData(true)}><RefreshCcw size={18} className={`text-slate-400 group-hover:text-slate-900 transition-all ${isRefreshing ? 'animate-spin text-slate-900' : ''}`} /></div>
            <div className="p-2.5 hover:bg-slate-50 rounded-xl cursor-pointer transition-all active:scale-90 group" onClick={() => setPage('settings')}><Settings size={18} className={`${page === 'settings' ? 'text-slate-900' : 'text-slate-400'} group-hover:text-slate-900`} /></div>
          </div>
          <div className="w-px h-6 bg-slate-100 mx-2" />
          <button className="bg-slate-900 text-white px-5 py-2 rounded-xl font-black uppercase text-[10px] tracking-widest flex items-center space-x-2.5 hover:bg-black transition-all active:scale-95 shadow-md" onClick={() => setShowLibrary(true)}><Plus size={16} strokeWidth={3} /><span>Add Widget</span></button>
          <div className="flex items-center space-x-3 ml-2 pl-4 border-l border-slate-100"><span className="text-[9px] font-black text-slate-300 uppercase tracking-widest italic">{isEditMode ? 'Active' : 'Locked'}</span><div onClick={() => setIsEditMode(!isEditMode)} className={`w-11 h-6 rounded-full relative cursor-pointer transition-all duration-300 ${isEditMode ? 'bg-emerald-500 shadow-inner shadow-black/20' : 'bg-slate-100'}`}><div className={`absolute top-1 w-4 h-4 rounded-full transition-all duration-300 shadow-sm ${isEditMode ? 'left-6 bg-white' : 'left-1 bg-slate-400'}`} /></div></div>
        </div>
      </header>

      <div className="flex-1 flex overflow-hidden">
        {/* SIDEBAR */}
        <aside className={`bg-white border-r border-slate-100 flex flex-col transition-all duration-500 ease-[cubic-bezier(0.16,1,0.3,1)] ${isSidebarExpanded ? 'w-72' : 'w-20'} shadow-[1px_0_10px_rgba(0,0,0,0.01)]`}>
          <div className="flex-1 py-6 overflow-y-auto px-3 scrollbar-hide">
            {navGroups.map((group) => (
              <div key={group.id} className="mb-6">
                {isSidebarExpanded && (
                  <div className="flex items-center justify-between px-4 mb-2 cursor-pointer group" onClick={() => toggleGroup(group.id)}>
                    <span className="text-[10px] font-black uppercase tracking-[0.2em] text-slate-300 group-hover:text-slate-500 transition-colors italic underline underline-offset-4 decoration-slate-100">{group.label}</span>
                    <ChevronDown size={10} className={`text-slate-300 transition-transform duration-300 ${expandedGroups.includes(group.id) ? '' : '-rotate-90'}`} />
                  </div>
                )}
                <div className={`space-y-1 transition-all ${!isSidebarExpanded || expandedGroups.includes(group.id) ? 'max-h-[1000px] opacity-100' : 'max-h-0 opacity-0 overflow-hidden'}`}>
                  {group.items.map((item) => (
                    <div key={item.id} className={`flex items-center transition-all duration-300 cursor-pointer rounded-xl group relative ${page === item.id ? 'bg-slate-900 text-white shadow-xl shadow-slate-900/20' : 'text-slate-400 hover:text-slate-900 hover:bg-slate-50'} ${isSidebarExpanded ? 'px-4 py-3' : 'justify-center p-3.5'}`} onClick={() => setPage(item.id)}>
                      <item.icon size={18} strokeWidth={page === item.id ? 2.5 : 2} className="shrink-0" />
                      {isSidebarExpanded && <span className="ml-4 text-[11px] font-bold tracking-wide">{item.label}</span>}
                      {page === item.id && !isSidebarExpanded && <div className="absolute -left-3 top-1/2 -translate-y-1/2 w-1 h-6 bg-slate-900 rounded-r-full" />}
                    </div>
                  ))}
                </div>
              </div>
            ))}
          </div>
          <div className="mt-auto border-t border-slate-50 p-4 space-y-4">
             <div className={`flex items-center p-3 rounded-xl hover:bg-rose-50 text-slate-400 hover:text-rose-600 transition-all cursor-pointer ${isSidebarExpanded ? 'px-4' : 'justify-center'}`}><LogOut size={18} />{isSidebarExpanded && <span className="ml-4 text-[11px] font-bold">Flush Session</span>}</div>
             {isSidebarExpanded && (<div className="bg-slate-50 rounded-[20px] p-5 border border-slate-100 shadow-inner overflow-hidden relative"><div className="absolute -right-4 -bottom-4 opacity-5 transform rotate-12 scale-150"><img src="/assets/images/realistic-icon-black-white-inside-vampire-cloak-with-golden-detail-vector-illustration.png" className="w-16 h-16 grayscale brightness-[5]" alt="" /></div><div className="flex justify-between items-center mb-3"><span className="text-[9px] font-black text-slate-400 uppercase tracking-[0.1em]">KERNAL_LINK</span><span className="text-[10px] font-black text-emerald-500 font-mono">ACTIVE</span></div><div className="w-full h-1 bg-slate-200 rounded-full overflow-hidden"><div className="bg-slate-900 h-full w-[94%] animate-pulse" /></div></div>)}
          </div>
        </aside>

        <main className="flex-1 p-10 overflow-y-auto bg-[#F8FAFC]" ref={containerRef}>
          <div className="max-w-[1800px] mx-auto h-full">
            <div className="flex items-center justify-between mb-10">
               <div>
                  <h2 className="text-2xl font-black text-slate-900 tracking-tight capitalize italic">{page.replace('_', ' ')} <span className="ml-4 px-2.5 py-1 bg-emerald-50 text-emerald-600 text-[10px] font-black uppercase rounded-lg tracking-widest border border-emerald-100 not-italic">LIVE_NODE</span></h2>
                  <p className="text-slate-400 text-xs mt-1 font-medium italic underline underline-offset-4 decoration-slate-100">KERNAL_AUDIT: <span className="font-mono text-slate-600 font-bold">{status?.address?.slice(0, 24)}...</span></p>
               </div>
               <div className="flex items-center space-x-3 bg-white p-2.5 rounded-3xl border border-slate-100 shadow-sm"><div className="p-2 bg-slate-50 rounded-2xl text-slate-500"><Clock size={16} /></div><div className="pr-6 border-r border-slate-50"><div className="text-[9px] font-black text-slate-400 uppercase leading-none mb-1">LOCAL_SEC_TIME</div><div className="text-xs font-bold text-slate-900 leading-none font-mono italic underline">{new Date().toLocaleTimeString()}</div></div><div className="px-6"><div className="text-[9px] font-black text-slate-400 uppercase leading-none mb-1">MESH_REACHABILITY</div><div className="flex items-center text-xs font-bold text-slate-900 leading-none"><div className="w-1.5 h-1.5 bg-emerald-500 rounded-full mr-2 shadow-[0_0_8px_rgba(16,185,129,0.5)] animate-pulse" />NOMINAL</div></div></div>
            </div>

            {page === 'dashboard' ? (
              <ResponsiveGridLayout
                className="layout"
                layouts={{ lg: layout }}
                breakpoints={{ lg: 1200, md: 996, sm: 768, xs: 480, xxs: 0 }}
                cols={{ lg: 12, md: 10, sm: 6, xs: 4, xxs: 2 }}
                rowHeight={60}
                width={containerWidth}
                isDraggable={isEditMode}
                isResizable={isEditMode}
                onLayoutChange={(newLayout: any) => setLayout(newLayout)}
              >
                {layout.map((w: any) => {
                  const moduleType = w.type || w.i.split('_')[0];
                  const catalogItem = WIDGET_CATALOG.find(c => c.id === moduleType);
                  return (
                    <div key={w.i}>
                      <Card 
                        title={catalogItem?.title || moduleType} 
                        subTitle={catalogItem?.subTitle}
                        isEditMode={isEditMode}
                        settings={widgetSettings[w.i]}
                        updateSettings={(s: any) => updateWidgetSettings(w.i, s)}
                        onRemove={() => setLayout((prev: any[]) => prev.filter((l: any) => l.i !== w.i))}
                      >
                        {Widgets[moduleType] ? React.createElement(Widgets[moduleType], { status, relays, searchQuery, filterStatus, settings: widgetSettings[w.i] }) : <div className="text-slate-300 italic text-[10px]">Component Missing</div>}
                      </Card>
                    </div>
                  );
                })}
              </ResponsiveGridLayout>
            ) : page === 'identity' ? (
              <IdentityView status={status} />
            ) : page === 'messenger' ? (
              <MessagingView />
            ) : page === 'terminal' ? (
              <TerminalView logs={structuredLogs} />
            ) : (
              <div className="flex flex-col items-center justify-center min-h-[500px] border-2 border-dashed border-slate-200 rounded-[40px] bg-white/40 shadow-inner">
                <div className="p-10 bg-white rounded-[40px] shadow-2xl border border-slate-50 text-center space-y-8 max-w-xl">
                   <div className="w-24 h-24 bg-slate-50 rounded-3xl flex items-center justify-center mx-auto border-2 border-dashed border-slate-200 group">
                      <SecurityIcon size={48} className="text-slate-300 group-hover:scale-110 transition-transform" strokeWidth={1.5} />
                   </div>
                   <div>
                      <h2 className="text-3xl font-black uppercase tracking-tight text-slate-900">Module_{page}_Standby</h2>
                      <p className="text-slate-400 text-sm leading-relaxed font-medium italic">Layer integration pending Revision 8.0 hooks.</p>
                   </div>
                </div>
              </div>
            )}
          </div>
        </main>
      </div>

      {/* PROVISIONING MODAL */}
      {showLibrary && (
        <div className="fixed inset-0 bg-slate-900/40 backdrop-blur-md z-[200] flex items-center justify-center p-8 transition-all">
          <div className="bg-white rounded-[40px] shadow-[0_50px_100px_rgba(0,0,0,0.2)] w-full max-w-5xl border border-slate-100 overflow-hidden flex flex-col max-h-[85vh]">
            <div className="p-10 border-b border-slate-100 flex justify-between items-center bg-slate-50/30">
               <div><h2 className="text-3xl font-black text-slate-900 tracking-tight italic uppercase">Provision Module</h2><p className="text-slate-400 text-sm font-medium mt-1 uppercase tracking-widest opacity-60">Provision your mesh workspace with dynamic modules</p></div>
               <button onClick={() => setShowLibrary(false)} className="p-3 hover:bg-slate-200 rounded-2xl transition-colors shadow-sm bg-white"><X size={24} className="text-slate-400" /></button>
            </div>
            <div className="flex-1 overflow-y-auto p-10 grid grid-cols-2 gap-8 scrollbar-hide">
              {WIDGET_CATALOG.map((w) => (
                <div key={w.id} className={`p-8 border border-slate-100 rounded-3xl cursor-pointer hover:border-slate-900 hover:shadow-2xl transition-all group flex items-center justify-between ${layout.find((l: any) => (l.type || l.i.split('_')[0]) === w.id) ? 'opacity-40 pointer-events-none grayscale' : 'bg-slate-50'}`} onClick={() => addWidget(w)}>
                  <div className="flex items-center space-x-8">
                    <div className="w-20 h-20 bg-white rounded-2xl border border-slate-100 flex items-center justify-center group-hover:scale-110 transition-all shadow-md text-slate-400">
                       {w.id === 'RelayAudit' && <List size={28} />}
                       {w.id === 'HardwareLoad' && <HardwareIcon size={28} />}
                       {w.id === 'NetworkStability' && <ActivityIcon size={28} />}
                       {w.id === 'MeanHandlingTime' && <Clock size={28} />}
                       {['AvailableHops', 'AvgHSTime'].includes(w.id) && <BarChart2 size={28} />}
                       {w.id === 'MeshTraffic' && <ActivityIcon size={28} />}
                       {w.id === 'SecurityAudit' && <SecurityIcon size={28} />}
                       {w.id === 'UptimeMonitor' && <Timer size={28} />}
                    </div>
                    <div>
                      <h4 className="font-black text-slate-900 uppercase tracking-widest text-xs italic">{w.title}</h4>
                      <p className="text-slate-400 text-[10px] font-bold mt-1 uppercase tracking-tighter opacity-60">{w.subTitle}</p>
                    </div>
                  </div>
                  <Plus size={20} className="text-slate-300 group-hover:text-slate-900 transition-colors" strokeWidth={3} />
                </div>
              ))}
            </div>
          </div>
        </div>
      )}

      {/* FOOTER */}
      <footer className="h-12 bg-white border-t border-slate-100 flex items-center justify-between px-8 text-[9px] font-black uppercase tracking-widest text-slate-300">
        <div className="flex space-x-12">
          <span>KERNEL_ID: <span className="font-mono text-slate-400 underline">0x{status?.address?.slice(0, 12)}</span></span>
          <span>UPTIME: <span className="text-slate-400 italic font-mono">{status?.uptime || 'INIT_BOOT'}</span></span>
        </div>
        <div className="flex items-center space-x-3 bg-slate-50 px-4 py-1.5 rounded-full border border-slate-100 shadow-sm">
          <div className="w-1.5 h-1.5 bg-emerald-500 rounded-full animate-pulse shadow-[0_0_8px_rgba(16,185,129,0.5)]" />
          <span className="text-slate-500 tracking-[0.1em]">REALTIME_KERNAL_TELEMETRY_SYNC</span>
        </div>
      </footer>
    </div>
  );
}
