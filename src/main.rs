/* Extern Crates */
extern crate crossbeam_channel;
extern crate rand;
extern crate scheduled_thread_pool;

/* Use Statements */
use crossbeam_channel::bounded;
use rand::Rng;
use scheduled_thread_pool::ScheduledThreadPool;
//use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, Instant};
use std::{format, thread};

fn main() {
    /* Threadpool */
    let pool = ScheduledThreadPool::new(10);
    let atc_pool = ScheduledThreadPool::new(2);
    /* Data Structures */
    let (tx_plane_buffer, rx_plane_buffer) = bounded(3);
    let tx_plane_buffer_clone = tx_plane_buffer.clone();
    let (tx_dockbay, rx_dockbay) = bounded(6);
    let tx_dockbay_clone = tx_dockbay.clone();
    let (tx_event2atc, rx_event2atc) = bounded(2);
    let rx_event2atc_clone = rx_event2atc.clone();
    let (tx_atc2cleaner, rx_atc2cleaner) = bounded(2);
    let tx_atc2cleaner_clone = tx_atc2cleaner.clone();
    let (tx_cleaner2atc, rx_cleaner2atc) = bounded(2);
    let rx_cleaner2atc_clone = rx_cleaner2atc.clone();

    // let event = Arc::new((Mutex::new(false), Condvar::new()));
    // let event2 = Arc::clone(&event);
    // let event3 = Arc::clone(&event);

    // Generate Event Thread
    let gen_event = pool.execute_with_fixed_delay(
        Duration::from_millis(0),
        Duration::from_millis(5000),
        move || {
            let mut rng = rand::thread_rng();
            let chance_event = rng.gen_range(0..101);
            if chance_event > 70 {
                println!(
                    "Gen_Event: Event is generated true {} {:?}",
                    chance_event,
                    Instant::now()
                );
                tx_event2atc.send(true).unwrap();
                tx_event2atc.send(true).unwrap();
                // Wait until all the event message has been sent out.
                while !tx_event2atc.is_empty() {
                    thread::sleep(Duration::from_millis(500));
                }
            } else {
                println!("Gen_Event: {} {:?}", chance_event, Instant::now());
            }
        },
    );

    /* Airport Traffic Control */
    // Incoming ATC
    let atc_incoming = atc_pool.execute_with_fixed_delay(
        Duration::from_secs(0),
        Duration::from_millis(2000),
        move || {
            let event_val = match rx_event2atc.try_recv() {
                Ok(val) => val,
                Err(_e) => false, //temp
            };
            if event_val {
                println!("IATC: Event Happened {:?}", Instant::now());
                tx_atc2cleaner.send(true).unwrap();
                let finish_cleaning = rx_cleaner2atc.recv().unwrap();
                if finish_cleaning {
                    println!("IATC: Free to resume operation  {:?}", Instant::now());
                }
                else{
                    panic!("Finish cleaning send false??");
                }
            } else {
                if !tx_dockbay.is_full() {
                    match rx_plane_buffer.recv_timeout(Duration::from_millis(500)) {
                        Ok(incoming_plane) => {
                            println!(
                                "IATC: Incoming Airplane {}, A runway and docking bay has been assigned for you {:?}",
                                incoming_plane, Instant::now()
                            );
                            tx_dockbay.send(incoming_plane).unwrap();
                        },
                        Err(_e) => println!("No more plane in the buffer"),
                    };
                }else{
                    println!("IATC: Docking Bay is Full {:?}", Instant::now());
                }
            }
        },
    );

    // Departing ATC
    let atc_departing = atc_pool.execute_with_fixed_delay(
        Duration::from_millis(1500),
        Duration::from_millis(3000),
        move || {
            let event_val = match rx_event2atc_clone.try_recv() {
                Ok(val) => val,
                Err(_e) => false,
            };
            if event_val {
                println!("DATC: Event Happened {:?}", Instant::now());
                tx_atc2cleaner_clone.send(true).unwrap();
                let finish_cleaning = rx_cleaner2atc_clone.recv().unwrap();
                if finish_cleaning {
                    println!("DATC: Free to resume operation  {:?}", Instant::now());
                } else {
                    panic!("Finish cleaning send false??");
                }
            } else {
                match rx_dockbay.recv_timeout(Duration::from_millis(500)) {
                    Ok(departing_plane) => {
                        println!(
                            "DATC: Airplane {} has departed {:?}",
                            departing_plane,
                            Instant::now()
                        );
                    }
                    Err(_e) => println!("No more plane in the docking bay"),
                }
            }
        },
    );

    // Cleaner Thread
    let cleaner = pool.execute_at_fixed_rate(
        Duration::from_secs(0),
        Duration::from_millis(1000),
        move || {
            let confirmation1 = rx_atc2cleaner.recv().expect("Disconnected?");
            let confirmation2 = rx_atc2cleaner.recv().expect("Disonnected 2?");
            if confirmation1 && confirmation2 {
                //Further checking
                println!(
                    "Cleaner: Please wait while the cleaner is working! {:?}",
                    Instant::now()
                );
                thread::sleep(Duration::from_secs(5)); //Simulate cleaner working and then set event_val to false to simulate runway is now cleaned
                println!(
                    "Cleaner: Has finished cleaning the runway {:?}",
                    Instant::now()
                );
                tx_cleaner2atc.send(true).unwrap();
                tx_cleaner2atc.send(true).unwrap();
            } else {
                println!("Eh?? Something Wrong {:?}", Instant::now());
                panic!("Something went wrong on cleaner thread"); //Literally shouldnt happen
            }
        },
    );

    // Generate Airplane Thread (Generate 120 Airplanes before gracefully shutting down)
    for i in 1..21 {
        let tx_plane_buffer_clone = tx_plane_buffer.clone();
        pool.execute(move || {
            let airline_code = vec!["AA", "EL", "AK", "CA", "GA"];
            let mut rng = rand::thread_rng();
            let random_airline_code = rng.gen_range(0..5);
            let random_airline_number = rng.gen_range(100..999);
            let incoming_plane = format!(
                "{}{}",
                airline_code[random_airline_code], random_airline_number
            );
            let inc_plane_clone = incoming_plane.clone();
            match tx_plane_buffer_clone.send_timeout(incoming_plane, Duration::from_secs(1)) {
                Ok(()) => println!(
                    "GenPlane: Airplane {} is requesting for runway and docking bay {:?}",
                    inc_plane_clone, Instant::now()
                ),
                Err(_e) => {
                    println!(
                        "GenPlane: Airplane buffer is full, airplane is rerouted to another airport {:?}", Instant::now()
                    )
                }
            }
        });
        println!("Main: {} Airplane has been generated", i);
        thread::sleep(Duration::from_secs(3));
    }

    while !tx_dockbay_clone.is_empty() || !tx_plane_buffer_clone.is_empty() {}
    // println!("gen_event cancelled");
    // gen_event.cancel();
    // println!("atc_departing cancelled");
    // atc_departing.cancel();
    // println!("atc_incoming cancelled");
    // atc_incoming.cancel();
    // println!("cleaner cancelled");
    // cleaner.cancel();
    // println!("everything cancelled");
    thread::sleep(Duration::from_secs(5));
}
