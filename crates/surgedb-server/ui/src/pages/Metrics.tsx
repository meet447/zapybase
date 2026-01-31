import React, { useState, useEffect } from 'react';
import { 
  AreaChart, Area, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer,
  LineChart, Line, Legend
} from 'recharts';
import { MemoryStick, Clock, Activity, Database as DbIcon } from 'lucide-react';

const formatBytes = (bytes: number) => {
  if (bytes === 0) return '0 B';
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
};

const MetricHeader = ({ title, icon: Icon }: any) => (
  <div className="flex items-center gap-3 mb-8 border-b-2 border-black pb-4">
    <Icon className="w-5 h-5 text-surge-orange" />
    <h3 className="text-sm font-black tracking-widest uppercase">{title}</h3>
  </div>
);

export default function Metrics() {
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

        const historyData = await historyRes.json();
        const statsData = await statsRes.json();
        
        if (Array.isArray(historyData)) {
          setHistory(historyData);
        }
        setStats(statsData);
      } catch (err) {
        console.error("Failed to fetch metrics", err);
      }
    };

    fetchData();
    const interval = setInterval(fetchData, 6000);
    return () => clearInterval(interval);
  }, []);

  const chartData = history.map(h => ({
    ...h,
    time: new Date(h.timestamp).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit', second: '2-digit' })
  }));

  return (
    <div className="p-10 space-y-12">
      <div className="flex flex-col gap-2">
        <h2 className="text-4xl font-black tracking-tighter uppercase leading-none">SYSTEM_METRICS</h2>
        <div className="h-1.5 w-24 bg-surge-orange"></div>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-10">
        {/* Memory History */}
        <div className="wire-container p-8">
          <MetricHeader title="MEMORY_RESOURCES" icon={MemoryStick} />
          <div className="h-[300px]">
            <ResponsiveContainer width="100%" height="100%">
              <AreaChart data={chartData}>
                <CartesianGrid strokeDasharray="3 3" stroke="#e5e5e5" vertical={false} />
                <XAxis dataKey="time" stroke="#000000" fontSize={10} tickMargin={10} hide />
                <YAxis stroke="#000000" fontSize={10} tickLine={false} axisLine={false} />
                <Tooltip 
                  contentStyle={{ backgroundColor: '#FFFFFF', border: '2px solid #000000', borderRadius: '0', fontFamily: 'JetBrains Mono' }}
                />
                <Area type="stepAfter" dataKey="memory_usage_mb" name="MEMORY (MB)" stroke="#000000" fill="#000000" fillOpacity={0.05} strokeWidth={2} />
              </AreaChart>
            </ResponsiveContainer>
          </div>
        </div>

        {/* Throughput History */}
        <div className="wire-container p-8">
          <MetricHeader title="IO_THROUGHPUT" icon={Activity} />
          <div className="h-[300px]">
            <ResponsiveContainer width="100%" height="100%">
              <LineChart data={chartData}>
                <CartesianGrid strokeDasharray="3 3" stroke="#e5e5e5" vertical={false} />
                <XAxis dataKey="time" stroke="#000000" fontSize={10} tickMargin={10} hide />
                <YAxis stroke="#000000" fontSize={10} tickLine={false} axisLine={false} />
                <Tooltip 
                  contentStyle={{ backgroundColor: '#FFFFFF', border: '2px solid #000000', borderRadius: '0', fontFamily: 'JetBrains Mono' }}
                />
                <Legend verticalAlign="top" align="right" iconType="rect" wrapperStyle={{ fontSize: '10px', fontWeight: 'bold', paddingBottom: '20px' }} />
                <Line type="stepAfter" dataKey="read_requests" name="READS" stroke="#000000" strokeWidth={3} dot={false} />
                <Line type="stepAfter" dataKey="write_requests" name="WRITES" stroke="#E57E51" strokeWidth={3} dot={false} />
              </LineChart>
            </ResponsiveContainer>
          </div>
        </div>

        {/* Latency History */}
        <div className="wire-container p-8">
          <MetricHeader title="QUERY_LATENCY" icon={Clock} />
          <div className="h-[300px]">
            <ResponsiveContainer width="100%" height="100%">
              <AreaChart data={chartData}>
                <CartesianGrid strokeDasharray="3 3" stroke="#e5e5e5" vertical={false} />
                <XAxis dataKey="time" stroke="#000000" fontSize={10} tickMargin={10} hide />
                <YAxis stroke="#000000" fontSize={10} tickLine={false} axisLine={false} />
                <Tooltip 
                   contentStyle={{ backgroundColor: '#FFFFFF', border: '2px solid #000000', borderRadius: '0', fontFamily: 'JetBrains Mono' }}
                />
                <Area type="monotone" dataKey="avg_latency_ms" name="LATENCY (MS)" stroke="#E57E51" fill="#E57E51" fillOpacity={0.1} strokeWidth={2} />
              </AreaChart>
            </ResponsiveContainer>
          </div>
        </div>

        {/* Storage History */}
        <div className="wire-container p-8">
          <MetricHeader title="DISK_USAGE" icon={DbIcon} />
          <div className="h-[300px]">
            <ResponsiveContainer width="100%" height="100%">
              <AreaChart data={chartData}>
                <CartesianGrid strokeDasharray="3 3" stroke="#e5e5e5" vertical={false} />
                <XAxis dataKey="time" stroke="#000000" fontSize={10} tickMargin={10} hide />
                <YAxis stroke="#000000" fontSize={10} tickLine={false} axisLine={false} />
                <Tooltip 
                   contentStyle={{ backgroundColor: '#FFFFFF', border: '2px solid #000000', borderRadius: '0', fontFamily: 'JetBrains Mono' }}
                   formatter={(val: number) => [formatBytes(val), "STORAGE"]}
                />
                <Area type="stepAfter" dataKey="storage_usage_bytes" name="STORAGE" stroke="#000000" fill="#000000" fillOpacity={0.05} strokeWidth={2} />
              </AreaChart>
            </ResponsiveContainer>
          </div>
        </div>
      </div>
    </div>
  );
}
