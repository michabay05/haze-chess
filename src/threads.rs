use crate::engine::Engine;
use crate::search::{self, SearchData};

use std::thread::JoinHandle;

pub struct WorkerThread {
    pub id: usize,
    pub handle: Option<JoinHandle<()>>,
}

pub fn launch_search_thread(engine: &mut Engine, depth: u32) {
    let data = SearchData::from_engine(&engine);
    let worker_thread_count = engine.worker_thread_count;
    let th = std::thread::spawn(move || {
        search::search_pos(&data, depth, worker_thread_count);
    });

    engine.search_thread = Some(th);
}

pub fn create_search_workers(data: &SearchData, depth: u32, thread_count: usize) -> Vec<WorkerThread> {
    let mut workers = vec![];
    for i in 0..thread_count {
        let data = data.clone();
        let th = std::thread::spawn(move || {
            search::worker_search_pos(data, depth, i);
        });
        let worker = WorkerThread {
            id: i,
            handle: Some(th)
        };
        workers.push(worker);
    }
    workers
}
