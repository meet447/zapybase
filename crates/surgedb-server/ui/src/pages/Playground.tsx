import React, { useState, useEffect } from 'react';
import { Search as SearchIcon, Play, Database, Info, Terminal, Settings2, Code, ChevronRight } from 'lucide-react';

export default function Playground() {
  const [collections, setCollections] = useState<string[]>([]);
  const [selectedCollection, setSelectedCollection] = useState('');
  const [vectorInput, setVectorInput] = useState('');
  const [k, setK] = useState(5);
  const [results, setResults] = useState<any[]>([]);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    fetch('/api/collections')
      .then(res => res.json())
      .then(data => {
        setCollections(data);
        if (data.length > 0) setSelectedCollection(data[0]);
      });
  }, []);

  const handleSearch = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!selectedCollection) return;
    
    setLoading(true);
    try {
      // Clean up vector input
      const cleanVector = vectorInput
        .replace(/[\[\]]/g, '')
        .split(',')
        .map(n => parseFloat(n.trim()))
        .filter(n => !isNaN(n));

      const res = await fetch(`/api/collections/${selectedCollection}/search`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          vector: cleanVector,
          k: Number(k)
        })
      });
      const data = await res.json();
      setResults(Array.isArray(data) ? data : []);
    } catch (err) {
      console.error(err);
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="p-10 space-y-10 font-mono">
      <div className="flex flex-col gap-2">
        <h2 className="text-4xl font-black tracking-tighter uppercase leading-none text-black">SEARCH_PLAYGROUND</h2>
        <div className="h-1.5 w-32 bg-surge-orange"></div>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-12 gap-10">
        {/* Search Config */}
        <div className="lg:col-span-4 space-y-8">
          <div className="wire-container p-8">
            <div className="flex items-center gap-3 mb-8 border-b-2 border-black pb-4">
              <Settings2 className="w-5 h-5 text-surge-orange" />
              <h3 className="text-sm font-black tracking-widest uppercase">QUERY_CONFIG</h3>
            </div>
            
            <form onSubmit={handleSearch} className="space-y-8">
              <div className="space-y-2">
                <label className="text-[10px] font-black uppercase tracking-widest text-black/50">COLLECTION_TARGET</label>
                <select 
                  value={selectedCollection}
                  onChange={e => setSelectedCollection(e.target.value)}
                  className="w-full bg-white border-2 border-black px-4 py-3 text-xs font-black outline-none appearance-none"
                >
                  {collections.map(c => <option key={c} value={c}>{c}</option>)}
                </select>
              </div>

              <div className="space-y-2">
                <label className="text-[10px] font-black uppercase tracking-widest text-black/50">TOP_K_NEIGHBORS</label>
                <input 
                  type="number" 
                  value={k}
                  onChange={e => setK(Number(e.target.value))}
                  className="w-full bg-white border-2 border-black px-4 py-3 text-xs font-black outline-none"
                />
              </div>

              <div className="space-y-2">
                <label className="text-[10px] font-black uppercase tracking-widest text-black/50">QUERY_EMBEDDING_ARRAY</label>
                <textarea 
                  value={vectorInput}
                  onChange={e => setVectorInput(e.target.value)}
                  placeholder="[0.12, -0.45, 0.89, ...]"
                  rows={8}
                  className="w-full bg-white border-2 border-black px-4 py-4 text-[11px] font-bold outline-none font-mono resize-none leading-relaxed"
                />
              </div>

              <button 
                disabled={loading || !vectorInput}
                className="w-full btn-brutal btn-brutal-orange text-xs py-3"
              >
                {loading ? 'EXECUTING_SEARCH...' : 'INITIATE_SEARCH'}
              </button>
            </form>
          </div>

          <div className="border-2 border-dashed border-black/20 p-6 bg-surge-gray/10">
            <div className="flex gap-4 items-start">
              <Info className="w-5 h-5 text-surge-orange shrink-0 mt-1" />
              <p className="text-[10px] font-bold text-black/40 leading-relaxed uppercase tracking-wider">
                RESULTS ARE RANKED BY SIMILARITY DISTANCE. LOWER VALUES INDICATE HIGHER COSINE SIMILARITY OR LOWER EUCLIDEAN DISTANCE.
              </p>
            </div>
          </div>
        </div>

        {/* Results Area */}
        <div className="lg:col-span-8">
          <div className="wire-container min-h-[600px] flex flex-col">
            <div className="p-4 border-b-2 border-black bg-surge-gray flex items-center justify-between">
              <div className="flex items-center gap-3">
                <Terminal className="w-4 h-4 text-surge-orange" />
                <h3 className="text-[10px] font-black tracking-widest uppercase">SEARCH_OUTPUT_STREAM</h3>
              </div>
              {results.length > 0 && (
                <span className="text-[10px] font-black bg-black text-white px-3 py-0.5">
                  {results.length}_HITS
                </span>
              )}
            </div>

            <div className="flex-1 overflow-auto p-8 space-y-6 custom-scrollbar bg-surge-gray/10">
              {results.map((res, i) => (
                <div 
                  key={res.id} 
                  className="bg-white border-2 border-black p-6 flex flex-col gap-4 animate-in fade-in slide-in-from-bottom-4 duration-300"
                  style={{ boxShadow: '4px 4px 0px 0px rgba(0,0,0,1)' }}
                >
                  <div className="flex items-center justify-between border-b-2 border-black pb-4">
                    <div className="flex items-center gap-4">
                       <div className="w-10 h-10 bg-black text-white flex items-center justify-center font-black text-sm italic">#{i+1}</div>
                       <div className="flex flex-col">
                          <span className="text-xs font-black uppercase tracking-tighter">{res.id}</span>
                          <span className="text-[9px] font-bold text-black/40 uppercase tracking-widest">INTERNAL_ID</span>
                       </div>
                    </div>
                    <div className="flex flex-col items-end">
                       <span className="text-sm font-black text-surge-orange leading-none">{res.distance.toFixed(6)}</span>
                       <span className="text-[9px] font-black uppercase tracking-widest text-black/40 mt-1">SIM_DIST</span>
                    </div>
                  </div>

                  {res.metadata && (
                    <div className="space-y-2">
                       <div className="flex items-center gap-2">
                          <Code className="w-3 h-3 text-black/30" />
                          <span className="text-[9px] font-black uppercase tracking-widest text-black/40">METADATA_EXTRACT</span>
                       </div>
                       <div className="bg-surge-gray p-4 border-2 border-black/5">
                          <pre className="text-[11px] font-mono font-bold text-black/70 overflow-hidden text-ellipsis whitespace-pre-wrap">
                            {JSON.stringify(res.metadata, null, 4)}
                          </pre>
                       </div>
                    </div>
                  )}
                </div>
              ))}

              {!loading && results.length === 0 && (
                <div className="h-full flex flex-col items-center justify-center text-center py-40 opacity-20">
                  <Terminal className="w-16 h-16 text-black mb-6" />
                  <h3 className="text-xs font-black uppercase tracking-[0.2em]">WAITING_FOR_INPUT</h3>
                </div>
              )}
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
