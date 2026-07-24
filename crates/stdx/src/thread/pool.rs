//! [`Pool`] implements a basic custom thread pool
//! inspired by the [`threadpool` crate](http://docs.rs/threadpool).
//! When you spawn a task you specify a thread intent
//! so the pool can schedule it to run on a thread with that intent.
//! rust-analyzer uses this to prioritize work based on latency requirements.
//!
//! The thread pool is implemented entirely using
//! the threading utilities in [`crate::thread`].

use std::{
    cell::Cell,
    marker::PhantomData,
    panic::{self, UnwindSafe},
    sync::{
        Arc, Condvar, Mutex,
        atomic::{AtomicBool, Ordering},
    },
};

use crossbeam_channel::{Receiver, Sender};
use crossbeam_utils::sync::WaitGroup;

use crate::thread::{Builder, JoinHandle, ThreadIntent};

thread_local! {
    static ACTIVE_POOL: Cell<Option<Arc<PoolShared>>> = const { Cell::new(None) };
}

pub struct Pool {
    // `_handles` is never read: the field is present
    // only for its `Drop` impl.

    // The worker threads exit once the channel closes;
    // make sure to keep `job_sender` above `handles`
    // so that the channel is actually closed
    // before we join the worker threads!
    job_sender: Sender<Job>,
    _handles: Box<[JoinHandle]>,
    shared: Arc<PoolShared>,
}

struct PoolShared {
    extant_tasks: Mutex<usize>,
    empty_queue: Condvar,
    cancellation_requested: AtomicBool,
}

struct Job {
    requested_intent: ThreadIntent,
    f: Box<dyn FnOnce() + Send + UnwindSafe + 'static>,
}

impl Pool {
    /// # Panics
    ///
    /// Panics if job panics
    #[must_use]
    pub fn new(threads: usize) -> Self {
        const STACK_SIZE: usize = 8 * 1024 * 1024;
        const INITIAL_INTENT: ThreadIntent = ThreadIntent::Worker;

        let (job_sender, job_receiver) = crossbeam_channel::unbounded();
        let shared = Arc::new(PoolShared {
            extant_tasks: Mutex::new(0),
            empty_queue: Condvar::new(),
            cancellation_requested: AtomicBool::new(false),
        });

        let mut handles = Vec::with_capacity(threads);
        for idx in 0..threads {
            let handle = Builder::new(INITIAL_INTENT, format!("Worker{idx}",))
                .stack_size(STACK_SIZE)
                .allow_leak(true)
                .spawn({
                    let shared = Arc::clone(&shared);
                    let job_receiver: Receiver<Job> = job_receiver.clone();
                    move || {
                        ACTIVE_POOL.set(Some(Arc::clone(&shared)));

                        let mut current_intent = INITIAL_INTENT;
                        for job in job_receiver {
                            if !shared.cancellation_requested.load(Ordering::Acquire) {
                                if job.requested_intent != current_intent {
                                    job.requested_intent.apply_to_current_thread();
                                    current_intent = job.requested_intent;
                                }
                                // discard the panic, we should've logged the backtrace already
                                drop(panic::catch_unwind(job.f));
                            }

                            let mut extant_tasks = shared.extant_tasks.lock().unwrap();
                            *extant_tasks -= 1;
                            if *extant_tasks == 0 {
                                shared.empty_queue.notify_all();
                            }
                        }
                    }
                })
                .expect("failed to spawn thread");

            handles.push(handle);
        }

        Self { _handles: handles.into_boxed_slice(), shared, job_sender }
    }

    pub fn spawn<F>(&self, intent: ThreadIntent, f: F)
    where
        F: FnOnce() + Send + UnwindSafe + 'static,
    {
        let f = Box::new(move || {
            if cfg!(debug_assertions) {
                intent.assert_is_used_on_current_thread();
            }
            f();
        });

        let job = Job { requested_intent: intent, f };
        *self.shared.extant_tasks.lock().unwrap() += 1;
        self.job_sender.send(job).unwrap();
    }

    pub fn scoped<'pool, 'scope, F, R>(&'pool self, f: F) -> R
    where
        F: FnOnce(&Scope<'pool, 'scope>) -> R,
    {
        let wg = WaitGroup::new();
        let scope = Scope { pool: self, wg, _marker: PhantomData };
        let r = f(&scope);
        scope.wg.wait();
        r
    }

    #[must_use]
    pub fn len(&self) -> usize {
        *self.shared.extant_tasks.lock().unwrap()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub fn unwind_if_cancelled() {
        let shared = ACTIVE_POOL.take();
        if let Some(shared) = shared {
            let should_cancel = shared.cancellation_requested.load(Ordering::Acquire);
            ACTIVE_POOL.set(Some(shared));
            if should_cancel {
                // We use resume and not panic here to avoid running the panic
                // hook (that is, to avoid collecting and printing backtrace).
                std::panic::resume_unwind(Box::new(PoolTaskCancelled))
            }
        }
    }

    /// Cancels running and pending tasks, and waits for them to complete.
    /// After calling this method, the pool is tainted and no new tasks will run forever.
    pub fn cancel_and_taint(&mut self) {
        self.shared.cancellation_requested.store(true, Ordering::Release);

        let extant_tasks = self.shared.extant_tasks.lock().unwrap();
        drop(
            self.shared
                .empty_queue
                .wait_while(extant_tasks, |extant_tasks| *extant_tasks != 0)
                .unwrap(),
        );
    }
}

impl Drop for Pool {
    fn drop(&mut self) {
        self.cancel_and_taint();
    }
}

struct PoolTaskCancelled;

pub struct Scope<'pool, 'scope> {
    pool: &'pool Pool,
    wg: WaitGroup,
    _marker: PhantomData<fn(&'scope ()) -> &'scope ()>,
}

impl<'scope> Scope<'_, 'scope> {
    pub fn spawn<F>(&self, intent: ThreadIntent, f: F)
    where
        F: 'scope + FnOnce() + Send + UnwindSafe,
    {
        let wg = self.wg.clone();
        let f = Box::new(move || {
            if cfg!(debug_assertions) {
                intent.assert_is_used_on_current_thread();
            }
            f();
            drop(wg);
        });

        let job = Job {
            requested_intent: intent,
            f: unsafe {
                std::mem::transmute::<
                    Box<dyn 'scope + FnOnce() + Send + UnwindSafe>,
                    Box<dyn 'static + FnOnce() + Send + UnwindSafe>,
                >(f)
            },
        };
        *self.pool.shared.extant_tasks.lock().unwrap() += 1;
        self.pool.job_sender.send(job).unwrap();
    }
}
