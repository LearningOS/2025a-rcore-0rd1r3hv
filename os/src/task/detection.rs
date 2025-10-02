use alloc::vec;
use alloc::vec::Vec;
use core::mem::variant_count;

/// mark resource type
pub enum ResourceType {
    /// if it's a mutex
    Mutex,
    /// if it's a semaphore
    Semaphore,
}

const NUM_RESOURCE_TYPES: usize = variant_count::<ResourceType>();

#[derive(Clone)]
struct ResourceBank {
    available: Vec<usize>,
    allocation: Vec<Vec<usize>>,
    need: Vec<Vec<usize>>,
}

impl ResourceBank {
    pub fn new() -> Self {
        ResourceBank { available: Vec::new(), allocation: vec![Vec::new()], need: vec![Vec::new()] }
    }
}

/// for deadlock check
pub struct DeadlockDetector {
    /// undefined behavior when enable after some resources are allocated
    enabled: bool,
    banks: Vec<ResourceBank>,
}

impl DeadlockDetector {
    /// get a new instance
    pub fn new() -> Self {
        Self { enabled: false, banks: vec![ResourceBank::new(); NUM_RESOURCE_TYPES] }
    }
    /// set the available resource
    pub fn set_available(&mut self, rtype: ResourceType, idx: usize, num: usize) {
        if self.enabled == false {
            return
        }
        let available = &mut self.banks[rtype as usize].available;
        if available.len() == idx {
            available.push(0);
        }
        available[idx] = num;
    }
    /// increase the available resource
    pub fn increase_available(&mut self, rtype: ResourceType, idx: usize, num: usize) {
        if self.enabled == false {
            return
        }
        self.banks[rtype as usize].available[idx] += num;
    }
    /// increase need and check if the system is safe currently
    pub fn increase_need_and_check(&mut self, rtype: ResourceType, tid: usize, rid: usize, num: usize) -> bool {
        if self.enabled == false {
            return true
        }
        let bank = &mut self.banks[rtype as usize];
        let mut work = bank.available.clone();
        let mut finish = vec![false; bank.need.len()];
        let allocation = &bank.allocation;
        let need = &mut bank.need;
        while need.len() < tid + 1 {
            need.push(Vec::new());
        }
        while need[tid].len() < rid + 1 {
            need[tid].push(0);
        }
        need[tid][rid] += num;
        // Finish[i] == false;
        // Need[i,j] <= Work[j];
        // length of need will not be longer than work
        while let Some(tid) = finish.iter().zip(need.iter()).position(|(finish, need)|
                !finish && need.iter().zip(work.iter()).all(|(need, work)| need <= work)
        ) {
            // Work[j] = Work[j] + Allocation[i, j];
            // Finish[i] = true;
            // if allocation is none we don't need to modify work
            allocation.get(tid).map(|allocation| {
                work.iter_mut().zip(allocation.iter()).for_each(|(work, allocation)| *work += allocation);
            });
            finish[tid] = true;
        }
        if finish.iter().all(|finish| *finish == true) {
            true
        } else {
            need[tid][rid] -= num;
            false
        }
    }
    /// decrease need (need is not 0)
    pub fn decrease_need(&mut self, rtype: ResourceType, tid: usize, rid: usize, num: usize) {
        self.banks[rtype as usize].need[tid][rid] -= num;
    }
    /// allocate some resource (decrease available, decrease need, increase allocation) (available and need must not be 0)
    pub fn allocate(&mut self, rtype: ResourceType, tid: usize, rid: usize, num: usize) {
        if self.enabled == false {
            return
        }
        let bank = &mut self.banks[rtype as usize];
        bank.available[rid] -= num;
        bank.need[tid][rid] -=num;
        let allocation = &mut bank.allocation;
        while allocation.len() < tid + 1 {
            allocation.push(Vec::new());
        }
        while allocation[tid].len() < rid + 1 {
            allocation[tid].push(0);
        }
        allocation[tid][rid] += num;
    }
    /// deallocate some resource (increase available, decrease allocation) (allocation and need must not be 0)
    pub fn deallocate(&mut self, rtype: ResourceType, tid: usize, rid: usize, num: usize) {
        if self.enabled == false {
            return
        }
        let bank = &mut self.banks[rtype as usize];
        bank.available[rid] += num;
        bank.allocation[tid][rid] -= num;
    }
    /// enable or disable
    pub fn set_enabled(&mut self, enabled: bool) -> isize {
        self.enabled = enabled;
        0
    }
}