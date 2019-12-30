const MAX_WORKER: usize = 4;

// Here is the work we want done: a simple non-recursive Fibonacci calculation.
fn fib(n: u64) -> u64 {
    // Special case: the 0th Fib. number is 0.
    if n == 0 {
        return 0;
    }
    // Special case: the 1st Fib. number is 1.
    if n == 1 {
        return 1;
    }

    let mut iteration = 0;
    let mut sum = 0;
    let mut last = fib(0);
    let mut current = fib(1);

    // Loop through all the Fib. numbers until we get to
    // the nth one.
    while iteration < n - 1 {
        sum = last + current;
        last = current;
        current = sum;
        iteration += 1;
    }

    return sum;
}

// Mutex stands for MUTually EXclusive. It essentially ensures that only
// one thread has access to a given resource at one time.
use std::sync::Mutex;

// A VecDeque is a double-ended queue, but we will only be using it in forward
// mode; that is, we will push onto the back and pull from the front.
use std::collections::VecDeque;

// Finally we wrap the whole thing in Arc (Atomic Reference Counting) so that
// we can safely share it with other threads. Arc (std::sync::arc) is a lot
// like Rc (std::rc::Rc), in that it allows multiple references to some memory
// which is freed when no references remain, except that it is atomic, making
// it comparitively slow but able to be shared across the thread boundary.
use std::sync::Arc;

// All three of these types are wrapped around a generic type T.
// T is required to be Send (a marker trait automatically implemented when
// it is safe to do so) because it denotes types that are safe to move between
// threads, which is the whole point of the WorkQueue.
// For this implementation, T is required to be Copy as well, for simplicity.

/// A generic work queue for work elements which can be trivially copied.
/// Any producer of work can add elements and any worker can consume them.
/// WorkQueue derives Clone so that it can be distributed among threads.
#[derive(Clone)]
struct WorkQueue<T: Send + Copy> {
    inner: Arc<Mutex<VecDeque<T>>>,
}

impl<T: Send + Copy> WorkQueue<T> {
    // Creating one of these by hand would be kind of a pain, so let's provide a
    // convenience function.

    /// Creates a new WorkQueue, ready to be used.
    fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    // This is the function workers will use to acquire work from the queue.
    // They will call it in a loop, checking to see if there is any work available.

    /// Blocks the current thread until work is available, then
    /// gets the data required to perform that work.
    ///
    /// # Errors
    /// Returns None if there is no more work in the queue.
    ///
    /// # Panics
    /// Panics if the underlying mutex became poisoned. This is exceedingly
    /// unlikely.
    fn get_work(&self) -> Option<T> {
        // Try to get a lock on the Mutex. If this fails, there is a
        // problem with the mutex - it's poisoned, meaning that a thread that
        // held the mutex lock panicked before releasing it. There is no way
        // to guarantee that all its invariants are upheld, so we need to not
        // use it in that case.
        let maybe_queue = self.inner.lock();
        // A lot is going on here. self.inner is an Arc of Mutex. Arc can deref
        // into its internal type, so we can call the methods of that inner
        // type (Mutex) without dereferencing, so this is like
        //      *(self.inner).lock()
        // but doesn't look awful. Mutex::lock() returns a
        // Result<MutexGuard<VecDeque<T>>>.

        // Unwrapping with if let, we get a MutexGuard, which is an RAII guard
        // that unlocks the Mutex when it goes out of scope.
        if let Ok(mut queue) = maybe_queue {
            // queue is a MutexGuard<VecDeque>, so this is like
            //      (*queue).pop_front()
            // Returns Some(item) or None if there are no more items.
            queue.pop_front()

        // The function has returned, so queue goes out of scope and the
        // mutex unlocks.
        } else {
            // There's a problem with the mutex.
            panic!("WorkQueue::get_work() tried to lock a poisoned mutex");
        }
    }

    // Both the controller (main thread) and possibly workers can use this
    // function to add work to the queue.

    /// Blocks the current thread until work can be added, then
    /// adds that work to the end of the queue.
    /// Returns the amount of work now in the queue.
    ///
    /// # Panics
    /// Panics if the underlying mutex became poisoned. This is exceedingly
    /// unlikely.
    fn add_work(&self, work: T) -> usize {
        // As above, try to get a lock on the mutex.
        if let Ok(mut queue) = self.inner.lock() {
            // As above, we can use the MutexGuard<VecDeque<T>> to access
            // the internal VecDeque.
            queue.push_back(work);
            // Now return the length of the queue.
            queue.len()
        } else {
            panic!("WorkQueue::add_work() tried to lock a poisoned mutex");
        }
    }
}

// Now we have a way of getting work from one thread to many threads.
// We need one more thing: a way to tell the workers when they're done.
// We could just say that they should give up when the work queue is empty.
// This will work in situations like this one, but not in situations where
// new work can be created after all the initial work is complete.
// In our case, it gives the capability to start workers before adding anything
// to the work queue, so they can work while the controller is adding work.
//
// For this we'll use another Mutex-based thing, but this time it will just
// wrap a boolean value. We'll call this a syncronization flag, or SyncFlag.
//
// However, we only want the controller thread to be able to tell the workers
// that they're done, so we'll go the route of std::sync::mpsc and create a
// Transmitter and Reciever version of the struct.
//
// The transmitter is where messages are sent; it has the ability to set the
// underlying Boolean value. The receiver only has the ability to read that
// value.

/// SyncFlagTx is the transmitting (mutable) half of a Single Producer,
/// Multiple Consumer Boolean (e.g. the opposite of std::sync::mpsc).
/// A single controller can use this to send info to any number of worker
/// threads, for instance.
///
/// SyncFlagTx is not Clone because it should only exist in one place.
struct SyncFlagTx {
    inner: Arc<Mutex<bool>>,
}

impl SyncFlagTx {
    // This function will be used by the controller thread to tell the worker
    // threads about the end of computation.

    /// Sets the interior value of the SyncFlagTx which will be read by any
    /// SyncFlagRx that exist for this SyncFlag.
    ///
    /// # Errors
    /// If the underlying mutex is poisoned this may return an error.
    fn set(&mut self, state: bool) -> Result<(), ()> {
        if let Ok(mut v) = self.inner.lock() {
            // The * (deref operator) means assigning to what's inside the
            // MutexGuard, not the guard itself (which would be silly)
            *v = state;
            Ok(())
        } else {
            Err(())
        }
    }
}

/// SyncFlagRx is the receiving (immutable) half of a Single Producer,
/// Multiple Consumer Boolean (e.g. the opposite of std::sync::mpsc).
/// An number of worker threads can use this to get info from a single
/// controller, for instance.
///
/// SyncFlagRx is Clone so it can be shared across threads.
#[derive(Clone)]
struct SyncFlagRx {
    inner: Arc<Mutex<bool>>,
}

impl SyncFlagRx {
    // This function will be used by the worker threads to check if they should
    // stop looking for work to do.

    /// Gets the interior state of the SyncFlagRx to whatever the corresponding
    /// SyncFlagTx last set it to.
    ///
    /// # Errors
    /// If the underlying mutex is poisoned this might return an error.
    fn get(&self) -> Result<bool, ()> {
        if let Ok(v) = self.inner.lock() {
            // Deref the MutexGuard to get at the bool inside
            Ok(*v)
        } else {
            Err(())
        }
    }
}

/// Create a new SyncFlagTx and SyncFlagRx that can be used to share a bool
/// across a number of threads.
fn new_syncflag(initial_state: bool) -> (SyncFlagTx, SyncFlagRx) {
    let state = Arc::new(Mutex::new(initial_state));
    let tx = SyncFlagTx {
        inner: state.clone(),
    };
    let rx = SyncFlagRx {
        inner: state.clone(),
    };

    return (tx, rx);
}

fn main() {
    // Create a new work queue to keep track of what work needs to be done.
    // Note that the queue is internally mutable (or, rather, the Mutex is),
    // but this binding doesn't need to be mutable. This isn't unsound because
    // the Mutex ensures at runtime that no two references can be used;
    // therefore no mutation can occur at the same time as aliasing.
    let queue = WorkQueue::new();

    // Create a MPSC (Multiple Producer, Single Consumer) channel. Every worker
    // is a producer, the main thread is a consumer; the producers put their
    // work into the channel when it's done.
    use std::sync::mpsc::channel;
    let (results_tx, results_rx) = channel();

    // Create a SyncFlag to share whether or not there are more jobs to be done.
    let (mut more_jobs_tx, more_jobs_rx) = new_syncflag(true);

    // std::thread allows us to spawn threads to do work in.
    use std::thread;
    // This Vec will hold thread join handles to allow us to not exit while work
    // is still being done. These handles provide a .join() method which blocks
    // the current thread until the thread referred to by the handle exits.
    let mut threads = Vec::new();

    println!("Spawning {} workers.", MAX_WORKER);

    for thread_num in 0..MAX_WORKER {
        // Get a reference to the queue for the thread to use
        // .clone() here doesn't clone the actual queue data, but rather the
        // internal Arc produces a new reference for use in the new queue
        // instance.
        let thread_queue = queue.clone();

        // Similarly, create a new transmitter for the thread to use
        let thread_results_tx = results_tx.clone();

        // ... and a SyncFlagRx for the thread.
        let thread_more_jobs_rx = more_jobs_rx.clone();

        // thread::spawn takes a closure (an anonymous function that "closes"
        // over its environment). The move keyword means it takes ownership of
        // those variables, meaning they can't be used again in the main thread.
        let handle = thread::spawn(move || {
            // A varaible to keep track of how much work was done.
            let mut work_done = 0;

            // Loop while there's expected to be work, looking for work.
            while thread_more_jobs_rx.get().unwrap() {
                // If work is available, do that work.
                if let Some(work) = thread_queue.get_work() {
                    // Do some work.
                    let result = fib(work);

                    // Record that some work was done.
                    work_done += 1;

                    // Send the work and the result of that work.
                    //
                    // Sending could fail. If so, there's no use in
                    // doing any more work, so abort.
                    match thread_results_tx.send((work, result)) {
                        Ok(_) => (),
                        Err(_) => {
                            break;
                        }
                    }
                }

                // Signal to the operating system that now is a good time
                // to give another thread a chance to run.
                //
                // This isn't strictly necessary - the OS can preemptively
                // switch between threads, without asking - but it helps make
                // sure that other threads do get a chance to get some work.
                std::thread::yield_now();
            }

            // Report the amount of work done.
            println!("Thread {} did {} jobs.", thread_num, work_done);
        });

        // Add the handle for the newly spawned thread to the list of handles
        threads.push(handle);
    }

    println!("Workers successfully started.");

    println!("Adding jobs to the queue.");
    // Variables to keep track of the number of jobs we expect to do.
    let mut jobs_remaining = 0;
    let mut jobs_total = 0;

    // Just add some numbers to the queue.
    // These numbers will be passed into fib(), so they need to stay pretty
    // small.
    for work in 0..90 {
        // Add each one several times.
        for _ in 0..100 {
            jobs_remaining = queue.add_work(work);
            jobs_total += 1;
        }
    }

    // Report that some jobs were inserted, and how many are left to be done.
    // This is interesting because the workers have been taking jobs out of the queue
    // the whole time the control thread has been putting them in!
    //
    // Try removing the use of std::thread::yield_now() in the thread closure.
    // You'll probably (depending on your system) notice that the number remaining
    // after insertion goes way up. That's because the operating system is usually
    // (not always, but usually) fairly conservative about interrupting a thread
    // that is actually doing work.
    //
    // Similarly, if you add a call to yield_now() in the loop above, you'll see the
    // number remaining probably drop to 1 or 2. This can also change depending on
    // how optimized the output code is - try `cargo run --release` vs `cargo run`.
    //
    // This inconsistency should drive home to you that you as the programmer can't
    // make any assumptions at all about when and in what order things will happen
    // in parallel code unless you use thread control primatives as demonstrated
    // in this program.
    println!(
        "Total of {} jobs inserted into the queue ({} remaining at this time).",
        jobs_total, jobs_remaining
    );

    // Get completed work from the channel while there's work to be done.
    while jobs_total > 0 {
        match results_rx.recv() {
            // If the control thread successfully receives, a job was completed.
            Ok(_) => jobs_total -= 1,
            // If the control thread is the one left standing, that's pretty
            // problematic.
            Err(_) => {
                panic!("All workers died unexpectedly.");
            }
        }
    }

    // When all the jobs are completed, inform the workers.
    more_jobs_tx.set(false).unwrap();

    // If we didn't do that, the workers would just look for work forever.
    // This is useful because many applications of this technique don't
    // have a defined stopping point that is known in advance - that is,
    // they will have to perform a lot of work that isn't known at the time
    // the work queue is created.
    //
    // A SyncFlag can be used so that when the main thread encounters a
    // kill condition (e.g. Ctrl+C, or perhaps a fatal error of some kind),
    // it can gracefully shut down all of those workers at once.

    // Just make sure that all the other threads are done.
    for handle in threads {
        handle.join().unwrap();
    }
}
