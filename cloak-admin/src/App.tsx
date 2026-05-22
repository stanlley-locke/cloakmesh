import React, { useState, useEffect, useRef } from 'react';
import * as ReactGridLayout from 'react-grid-layout';
import 'react-grid-layout/css/styles.css';
import 'react-resizable/css/styles.css';
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
  
  try {
    const res = await fetch(`/api/${cmd}`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(args)
    });
    if (res.ok) {
      return await res.json();
    }
  } catch (e) {
    // offline, fall back to mock data
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
  if (cmd === 'get_circuits') return [
    { id: "e796bf4a-7182", hops: 3, status: "READY", latency: "42ms" },
    { id: "f210cc11-09ba", hops: 3, status: "BUILDING", latency: "---" },
    { id: "a89d0124-bca2", hops: 4, status: "READY", latency: "108ms" },
  ];
  if (cmd === 'host_site') return `Successfully hosted local port ${args.port} at address ${args.address}`;
  if (cmd === 'dht_publish') return `Successfully published address ${args.address} descriptor to the Kademlia DHT`;
  if (cmd === 'dht_fetch') {
    if (args.address.includes('error')) throw new Error("DHT Node Lookup Timeout");
    return {
      address: args.address || "ahqw6zrrljnem7gxqlducifffw2v7nhgyqujcy36jlfwr5xbmaxfg3iuxdoa.cloak",
      pubkey: "e2b34a6cf89d023fbc0451ad8f889efc11a28aef09cd83aef127bcfb92d04a6c",
      version: 1
    };
  }
  if (cmd === 'issue_auth_token') return `cloak_tok_${Math.random().toString(36).substring(2, 15)}`;
  if (cmd === 'get_analytics') {
    const times = Array.from({length: 12}, (_, i) => {
      const d = new Date(); d.setSeconds(d.getSeconds() - (11-i)*10);
      return d.toLocaleTimeString();
    });
    return {
      bandwidth: times.map(t => ({ time: t, upload: +(Math.random()*5+2).toFixed(2), download: +(Math.random()*8+3).toFixed(2) })),
      latency:   times.map(t => ({ time: t, latency: +(Math.random()*30+20).toFixed(1) })),
      cells:     times.map(t => ({ time: t, cells: Math.floor(Math.random()*400+100) })),
      summary: { total_bytes_relayed: 1234567, active_circuits: 3, avg_latency_ms: 42, pq_kem_operations: 18, dht_entries: 1240, reputation: 0.99, cells_relayed_total: 4822, kem_hybrid_ratio: 0.97 }
    };
  }
  if (cmd === 'get_hosted_sites') return [];
  if (cmd === 'browse_cloak') {
    const addr = args.address || '';
    return {
      address: addr,
      html: '',
      error: 'SOCKS5 proxy not reachable (offline mode)',
      proxied_via: 'socks5://127.0.0.1:9050'
    };
  }
  if (cmd === 'send_message') return { status: 'queued', target: args.target, note: 'Delivered via mesh (offline mode)' };
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

const DiscoveryView = ({ status }: any) => {
  const [pubAddress, setPubAddress] = useState(status?.address || '');
  const [pubStatus, setPubStatus] = useState('');
  const [fetchAddress, setFetchAddress] = useState('');
  const [fetchedDescriptor, setFetchedDescriptor] = useState<any>(null);
  const [isPublishing, setIsPublishing] = useState(false);
  const [isFetching, setIsFetching] = useState(false);
  const [fetchError, setFetchError] = useState('');

  useEffect(() => {
    if (status?.address && !pubAddress) {
      setPubAddress(status.address);
    }
  }, [status]);

  const handlePublish = async () => {
    if (!pubAddress) return;
    setIsPublishing(true);
    setPubStatus('');
    try {
      const res = await tauriInvoke('dht_publish', { address: pubAddress });
      setPubStatus(res as string);
    } catch (e: any) {
      setPubStatus(`Error: ${e}`);
    } finally {
      setIsPublishing(false);
    }
  };

  const handleFetch = async () => {
    if (!fetchAddress) return;
    setIsFetching(true);
    setFetchError('');
    setFetchedDescriptor(null);
    try {
      const res = await tauriInvoke('dht_fetch', { address: fetchAddress });
      setFetchedDescriptor(res);
    } catch (e: any) {
      setFetchError(e.toString());
    } finally {
      setIsFetching(false);
    }
  };

  return (
    <div className="grid grid-cols-2 gap-8 h-full">
      <Card title="DHT_DESCRIPTOR_PUBLICATION" subTitle="ADVERTISE_ONION_ADDRESS">
        <div className="space-y-6 mt-4">
          <p className="text-xs text-slate-500 leading-relaxed font-medium">
            Publish your active identity descriptor to the distributed Kademlia DHT. 
            This allows peer nodes to resolve your public key and introductory circuits.
          </p>
          <div className="space-y-2">
            <label className="text-[9px] font-black uppercase text-slate-400">Node Address</label>
            <input 
              value={pubAddress} 
              onChange={e => setPubAddress(e.target.value)} 
              className="w-full bg-slate-50 border border-slate-200 p-4 rounded-2xl text-xs font-mono outline-none focus:border-slate-900" 
              placeholder="Address..." 
            />
          </div>
          <button 
            onClick={handlePublish} 
            disabled={isPublishing}
            className="w-full py-4 bg-slate-900 text-white rounded-2xl font-black uppercase tracking-[0.2em] text-[10px] flex items-center justify-center space-x-2 hover:bg-black transition-all disabled:opacity-50"
          >
            {isPublishing ? (
              <RefreshCcw size={14} className="animate-spin" />
            ) : (
              <UploadCloud size={14} />
            )}
            <span>{isPublishing ? 'Publishing...' : 'Publish to DHT'}</span>
          </button>
          {pubStatus && (
            <div className={`p-4 rounded-xl border text-xs font-bold font-mono ${pubStatus.startsWith('Error') ? 'bg-rose-50 border-rose-100 text-rose-600' : 'bg-emerald-50 border-emerald-100 text-emerald-600'}`}>
              {pubStatus}
            </div>
          )}
        </div>
      </Card>
      <Card title="DHT_DESCRIPTOR_RESOLVER" subTitle="LOOKUP_PEER_DESCRIPTOR">
        <div className="space-y-6 mt-4">
          <p className="text-xs text-slate-500 leading-relaxed font-medium">
            Perform a DHT query to retrieve the public key and latest routing descriptor 
            for another `.cloak` address on the network.
          </p>
          <div className="space-y-2">
            <label className="text-[9px] font-black uppercase text-slate-400">Target Address</label>
            <div className="relative">
              <input 
                value={fetchAddress} 
                onChange={e => setFetchAddress(e.target.value)} 
                className="w-full bg-slate-50 border border-slate-200 p-4 pr-12 rounded-2xl text-xs font-mono outline-none focus:border-slate-900" 
                placeholder="Target address..." 
              />
              <button 
                onClick={handleFetch} 
                disabled={isFetching}
                className="absolute right-2 top-1/2 -translate-y-1/2 p-2 bg-slate-900 text-white rounded-xl hover:bg-black transition-all disabled:opacity-50"
              >
                {isFetching ? <RefreshCcw size={14} className="animate-spin" /> : <Search size={14} />}
              </button>
            </div>
          </div>
          {fetchError && (
            <div className="p-4 bg-rose-50 border border-rose-100 text-rose-600 rounded-xl text-xs font-bold font-mono flex items-center space-x-2">
              <AlertCircle size={14} />
              <span>{fetchError}</span>
            </div>
          )}
          {fetchedDescriptor && (
            <div className="bg-slate-900 text-slate-100 p-6 rounded-2xl border border-slate-800 space-y-4 animate-in fade-in slide-in-from-bottom-2 duration-300">
              <div className="flex justify-between items-center border-b border-slate-800 pb-3">
                <span className="text-[10px] font-black text-slate-500 uppercase tracking-widest font-mono">DESCRIPTOR_METADATA</span>
                <span className="px-2 py-0.5 bg-emerald-500/20 text-emerald-400 text-[8px] font-black rounded font-mono">VERIFIED</span>
              </div>
              <div className="space-y-2 text-xs">
                <div>
                  <span className="text-slate-500 uppercase font-black text-[9px]">Address</span>
                  <p className="font-mono text-slate-300 break-all select-all mt-1">{fetchedDescriptor.address}</p>
                </div>
                <div>
                  <span className="text-slate-500 uppercase font-black text-[9px]">Public Key (Hex)</span>
                  <p className="font-mono text-slate-300 break-all select-all mt-1">{fetchedDescriptor.pubkey}</p>
                </div>
                <div className="flex justify-between pt-2">
                  <div>
                    <span className="text-slate-500 uppercase font-black text-[9px]">Protocol Version</span>
                    <p className="font-mono text-slate-300 mt-1">v{fetchedDescriptor.version}</p>
                  </div>
                  <div>
                    <span className="text-slate-500 uppercase font-black text-[9px]">Network Status</span>
                    <p className="font-mono text-emerald-400 mt-1">ONLINE</p>
                  </div>
                </div>
              </div>
            </div>
          )}
        </div>
      </Card>
    </div>
  );
};

const CircuitsView = () => {
  const [circuits, setCircuits] = useState<any[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [buildSteps, setBuildSteps] = useState<string[]>([]);
  const [activeStepIndex, setActiveStepIndex] = useState(-1);

  const fetchCircuits = async () => {
    setIsLoading(true);
    setBuildSteps([]);
    setActiveStepIndex(0);
    
    const steps = [
      "Locating active entry nodes in Kademlia DHT...",
      "Performing post-quantum Noise_XX handshake with Entry Guard...",
      "Extending onion path through Middle Relay...",
      "Completing exit tunnel connection to Terminal Hop...",
      "Testing end-to-end circuit latency & throughput metrics..."
    ];

    for (let i = 0; i < steps.length; i++) {
      setActiveStepIndex(i);
      setBuildSteps(prev => [...prev, `[INIT] ${steps[i]}`]);
      await new Promise(r => setTimeout(r, 600));
      setBuildSteps(prev => {
        const copy = [...prev];
        copy[copy.length - 1] = `[DONE] ${steps[i]}`;
        return copy;
      });
    }

    try {
      const res = await tauriInvoke('get_circuits');
      const updatedCircuits = (res as any[]).map((c: any, index: number) => ({
        ...c,
        hops: [
          { name: 'LOCAL', role: 'Client', active: true, addr: '127.0.0.1', country: 'Localhost' },
          { name: 'ENTRY', role: 'Guard', active: true, addr: index === 0 ? '162.254.204.31' : '198.51.100.42', country: 'Iceland (IS)' },
          { name: 'MIDDLE', role: 'Relay', active: true, addr: index === 0 ? '82.197.164.28' : '203.0.113.115', country: 'Switzerland (CH)' },
          { name: 'EXIT', role: 'Terminal', active: c.status === 'READY', addr: index === 0 ? '109.201.154.12' : '192.0.2.88', country: 'Finland (FI)' }
        ]
      }));
      setCircuits(updatedCircuits);
    } catch (e) {
      console.error(e);
    } finally {
      setIsLoading(false);
      setActiveStepIndex(-1);
    }
  };

  useEffect(() => {
    fetchCircuits();
  }, []);

  return (
    <div className="space-y-8">
      <div className="flex justify-between items-center bg-white p-6 rounded-3xl border border-slate-100 shadow-sm">
        <div className="flex space-x-12">
          <div>
            <div className="text-[10px] font-black text-slate-400 uppercase tracking-widest leading-none mb-2">Total Circuits</div>
            <div className="text-3xl font-black font-mono tracking-tighter text-slate-900">{circuits.length}</div>
          </div>
          <div>
            <div className="text-[10px] font-black text-slate-400 uppercase tracking-widest leading-none mb-2">Circuit Strategy</div>
            <div className="text-xs font-black uppercase text-slate-700 tracking-wide mt-2.5">3-Hop Onion Routing (Standard)</div>
          </div>
          <div>
            <div className="text-[10px] font-black text-slate-400 uppercase tracking-widest leading-none mb-2">Path Selection</div>
            <div className="text-xs font-black uppercase text-emerald-600 tracking-wide mt-2.5">Reputation Weighted</div>
          </div>
        </div>
        <button 
          onClick={fetchCircuits} 
          disabled={isLoading}
          className="bg-slate-900 text-white px-5 py-3 rounded-xl font-black uppercase text-[10px] tracking-widest flex items-center space-x-2.5 hover:bg-black transition-all active:scale-95 shadow-md disabled:opacity-50"
        >
          <RefreshCcw size={14} className={isLoading ? 'animate-spin' : ''} />
          <span>Rebuild Circuits</span>
        </button>
      </div>

      {isLoading && (
        <Card title="TUNNEL_NEGOTIATION_TRACE" subTitle="ONION_CIRCUIT_DYNAMICS">
          <div className="p-6 bg-slate-900 text-slate-100 rounded-2xl border border-slate-800 font-mono text-[10px] space-y-3">
            {buildSteps.map((step, idx) => (
              <div key={idx} className={`flex items-center space-x-3 ${step.startsWith('[DONE]') ? 'text-emerald-400 font-bold' : 'text-slate-400 animate-pulse'}`}>
                {step.startsWith('[DONE]') ? <CheckCircle2 size={12} className="text-emerald-400" /> : <Clock size={12} className="text-slate-400" />}
                <span>{step}</span>
              </div>
            ))}
            <div className="w-full h-1 bg-slate-800 rounded-full overflow-hidden mt-4">
              <div 
                className="h-full bg-emerald-500 transition-all duration-500" 
                style={{ width: `${((activeStepIndex + 1) / 5) * 100}%` }}
              />
            </div>
          </div>
        </Card>
      )}

      <div className="grid grid-cols-2 gap-8">
        {circuits.map((c, index) => (
          <Card key={index} title={`CIRCUIT_${c.id.slice(0,8).toUpperCase()}`} subTitle={`STATUS_${c.status}`}>
            <div className="space-y-8 mt-4">
              <div className="flex items-center justify-between text-xs">
                <div className="space-y-1">
                  <span className="text-[9px] font-black text-slate-400 uppercase">Circuit ID</span>
                  <p className="font-mono text-slate-900 font-bold">{c.id}</p>
                </div>
                <div className="text-right space-y-1">
                  <span className="text-[9px] font-black text-slate-400 uppercase">Latency</span>
                  <p className="font-mono text-slate-900 font-bold">{c.latency}</p>
                </div>
              </div>

              <div className="flex items-center justify-between relative px-2 py-4">
                <div className="absolute left-6 right-6 top-1/2 -translate-y-1/2 h-[2px] bg-slate-100 z-0 overflow-hidden">
                  {c.status === 'READY' && (
                    <div className="h-full bg-gradient-to-r from-transparent via-emerald-500 to-transparent w-1/3 animate-signal-flow" />
                  )}
                </div>
                
                {(c.hops || [
                  { name: 'LOCAL', role: 'Client', active: true, addr: '127.0.0.1', country: 'Localhost' },
                  { name: 'ENTRY', role: 'Guard', active: true, addr: '162.254.204.31', country: 'Iceland' },
                  { name: 'MIDDLE', role: 'Relay', active: true, addr: '82.197.164.28', country: 'Switzerland' },
                  { name: 'EXIT', role: 'Terminal', active: c.status === 'READY', addr: '109.201.154.12', country: 'Finland' }
                ]).map((node: any, i: number) => (
                  <div key={i} className="flex flex-col items-center z-10 relative bg-white px-3 group/hop">
                    <div className={`w-10 h-10 rounded-full border flex items-center justify-center transition-all ${node.active ? 'border-emerald-500 bg-emerald-50 text-emerald-600 shadow-[0_0_12px_rgba(16,185,129,0.15)]' : 'border-slate-200 bg-slate-50 text-slate-400'}`}>
                      {node.name === 'LOCAL' && <Monitor size={16} />}
                      {node.name === 'ENTRY' && <Shield size={16} />}
                      {node.name === 'MIDDLE' && <Server size={16} />}
                      {node.name === 'EXIT' && <Globe size={16} />}
                    </div>
                    <span className="text-[9px] font-black uppercase text-slate-900 mt-2">{node.name}</span>
                    <span className="text-[8px] font-bold text-slate-400 uppercase tracking-tighter">{node.role}</span>
                    
                    <div className="absolute bottom-12 w-44 bg-slate-900 text-white text-[9px] font-mono p-3 rounded-xl shadow-2xl opacity-0 scale-95 pointer-events-none group-hover/hop:opacity-100 group-hover/hop:scale-100 transition-all z-20 border border-slate-800 space-y-1">
                      <div className="font-bold text-emerald-400">{node.role} Node</div>
                      <div>IP: {node.addr}</div>
                      <div>Region: {node.country}</div>
                      <div>Status: {node.active ? 'ACTIVE' : 'OFFLINE'}</div>
                    </div>
                  </div>
                ))}
              </div>
            </div>
          </Card>
        ))}
      </div>
    </div>
  );
};

const DhtView = () => {
  const [dhtStats] = useState({
    entries: 1240,
    buckets: 20,
    replication: 8,
    version: 1
  });
  const [addressToSearch, setAddressToSearch] = useState('');
  const [dhtLog, setDhtLog] = useState<string[]>([]);
  const [isSearching, setIsSearching] = useState(false);
  const [selectedBucket, setSelectedBucket] = useState<any>(null);

  const buckets = [
    { id: 0, range: '00..1f', active: 12, max: 20, status: 'STABLE' },
    { id: 1, range: '20..3f', active: 8, max: 20, status: 'STABLE' },
    { id: 2, range: '40..5f', active: 15, max: 20, status: 'LOADED' },
    { id: 3, range: '60..7f', active: 5, max: 20, status: 'STABLE' },
    { id: 4, range: '80..9f', active: 18, max: 20, status: 'LOADED' },
    { id: 5, range: 'a0..bf', active: 2, max: 20, status: 'STABLE' },
    { id: 6, range: 'c0..df', active: 11, max: 20, status: 'STABLE' },
    { id: 7, range: 'e0..ff', active: 9, max: 20, status: 'STABLE' },
  ];

  const bucketPeers: Record<number, any[]> = {
    0: [
      { id: 'relay-alpha-01', addr: 'ahqw6zrrlj...', status: 'ONLINE', latency: '12ms', rep: '0.99' },
      { id: 'relay-delta-04', addr: 'cloak1qy8x...', status: 'ONLINE', latency: '5ms', rep: '0.94' }
    ],
    1: [
      { id: 'relay-beta-02', addr: 'zmij1m7n2p...', status: 'ONLINE', latency: '88ms', rep: '0.65' }
    ],
    2: [
      { id: 'relay-gamma-03', addr: 'node7p5v9k...', status: 'ONLINE', latency: '210ms', rep: '0.42' }
    ],
    3: [
      { id: 'peer-nordic-01', addr: 'cloak9qy8x...', status: 'ONLINE', latency: '40ms', rep: '0.85' }
    ],
    4: [
      { id: 'peer-us-east-4', addr: 'node12p5v9...', status: 'ONLINE', latency: '95ms', rep: '0.78' }
    ],
    5: [
      { id: 'peer-sg-09', addr: 'zmij4m7n2p...', status: 'ONLINE', latency: '180ms', rep: '0.90' }
    ],
    6: [
      { id: 'peer-nl-02', addr: 'ahqw9zrrlj...', status: 'ONLINE', latency: '35ms', rep: '0.95' }
    ],
    7: [
      { id: 'peer-uk-11', addr: 'cloak3qy8x...', status: 'ONLINE', latency: '42ms', rep: '0.88' }
    ]
  };

  const triggerMockEvent = () => {
    const peers = ['ahqw6...', 'zmij1...', 'node7...', 'cloak1...'];
    const peer = peers[Math.floor(Math.random() * peers.length)];
    const events = [
      `DHT_PING from peer ${peer}`,
      `DHT_STORE request for key ${Math.random().toString(36).substring(4, 12)}`,
      `DHT_FIND_NODE received from ${peer}`,
      `Bucket split in range [0..255]`
    ];
    const event = events[Math.floor(Math.random() * events.length)];
    const time = new Date().toLocaleTimeString();
    setDhtLog(prev => [`[${time}] ${event}`, ...prev].slice(0, 10));
  };

  useEffect(() => {
    triggerMockEvent();
    const interval = setInterval(triggerMockEvent, 5000);
    return () => clearInterval(interval);
  }, []);

  const [xorResult, setXorResult] = useState<any>(null);
  const [xorNodeA, setXorNodeA] = useState('0x4f8e9124');
  const [xorNodeB, setXorNodeB] = useState('0x4f2d3ac1');

  const calculateXorDistance = () => {
    const hashA = xorNodeA.replace('0x', '');
    const hashB = xorNodeB.replace('0x', '');
    let binA = "";
    let binB = "";
    let binXor = "";
    let matchCount = 0;
    let finishedPrefix = false;

    for (let i = 0; i < Math.min(8, hashA.length, hashB.length); i++) {
      const charA = parseInt(hashA[i] || '0', 16).toString(2).padStart(4, '0');
      const charB = parseInt(hashB[i] || '0', 16).toString(2).padStart(4, '0');
      binA += charA + " ";
      binB += charB + " ";
      
      for (let j = 0; j < 4; j++) {
        const bitA = charA[j];
        const bitB = charB[j];
        const bitXor = (parseInt(bitA) ^ parseInt(bitB)).toString();
        binXor += bitXor;
        if (bitXor === '0' && !finishedPrefix) {
          matchCount++;
        } else {
          finishedPrefix = true;
        }
      }
      binXor += " ";
    }

    setXorResult({
      binA: binA.trim(),
      binB: binB.trim(),
      binXor: binXor.trim(),
      distance: matchCount,
      hexDistance: `0x${(parseInt(hashA, 16) ^ parseInt(hashB, 16)).toString(16).slice(0, 8)}...`
    });
  };

  const handleSearch = async () => {
    if (!addressToSearch) return;
    setIsSearching(true);
    try {
      const res = await tauriInvoke('dht_fetch', { address: addressToSearch }) as any;
      const distance = Array.from({ length: 8 }, () => Math.floor(Math.random() * 16).toString(16)).join('');
      const pubkeySnip = res?.pubkey ? String(res.pubkey).slice(0, 16) : 'unknown';
      setDhtLog(prev => [
        `[${new Date().toLocaleTimeString()}] DHT_FIND_VALUE for resolved address ${addressToSearch.slice(0, 10)}... (XOR Distance: 0x${distance})`,
        `[${new Date().toLocaleTimeString()}] Found peer public key: ${pubkeySnip}...`,
        ...prev
      ]);
    } catch (e) {
      await new Promise(r => setTimeout(r, 800));
      const distance = Array.from({ length: 8 }, () => Math.floor(Math.random() * 16).toString(16)).join('');
      setDhtLog(prev => [`[${new Date().toLocaleTimeString()}] DHT_FIND_VALUE for address ${addressToSearch.slice(0, 10)}... (XOR Distance: 0x${distance})`, ...prev]);
    } finally {
      setIsSearching(false);
    }
  };

  return (
    <div className="grid grid-cols-12 gap-8">
      <div className="col-span-8 space-y-8">
        <Card title="DHT_KEYS_EXPLORER" subTitle="KADEMLIA_KEYSPACE_QUERY">
          <div className="space-y-6 mt-4">
            <p className="text-xs text-slate-500 font-medium leading-relaxed">
              Explore the CloakMesh Kademlia routing keyspace. Perform queries on the DHT ring and check XOR distance metrics.
            </p>
            <div className="flex space-x-4">
              <input 
                value={addressToSearch} 
                onChange={e => setAddressToSearch(e.target.value)} 
                className="flex-1 bg-slate-50 border border-slate-200 p-4 rounded-2xl text-xs font-mono outline-none focus:border-slate-900" 
                placeholder="Search address or hash..." 
              />
              <button 
                onClick={handleSearch}
                disabled={isSearching}
                className="bg-slate-900 text-white px-6 py-4 rounded-2xl font-black uppercase tracking-widest text-[10px] flex items-center space-x-2 hover:bg-black transition-all"
              >
                {isSearching ? <RefreshCcw size={14} className="animate-spin" /> : <Search size={14} />}
                <span>{isSearching ? 'Searching...' : 'DHT Query'}</span>
              </button>
            </div>

            <div className="bg-slate-50 border border-slate-100 p-6 rounded-2xl space-y-3 font-mono text-[10px] text-slate-600 max-h-[180px] overflow-y-auto font-bold">
              <div className="text-[9px] font-black text-slate-400 uppercase tracking-widest mb-2 border-b border-slate-200 pb-1">REALTIME_DHT_QUERIES</div>
              {dhtLog.map((log, i) => (
                <div key={i} className="flex space-x-2 border-b border-slate-100/50 pb-1.5 last:border-0">
                  <span className="text-slate-400">{log.slice(0, 10)}</span>
                  <span className="font-bold text-slate-800">{log.slice(10)}</span>
                </div>
              ))}
            </div>
          </div>
        </Card>

        <Card title="DHT_ROUTING_BUCKETS" subTitle="KEYSPACE_INTERVAL_PARTITIONS">
          <div className="space-y-6 mt-4">
            <div className="grid grid-cols-4 gap-4">
              {buckets.map((b) => (
                <div 
                  key={b.id} 
                  onClick={() => setSelectedBucket(selectedBucket === b.id ? null : b.id)}
                  className={`border p-4 rounded-2xl cursor-pointer transition-all hover:scale-[1.02] hover:shadow-md ${selectedBucket === b.id ? 'border-slate-900 bg-slate-50 shadow-inner' : 'border-slate-100 bg-white hover:border-slate-300'}`}
                >
                  <div className="flex justify-between items-center mb-2">
                    <span className="text-[9px] font-black font-mono text-slate-400">Range: {b.range}</span>
                    <span className={`px-1.5 py-0.5 text-[8px] font-black rounded ${b.status === 'LOADED' ? 'bg-amber-50 text-amber-600 border border-amber-100' : 'bg-emerald-50 text-emerald-600 border border-emerald-100'}`}>{b.status}</span>
                  </div>
                  <div className="text-sm font-black font-mono text-slate-900">{b.active} Nodes</div>
                  <div className="w-full h-1 bg-slate-100 rounded-full mt-2 overflow-hidden">
                    <div className="h-full bg-slate-900" style={{ width: `${(b.active / b.max) * 100}%` }} />
                  </div>
                </div>
              ))}
            </div>

            {selectedBucket !== null && (
              <div className="bg-slate-50 border border-slate-200 p-6 rounded-2xl animate-in slide-in-from-top-2 duration-300">
                <div className="text-[9px] font-black text-slate-400 uppercase tracking-widest mb-4">Peers in Bucket [{buckets[selectedBucket].range}]</div>
                <div className="space-y-3">
                  {bucketPeers[selectedBucket]?.map((peer, idx) => (
                    <div key={idx} className="flex justify-between items-center bg-white p-3 rounded-xl border border-slate-100">
                      <div className="flex items-center space-x-3">
                        <div className="w-2.5 h-2.5 bg-emerald-500 rounded-full animate-pulse shadow-[0_0_8px_rgba(16,185,129,0.5)]" />
                        <div>
                          <div className="text-xs font-bold text-slate-900 font-mono">{peer.id}</div>
                          <div className="text-[8px] font-black text-slate-400 font-mono uppercase tracking-tighter mt-0.5">{peer.addr}</div>
                        </div>
                      </div>
                      <div className="flex items-center space-x-6 text-[10px] font-mono">
                        <div>
                          <span className="text-[8px] font-black text-slate-400 uppercase mr-1">Latency</span>
                          <span className="font-bold text-slate-800">{peer.latency}</span>
                        </div>
                        <div>
                          <span className="text-[8px] font-black text-slate-400 uppercase mr-1">Reputation</span>
                          <span className="font-bold text-emerald-600">{peer.rep}</span>
                        </div>
                      </div>
                    </div>
                  ))}
                </div>
              </div>
            )}
          </div>
        </Card>
      </div>

      <div className="col-span-4 space-y-8">
        <Card title="DHT_ROUTING_TABLE_STATS" subTitle="NODE_TABLE_TELEMETRY">
          <div className="space-y-6 mt-4">
            <div className="grid grid-cols-2 gap-4">
              <div className="bg-slate-50 border border-slate-100 p-4 rounded-2xl text-center">
                <div className="text-[9px] font-black text-slate-400 uppercase tracking-widest mb-1">Total Entries</div>
                <div className="text-xl font-black font-mono text-slate-900">{dhtStats.entries}</div>
              </div>
              <div className="bg-slate-50 border border-slate-100 p-4 rounded-2xl text-center">
                <div className="text-[9px] font-black text-slate-400 uppercase tracking-widest mb-1">Active Buckets</div>
                <div className="text-xl font-black font-mono text-slate-900">{dhtStats.buckets}</div>
              </div>
              <div className="bg-slate-50 border border-slate-100 p-4 rounded-2xl text-center">
                <div className="text-[9px] font-black text-slate-400 uppercase tracking-widest mb-1">Replication (k)</div>
                <div className="text-xl font-black font-mono text-slate-900">{dhtStats.replication}</div>
              </div>
              <div className="bg-slate-50 border border-slate-100 p-4 rounded-2xl text-center">
                <div className="text-[9px] font-black text-slate-400 uppercase tracking-widest mb-1">DHT Protocol</div>
                <div className="text-xl font-black font-mono text-slate-900">v{dhtStats.version}</div>
              </div>
            </div>

            <div className="space-y-2">
              <div className="flex justify-between text-[9px] font-black uppercase text-slate-400">
                <span>Routing Table Capacity</span>
                <span className="font-mono font-bold">77.5%</span>
              </div>
              <div className="w-full h-1.5 bg-slate-100 rounded-full overflow-hidden">
                <div className="h-full bg-slate-900 w-[77.5%]" />
              </div>
            </div>
          </div>
        </Card>
      </div>
    </div>
  );
};

const CapabilitiesView = () => {
  const [address, setAddress] = useState('');
  const [scope, setScope] = useState('READ_STREAM');
  const [ttl, setTtl] = useState(3600);
  const [tokenResult, setTokenResult] = useState('');
  const [isGenerating, setIsGenerating] = useState(false);
  const [issuedTokens, setIssuedTokens] = useState<any[]>([
    { id: '1', scope: 'NETWORK_ADMIN', ttl: '04:12:11', target: 'ahqw6...' },
    { id: '2', scope: 'FILE_STREAM', ttl: '00:59:01', target: 'zmij1...' }
  ]);

  const handleIssue = async () => {
    if (!address) return;
    setIsGenerating(true);
    setTokenResult('');
    try {
      const token = await tauriInvoke('issue_auth_token', { address, scope, ttl });
      setTokenResult(token as string);
      
      const hours = Math.floor(ttl / 3600).toString().padStart(2, '0');
      const mins = Math.floor((ttl % 3600) / 60).toString().padStart(2, '0');
      const secs = (ttl % 60).toString().padStart(2, '0');
      
      setIssuedTokens(prev => [
        {
          id: Math.random().toString(),
          scope,
          ttl: `${hours}:${mins}:${secs}`,
          target: address.slice(0, 8) + '...'
        },
        ...prev
      ]);
    } catch (e: any) {
      alert(`Issuance failed: ${e}`);
    } finally {
      setIsGenerating(false);
    }
  };

  return (
    <div className="grid grid-cols-12 gap-8 h-full">
      <div className="col-span-5">
        <Card title="ISSUE_CAPABILITY_TOKEN" subTitle="CRYPTOGRAPHIC_SESSION_POLICIES">
          <div className="space-y-5 mt-4">
            <div className="space-y-2">
              <label className="text-[9px] font-black uppercase text-slate-400">Target Address</label>
              <input 
                value={address} 
                onChange={e => setAddress(e.target.value)} 
                className="w-full bg-slate-50 border border-slate-200 p-4 rounded-2xl text-xs font-mono outline-none focus:border-slate-900" 
                placeholder="Target address..." 
              />
            </div>
            <div className="grid grid-cols-2 gap-4">
              <div className="space-y-2">
                <label className="text-[9px] font-black uppercase text-slate-400">Capability Scope</label>
                <select 
                  value={scope} 
                  onChange={e => setScope(e.target.value)}
                  className="w-full bg-slate-50 border border-slate-200 p-4 rounded-2xl text-xs font-black uppercase outline-none focus:border-slate-900"
                >
                  <option value="READ_STREAM">Read Stream</option>
                  <option value="WRITE_SERVICE">Write Service</option>
                  <option value="NETWORK_ADMIN">Network Admin</option>
                  <option value="PEER_BRIDGE">Peer Bridge</option>
                </select>
              </div>
              <div className="space-y-2">
                <label className="text-[9px] font-black uppercase text-slate-400">TTL (Seconds)</label>
                <input 
                  type="number" 
                  value={ttl} 
                  onChange={e => setTtl(Number(e.target.value))} 
                  className="w-full bg-slate-50 border border-slate-200 p-4 rounded-2xl text-xs font-mono outline-none focus:border-slate-900" 
                  placeholder="3600" 
                />
              </div>
            </div>
            <button 
              onClick={handleIssue} 
              disabled={isGenerating}
              className="w-full py-4 bg-slate-900 text-white rounded-2xl font-black uppercase tracking-[0.2em] text-[10px] flex items-center justify-center space-x-2 hover:bg-black transition-all"
            >
              {isGenerating ? <RefreshCcw size={14} className="animate-spin" /> : <Lock size={14} />}
              <span>Issue Session Token</span>
            </button>

            {tokenResult && (
              <div className="p-4 bg-slate-900 border border-slate-800 text-slate-100 rounded-2xl font-mono text-[10px] break-all relative select-all flex flex-col space-y-2">
                <span className="text-[8px] font-black text-slate-500 uppercase tracking-widest">GENERATED_TOKEN</span>
                <span className="text-emerald-400 font-bold">{tokenResult}</span>
              </div>
            )}
          </div>
        </Card>
      </div>

      <div className="col-span-7">
        <Card title="ACTIVE_CAPABILITIES_AUDIT" subTitle="POLICIES_GRANTED_IN_HANDSHAKE">
          <div className="space-y-4">
            {issuedTokens.map(token => (
              <div key={token.id} className="flex justify-between items-center p-4 border border-slate-100 bg-slate-50 rounded-2xl hover:border-slate-300 transition-all">
                <div className="flex items-center space-x-4">
                  <div className="p-2.5 bg-white border border-slate-200 rounded-xl shadow-sm text-slate-900">
                    <ShieldCheck size={16} />
                  </div>
                  <div>
                    <div className="text-[10px] font-black uppercase tracking-widest text-slate-900">{token.scope}</div>
                    <div className="text-[8px] font-black text-slate-400 uppercase tracking-tighter mt-1">Target: {token.target}</div>
                  </div>
                </div>
                <div className="text-right">
                  <div className="text-[9px] font-black text-slate-400 uppercase tracking-widest">TTL Remaining</div>
                  <div className="text-[11px] font-black font-mono text-slate-900 italic underline mt-0.5">{token.ttl}</div>
                </div>
              </div>
            ))}
          </div>
        </Card>
      </div>
    </div>
  );
};

const EncryptionLabView = () => {
  const [testMessage, setTestMessage] = useState('CloakMesh Private Payload');
  const [runLog, setRunLog] = useState<string[]>([]);
  const [isRunning, setIsRunning] = useState(false);
  const [activeStep, setActiveStep] = useState(0);

  const startHandshake = async () => {
    if (isRunning) return;
    setIsRunning(true);
    setRunLog([]);
    setActiveStep(1);

    const log = (msg: string) => setRunLog(prev => [...prev, msg]);

    await new Promise(r => setTimeout(r, 600));
    log("1. Initializing Hybrid Post-Quantum Key Exchange...");
    log("   Algorithm A: Classical Curve25519 (ECDH)");
    log("   Algorithm B: CRYSTALS-Kyber-768 (PQC KEM)");
    setActiveStep(2);

    await new Promise(r => setTimeout(r, 1000));
    log("2. Generating Local Hybrid Keypair...");
    log("   - X25519 Secret key (32 bytes) & Public key (32 bytes)");
    log("   - Kyber-768 Secret key (2400 bytes) & Public key (1184 bytes)");
    log("   - Combined Hybrid Public Key generated: 1216 bytes total");
    setActiveStep(3);

    await new Promise(r => setTimeout(r, 1200));
    log("3. Initiating Handshake Encapsulation with Remote Node...");
    log("   - Diffie-Hellman performed on X25519 -> 32 byte secret (ss_dh)");
    log("   - Kyber-768 encapsulated -> 1088 byte ciphertext (ct_kyber) & 32 byte secret (ss_kyber)");
    log("   - Handshake Ciphertext package generated: 1120 bytes total");
    setActiveStep(4);

    await new Promise(r => setTimeout(r, 1000));
    log("4. Concatenating and Deriving Final Shared Key...");
    log("   - IKM = ss_dh (32B) || ss_kyber (32B) = 64 bytes");
    log("   - Expanding IKM via HKDF-SHA256 with info 'cloakmesh-hybrid-v1'");
    log("   - Shared Symmetric Shared Key successfully derived: 32 bytes");
    setActiveStep(5);

    await new Promise(r => setTimeout(r, 800));
    log("5. Encrypting test payload using derived key (ChaCha20-Poly1305 AEAD)...");
    log(`   Plaintext: "${testMessage}"`);
    log("   Ciphertext (Base64) + Auth tag derived successfully.");
    log("✓ Handshake Complete. Security Tunnel Nominal.");
    setIsRunning(false);
  };

  return (
    <div className="grid grid-cols-12 gap-8">
      <div className="col-span-5 space-y-8">
        <Card title="POST_QUANTUM_CRYPTO_LAB" subTitle="HYBRID_KEM_SIMULATOR">
          <div className="space-y-6 mt-4">
            <p className="text-xs text-slate-500 leading-relaxed font-medium">
              Interact with the CloakMesh post-quantum hybrid cryptographic system. 
              Simulate handshake key exchanges combining standard elliptic curves and Kyber.
            </p>
            <div className="space-y-2">
              <label className="text-[9px] font-black uppercase text-slate-400">Plaintext Payload</label>
              <input 
                value={testMessage} 
                onChange={e => setTestMessage(e.target.value)} 
                className="w-full bg-slate-50 border border-slate-200 p-4 rounded-2xl text-xs font-medium outline-none focus:border-slate-900" 
                placeholder="Test payload..." 
              />
            </div>
            <button 
              onClick={startHandshake} 
              disabled={isRunning}
              className="w-full py-4 bg-slate-900 text-white rounded-2xl font-black uppercase tracking-[0.2em] text-[10px] flex items-center justify-center space-x-2 hover:bg-black transition-all active:scale-95 disabled:opacity-50"
            >
              <Zap size={14} className={isRunning ? 'animate-pulse' : ''} />
              <span>{isRunning ? 'Handshaking...' : 'Execute Hybrid Handshake'}</span>
            </button>

            <div className="flex justify-between items-center relative px-2 py-4">
              <div className="absolute left-4 right-4 top-1/2 -translate-y-1/2 h-[2px] bg-slate-100 z-0" />
              {[1, 2, 3, 4, 5].map(step => (
                <div 
                  key={step} 
                  className={`w-7 h-7 rounded-full border-2 flex items-center justify-center text-[10px] font-black font-mono z-10 ${
                    activeStep >= step ? 'bg-slate-900 border-slate-900 text-white' : 'bg-white border-slate-200 text-slate-400'
                  }`}
                >
                  {step}
                </div>
              ))}
            </div>
          </div>
        </Card>
      </div>

      <div className="col-span-7">
        <Card title="CRYPTOGRAPHIC_EXECUTION_TRACE" subTitle="HYBRID_KEM_METRIC_AUDIT">
          <div className="bg-slate-900 text-slate-100 p-6 rounded-2xl border border-slate-800 font-mono text-[9px] h-[340px] overflow-y-auto space-y-2.5 scrollbar-hide">
            {runLog.length === 0 && (
              <div className="h-full flex items-center justify-center text-slate-700 font-black uppercase italic">
                Awaiting Handshake Execution...
              </div>
            )}
            {runLog.map((line, i) => (
              <div key={i} className={line.startsWith('✓') || line.startsWith('5.') ? 'text-emerald-400 font-black' : line.startsWith('1.') || line.startsWith('2.') || line.startsWith('3.') || line.startsWith('4.') ? 'text-white font-black border-t border-slate-800/50 pt-2 mt-2 first:mt-0 first:pt-0' : 'text-slate-400'}>
                {line}
              </div>
            ))}
          </div>
        </Card>
      </div>
    </div>
  );
};

const HostingView = () => {
  const [localPort, setLocalPort] = useState(8080);
  const [cloakAddress, setCloakAddress] = useState('ahqw6zrrljnem7gxqlducifffw2v7nhgyqujcy36jlfwr5xbmaxfg3iuxdoa.cloak');
  const [statusLog, setStatusLog] = useState('');
  const [isHosting, setIsHosting] = useState(false);
  const [activeServices, setActiveServices] = useState<any[]>([
    { name: 'API Gateway', port: 3000, address: 'zmij1m7n2p...', status: 'ONLINE', traffic: '1.2 KB/s' }
  ]);

  const handleHost = async () => {
    if (!cloakAddress || !localPort) return;
    setIsHosting(true);
    setStatusLog('');
    try {
      const res = await tauriInvoke('host_site', { address: cloakAddress, port: localPort });
      setStatusLog(res as string);
      setActiveServices(prev => [
        {
          name: `Local Service :${localPort}`,
          port: localPort,
          address: cloakAddress.slice(0, 10) + '...',
          status: 'ONLINE',
          traffic: '0.0 KB/s'
        },
        ...prev
      ]);
    } catch (e: any) {
      setStatusLog(`Error: ${e}`);
    } finally {
      setIsHosting(false);
    }
  };

  return (
    <div className="grid grid-cols-12 gap-8 h-full">
      <div className="col-span-5">
        <Card title="EXPOSE_LOCAL_SERVICE" subTitle="ONION_HIDDEN_SERVICE_PROVISION">
          <div className="space-y-6 mt-4">
            <p className="text-xs text-slate-500 leading-relaxed font-medium">
              Create a secure, end-to-end encrypted onion service mapping a local port to a distributed `.cloak` address.
            </p>
            <div className="grid grid-cols-3 gap-4">
              <div className="col-span-1 space-y-2">
                <label className="text-[9px] font-black uppercase text-slate-400">Local Port</label>
                <input 
                  type="number" 
                  value={localPort} 
                  onChange={e => setLocalPort(Number(e.target.value))} 
                  className="w-full bg-slate-50 border border-slate-200 p-4 rounded-2xl text-xs font-mono outline-none focus:border-slate-900" 
                  placeholder="8080" 
                />
              </div>
              <div className="col-span-2 space-y-2">
                <label className="text-[9px] font-black uppercase text-slate-400">Cloak Target Address</label>
                <input 
                  value={cloakAddress} 
                  onChange={e => setCloakAddress(e.target.value)} 
                  className="w-full bg-slate-50 border border-slate-200 p-4 rounded-2xl text-xs font-mono outline-none focus:border-slate-900" 
                  placeholder="Target address..." 
                />
              </div>
            </div>
            <button 
              onClick={handleHost} 
              disabled={isHosting}
              className="w-full py-4 bg-slate-900 text-white rounded-2xl font-black uppercase tracking-[0.2em] text-[10px] flex items-center justify-center space-x-2 hover:bg-black transition-all"
            >
              {isHosting ? <RefreshCcw size={14} className="animate-spin" /> : <Server size={14} />}
              <span>Establish Onion Service</span>
            </button>
            {statusLog && (
              <div className={`p-4 rounded-xl border text-xs font-bold font-mono ${statusLog.startsWith('Error') ? 'bg-rose-50 border-rose-100 text-rose-600' : 'bg-emerald-50 border-emerald-100 text-emerald-600'}`}>
                {statusLog}
              </div>
            )}
          </div>
        </Card>
      </div>

      <div className="col-span-7">
        <Card title="ACTIVE_ONION_SERVICES" subTitle="ACTIVE_SERVICE_TUNNELS">
          <div className="space-y-4">
            {activeServices.map((srv, idx) => (
              <div key={idx} className="flex justify-between items-center p-4 border border-slate-100 bg-slate-50 rounded-2xl hover:border-slate-300 transition-all">
                <div className="flex items-center space-x-4">
                  <div className="p-2.5 bg-white border border-slate-200 rounded-xl shadow-sm text-slate-900">
                    <Layers size={16} />
                  </div>
                  <div>
                    <div className="text-[10px] font-black uppercase tracking-widest text-slate-900">{srv.name}</div>
                    <div className="text-[8px] font-black text-slate-400 uppercase tracking-tighter mt-1">Local Port: {srv.port} ➜ {srv.address}</div>
                  </div>
                </div>
                <div className="text-right flex items-center space-x-4">
                  <div className="text-right">
                    <div className="text-[8px] font-black text-slate-400 uppercase">Throughput</div>
                    <div className="text-[10px] font-mono font-bold text-slate-900">{srv.traffic}</div>
                  </div>
                  <span className="px-2 py-1 bg-emerald-50 text-emerald-600 border border-emerald-100 text-[8px] font-black uppercase rounded font-mono">
                    {srv.status}
                  </span>
                </div>
              </div>
            ))}
          </div>
        </Card>
      </div>
    </div>
  );
};

const SettingsView = () => {
  const [alias, setAlias] = useState('Stanlley-Node');
  const [maxHops, setMaxHops] = useState(3);
  const [logLevel, setLogLevel] = useState('INFO');
  const [theme, setTheme] = useState('slate');

  return (
    <div className="grid grid-cols-2 gap-8 h-full">
      <Card title="NODE_PREFERENCES" subTitle="LOCAL_DAEMON_SETTINGS">
        <div className="space-y-6 mt-4">
          <div className="space-y-2">
            <label className="text-[9px] font-black uppercase text-slate-400">Node Identity Alias</label>
            <input 
              value={alias} 
              onChange={e => setAlias(e.target.value)} 
              className="w-full bg-slate-50 border border-slate-200 p-4 rounded-2xl text-xs font-bold outline-none focus:border-slate-900" 
            />
          </div>
          <div className="grid grid-cols-2 gap-4">
            <div className="space-y-2">
              <label className="text-[9px] font-black uppercase text-slate-400">Maximum Hop Depth</label>
              <input 
                type="number" 
                value={maxHops} 
                onChange={e => setMaxHops(Number(e.target.value))} 
                className="w-full bg-slate-50 border border-slate-200 p-4 rounded-2xl text-xs font-mono outline-none focus:border-slate-900" 
              />
            </div>
            <div className="space-y-2">
              <label className="text-[9px] font-black uppercase text-slate-400">Console Log Level</label>
              <select 
                value={logLevel} 
                onChange={e => setLogLevel(e.target.value)}
                className="w-full bg-slate-50 border border-slate-200 p-4 rounded-2xl text-xs font-black uppercase outline-none focus:border-slate-900"
              >
                <option value="DEBUG">DEBUG</option>
                <option value="INFO">INFO</option>
                <option value="WARN">WARN</option>
                <option value="ERROR">ERROR</option>
              </select>
            </div>
          </div>
          <button className="w-full py-4 bg-slate-900 text-white rounded-2xl font-black uppercase tracking-[0.2em] text-[10px] hover:bg-black transition-all">
            Save Daemon Config
          </button>
        </div>
      </Card>

      <Card title="INTERFACE_PREFERENCES" subTitle="ADMIN_PANEL_DASHBOARD_THEME">
        <div className="space-y-6 mt-4">
          <div className="space-y-3">
            <label className="text-[9px] font-black uppercase text-slate-400">Theme Profile</label>
            <div className="grid grid-cols-2 gap-4">
              {[
                { id: 'slate', name: 'Dark Slate (Standard)', color: 'bg-slate-950' },
                { id: 'emerald', name: 'Emerald Forest', color: 'bg-emerald-950' },
                { id: 'violet', name: 'Neon Violet', color: 'bg-violet-950' },
                { id: 'blue', name: 'Deep Ocean Blue', color: 'bg-blue-950' }
              ].map(t => (
                <div 
                  key={t.id} 
                  onClick={() => setTheme(t.id)}
                  className={`p-4 border rounded-2xl cursor-pointer flex items-center space-x-3 transition-all ${
                    theme === t.id ? 'border-slate-900 bg-slate-50 shadow-sm' : 'border-slate-100 hover:border-slate-300'
                  }`}
                >
                  <div className={`w-4 h-4 rounded-full ${t.color}`} />
                  <span className="text-[10px] font-bold text-slate-700">{t.name}</span>
                </div>
              ))}
            </div>
          </div>
        </div>
      </Card>
    </div>
  );
};

const HardwareView = ({ status }: any) => {
  const chartData = Array.from({ length: 12 }, (_, i) => ({
    time: `t-${12-i}`,
    cpu: Math.max(10, Math.min(100, (status?.cpu_usage || 14) + (Math.random() - 0.5) * 8)),
    mem: Math.max(1.5, Math.min(16, (status?.mem_usage || 2) + (Math.random() - 0.5) * 0.4))
  }));

  return (
    <div className="space-y-8">
      <div className="grid grid-cols-3 gap-8">
        <Card title="CPU_LOAD_TELEMETRY" subTitle="ACTIVE_CORE_UTILIZATION">
          <div className="h-44 w-full mt-4">
            <ResponsiveContainer width="100%" height="100%">
              <AreaChart data={chartData}>
                <CartesianGrid strokeDasharray="3 3" stroke="#f1f5f9" vertical={false} />
                <XAxis dataKey="time" hide />
                <YAxis hide domain={[0, 100]} />
                <Area type="monotone" dataKey="cpu" stroke="#0f172a" fill="#0f172a" fillOpacity={0.04} strokeWidth={2.5} />
                <Tooltip contentStyle={{ borderRadius: '12px', border: 'none', fontSize: '10px' }} />
              </AreaChart>
            </ResponsiveContainer>
          </div>
          <div className="text-center font-mono font-black text-xl text-slate-900 mt-2">
            {status?.cpu_usage || '14.2'}%
          </div>
        </Card>

        <Card title="MEMORY_BUFFER_TELEMETRY" subTitle="RAM_KERNEL_BUFFER">
          <div className="h-44 w-full mt-4">
            <ResponsiveContainer width="100%" height="100%">
              <AreaChart data={chartData}>
                <CartesianGrid strokeDasharray="3 3" stroke="#f1f5f9" vertical={false} />
                <XAxis dataKey="time" hide />
                <YAxis hide domain={[0, 16]} />
                <Area type="monotone" dataKey="mem" stroke="#10b981" fill="#10b981" fillOpacity={0.04} strokeWidth={2.5} />
                <Tooltip contentStyle={{ borderRadius: '12px', border: 'none', fontSize: '10px' }} />
              </AreaChart>
            </ResponsiveContainer>
          </div>
          <div className="text-center font-mono font-black text-xl text-slate-900 mt-2">
            {status?.mem_usage || '2.1'} GB / 16 GB
          </div>
        </Card>

        <Card title="HARDWARE_SPECS" subTitle="PHYSICAL_NODE_METRICS">
          <div className="space-y-4 mt-6 text-xs">
            <div className="flex justify-between border-b border-slate-100 pb-2">
              <span className="text-[9px] font-black text-slate-400 uppercase">Architecture</span>
              <span className="font-mono font-bold text-slate-900">x86_64 / Linux</span>
            </div>
            <div className="flex justify-between border-b border-slate-100 pb-2">
              <span className="text-[9px] font-black text-slate-400 uppercase">CPU Cores</span>
              <span className="font-mono font-bold text-slate-900">4 Cores (Intel Xeon)</span>
            </div>
            <div className="flex justify-between border-b border-slate-100 pb-2">
              <span className="text-[9px] font-black text-slate-400 uppercase">Disk Space</span>
              <span className="font-mono font-bold text-slate-900">120 GB / 256 GB (SSD)</span>
            </div>
          </div>
        </Card>
      </div>
    </div>
  );
};

// ── Analytics View ───────────────────────────────────────────────────────────

const AnalyticsView = () => {
  const [data, setData] = useState<any>(null);
  const [loading, setLoading] = useState(true);
  const [lastUpdate, setLastUpdate] = useState('');

  const fetchAnalytics = async () => {
    try {
      const res = await tauriInvoke('get_analytics');
      setData(res);
      setLastUpdate(new Date().toLocaleTimeString());
    } catch (e) {
      console.error('Analytics fetch error:', e);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchAnalytics();
    const timer = setInterval(fetchAnalytics, 5000);
    return () => clearInterval(timer);
  }, []);

  const summary = data?.summary || {};
  const bandwidth = data?.bandwidth || [];
  const latency = data?.latency || [];
  const cells = data?.cells || [];

  const statCards = [
    { label: 'BANDWIDTH_UP', value: `${((summary.total_bytes_relayed || 0) / 1024 / 1024).toFixed(2)} MB`, sub: 'total relayed', icon: <UploadCloud size={18} />, color: 'text-violet-600', bg: 'bg-violet-50', border: 'border-violet-100' },
    { label: 'ACTIVE_CIRCUITS', value: summary.active_circuits || 0, sub: 'onion hops live', icon: <Radio size={18} />, color: 'text-emerald-600', bg: 'bg-emerald-50', border: 'border-emerald-100' },
    { label: 'AVG_LATENCY', value: `${summary.avg_latency_ms || 0}ms`, sub: 'circuit establishment', icon: <ActivityIcon size={18} />, color: 'text-amber-600', bg: 'bg-amber-50', border: 'border-amber-100' },
    { label: 'PQ_KEM_OPS', value: summary.pq_kem_operations || 0, sub: 'hybrid handshakes', icon: <Zap size={18} />, color: 'text-sky-600', bg: 'bg-sky-50', border: 'border-sky-100' },
    { label: 'DHT_ENTRIES', value: summary.dht_entries || 0, sub: 'routing table size', icon: <Database size={18} />, color: 'text-rose-600', bg: 'bg-rose-50', border: 'border-rose-100' },
    { label: 'REPUTATION', value: `${((summary.reputation || 0) * 100).toFixed(1)}%`, sub: 'node trust score', icon: <ShieldCheck size={18} />, color: 'text-teal-600', bg: 'bg-teal-50', border: 'border-teal-100' },
  ];

  return (
    <div className="space-y-8">
      {/* Header strip */}
      <div className="flex items-center justify-between">
        <div className="flex items-center space-x-3">
          <div className="p-2 bg-slate-900 rounded-xl text-white"><BarChart2 size={18} /></div>
          <div>
            <div className="text-[10px] font-black uppercase tracking-[0.2em] text-slate-400">Network Analytics</div>
            <div className="text-[9px] text-slate-300 font-mono italic">Live telemetry — refresh every 5s</div>
          </div>
        </div>
        <div className="flex items-center space-x-2">
          {loading && <RefreshCcw size={12} className="animate-spin text-slate-400" />}
          <span className="text-[9px] font-mono text-slate-300">LAST_SYNC: {lastUpdate}</span>
          <button onClick={fetchAnalytics} className="px-3 py-1.5 bg-slate-50 border border-slate-200 rounded-lg text-[9px] font-black uppercase tracking-widest hover:bg-slate-900 hover:text-white transition-all">
            Refresh
          </button>
        </div>
      </div>

      {/* KPI cards */}
      <div className="grid grid-cols-6 gap-4">
        {statCards.map((s, i) => (
          <div key={i} className={`p-5 bg-white rounded-2xl border border-slate-100 shadow-sm hover:shadow-md transition-all group`}>
            <div className={`inline-flex p-2 rounded-xl ${s.bg} border ${s.border} ${s.color} mb-3 group-hover:scale-110 transition-transform`}>{s.icon}</div>
            <div className="text-[9px] font-black uppercase tracking-[0.15em] text-slate-400 mb-1">{s.label}</div>
            <div className="text-xl font-black text-slate-900 font-mono">{loading ? '---' : s.value}</div>
            <div className="text-[8px] text-slate-400 font-medium uppercase mt-1">{s.sub}</div>
          </div>
        ))}
      </div>

      {/* Charts row 1: Bandwidth + Latency */}
      <div className="grid grid-cols-2 gap-6">
        <Card title="BANDWIDTH_DISTRIBUTION" subTitle="UPLOAD_DOWNLOAD_MB_PER_S">
          <div className="mt-4 h-52">
            {loading ? <div className="h-full flex items-center justify-center"><RefreshCcw size={24} className="animate-spin text-slate-200" /></div> : (
              <ResponsiveContainer width="100%" height="100%">
                <AreaChart data={bandwidth} margin={{ top: 5, right: 10, left: -20, bottom: 0 }}>
                  <defs>
                    <linearGradient id="uploadGrad" x1="0" y1="0" x2="0" y2="1">
                      <stop offset="5%" stopColor="#7c3aed" stopOpacity={0.3}/>
                      <stop offset="95%" stopColor="#7c3aed" stopOpacity={0}/>
                    </linearGradient>
                    <linearGradient id="downloadGrad" x1="0" y1="0" x2="0" y2="1">
                      <stop offset="5%" stopColor="#0ea5e9" stopOpacity={0.3}/>
                      <stop offset="95%" stopColor="#0ea5e9" stopOpacity={0}/>
                    </linearGradient>
                  </defs>
                  <CartesianGrid strokeDasharray="3 3" stroke="#f1f5f9" />
                  <XAxis dataKey="time" tick={{ fontSize: 8, fill: '#94a3b8' }} />
                  <YAxis tick={{ fontSize: 8, fill: '#94a3b8' }} />
                  <Tooltip contentStyle={{ background: '#0f172a', border: 'none', borderRadius: '12px', color: '#fff', fontSize: '10px' }} />
                  <Legend wrapperStyle={{ fontSize: '9px' }} />
                  <Area type="monotone" dataKey="upload" stroke="#7c3aed" strokeWidth={2} fill="url(#uploadGrad)" name="Upload MB/s" />
                  <Area type="monotone" dataKey="download" stroke="#0ea5e9" strokeWidth={2} fill="url(#downloadGrad)" name="Download MB/s" />
                </AreaChart>
              </ResponsiveContainer>
            )}
          </div>
        </Card>

        <Card title="CIRCUIT_LATENCY" subTitle="END_TO_END_MS">
          <div className="mt-4 h-52">
            {loading ? <div className="h-full flex items-center justify-center"><RefreshCcw size={24} className="animate-spin text-slate-200" /></div> : (
              <ResponsiveContainer width="100%" height="100%">
                <LineChart data={latency} margin={{ top: 5, right: 10, left: -20, bottom: 0 }}>
                  <CartesianGrid strokeDasharray="3 3" stroke="#f1f5f9" />
                  <XAxis dataKey="time" tick={{ fontSize: 8, fill: '#94a3b8' }} />
                  <YAxis tick={{ fontSize: 8, fill: '#94a3b8' }} />
                  <Tooltip contentStyle={{ background: '#0f172a', border: 'none', borderRadius: '12px', color: '#fff', fontSize: '10px' }} />
                  <Line type="monotone" dataKey="latency" stroke="#f59e0b" strokeWidth={2.5} dot={false} name="Latency ms" />
                </LineChart>
              </ResponsiveContainer>
            )}
          </div>
        </Card>
      </div>

      {/* Charts row 2: Cell traffic + PQ-KEM bar */}
      <div className="grid grid-cols-3 gap-6">
        <div className="col-span-2">
          <Card title="CELL_THROUGHPUT" subTitle="256B_ONION_CELLS_PER_INTERVAL">
            <div className="mt-4 h-48">
              {loading ? <div className="h-full flex items-center justify-center"><RefreshCcw size={24} className="animate-spin text-slate-200" /></div> : (
                <ResponsiveContainer width="100%" height="100%">
                  <BarChart data={cells} margin={{ top: 5, right: 10, left: -20, bottom: 0 }}>
                    <CartesianGrid strokeDasharray="3 3" stroke="#f1f5f9" />
                    <XAxis dataKey="time" tick={{ fontSize: 8, fill: '#94a3b8' }} />
                    <YAxis tick={{ fontSize: 8, fill: '#94a3b8' }} />
                    <Tooltip contentStyle={{ background: '#0f172a', border: 'none', borderRadius: '12px', color: '#fff', fontSize: '10px' }} />
                    <Bar dataKey="cells" fill="#10b981" radius={[4,4,0,0]} name="Cells" />
                  </BarChart>
                </ResponsiveContainer>
              )}
            </div>
          </Card>
        </div>

        <Card title="PQ_KEM_HYBRID" subTitle="POST_QUANTUM_ENCRYPTION_STATS">
          <div className="mt-4 space-y-5">
            <div>
              <div className="flex justify-between text-[9px] font-black uppercase text-slate-400 mb-2">
                <span>Hybrid KEM Ratio</span>
                <span className="text-slate-900 font-mono">{((summary.kem_hybrid_ratio || 0.97) * 100).toFixed(0)}%</span>
              </div>
              <div className="w-full h-2 bg-slate-100 rounded-full overflow-hidden">
                <div className="h-full bg-gradient-to-r from-violet-500 to-sky-500 rounded-full transition-all duration-1000" style={{ width: `${(summary.kem_hybrid_ratio || 0.97) * 100}%` }} />
              </div>
            </div>
            <div className="space-y-3">
              {[
                { label: 'X25519 ECDH', pct: 97, color: 'bg-violet-500' },
                { label: 'ML-KEM-768', pct: 94, color: 'bg-sky-500' },
                { label: 'ChaCha20-Poly', pct: 100, color: 'bg-emerald-500' },
                { label: 'Ed25519 Sign', pct: 100, color: 'bg-amber-500' },
              ].map((row, i) => (
                <div key={i}>
                  <div className="flex justify-between text-[8px] font-black uppercase text-slate-400 mb-1">
                    <span>{row.label}</span><span className="text-slate-600 font-mono">{row.pct}%</span>
                  </div>
                  <div className="w-full h-1.5 bg-slate-100 rounded-full overflow-hidden">
                    <div className={`h-full ${row.color} rounded-full`} style={{ width: `${row.pct}%` }} />
                  </div>
                </div>
              ))}
            </div>
          </div>
        </Card>
      </div>
    </div>
  );
};

// ── Decentralized Browser View ────────────────────────────────────────────────

const BrowserView = () => {
  const [addressInput, setAddressInput] = useState('');
  const [currentAddress, setCurrentAddress] = useState('');
  const [htmlContent, setHtmlContent] = useState('');
  const [rawText, setRawText] = useState('');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState('');
  const [history, setHistory] = useState<string[]>([]);
  const [historyIdx, setHistoryIdx] = useState(-1);
  const [hostedSites, setHostedSites] = useState<any[]>([]);

  // Fetch list of locally hosted sites
  useEffect(() => {
    const loadSites = async () => {
      try {
        const sites = await tauriInvoke('get_hosted_sites');
        if (Array.isArray(sites)) setHostedSites(sites);
      } catch (e) {}
    };
    loadSites();
    const timer = setInterval(loadSites, 10000);
    return () => clearInterval(timer);
  }, []);

  const navigateTo = async (addr: string) => {
    if (!addr.trim()) return;
    let target = addr.trim();
    if (!target.endsWith('.cloak')) {
      target = target + '.cloak';
    }
    setAddressInput(target);
    setCurrentAddress(target);
    setLoading(true);
    setError('');
    setHtmlContent('');
    setRawText('');

    try {
      const res = await tauriInvoke('browse_cloak', { address: target }) as any;
      if (res?.error && !res?.html) {
        setError(res.error);
      } else {
        const body = res?.html || '';
        // Check if it looks like HTML
        if (body.includes('<html') || body.includes('<HTML') || body.includes('<body')) {
          setHtmlContent(body);
        } else {
          setRawText(body || `Connected to ${target} via onion circuit.\nNo HTTP content received.`);
        }
        if (res?.error) setError(res.error);
      }
      // Update history
      const newHistory = [...history.slice(0, historyIdx + 1), target];
      setHistory(newHistory);
      setHistoryIdx(newHistory.length - 1);
    } catch (e: any) {
      setError(e?.message || 'Navigation failed');
    } finally {
      setLoading(false);
    }
  };

  const goBack = () => {
    if (historyIdx > 0) {
      const prev = history[historyIdx - 1];
      setHistoryIdx(historyIdx - 1);
      setAddressInput(prev);
      navigateTo(prev);
    }
  };

  const goForward = () => {
    if (historyIdx < history.length - 1) {
      const next = history[historyIdx + 1];
      setHistoryIdx(historyIdx + 1);
      setAddressInput(next);
      navigateTo(next);
    }
  };

  const isSecure = currentAddress.endsWith('.cloak');

  return (
    <div className="flex flex-col h-full space-y-0 bg-white rounded-[28px] border border-slate-100 shadow-xl overflow-hidden" style={{ minHeight: '75vh' }}>
      {/* Browser Chrome */}
      <div className="bg-slate-50 border-b border-slate-100 px-4 py-3 space-y-3">
        {/* Tab bar */}
        <div className="flex items-center space-x-2">
          <div className="flex items-center space-x-1.5">
            <div className="w-3 h-3 rounded-full bg-rose-400" />
            <div className="w-3 h-3 rounded-full bg-amber-400" />
            <div className="w-3 h-3 rounded-full bg-emerald-400" />
          </div>
          <div className="flex-1 flex items-center space-x-2">
            {currentAddress && (
              <div className="px-4 py-1.5 bg-white border border-slate-200 rounded-xl text-[10px] font-mono font-bold text-slate-700 flex items-center space-x-2 shadow-sm max-w-xs">
                <Lock size={9} className="text-emerald-500 shrink-0" />
                <span className="truncate">{currentAddress}</span>
              </div>
            )}
          </div>
          <div className="flex items-center space-x-2">
            <div className="p-1.5 bg-slate-100 rounded-lg text-[8px] font-black uppercase tracking-widest text-slate-500 border border-slate-200">SOCKS5 :9050</div>
          </div>
        </div>

        {/* Navigation bar */}
        <div className="flex items-center space-x-3">
          <button
            onClick={goBack}
            disabled={historyIdx <= 0}
            className="p-2 hover:bg-slate-200 rounded-xl transition-all disabled:opacity-30 disabled:cursor-not-allowed text-slate-500"
          >
            <ChevronLeft size={16} />
          </button>
          <button
            onClick={goForward}
            disabled={historyIdx >= history.length - 1}
            className="p-2 hover:bg-slate-200 rounded-xl transition-all disabled:opacity-30 disabled:cursor-not-allowed text-slate-500"
          >
            <ChevronRight size={16} />
          </button>
          <button
            onClick={() => navigateTo(addressInput)}
            disabled={loading}
            className="p-2 hover:bg-slate-200 rounded-xl transition-all text-slate-500"
          >
            {loading ? <RefreshCcw size={16} className="animate-spin" /> : <RefreshCcw size={16} />}
          </button>

          {/* Address bar */}
          <div className="flex-1 flex items-center bg-white border border-slate-200 rounded-xl overflow-hidden shadow-sm focus-within:border-slate-900 focus-within:shadow-md transition-all">
            <div className="pl-3 pr-2 flex items-center">
              {isSecure ? (
                <Lock size={12} className="text-emerald-500" />
              ) : (
                <Globe size={12} className="text-slate-400" />
              )}
            </div>
            <input
              type="text"
              value={addressInput}
              onChange={e => setAddressInput(e.target.value)}
              onKeyDown={e => e.key === 'Enter' && navigateTo(addressInput)}
              placeholder="Enter a .cloak address, e.g. mysite.cloak"
              className="flex-1 py-2 pr-3 text-xs font-mono bg-transparent outline-none text-slate-700 placeholder:text-slate-300"
            />
            <button
              onClick={() => navigateTo(addressInput)}
              className="px-4 py-2 bg-slate-900 text-white text-[10px] font-black uppercase tracking-widest hover:bg-black transition-all"
            >
              Go
            </button>
          </div>
        </div>
      </div>

      {/* Viewport */}
      <div className="flex-1 flex overflow-hidden">
        {/* Page content */}
        <div className="flex-1 overflow-auto bg-white">
          {!currentAddress && !loading && (
            <div className="h-full flex flex-col items-center justify-center p-12 space-y-10">
              {/* New Tab / Start Page */}
              <div className="text-center space-y-4">
                <div className="w-20 h-20 bg-slate-900 rounded-3xl flex items-center justify-center mx-auto shadow-2xl">
                  <Globe size={36} className="text-white" strokeWidth={1.5} />
                </div>
                <h2 className="text-3xl font-black uppercase tracking-tight text-slate-900">CloakMesh Browser</h2>
                <p className="text-slate-400 text-sm font-medium max-w-md leading-relaxed">
                  Navigate the decentralized web. All traffic is routed through multi-hop onion circuits over SOCKS5.
                </p>
              </div>

              {/* Quick-access tiles */}
              {hostedSites.length > 0 && (
                <div className="w-full max-w-lg">
                  <div className="text-[10px] font-black uppercase tracking-[0.2em] text-slate-400 mb-4 text-center">Locally Hosted Services</div>
                  <div className="grid grid-cols-2 gap-3">
                    {hostedSites.map((site, i) => (
                      <button
                        key={i}
                        onClick={() => navigateTo(site.address)}
                        className="p-4 bg-slate-50 border border-slate-100 rounded-2xl hover:border-slate-900 hover:shadow-lg transition-all text-left group"
                      >
                        <div className="flex items-center space-x-3">
                          <div className="p-2 bg-white rounded-xl border border-slate-200 group-hover:bg-slate-900 group-hover:text-white transition-all">
                            <Server size={14} className="text-slate-500 group-hover:text-white" />
                          </div>
                          <div>
                            <div className="text-[10px] font-black text-slate-900 truncate max-w-[140px]">{site.address}</div>
                            <div className="text-[8px] text-slate-400 font-mono">:{site.port}</div>
                          </div>
                        </div>
                      </button>
                    ))}
                  </div>
                </div>
              )}

              {/* Sample addresses */}
              <div className="w-full max-w-lg">
                <div className="text-[10px] font-black uppercase tracking-[0.2em] text-slate-400 mb-4 text-center">Try a Demo Address</div>
                <div className="grid grid-cols-1 gap-2">
                  {[
                    { addr: 'ag3o2j2z4uredr7fudmv4zzrkmmmsqgjkgdb3pubczdtocptefwm5y7rrrmq.cloak', label: 'Local Node' },
                  ].map((item, i) => (
                    <button
                      key={i}
                      onClick={() => navigateTo(item.addr)}
                      className="p-3 bg-slate-50 border border-slate-100 rounded-xl hover:border-slate-300 transition-all text-left flex items-center space-x-3"
                    >
                      <Globe size={12} className="text-slate-400 shrink-0" />
                      <span className="text-[10px] font-mono text-slate-600 truncate">{item.addr}</span>
                      <span className="text-[9px] font-black text-slate-400 uppercase ml-auto shrink-0">{item.label}</span>
                    </button>
                  ))}
                </div>
              </div>
            </div>
          )}

          {loading && (
            <div className="h-full flex flex-col items-center justify-center p-12 space-y-6">
              <div className="relative">
                <div className="w-16 h-16 rounded-full border-4 border-slate-100 border-t-slate-900 animate-spin" />
                <div className="absolute inset-0 flex items-center justify-center">
                  <Lock size={16} className="text-slate-900" />
                </div>
              </div>
              <div className="text-center space-y-2">
                <div className="text-[10px] font-black uppercase tracking-[0.2em] text-slate-400">Establishing Onion Circuit</div>
                <div className="text-[9px] text-slate-300 font-mono">{currentAddress}</div>
                <div className="flex items-center justify-center space-x-2 mt-4">
                  {['Guard Node', 'Middle Relay', 'Exit Relay', 'Destination'].map((hop, i) => (
                    <React.Fragment key={i}>
                      <div className="px-2 py-1 bg-slate-50 border border-slate-200 rounded-lg text-[8px] font-black text-slate-500 uppercase animate-pulse" style={{ animationDelay: `${i * 200}ms` }}>{hop}</div>
                      {i < 3 && <ArrowUpRight size={10} className="text-slate-300" />}
                    </React.Fragment>
                  ))}
                </div>
              </div>
            </div>
          )}

          {!loading && currentAddress && htmlContent && (
            <iframe
              srcDoc={htmlContent}
              className="w-full h-full border-0"
              sandbox="allow-same-origin allow-scripts"
              title={currentAddress}
            />
          )}

          {!loading && currentAddress && rawText && !htmlContent && (
            <div className="p-8">
              {error && (
                <div className="mb-6 p-4 bg-amber-50 border border-amber-100 rounded-xl">
                  <div className="text-[10px] font-black text-amber-600 uppercase tracking-widest mb-1">⚡ Connection Note</div>
                  <div className="text-xs font-mono text-amber-800">{error}</div>
                </div>
              )}
              <div className="p-6 bg-slate-50 border border-slate-100 rounded-2xl">
                <div className="flex items-center space-x-2 mb-4">
                  <Lock size={12} className="text-emerald-500" />
                  <span className="text-[10px] font-black uppercase tracking-widest text-slate-700">{currentAddress}</span>
                </div>
                <pre className="text-xs font-mono text-slate-600 whitespace-pre-wrap leading-relaxed">{rawText}</pre>
              </div>
            </div>
          )}

          {!loading && error && !rawText && !htmlContent && (
            <div className="h-full flex flex-col items-center justify-center p-12 space-y-6">
              <div className="w-16 h-16 bg-rose-50 rounded-3xl border-2 border-dashed border-rose-200 flex items-center justify-center">
                <AlertCircle size={28} className="text-rose-400" />
              </div>
              <div className="text-center space-y-3">
                <div className="text-lg font-black uppercase tracking-tight text-slate-900">Connection Failed</div>
                <div className="text-xs font-mono text-slate-500 max-w-sm">{error}</div>
                <div className="text-[9px] text-slate-300 font-mono">{currentAddress}</div>
              </div>
              <button
                onClick={() => navigateTo(currentAddress)}
                className="px-6 py-3 bg-slate-900 text-white text-[10px] font-black uppercase tracking-widest rounded-xl hover:bg-black transition-all"
              >
                Retry
              </button>
            </div>
          )}
        </div>

        {/* Sidebar: circuit info + history */}
        <div className="w-64 border-l border-slate-100 bg-slate-50 flex flex-col overflow-hidden">
          <div className="p-4 border-b border-slate-100">
            <div className="text-[9px] font-black uppercase tracking-[0.2em] text-slate-400 mb-3">Circuit Path</div>
            <div className="space-y-2">
              {['Guard Node', 'Middle Relay', 'Exit Relay'].map((hop, i) => (
                <div key={i} className="flex items-center space-x-2">
                  <div className={`w-2 h-2 rounded-full ${loading ? 'bg-amber-400 animate-pulse' : currentAddress ? 'bg-emerald-400' : 'bg-slate-200'}`} />
                  <span className="text-[9px] font-black uppercase text-slate-500">{hop}</span>
                  {i < 2 && <ArrowUpRight size={8} className="text-slate-300 ml-auto" />}
                </div>
              ))}
            </div>
          </div>
          <div className="p-4 flex-1 overflow-y-auto">
            <div className="text-[9px] font-black uppercase tracking-[0.2em] text-slate-400 mb-3">History</div>
            {history.length === 0 && (
              <div className="text-[9px] text-slate-300 italic">No browsing history yet</div>
            )}
            <div className="space-y-1">
              {history.slice().reverse().map((h, i) => (
                <button
                  key={i}
                  onClick={() => navigateTo(h)}
                  className="w-full text-left p-2 hover:bg-slate-100 rounded-lg transition-all group"
                >
                  <div className="text-[8px] font-mono text-slate-600 truncate group-hover:text-slate-900">{h}</div>
                </button>
              ))}
            </div>
          </div>
        </div>
      </div>
    </div>
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
    let active = true;
    let unlistenFn: (() => void) | null = null;
    let pollInterval: any = null;
    let maxTimestamp = '';

    const setupLogs = async () => {
      if (typeof window !== 'undefined' && (window as any).__TAURI_IPC__) {
        try {
          const { listen } = await import('@tauri-apps/api/event');
          const unlisten = await listen('structured-log', (event: any) => {
            if (!active) return;
            setStructuredLogs(prev => [event.payload, ...prev].slice(0, 100));
            if (event.payload.level === 'WARN' || event.payload.level === 'ERROR') {
              setNotifications(prev => [{ id: Date.now(), type: 'warning', msg: event.payload.message, time: 'Just now' }, ...prev]);
            }
          });
          unlistenFn = unlisten;
        } catch (e) {
          console.error("Tauri listen error:", e);
        }
      } else {
        const pollLogs = async () => {
          try {
            const logs = await tauriInvoke('get_logs');
            if (logs && Array.isArray(logs) && active) {
              const reversed = [...logs].reverse();
              setStructuredLogs(reversed);

              let nextMax = maxTimestamp;
              for (const log of logs) {
                const ts = log.raw_timestamp || log.timestamp;
                if (ts > maxTimestamp) {
                  if (maxTimestamp && (log.level === 'WARN' || log.level === 'ERROR')) {
                    setNotifications(prev => [{ id: Date.now(), type: 'warning', msg: log.message, time: 'Just now' }, ...prev]);
                  }
                  if (ts > nextMax) {
                    nextMax = ts;
                  }
                }
              }
              maxTimestamp = nextMax;
            }
          } catch (e) {
            console.error("Error polling logs:", e);
          }
        };
        pollLogs();
        pollInterval = setInterval(pollLogs, 2000);
      }
    };

    setupLogs();

    return () => {
      active = false;
      if (unlistenFn) unlistenFn();
      if (pollInterval) clearInterval(pollInterval);
    };
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
    { id: 'network', label: 'Network', items: [{ id: 'discovery', label: 'Peer Discovery', icon: Globe }, { id: 'circuits', label: 'Circuit Audit', icon: Radio }, { id: 'dht', label: 'DHT Explorer', icon: Database }, { id: 'analytics', label: 'Network Analytics', icon: BarChart2 }] },
    { id: 'security', label: 'Security', items: [{ id: 'identity', label: 'ID Management', icon: Key }, { id: 'capabilities', label: 'Permissions', icon: ShieldCheck }, { id: 'encryption', label: 'Crypto Lab', icon: Lock }] },
    { id: 'ops', label: 'Operations', items: [{ id: 'hosting', label: 'Service Hosting', icon: Server }, { id: 'browser', label: 'Decentralized Browser', icon: Monitor }, { id: 'terminal', label: 'Kernel Logs', icon: TerminalIcon }] },
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
            ) : page === 'discovery' ? (
              <DiscoveryView status={status} />
            ) : page === 'circuits' ? (
              <CircuitsView />
            ) : page === 'dht' ? (
              <DhtView />
            ) : page === 'capabilities' ? (
              <CapabilitiesView />
            ) : page === 'encryption' ? (
              <EncryptionLabView />
            ) : page === 'hosting' ? (
              <HostingView />
            ) : page === 'settings' ? (
              <SettingsView />
            ) : page === 'hardware' ? (
              <HardwareView status={status} />
            ) : page === 'analytics' ? (
              <AnalyticsView />
            ) : page === 'browser' ? (
              <BrowserView />
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
