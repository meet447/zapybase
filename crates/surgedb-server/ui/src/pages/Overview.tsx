import React, { useState, useEffect } from 'react';
import { Link } from 'react-router-dom';
import { 
  MemoryStick, 
  ArrowUpRight, 
  ArrowDownLeft, 
  Database as DbIcon, 
  Clock,
  Layers,
  ChevronRight,
  Activity,
  ArrowRight
} from 'lucide-react';
import { 
  AreaChart, Area, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer
} from 'recharts';

const formatBytes = (bytes: number) => {
  if (bytes === 0) return '0 B';
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
};

const StatCard = ({ title, value, unit, color }: any) => (
  <div className="wire-container p-6 relative overflow-hidden group">
    <div className={`absolute top-0 right-0 w-16 h-16 opacity-10 transition-transform group-hover:scale-110`} style={{ backgroundColor: color }}></div>
    <div className="text-[10px] font-black text-black/40 tracking-[0.2em] mb-4 uppercase">{title}</div>
    <div className="flex items-baseline gap-2">
      <span className="text-3xl font-black tracking-tighter text-black leading-none">{value}</span>
      {unit && <span className="text-xs font-bold text-black/60 uppercase">{unit}</span>}
    </div>
  </div>
);

export default function Overview() {
  const [history, setHistory] = useState<any[]>([]);
  const [stats, setStats] = useState<any>(null);

  useEffect(() => {
    const fetchData = async () => {
      try {
        const [historyRes, statsRes] = await Promise.all([
          fetch('/api/metrics/history'),
          fetch('/api/stats')
        ]);
        if (!historyRes.ok || !statsRes.ok) return;
        setHistory(await historyRes.json());
        setStats(await statsRes.json());
      } catch (err) {
        console.error(err);
      }
    };
    fetchData();
    const interval = setInterval(fetchData, 6000);
    return () => clearInterval(interval);
  }, []);

  const latest = history[history.length - 1] || {
    memory_usage_mb: 0,
    read_requests: 0,
    write_requests: 0,
    avg_latency_ms: 0,
    storage_usage_bytes: 0
  };

  const chartData = history.slice(-20).map(h => ({
    ...h,
    time: new Date(h.timestamp).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit', second: '2-digit' })
  }));

  const collections = stats?.database?.collections ? Object.entries(stats.database.collections) : [];

  return (
    <div className="p-10 space-y-10">
      <div className="flex flex-col gap-2">
        <h2 className="text-4xl font-black tracking-tighter uppercase leading-none">SYSTEM_OVERVIEW</h2>
        <div className="h-1.5 w-24 bg-surge-orange"></div>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-8">
        <StatCard title="MEMORY_LOAD" value={latest.memory_usage_mb} unit="MB" color="#3b82f6" />
        <StatCard title="VECTOR_COUNT" value={stats?.database?.total_vectors || 0} color="#10b981" />
        <StatCard title="AVG_LATENCY" value={latest.avg_latency_ms.toFixed(2)} unit="MS" color="#a855f7" />
        <StatCard title="STORAGE_SIZE" value={formatBytes(latest.storage_usage_bytes).split(' ')[0]} unit={formatBytes(latest.storage_usage_bytes).split(' ')[1]} color="#E57E51" />
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-12 gap-10">
        {/* Quick Throughput Chart */}
        <div className="lg:col-span-8 wire-container p-8">
          <div className="flex items-center justify-between mb-8 border-b-2 border-black pb-4">
            <div className="flex items-center gap-3">
              <Activity className="w-5 h-5 text-surge-orange" />
              <h3 className="text-sm font-black tracking-widest uppercase">TRAFFIC_THROUGHPUT</h3>
            </div>
            <Link to="/metrics" className="text-[10px] font-black hover:text-surge-orange transition-colors">FULL_METRICS →</Link>
          </div>
          <div className="h-[300px]">
            <ResponsiveContainer width="100%" height="100%">
              <AreaChart data={chartData}>
                <CartesianGrid strokeDasharray="3 3" stroke="#e5e5e5" vertical={false} />
                <XAxis dataKey="time" hide />
                <YAxis stroke="#000000" fontSize={10} tickLine={false} axisLine={false} />
                <Tooltip 
                  contentStyle={{ backgroundColor: '#FFFFFF', border: '2px solid #000000', borderRadius: '0', fontFamily: 'JetBrains Mono' }} 
                  itemStyle={{ fontSize: '10px', fontWeight: '800' }}
                />
                <Area type="stepAfter" dataKey="read_requests" name="READS" stroke="#000000" fill="#000000" fillOpacity={0.05} strokeWidth={2} />
                <Area type="stepAfter" dataKey="write_requests" name="WRITES" stroke="#E57E51" fill="#E57E51" fillOpacity={0.1} strokeWidth={2} />
              </AreaChart>
            </ResponsiveContainer>
          </div>
        </div>

        {/* Collections Overview */}
        <div className="lg:col-span-4 wire-container p-8 flex flex-col">
          <div className="flex items-center justify-between mb-8 border-b-2 border-black pb-4">
            <div className="flex items-center gap-3">
              <Layers className="w-5 h-5 text-surge-orange" />
              <h3 className="text-sm font-black tracking-widest uppercase">COLLECTIONS</h3>
            </div>
            <span className="text-[10px] font-bold bg-black text-white px-2 py-0.5">{collections.length}</span>
          </div>
          
          <div className="space-y-4 flex-1 overflow-auto max-h-[300px] pr-2 custom-scrollbar">
            {collections.map(([name, data]: [string, any]) => (
              <Link 
                key={name} 
                to={`/collections`} 
                className="flex items-center justify-between p-4 border-2 border-black hover:bg-surge-gray transition-all group active:translate-x-1 active:translate-y-1 active:shadow-none"
                style={{ boxShadow: '2px 2px 0px 0px rgba(0,0,0,1)' }}
              >
                <div className="flex flex-col gap-1 min-w-0">
                  <span className="text-xs font-black truncate uppercase tracking-tighter">{name}</span>
                  <span className="text-[10px] font-bold text-black/50">{data.vector_count} VECTORS</span>
                </div>
                <ArrowRight className="w-4 h-4 text-black group-hover:text-surge-orange transition-colors" />
              </Link>
            ))}
            {collections.length === 0 && (
              <div className="text-center py-10 border-2 border-dashed border-black/20 italic text-[10px] font-bold text-black/40">
                NO_COLLECTIONS_INITIATED
              </div>
            )}
          </div>
          
          <Link to="/collections" className="btn-brutal btn-brutal-white text-center text-xs mt-8">
            MANAGE_DATABASE
          </Link>
        </div>
      </div>
    </div>
  );
}
