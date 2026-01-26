//! Graph storage implementation
//!
//! Handles reading/writing the graph index on disk.

use super::layout::{node_size_bytes, GraphHeader, GRAPH_MAGIC, GRAPH_VERSION};
use crate::error::{Error, Result};
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;

pub struct GraphStorage {
    file: File,
    header: GraphHeader,
    node_size: usize,
}

impl GraphStorage {
    /// Create a new graph file
    pub fn create(path: impl AsRef<Path>, max_degree: u32, entry_point: u32) -> Result<Self> {
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)?;

        let header = GraphHeader {
            magic: *GRAPH_MAGIC,
            version: GRAPH_VERSION,
            num_nodes: 0,
            max_degree,
            entry_point,
            _padding: [0; 4076],
        };

        // Write header
        let mut header_bytes = Vec::with_capacity(4096);
        header_bytes.extend_from_slice(&header.magic);
        header_bytes.extend_from_slice(&header.version.to_le_bytes());
        header_bytes.extend_from_slice(&header.num_nodes.to_le_bytes());
        header_bytes.extend_from_slice(&header.max_degree.to_le_bytes());
        header_bytes.extend_from_slice(&header.entry_point.to_le_bytes());
        header_bytes.extend_from_slice(&header._padding);

        file.write_all(&header_bytes)?;
        file.sync_all()?;

        Ok(Self {
            file,
            header,
            node_size: node_size_bytes(max_degree),
        })
    }

    /// Open an existing graph file
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let mut file = OpenOptions::new().read(true).write(true).open(path)?;

        let mut magic = [0u8; 4];
        file.read_exact(&mut magic)?;
        if &magic != GRAPH_MAGIC {
            return Err(Error::Storage("Invalid graph file magic".into()));
        }

        let mut version_bytes = [0u8; 4];
        file.read_exact(&mut version_bytes)?;
        let version = u32::from_le_bytes(version_bytes);
        if version != GRAPH_VERSION {
            return Err(Error::Storage(format!("Unsupported version: {}", version)));
        }

        let mut num_nodes_bytes = [0u8; 8];
        file.read_exact(&mut num_nodes_bytes)?;
        let num_nodes = u64::from_le_bytes(num_nodes_bytes);

        let mut max_degree_bytes = [0u8; 4];
        file.read_exact(&mut max_degree_bytes)?;
        let max_degree = u32::from_le_bytes(max_degree_bytes);

        let mut entry_point_bytes = [0u8; 4];
        file.read_exact(&mut entry_point_bytes)?;
        let entry_point = u32::from_le_bytes(entry_point_bytes);

        // Skip padding
        file.seek(SeekFrom::Start(4096))?;

        let header = GraphHeader {
            magic,
            version,
            num_nodes,
            max_degree,
            entry_point,
            _padding: [0; 4076],
        };

        Ok(Self {
            file,
            header,
            node_size: node_size_bytes(max_degree),
        })
    }

    /// Read neighbors for a node
    pub fn get_neighbors(&mut self, node_id: u32) -> Result<Vec<u32>> {
        let offset = 4096 + (node_id as u64 * self.node_size as u64);
        self.file.seek(SeekFrom::Start(offset))?;

        let mut count_bytes = [0u8; 4];
        self.file.read_exact(&mut count_bytes)?;
        let count = u32::from_le_bytes(count_bytes) as usize;

        let mut neighbors = Vec::with_capacity(count);
        for _ in 0..count {
            let mut neighbor_bytes = [0u8; 4];
            self.file.read_exact(&mut neighbor_bytes)?;
            neighbors.push(u32::from_le_bytes(neighbor_bytes));
        }

        Ok(neighbors)
    }

    /// Write neighbors for a node (append only supported for new nodes essentially, unless updating in place)
    /// Updating in place is fine since node size is fixed.
    pub fn set_neighbors(&mut self, node_id: u32, neighbors: &[u32]) -> Result<()> {
        let offset = 4096 + (node_id as u64 * self.node_size as u64);
        self.file.seek(SeekFrom::Start(offset))?;

        let bytes = super::layout::serialize_node(neighbors, self.header.max_degree as usize);
        self.file.write_all(&bytes)?;

        // Update node count if we wrote past the end
        if (node_id as u64) >= self.header.num_nodes {
            self.header.num_nodes = node_id as u64 + 1;
            self.update_header()?;
        }

        Ok(())
    }

    fn update_header(&mut self) -> Result<()> {
        self.file.seek(SeekFrom::Start(8))?; // Skip magic + version
        self.file.write_all(&self.header.num_nodes.to_le_bytes())?;
        Ok(())
    }
}
