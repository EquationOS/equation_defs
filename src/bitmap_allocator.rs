use alloc::vec::Vec;

use allocator::{AllocError, AllocResult, BaseAllocator};
use bitmap_allocator::BitAlloc;
use bitmaps::{Bitmap, Bits, BitsImpl};
use memory_addr::{PAGE_SIZE_1G as MAX_ALIGN_1GB, align_down, align_up, is_aligned};

use crate::bitmap::{BitAlloc512, SegmentBitAllocCascade};

/// Page-granularity allocator.
/// refer to [`PageAllocator`] in https://github.com/arceos-org/allocator.git for more details.
/// This is just a simplified version which removes the `PAGE_SIZE` constant
pub trait PageAllocator: BaseAllocator {
    /// Allocate contiguous memory pages with given count and alignment.
    fn alloc_pages(&mut self, num_pages: usize, align_pow2: usize) -> AllocResult<usize>;

    /// Deallocate contiguous memory pages with given position and count.
    fn dealloc_pages(&mut self, pos: usize, num_pages: usize);

    /// Allocate contiguous memory pages with given base address, count and alignment.
    fn alloc_pages_at(
        &mut self,
        base: usize,
        num_pages: usize,
        align_pow2: usize,
    ) -> AllocResult<usize>;

    /// Returns the total number of memory pages.
    fn total_pages(&self) -> usize;

    /// Returns the number of allocated memory pages.
    fn used_pages(&self) -> usize;

    /// Returns the number of available memory pages.
    fn available_pages(&self) -> usize;
}

/// A Segment-aware page-granularity memory allocator based on the [bitmap_allocator].
///
/// It internally uses a bitmap, each bit indicates whether a page has been
/// allocated.
///
/// The `self.page_size` must be a power of two.
#[repr(C)]
pub struct SegmentBitmapPageAllocator<const SIZE: usize>
where
    BitsImpl<{ SIZE }>: Bits,
{
    base: usize,
    segment_granularity: usize,

    page_size: usize,
    used_pages: usize,
    total_pages: usize,

    /// Mark if the physical memory backend is allocated for this sub segments.
    /// 1 indicates allocated, 0 indicates not allocated.
    allocated_bitset: Bitmap<SIZE>,
    inner: SegmentBitAllocCascade<BitAlloc512, SIZE>,
}

impl<const SIZE: usize> SegmentBitmapPageAllocator<{ SIZE }>
where
    BitsImpl<{ SIZE }>: Bits,
{
    pub fn base(&self) -> usize {
        self.base
    }

    pub fn segment_granularity(&self) -> usize {
        self.segment_granularity
    }

    pub fn page_size(&self) -> usize {
        self.page_size
    }
    pub fn used_pages(&self) -> usize {
        self.used_pages
    }
    pub fn total_pages(&self) -> usize {
        self.total_pages
    }

    /// Constructs a new `BitmapPageAllocator` with the given page size from raw memory.
    pub fn init_with_page_size(
        &mut self,
        page_size: usize,
        segment_granularity: usize,
        start: usize,
        size: usize,
    ) {
        assert!(page_size.is_power_of_two());
        assert!(segment_granularity.is_power_of_two());
        assert!(is_aligned(start, segment_granularity));

        self.page_size = page_size;
        self.segment_granularity = segment_granularity;

        self.allocated_bitset.set(
            align_down(start, segment_granularity) / segment_granularity,
            true,
        );

        self.init(start, size);
    }

    pub fn increase_segment_at(&mut self, segment_base: usize) -> bool {
        assert!(is_aligned(segment_base, self.segment_granularity));

        let segment_idx = segment_base / self.segment_granularity;
        // Check if the segment is already allocated.
        if self.allocated_bitset.get(segment_idx) {
            return false;
        }

        // Mark the segment as allocated.
        self.allocated_bitset.set(segment_idx, true);

        // Allocate a new segment.
        let start = segment_idx * self.segment_granularity;
        let end = start + self.segment_granularity;

        // Initialize the inner allocator for the new segment.
        self.inner.insert(start..end);

        true
    }

    pub fn try_decrease_segment(&mut self) {
        let segment_idxes: Vec<usize> = self.allocated_bitset.into_iter().collect();

        for segment_idx in segment_idxes {
            if !self.inner.segment_is_free(segment_idx) {
                continue;
            }
            let start = segment_idx * self.segment_granularity;
            let end = start + self.segment_granularity;
            // Remove the inner allocator for the segment.
            self.inner.remove(start..end);

            // Mark the segment as deallocated.
            self.allocated_bitset.set(segment_idx, false);
        }
    }
}

impl<const SIZE: usize> BaseAllocator for SegmentBitmapPageAllocator<{ SIZE }>
where
    BitsImpl<{ SIZE }>: Bits,
{
    /// Just init first segment.
    fn init(&mut self, start: usize, size: usize) {
        assert!(self.page_size.is_power_of_two());

        // Range for real:  [align_up(start, self.page_size), align_down(start + size, self.page_size))
        let end = align_down(start + size, self.page_size);
        let start = align_up(start, self.page_size);
        self.total_pages = (end - start) / self.page_size;

        // Calculate the base offset stored in the real [`BitAlloc`] instance.
        self.base = align_down(start, MAX_ALIGN_1GB);

        // Range in bitmap: [start - self.base, start - self.base + total_pages * self.page_size)
        let start = start - self.base;
        let start_idx = start / self.page_size;

        self.inner.insert(start_idx..start_idx + self.total_pages);
    }

    fn add_memory(&mut self, _start: usize, _size: usize) -> AllocResult {
        Err(AllocError::NoMemory) // unsupported
    }
}

impl<const SIZE: usize> PageAllocator for SegmentBitmapPageAllocator<{ SIZE }>
where
    BitsImpl<{ SIZE }>: Bits,
{
    fn alloc_pages(&mut self, num_pages: usize, align_pow2: usize) -> AllocResult<usize> {
        // Check if the alignment is valid.
        if align_pow2 > MAX_ALIGN_1GB || !is_aligned(align_pow2, self.page_size) {
            return Err(AllocError::InvalidParam);
        }
        let align_pow2 = align_pow2 / self.page_size;
        if !align_pow2.is_power_of_two() {
            return Err(AllocError::InvalidParam);
        }
        let align_log2 = align_pow2.trailing_zeros() as usize;
        match num_pages.cmp(&1) {
            core::cmp::Ordering::Equal => self
                .inner
                .alloc()
                .map(|idx| idx * self.page_size + self.base),
            core::cmp::Ordering::Greater => self
                .inner
                .alloc_contiguous(None, num_pages, align_log2)
                .map(|idx| idx * self.page_size + self.base),
            _ => return Err(AllocError::InvalidParam),
        }
        .ok_or(AllocError::NoMemory)
        .inspect(|_| self.used_pages += num_pages)
    }

    /// Allocate pages at a specific address.
    fn alloc_pages_at(
        &mut self,
        base: usize,
        num_pages: usize,
        align_pow2: usize,
    ) -> AllocResult<usize> {
        // Check if the alignment is valid,
        // and the base address is aligned to the given alignment.
        if align_pow2 > MAX_ALIGN_1GB
            || !is_aligned(align_pow2, self.page_size)
            || !is_aligned(base, align_pow2)
        {
            return Err(AllocError::InvalidParam);
        }

        let align_pow2 = align_pow2 / self.page_size;
        if !align_pow2.is_power_of_two() {
            return Err(AllocError::InvalidParam);
        }
        let align_log2 = align_pow2.trailing_zeros() as usize;

        let idx = (base - self.base) / self.page_size;

        self.inner
            .alloc_contiguous(Some(idx), num_pages, align_log2)
            .map(|idx| idx * self.page_size + self.base)
            .ok_or(AllocError::NoMemory)
            .inspect(|_| self.used_pages += num_pages)
    }

    fn dealloc_pages(&mut self, pos: usize, num_pages: usize) {
        assert!(
            is_aligned(pos, self.page_size),
            "pos must be aligned to self.page_size"
        );
        if match num_pages.cmp(&1) {
            core::cmp::Ordering::Equal => self.inner.dealloc((pos - self.base) / self.page_size),
            core::cmp::Ordering::Greater => self
                .inner
                .dealloc_contiguous((pos - self.base) / self.page_size, num_pages),
            _ => false,
        } {
            self.used_pages -= num_pages;
        }
    }

    fn total_pages(&self) -> usize {
        self.total_pages
    }

    fn used_pages(&self) -> usize {
        self.used_pages
    }

    fn available_pages(&self) -> usize {
        self.total_pages - self.used_pages
    }
}
