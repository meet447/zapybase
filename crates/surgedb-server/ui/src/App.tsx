import React, { useState, useEffect } from 'react';
import { Routes, Route, Link, useLocation } from 'react-router-dom';
import { 
  LayoutDashboard, 
  Database, 
  Search, 
  Settings, 
  Activity,
  Menu,
  X,
  DatabaseZap,
  Box
} from 'lucide-react';
import { clsx, type ClassValue } from 'clsx';
import { twMerge } from 'tailwind-merge';

function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

import Overview from './pages/Overview';
import Metrics from './pages/Metrics';
import Collections from './pages/Collections';
import Playground from './pages/Playground';

export default function App() {
  const [isSidebarOpen, setIsSidebarOpen] = useState(true);
  const location = useLocation();

  const navItems = [
    { name: 'OVERVIEW', icon: LayoutDashboard, path: '/' },
    { name: 'COLLECTIONS', icon: Database, path: '/collections' },
    { name: 'PLAYGROUND', icon: Search, path: '/search' },
    { name: 'METRICS', icon: Activity, path: '/metrics' },
  ];

  return (
    <div className="flex h-screen bg-white text-black font-mono overflow-hidden">
      {/* Sidebar */}
      <aside className={cn(
        "bg-white border-r-2 border-black transition-all duration-300 flex flex-col z-20",
        isSidebarOpen ? "w-64" : "w-20"
      )}>
        <div className="h-20 flex items-center px-6 border-b-2 border-black">
          <Link to="/" className="flex items-center gap-3">
            <div className="w-8 h-8 bg-surge-orange border-2 border-black flex items-center justify-center">
              <DatabaseZap className="text-white w-5 h-5" />
            </div>
            {isSidebarOpen && <span className="font-black text-xl tracking-tighter italic uppercase">surgedb</span>}
          </Link>
        </div>

        <nav className="flex-1 py-6 space-y-2">
          {navItems.map((item) => (
            <Link
              key={item.path}
              to={item.path}
              className={cn(
                "flex items-center gap-3 px-6 py-3 font-bold transition-all relative group",
                location.pathname === item.path 
                  ? "text-surge-orange" 
                  : "text-black hover:bg-surge-gray"
              )}
            >
              {location.pathname === item.path && (
                <div className="absolute left-0 top-0 bottom-0 w-1.5 bg-surge-orange" />
              )}
              <item.icon className={cn("w-5 h-5 shrink-0", location.pathname === item.path ? "text-surge-orange" : "text-black")} />
              {isSidebarOpen && <span className="text-sm tracking-widest">{item.name}</span>}
            </Link>
          ))}
        </nav>

        <div className="p-6 border-t-2 border-black">
          <button 
            onClick={() => setIsSidebarOpen(!isSidebarOpen)}
            className="w-full flex items-center justify-center p-2 border-2 border-black hover:bg-surge-gray transition-colors"
          >
            {isSidebarOpen ? <X className="w-5 h-5" /> : <Menu className="w-5 h-5" />}
          </button>
        </div>
      </aside>

      {/* Main Content */}
      <main className="flex-1 overflow-auto bg-white">
        <header className="h-20 border-b-2 border-black flex items-center justify-between px-10 sticky top-0 bg-white/80 backdrop-blur-md z-10">
          <div className="flex items-center gap-4">
            <div className="w-2 h-2 bg-surge-orange animate-pulse rounded-full" />
            <h1 className="text-sm font-black tracking-[0.2em] uppercase">
              {navItems.find(i => i.path === location.pathname)?.name || 'SYSTEM'}
            </h1>
          </div>
          
          <div className="flex items-center gap-6">
             <div className="hidden md:flex gap-4 items-center">
                <span className="text-[10px] font-bold text-slate-400">STATUS</span>
                <span className="text-xs font-bold bg-black text-white px-3 py-1">LIVE_V1.0</span>
             </div>
             <Link to="/search" className="btn-brutal btn-brutal-orange text-xs py-1.5 px-4">
                GET STARTED
             </Link>
          </div>
        </header>

        <div className="max-w-[1400px] mx-auto min-h-[calc(100vh-80px)]">
          <Routes>
            <Route path="/" element={<Overview />} />
            <Route path="/collections/*" element={<Collections />} />
            <Route path="/search" element={<Playground />} />
            <Route path="/metrics" element={<Metrics />} />
          </Routes>
        </div>
      </main>
    </div>
  );
}
