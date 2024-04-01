use clokwerk::{Scheduler, TimeUnits};
use std::time::Duration;
use tokio;

async fn task_one() {
    println!("Task one is running.");
    // Task one logic here
}

async fn task_two() {
    println!("Task two is running.");
    // Task two logic here
}

pub async fn setup_scheduler() {
    let mut scheduler = Scheduler::with_tz(chrono::Utc);

    // Schedule task_one to run every 10 seconds
    scheduler.every(10.seconds()).run(|| {
        let task = task_one();
        tokio::spawn(task); // Spawn the task asynchronously
    });

    // Schedule task_two to run every 30 seconds
    scheduler.every(5.seconds()).run(|| {
        let task = task_two();
        tokio::spawn(task); // Spawn the task asynchronously
    });

    // Scheduler tick loop
    tokio::spawn(async move {
        loop {
            scheduler.run_pending();
            tokio::time::sleep(Duration::from_millis(100)).await; // Short sleep between checks
        }
    });
}
