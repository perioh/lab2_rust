use rand::{thread_rng, Rng};
use std::collections::VecDeque;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

fn main() {
    let buf_size = 4;
    let min_time = 2;
    let max_time = 5;
    let num_processes = 200;

    let ended = Arc::new(AtomicBool::new(false));

    let ended_clone = ended.clone();

    let cpu = Arc::new(Mutex::new(CPU::new()));

    let cpu_clone = cpu.clone();
    let process_thread = thread::spawn(move || {
        for _ in 0..num_processes {
            thread::sleep(Duration::from_millis(
                thread_rng().gen_range(min_time..max_time),
            ));
            let process = CPUProcess::new(thread_rng().gen_range(min_time..max_time));
            cpu_clone.lock().unwrap().insert_new(process, buf_size);
        }
        ended_clone.store(true, std::sync::atomic::Ordering::Relaxed)
    });

    let service_thread = thread::spawn(move || {
        let mut sender_dead = false;
        loop {
            let Some(mut q) = cpu.lock().unwrap().get_first() else {
                if sender_dead {
                    break;
                }
                continue;
            };

            while let Some(process) = q.extract() {
                thread::sleep(std::time::Duration::from_millis(process.interval));
                println!("Slept {}", process.interval)
            }

            let is_over = ended.load(std::sync::atomic::Ordering::Relaxed);
            if is_over {
                sender_dead = true;
            }
        }

        let cpu = cpu.lock().unwrap();
        println!(
            "avg buffer: {}, max buffer: {}",
            cpu.total_queue_elements as f64 / num_processes as f64,
            cpu.max_queue_elements
        )
    });

    process_thread.join().unwrap();
    service_thread.join().unwrap();
}

#[derive(Debug)]
struct CPUQueue {
    processes: Vec<CPUProcess>,
}

impl CPUQueue {
    fn new() -> Self {
        CPUQueue {
            processes: Vec::new(),
        }
    }

    fn insert(&mut self, process: CPUProcess) {
        self.processes.push(process);
    }

    fn extract(&mut self) -> Option<CPUProcess> {
        self.processes.pop()
    }
}

#[derive(Debug, Clone)]
struct CPUProcess {
    interval: u64,
}

impl CPUProcess {
    fn new(interval: u64) -> Self {
        CPUProcess { interval }
    }
}

struct CPU {
    queue: VecDeque<CPUQueue>,
    total_queue_elements: usize,
    max_queue_elements: usize,
}

impl CPU {
    fn new() -> Self {
        CPU {
            queue: VecDeque::new(),
            total_queue_elements: 0,
            max_queue_elements: 0,
        }
    }

    fn insert_new(&mut self, process: CPUProcess, buffer_size: usize) {
        let mut create_new = false;
        if let Some(first_queue) = self.queue.get_mut(0) {
            if first_queue.processes.len() < buffer_size {
                first_queue.insert(process.clone());
            } else {
                create_new = true;
            }
        } else {
            create_new = true;
        }

        if create_new {
            let mut new_queue = CPUQueue::new();
            new_queue.insert(process);
            self.queue.push_back(new_queue);
            println!("Buffer increased to {}", self.queue.len());
        }
        self.total_queue_elements += self.queue.len();
        if self.queue.len() > self.max_queue_elements {
            self.max_queue_elements = self.queue.len();
        }
    }

    fn get_first(&mut self) -> Option<CPUQueue> {
        if self.queue.len() == 0 {
            return None;
        }

        println!("Buffer decreased to {}", self.queue.len());

        let first_queue = self.queue.pop_front().expect("This must exist");

        Some(first_queue)
    }
}
