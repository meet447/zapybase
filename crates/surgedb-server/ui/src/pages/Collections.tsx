import React, { useState, useEffect } from 'react';
import { 
  Plus, 
  Trash2, 
  Search as SearchIcon, 
  ChevronLeft, 
  ChevronRight, 
  Database, 
  Box,
  Layers,
  ArrowRight,
  Filter,
  ExternalLink,
  Code,
  X
} from 'lucide-react';
import { clsx, type ClassValue } from 'clsx';
import { twMerge } from 'tailwind-merge';

function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

interface VectorEntry {
  id: string;
  metadata: any;
}

export default function Collections() {
  const [collections, setCollections] = useState<string[]>([]);
  const [stats, setStats] = useState<any>(null);
  const [selectedCollection, setSelectedCollection] = useState<string | null>(null);
  const [vectors, setVectors] = useState<VectorEntry[]>([]);
  const [isCreating, setIsCreating] = useState(false);
  const [newCollection, setNewCollection] = useState({ 
    name: '', 
    dimensions: 1536,
    distance_metric: 'Cosine',
    quantization: 'None'
  });
  const [loading, setLoading] = useState(false);
  const [selectedVector, setSelectedVector] = useState<any>(null);
  
  // Pagination
  const [page, setPage] = useState(0);
  const limit = 50;

  useEffect(() => {
    fetchCollections();
  }, []);

  useEffect(() => {
    if (selectedCollection) {
      setPage(0); // Reset page on collection switch
      fetchVectors(selectedCollection, 0);
    }
  }, [selectedCollection]);

  useEffect(() => {
    if (selectedCollection) {
      fetchVectors(selectedCollection, page);
    }
  }, [page]);

  const fetchCollections = async () => {
    try {
      const [colsRes, statsRes] = await Promise.all([
        fetch('/api/collections'),
        fetch('/api/stats')
      ]);
      const cols = await colsRes.json();
      const statsData = await statsRes.json();
      setCollections(cols);
      setStats(statsData);
    } catch (err) {
      console.error(err);
    }
  };

  const fetchVectors = async (name: string, p: number) => {
    try {
      const res = await fetch(`/api/collections/${name}/vectors?offset=${p * limit}&limit=${limit}`);
      const data = await res.json();
      setVectors(Array.isArray(data) ? data : []);
    } catch (err) {
      console.error(err);
    }
  };

  const handleCreate = async (e: React.FormEvent) => {
    e.preventDefault();
    setLoading(true);
    try {
      const res = await fetch('/api/collections', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          name: newCollection.name,
          dimensions: Number(newCollection.dimensions),
          distance_metric: newCollection.distance_metric,
          quantization: newCollection.quantization
        })
      });
      if (res.ok) {
        setIsCreating(false);
        setNewCollection({ 
          name: '', 
          dimensions: 1536,
          distance_metric: 'Cosine',
          quantization: 'None'
        });
        fetchCollections();
      }
    } catch (err) {
      console.error(err);
    } finally {
      setLoading(false);
    }
  };

  const handleDelete = async (name: string) => {
    if (!confirm(`Are you sure you want to delete collection "${name}"?`)) return;
    try {
      await fetch(`/api/collections/${name}`, { method: 'DELETE' });
      if (selectedCollection === name) setSelectedCollection(null);
      fetchCollections();
    } catch (err) {
      console.error(err);
    }
  };

  const fetchVectorDetail = async (id: string) => {
    try {
      const res = await fetch(`/api/collections/${selectedCollection}/vectors/${id}`);
      const data = await res.json();
      setSelectedVector(data);
    } catch (err) {
      console.error(err);
    }
  }

  return (
    <div className="flex h-[calc(100vh-80px)] overflow-hidden">
      {/* List Sidebar */}
      <div className="w-80 border-r-2 border-black bg-white flex flex-col shrink-0">
        <div className="p-8 border-b-2 border-black flex items-center justify-between">
          <h2 className="text-xs font-black tracking-widest uppercase">COLLECTIONS</h2>
          <button 
            onClick={() => setIsCreating(true)}
            className="w-8 h-8 border-2 border-black flex items-center justify-center hover:bg-surge-orange hover:text-white transition-colors"
          >
            <Plus className="w-4 h-4" />
          </button>
        </div>
        
        <div className="flex-1 overflow-auto p-4 space-y-3 custom-scrollbar">
          {collections.map(name => (
            <button
              key={name}
              onClick={() => setSelectedCollection(name)}
              className={cn(
                "w-full flex flex-col items-start p-4 border-2 border-black transition-all relative group",
                selectedCollection === name 
                  ? "bg-surge-orange text-white shadow-none translate-x-[2px] translate-y-[2px]" 
                  : "bg-white text-black hover:bg-surge-gray"
              )}
              style={selectedCollection !== name ? { boxShadow: '3px 3px 0px 0px rgba(0,0,0,1)' } : {}}
            >
              <div className="flex items-center justify-between w-full mb-1">
                <span className="text-xs font-black uppercase truncate tracking-tighter">{name}</span>
                <ChevronRight className={cn("w-3 h-3", selectedCollection === name ? "text-white" : "text-black")} />
              </div>
              {stats?.database?.collections?.[name] && (
                <span className={cn("text-[10px] font-bold", selectedCollection === name ? "text-white/80" : "text-black/40")}>
                  {stats.database.collections[name].vector_count} VECTORS
                </span>
              )}
            </button>
          ))}
        </div>
      </div>

      {/* Detail Content */}
      <div className="flex-1 overflow-auto bg-white p-10">
        {selectedCollection ? (
          <div className="space-y-10">
            <div className="flex items-end justify-between border-b-2 border-black pb-8">
              <div>
                <div className="flex items-center gap-4 mb-2">
                   <h2 className="text-4xl font-black tracking-tighter uppercase leading-none">{selectedCollection}</h2>
                   <div className="flex items-center gap-2">
                      <span className="px-3 py-1 border-2 border-black text-[10px] font-black bg-white uppercase">
                        {stats?.database?.collections?.[selectedCollection]?.dimensions}D
                      </span>
                      <span className="px-3 py-1 border-2 border-black text-[10px] font-black bg-black text-white uppercase">
                        {stats?.database?.collections?.[selectedCollection]?.quantization}
                      </span>
                   </div>
                </div>
                <p className="text-[10px] font-bold text-black/50 tracking-widest uppercase">DATASET_EXPLORATION_MODE</p>
              </div>
              <button 
                onClick={() => handleDelete(selectedCollection)}
                className="btn-brutal btn-brutal-white text-[10px] py-2"
              >
                DELETE_COLLECTION
              </button>
            </div>

            <div className="wire-container overflow-hidden flex flex-col">
              <div className="p-4 border-b-2 border-black bg-surge-gray flex items-center justify-between">
                 <div className="text-[10px] font-black tracking-[0.2em] uppercase flex items-center gap-2">
                  <Database className="w-3 h-3 text-surge-orange" /> RECORDS_BUFFER
                 </div>
                 <div className="text-[10px] font-bold bg-black text-white px-2 py-0.5">PAGE_{page + 1}</div>
              </div>

              <div className="overflow-x-auto">
                <table className="w-full text-left border-collapse">
                  <thead>
                    <tr className="border-b-2 border-black text-[10px] uppercase font-black bg-white">
                      <th className="px-6 py-4 border-r-2 border-black">ID_REF</th>
                      <th className="px-6 py-4 border-r-2 border-black">METADATA_JSON</th>
                      <th className="px-6 py-4 text-center">ACTION</th>
                    </tr>
                  </thead>
                  <tbody className="divide-y-2 divide-black">
                    {vectors.map(vec => (
                      <tr key={vec.id} className="hover:bg-surge-gray transition-colors group">
                        <td className="px-6 py-4 border-r-2 border-black">
                          <div className="text-xs font-black truncate max-w-[200px]" title={vec.id}>
                            {vec.id}
                          </div>
                        </td>
                        <td className="px-6 py-4 border-r-2 border-black">
                          <div className="text-[10px] font-bold text-black/60 font-mono line-clamp-1 max-w-2xl">
                            {vec.metadata ? JSON.stringify(vec.metadata) : 'NULL'}
                          </div>
                        </td>
                        <td className="px-6 py-4 text-center">
                          <button 
                            onClick={() => fetchVectorDetail(vec.id)}
                            className="text-[10px] font-black hover:text-surge-orange transition-colors underline decoration-2 underline-offset-4"
                          >
                            INSPECT
                          </button>
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
                
                {vectors.length === 0 && (
                  <div className="p-20 text-center border-b-2 border-black">
                    <Box className="w-12 h-12 text-black/10 mx-auto mb-4" />
                    <h3 className="text-xs font-black uppercase tracking-widest text-black/40">EMPTY_BUFFER_DETECTED</h3>
                  </div>
                )}
              </div>

              <div className="p-6 bg-white flex items-center justify-between">
                <div className="text-[10px] font-black text-black/40 tracking-widest">
                  TOTAL_PAGES: {Math.ceil((stats?.database?.collections?.[selectedCollection]?.vector_count || 0) / limit) || 1}
                </div>
                <div className="flex gap-4">
                  <button 
                    disabled={page === 0}
                    onClick={() => setPage(p => p - 1)}
                    className="btn-brutal btn-brutal-white py-1 px-3 text-[10px]"
                  >
                    PREV
                  </button>
                  <button 
                    disabled={vectors.length < limit}
                    onClick={() => setPage(p => p + 1)}
                    className="btn-brutal btn-brutal-white py-1 px-3 text-[10px]"
                  >
                    NEXT
                  </button>
                </div>
              </div>
            </div>
          </div>
        ) : (
          <div className="h-full flex flex-col items-center justify-center text-center opacity-20">
            <div className="w-24 h-24 border-4 border-black flex items-center justify-center mb-8 rotate-3">
              <Database className="w-12 h-12" />
            </div>
            <h2 className="text-2xl font-black uppercase tracking-[0.3em]">SELECT_COLLECTION</h2>
          </div>
        )}
      </div>

      {/* Vector Inspector Slide-over */}
      {selectedVector && (
        <div className="fixed inset-0 z-[60] flex justify-end">
          <div className="absolute inset-0 bg-black/40 backdrop-blur-[2px]" onClick={() => setSelectedVector(null)} />
          <div className="relative w-full max-w-2xl bg-white border-l-4 border-black shadow-2xl flex flex-col animate-in slide-in-from-right duration-300">
            <div className="p-8 border-b-4 border-black flex items-center justify-between bg-surge-gray">
              <div>
                <h3 className="text-xl font-black truncate max-w-md uppercase tracking-tighter">{selectedVector.id}</h3>
                <p className="text-[10px] font-bold text-black/50 mt-1 tracking-widest uppercase">VECTOR_RECORD_MANIFEST</p>
              </div>
              <button 
                onClick={() => setSelectedVector(null)}
                className="w-10 h-10 border-2 border-black flex items-center justify-center hover:bg-black hover:text-white transition-all"
              >
                <X className="w-6 h-6" />
              </button>
            </div>
            
            <div className="flex-1 overflow-auto p-10 space-y-12 custom-scrollbar">
              <section>
                <div className="flex items-center gap-2 mb-6">
                  <div className="w-1.5 h-6 bg-surge-orange" />
                  <h4 className="text-[10px] font-black text-black uppercase tracking-[0.2em]">METADATA_STREAM</h4>
                </div>
                <div className="wire-container p-6 bg-surge-gray/30">
                  <pre className="text-[11px] font-mono text-black overflow-auto max-h-60 leading-relaxed font-bold">
                    {JSON.stringify(selectedVector.metadata, null, 4)}
                  </pre>
                </div>
              </section>

              <section>
                <div className="flex items-center justify-between mb-6">
                  <div className="flex items-center gap-2">
                    <div className="w-1.5 h-6 bg-surge-orange" />
                    <h4 className="text-[10px] font-black text-black uppercase tracking-[0.2em]">EMBEDDING_ARRAY</h4>
                  </div>
                  <span className="text-[10px] font-black bg-black text-white px-2 py-0.5">{selectedVector.vector?.length}_DIM</span>
                </div>
                <div className="grid grid-cols-4 md:grid-cols-5 gap-3">
                  {selectedVector.vector?.slice(0, 100).map((v: number, i: number) => (
                    <div key={i} className="text-[10px] font-bold text-black/40 border-2 border-black/10 p-2 text-center font-mono">
                      {v.toFixed(4)}
                    </div>
                  ))}
                  {selectedVector.vector?.length > 100 && (
                    <div className="col-span-full text-center text-[10px] font-black uppercase py-4 border-2 border-dashed border-black/10 mt-2">
                      + {selectedVector.vector.length - 100} DIMENSIONS_REDACTED
                    </div>
                  )}
                </div>
              </section>
            </div>
          </div>
        </div>
      )}

      {/* Create Modal Overlay */}
      {isCreating && (
        <div className="fixed inset-0 bg-black/60 backdrop-blur-sm z-50 flex items-center justify-center p-4">
          <div className="bg-white border-4 border-black w-full max-w-lg overflow-hidden animate-in fade-in zoom-in duration-200">
            <div className="p-8 border-b-4 border-black bg-surge-gray flex items-center justify-between">
              <div>
                <h3 className="text-2xl font-black uppercase tracking-tighter">NEW_COLLECTION</h3>
                <p className="text-[10px] font-bold text-black/40 mt-1 tracking-widest uppercase">INITIALIZE_VECTOR_SPACE</p>
              </div>
              <button onClick={() => setIsCreating(false)} className="hover:rotate-90 transition-transform">
                <X className="w-6 h-6" />
              </button>
            </div>
            <form onSubmit={handleCreate} className="p-8 space-y-6 font-bold">
              <div className="space-y-2">
                <label className="text-[10px] uppercase tracking-widest">COLLECTION_NAME</label>
                <input 
                  autoFocus
                  required
                  type="text" 
                  value={newCollection.name}
                  onChange={e => setNewCollection({...newCollection, name: e.target.value})}
                  placeholder="E.G. CUSTOMER_INDEX_V1"
                  className="w-full bg-white border-2 border-black px-4 py-3 text-sm focus:bg-surge-gray outline-none transition-colors"
                />
              </div>
              <div className="space-y-2">
                <label className="text-[10px] uppercase tracking-widest">DIMENSIONALITY</label>
                <input 
                  required
                  type="number" 
                  value={newCollection.dimensions}
                  onChange={e => setNewCollection({...newCollection, dimensions: Number(e.target.value)})}
                  className="w-full bg-white border-2 border-black px-4 py-3 text-sm outline-none"
                />
              </div>
              
              <div className="grid grid-cols-2 gap-6">
                <div className="space-y-2">
                  <label className="text-[10px] uppercase tracking-widest">METRIC</label>
                  <select 
                    value={newCollection.distance_metric}
                    onChange={e => setNewCollection({...newCollection, distance_metric: e.target.value})}
                    className="w-full bg-white border-2 border-black px-4 py-3 text-sm outline-none appearance-none"
                  >
                    <option value="Cosine">COSINE</option>
                    <option value="Euclidean">EUCLIDEAN</option>
                    <option value="DotProduct">DOT_PRODUCT</option>
                  </select>
                </div>
                <div className="space-y-2">
                  <label className="text-[10px] uppercase tracking-widest">COMPRESSION</label>
                  <select 
                    value={newCollection.quantization}
                    onChange={e => setNewCollection({...newCollection, quantization: e.target.value})}
                    className="w-full bg-white border-2 border-black px-4 py-3 text-sm outline-none appearance-none"
                  >
                    <option value="None">NONE (F32)</option>
                    <option value="SQ8">SQ8 (INT8)</option>
                  </select>
                </div>
              </div>
              
              <div className="flex gap-4 mt-10">
                <button 
                  type="button"
                  onClick={() => setIsCreating(false)}
                  className="flex-1 btn-brutal btn-brutal-white text-xs"
                >
                  ABORT
                </button>
                <button 
                  type="submit"
                  disabled={loading}
                  className="flex-1 btn-brutal btn-brutal-orange text-xs"
                >
                  {loading ? 'INITIALIZING...' : 'CREATE_COLLECTION'}
                </button>
              </div>
            </form>
          </div>
        </div>
      )}
    </div>
  );
}
