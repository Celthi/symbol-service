use debugid::DebugId;
use uuid::Uuid;
const UUID_SIZE: usize = 16;

pub struct DebugEntry {
    pub debug_id: DebugId,
    pub location: String,
    pub note: String,
}
// From https://github.com/mozilla/dump_syms
pub fn compute_debug_id(identifier: &[u8], little_endian: bool) -> DebugId {
    // Make sure that we have exactly UUID_SIZE bytes available
    let mut data = [0; UUID_SIZE];
    let len = std::cmp::min(identifier.len(), UUID_SIZE);
    data[0..len].copy_from_slice(&identifier[0..len]);

    if little_endian {
        // The file ELF file targets a little endian architecture. Convert to
        // network byte order (big endian) to match the Breakpad processor's
        // expectations. For big endian object files, this is not needed.
        data[0..4].reverse(); // uuid field 1
        data[4..6].reverse(); // uuid field 2
        data[6..8].reverse(); // uuid field 3
    }

    Uuid::from_slice(&data)
        .map(DebugId::from_uuid)
        .unwrap_or_default()
}
