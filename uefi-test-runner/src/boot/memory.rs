use uefi::table::boot::{AllocateType, BootServices, MemoryType};

#[cfg(not(feature = "native"))]
use alloc::vec::Vec;

pub fn test(bt: &BootServices) {
    info!("Testing memory functions");

    allocate_pages(bt);
    vec_alloc();
    alloc_alignment();

    memory_map(bt);
}

fn allocate_pages(bt: &BootServices) {
    info!("Allocating some pages of memory");

    let ty = AllocateType::AnyPages;
    let mem_ty = MemoryType::LOADER_DATA;
    let pgs = bt
        .allocate_pages(ty, mem_ty, 1)
        .expect("Failed to allocate a page of memory");

    assert_eq!(pgs % 4096, 0, "Page pointer is not page-aligned");

    // Reinterpret the page as an array of bytes
    let buf = unsafe { &mut *(pgs as *mut [u8; 4096]) };

    // If these don't fail then we properly allocated some memory.
    buf[0] = 0xF0;
    buf[4095] = 0x23;

    // Clean up to avoid memory leaks.
    unsafe { bt.free_pages(pgs, 1) }.unwrap();
}

// Simple test to ensure our custom allocator works with the `alloc` crate.
fn vec_alloc() {
    info!("Allocating a vector through the `alloc` crate");

    #[allow(clippy::useless_vec)]
    let mut values = vec![-5, 16, 23, 4, 0];

    values.sort_unstable();

    assert_eq!(values[..], [-5, 0, 4, 16, 23], "Failed to sort vector");
}

// Simple test to ensure our custom allocator works with correct alignment.
fn alloc_alignment() {
    info!("Allocating a structure with alignment to 0x100");

    #[repr(align(0x100))]
    struct Block(
        // Ignore warning due to field not being read.
        #[allow(dead_code)] [u8; 0x100],
    );

    let value = vec![Block([1; 0x100])];
    assert_eq!(value.as_ptr() as usize % 0x100, 0, "Wrong alignment");
}

fn memory_map(bt: &BootServices) {
    info!("Testing memory map functions");

    // Get the memory descriptor size and an estimate of the memory map size
    let sizes = bt.memory_map_size();

    // 2 extra descriptors should be enough.
    let buf_sz = sizes.map_size + 2 * sizes.entry_size;

    // We will use vectors for convenience.
    let mut buffer = vec![0_u8; buf_sz];

    let mut memory_map = bt
        .memory_map(&mut buffer)
        .expect("Failed to retrieve UEFI memory map");

    memory_map.sort();

    // Collect the descriptors into a vector
    let descriptors = memory_map.entries().copied().collect::<Vec<_>>();

    // Ensured we have at least one entry.
    // Real memory maps usually have dozens of entries.
    assert!(!descriptors.is_empty(), "Memory map is empty");

    let mut curr_value = descriptors[0];

    for value in descriptors.iter().skip(1) {
        if value.phys_start <= curr_value.phys_start {
            panic!("memory map sorting failed");
        }
        curr_value = *value;
    }

    // This is pretty much a sanity test to ensure returned memory isn't filled with random values.
    let first_desc = descriptors[0];

    #[cfg(target_arch = "x86_64")]
    {
        let phys_start = first_desc.phys_start;
        assert_eq!(phys_start, 0, "Memory does not start at address 0");
    }
    let page_count = first_desc.page_count;
    assert!(page_count != 0, "Memory map entry has zero size");
}
