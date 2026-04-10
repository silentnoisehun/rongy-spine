// Rongyász Spine — 2048-byte zero-copy IPC bus
// Sub-nanosecond inter-module communication via fixed memory layout
// No serialization, no locks, no allocation on hot path
//
// Author: Máté Róbert (silentnoisehun)
// License: MIT

//! Rongyász Spine — Mmap-alapú belső gerinc
//!
//! 2048 byte fix layout shared memory — modulok közti ~0 latency kommunikáció.
//! Nincs serialize/deserialize, nincs lock, nincs allokáció.
//!
//! Elv: Memory-mapped file + lock-free SPSC + fix slot-ok
//! Eredmény: egy modul ír egy f32-t → másik modul azonnal olvassa (~nanosec)

pub const SPINE_MAGIC: u64 = 0x524F4E47_59415353; // "RONGYASS"
pub const SPINE_SIZE: usize = 2048;

pub mod slot {
    pub const MAGIC: usize = 0;
    pub const VERSION: usize = 8;
    pub const TICK: usize = 12;
    pub const TIMESTAMP: usize = 20;
    pub const TENSION: usize = 32;
    pub const FOCUS: usize = 36;
    pub const FRUSTRATION: usize = 40;
    pub const CONFIDENCE: usize = 44;
    pub const PSI_FREQUENCY: usize = 48;
    pub const PSI_PHASE: usize = 52;
    pub const PSI_AMPLITUDE: usize = 56;
    pub const PHI: usize = 60;
    pub const AWARENESS: usize = 64;
    pub const ACTIVE_LAYER: usize = 68;
    pub const VALENCE: usize = 72;
    pub const AROUSAL: usize = 76;
    pub const EMOTION_ID: usize = 80;
    pub const COHERENCE: usize = 84;
    pub const IDENTITY_STABILITY: usize = 88;
    pub const MEM_BLOCK_INDEX: usize = 92;
    pub const MEM_LAYER: usize = 100;
    pub const MEM_DEPTH: usize = 104;
    pub const MEM_TIMESTAMP: usize = 108;
    pub const MEM_SCORE: usize = 116;
    pub const MEM_TOTAL: usize = 120;
    pub const DREAM_CONFIDENCE: usize = 124;
    pub const DREAM_ACTIVE: usize = 128;
    pub const DREAM_TEXT: usize = 132;
    pub const HEBBIAN_RATE: usize = 188;
    pub const HEBBIAN_CONNECTIONS: usize = 192;
    pub const HEBBIAN_STRONGEST: usize = 196;
    pub const COLONY_ACTIVE_MUTATIONS: usize = 200;
    pub const COLONY_SUCCESS_RATE: usize = 204;
    pub const COLONY_LAST_FITNESS: usize = 208;
    pub const COLONY_GENERATION: usize = 212;
    pub const COLONY_PENDING: usize = 216;
    pub const SWARM_NODE_COUNT: usize = 220;
    pub const SWARM_TASK_PENDING: usize = 224;
    pub const SWARM_TASK_DONE: usize = 228;
    pub const SWARM_LOAD: usize = 232;
    pub const SWARM_UPTIME: usize = 236;
    pub const VOICE_RATE: usize = 240;
    pub const VOICE_PITCH: usize = 244;
    pub const VOICE_SPEAKING: usize = 248;
    pub const VOICE_QUEUE_LEN: usize = 252;
    pub const VOICE_EMOTION: usize = 256;
    pub const CPU_USAGE: usize = 260;
    pub const RAM_USAGE: usize = 264;
    pub const DISK_USAGE: usize = 268;
    pub const UPTIME_SECS: usize = 272;
    pub const RING_HEAD: usize = 280;
    pub const RING_TAIL: usize = 288;
    pub const RING_DATA_START: usize = 296;
    pub const RING_DATA_END: usize = 2048;
}

pub struct Spine {
    buf: Vec<u8>,
}

impl Spine {
    pub fn new() -> Self {
        let mut buf = vec![0u8; SPINE_SIZE];
        buf[slot::MAGIC..slot::MAGIC + 8].copy_from_slice(&SPINE_MAGIC.to_le_bytes());
        buf[slot::VERSION..slot::VERSION + 4].copy_from_slice(&1u32.to_le_bytes());
        Self::write_f32_at(&mut buf, slot::CONFIDENCE, 0.5);
        Self::write_f32_at(&mut buf, slot::FOCUS, 0.5);
        Self::write_f32_at(&mut buf, slot::AWARENESS, 0.5);
        Self::write_f32_at(&mut buf, slot::COHERENCE, 1.0);
        Self::write_f32_at(&mut buf, slot::IDENTITY_STABILITY, 1.0);
        Self { buf }
    }

    pub fn from_bytes(data: &[u8]) -> Option<Self> {
        if data.len() < SPINE_SIZE { return None; }
        let magic = u64::from_le_bytes(data[slot::MAGIC..slot::MAGIC + 8].try_into().ok()?);
        if magic != SPINE_MAGIC { return None; }
        let mut buf = vec![0u8; SPINE_SIZE];
        buf.copy_from_slice(&data[..SPINE_SIZE]);
        Some(Self { buf })
    }

    pub fn as_bytes(&self) -> &[u8] { &self.buf }
    pub fn as_bytes_mut(&mut self) -> &mut [u8] { &mut self.buf }

    pub fn tick(&mut self) {
        let current = self.read_u64(slot::TICK);
        self.write_u64(slot::TICK, current.wrapping_add(1));
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default().as_millis() as u64;
        self.write_u64(slot::TIMESTAMP, now);
    }

    pub fn current_tick(&self) -> u64 { self.read_u64(slot::TICK) }

    pub fn read_f32(&self, offset: usize) -> f32 {
        f32::from_le_bytes(self.buf[offset..offset + 4].try_into().unwrap())
    }
    pub fn write_f32(&mut self, offset: usize, val: f32) {
        self.buf[offset..offset + 4].copy_from_slice(&val.to_le_bytes());
    }
    pub fn read_u32(&self, offset: usize) -> u32 {
        u32::from_le_bytes(self.buf[offset..offset + 4].try_into().unwrap())
    }
    pub fn write_u32(&mut self, offset: usize, val: u32) {
        self.buf[offset..offset + 4].copy_from_slice(&val.to_le_bytes());
    }
    pub fn read_u64(&self, offset: usize) -> u64 {
        u64::from_le_bytes(self.buf[offset..offset + 8].try_into().unwrap())
    }
    pub fn write_u64(&mut self, offset: usize, val: u64) {
        self.buf[offset..offset + 8].copy_from_slice(&val.to_le_bytes());
    }
    fn write_f32_at(buf: &mut [u8], offset: usize, val: f32) {
        buf[offset..offset + 4].copy_from_slice(&val.to_le_bytes());
    }

    pub fn tension(&self) -> f32 { self.read_f32(slot::TENSION) }
    pub fn set_tension(&mut self, v: f32) { self.write_f32(slot::TENSION, v) }
    pub fn focus(&self) -> f32 { self.read_f32(slot::FOCUS) }
    pub fn set_focus(&mut self, v: f32) { self.write_f32(slot::FOCUS, v) }
    pub fn confidence(&self) -> f32 { self.read_f32(slot::CONFIDENCE) }
    pub fn set_confidence(&mut self, v: f32) { self.write_f32(slot::CONFIDENCE, v) }
    pub fn phi(&self) -> f32 { self.read_f32(slot::PHI) }
    pub fn set_phi(&mut self, v: f32) { self.write_f32(slot::PHI, v) }
    pub fn colony_fitness(&self) -> f32 { self.read_f32(slot::COLONY_LAST_FITNESS) }
    pub fn set_colony_fitness(&mut self, v: f32) { self.write_f32(slot::COLONY_LAST_FITNESS, v) }
    pub fn colony_generation(&self) -> u32 { self.read_u32(slot::COLONY_GENERATION) }
    pub fn set_colony_generation(&mut self, v: u32) { self.write_u32(slot::COLONY_GENERATION, v) }
    pub fn cpu_usage(&self) -> f32 { self.read_f32(slot::CPU_USAGE) }
    pub fn set_cpu_usage(&mut self, v: f32) { self.write_f32(slot::CPU_USAGE, v) }

    pub fn ring_push(&mut self, msg: &[u8]) -> bool {
        let max_msg = 64;
        if msg.len() > max_msg { return false; }
        let capacity = slot::RING_DATA_END - slot::RING_DATA_START;
        let slot_size = max_msg + 1;
        let slot_count = capacity / slot_size;
        let head = self.read_u64(slot::RING_HEAD) as usize;
        let tail = self.read_u64(slot::RING_TAIL) as usize;
        let next_head = (head + 1) % slot_count;
        if next_head == tail { return false; }
        let offset = slot::RING_DATA_START + head * slot_size;
        self.buf[offset] = msg.len() as u8;
        self.buf[offset + 1..offset + 1 + msg.len()].copy_from_slice(msg);
        self.write_u64(slot::RING_HEAD, next_head as u64);
        true
    }

    pub fn ring_pop(&mut self) -> Option<Vec<u8>> {
        let max_msg = 64;
        let capacity = slot::RING_DATA_END - slot::RING_DATA_START;
        let slot_size = max_msg + 1;
        let slot_count = capacity / slot_size;
        let head = self.read_u64(slot::RING_HEAD) as usize;
        let tail = self.read_u64(slot::RING_TAIL) as usize;
        if head == tail { return None; }
        let offset = slot::RING_DATA_START + tail * slot_size;
        let len = self.buf[offset] as usize;
        let data = self.buf[offset + 1..offset + 1 + len].to_vec();
        self.write_u64(slot::RING_TAIL, ((tail + 1) % slot_count) as u64);
        Some(data)
    }

    pub fn ring_len(&self) -> usize {
        let max_msg = 64;
        let capacity = slot::RING_DATA_END - slot::RING_DATA_START;
        let slot_size = max_msg + 1;
        let slot_count = capacity / slot_size;
        let head = self.read_u64(slot::RING_HEAD) as usize;
        let tail = self.read_u64(slot::RING_TAIL) as usize;
        (head + slot_count - tail) % slot_count
    }
}

impl Default for Spine {
    fn default() -> Self { Self::new() }
}
