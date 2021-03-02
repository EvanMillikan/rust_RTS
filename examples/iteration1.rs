/* Extern Crates */
extern crate crossbeam_channel;
extern crate rand;
extern crate scheduled_thread_pool;

/* Use Statements */
use crossbeam_channel::{bounded, unbounded};
use rand::Rng;
use scheduled_thread_pool::ScheduledThreadPool;
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

    // Duration from event is generated true until incoming ATC notices
    let (tx_timer_event1, rx_timer_event1) = unbounded();
    // Duration from event is generated true until departing ATC notices.
    let (tx_timer_event2, rx_timer_event2) = unbounded();
    // Duration from event is generated true until cleaner is dispatched
    let (tx_timer_event3, rx_timer_event3) = unbounded();
    // Duration from incoming ATC notice until cleaner is notified
    let (tx_timer_event4, rx_timer_event4) = unbounded();
    // Duration from departing ATC notice until cleaner is notified
    let (tx_timer_event5, rx_timer_event5) = unbounded();
    // Duration from cleaner has finished cleaning until incoming ATC commence regular task
    let (tx_timer_event6, rx_timer_event6) = unbounded();
    // Duration from cleaner has finished cleaning until departing ATC commence regular task
    let (tx_timer_event7, rx_timer_event7) = unbounded();

    // Generate Event Thread
    let gen_event = pool.execute_with_fixed_delay(
        Duration::from_millis(0),
        Duration::from_millis(2500),
        move || {
            let mut rng = rand::thread_rng();
            let chance_event = rng.gen_range(0..101);
            if chance_event > 50 {
                println!(
                    "Gen_Event: Event is generated true {} {:?}",
                    chance_event,
                    Instant::now()
                );
                let event_start = Instant::now();
                let event_start_clone = event_start.clone();
                tx_event2atc.send((true, event_start)).unwrap();
                tx_event2atc.send((true, event_start_clone)).unwrap();
                // Wait until all the event message has been sent out.
                while !tx_event2atc.is_empty() {
                    thread::sleep(Duration::from_millis(500));
                }
            } else {
                println!("Gen_Event: {} {:?}", chance_event, Instant::now());
            }
        },
    );

    // Cleaner Thread
    let cleaner = pool.execute_at_fixed_rate(
        Duration::from_secs(0),
        Duration::from_millis(500),
        move || {
            let (confirmation1, event_start_timer1, iatc_start_timer) = match rx_atc2cleaner.recv()
            {
                Ok((val, timer1, timer2)) => (val, timer1, timer2),
                Err(_) => (false, Instant::now(), Instant::now()), //Err when channel is disconnected
            };
            let (confirmation2, _, datc_start_timer) = match rx_atc2cleaner.recv() {
                // _ because value is the same as event_start_timer1 so its an unused variable
                Ok((val, timer1, timer2)) => (val, timer1, timer2),
                Err(_) => (false, Instant::now(), Instant::now()), //Err when channel is disconnected
            };
            if confirmation1 && confirmation2 {
                //Further checking
                println!(
                    "Cleaner: Please wait while the cleaner is working! {:?}",
                    Instant::now()
                );
                tx_timer_event3
                    .send(event_start_timer1.elapsed().as_nanos())
                    .unwrap();
                tx_timer_event4
                    .send(iatc_start_timer.elapsed().as_nanos())
                    .unwrap();
                tx_timer_event5
                    .send(datc_start_timer.elapsed().as_nanos())
                    .unwrap();
                thread::sleep(Duration::from_millis(2500)); //Simulate cleaner working and then set event_val to false to simulate runway is now cleaned
                println!(
                    "Cleaner: Has finished cleaning the runway {:?}",
                    Instant::now()
                );
                let cleaner_finish = Instant::now();
                let cleaner_finish_timer = cleaner_finish.clone(); //Clone here so that the two values send is the same
                tx_cleaner2atc.send((true, cleaner_finish)).unwrap();
                tx_cleaner2atc.send((true, cleaner_finish_timer)).unwrap();
            } else {
                println!("Cleaner thread is shutting down");
            }
        },
    );

    /* Airport Traffic Control */
    // Incoming ATC
    let atc_incoming = atc_pool.execute_with_fixed_delay(
        Duration::from_secs(0),
        Duration::from_millis(1000),
        move || {
            let (event_val, event_start_timer) = match rx_event2atc.try_recv() {
                Ok((val,time)) => (val, time),
                Err(_e) => (false, Instant::now()), // temp value if try_recv() returns an error because the channel is empty
            };
            if event_val {
                tx_timer_event1.send(event_start_timer.elapsed().as_nanos()).unwrap(); // Duration from the moment event is detected until incoming atc detect
                println!("IATC: Event Happened {:?}", Instant::now());
                tx_atc2cleaner.send((true, event_start_timer, Instant::now())).unwrap();
                let (finish_cleaning, cleaner_finish_timer) = rx_cleaner2atc.recv().unwrap();
                if finish_cleaning {
                    tx_timer_event6.send(cleaner_finish_timer.elapsed().as_nanos()).unwrap();
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
        Duration::from_millis(750),
        Duration::from_millis(1500),
        move || {
            let (event_val, event_start_timer) = match rx_event2atc_clone.try_recv() {
                Ok((val, time)) => (val, time),
                Err(_e) => (false, Instant::now()), //Need to check if disconnected or empty
            };
            if event_val {
                tx_timer_event2
                    .send(event_start_timer.elapsed().as_nanos())
                    .unwrap(); // Duration from the moment event is detected until departing atc detect
                println!("DATC: Event Happened {:?}", Instant::now());
                tx_atc2cleaner_clone
                    .send((true, event_start_timer, Instant::now()))
                    .unwrap();
                let (finish_cleaning, cleaner_finish_timer) = rx_cleaner2atc_clone.recv().unwrap();
                if finish_cleaning {
                    tx_timer_event7
                        .send(cleaner_finish_timer.elapsed().as_nanos())
                        .unwrap();
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

    // Generate Airplane Thread (Generate 60 Airplanes before gracefully shutting down)
    for i in 1..61 {
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
        thread::sleep(Duration::from_millis(1500)); // 1.5 seconds between every incoming airplane
    }

    while !tx_dockbay_clone.is_empty() || !tx_plane_buffer_clone.is_empty() {} //Wait for docking bay and buffer clone to be empty before terminating all thread
    thread::sleep(Duration::from_secs(5));
    gen_event.cancel();
    atc_departing.cancel();
    atc_incoming.cancel();
    cleaner.cancel();

    // Print only the value so it is easier to be copy pasted to an excel sheet.
    println!("Event 1");
    for val in rx_timer_event1.iter() {
        println!("{}", val);
    }
    println!("Event 2");
    for val in rx_timer_event2.iter() {
        println!("{}", val);
    }
    println!("Event 3");
    for val in rx_timer_event3.iter() {
        println!("{}", val);
    }
    println!("Event 4");
    for val in rx_timer_event4.iter() {
        println!("{}", val);
    }
    println!("Event 5");
    for val in rx_timer_event5.iter() {
        println!("{}", val);
    }
    println!("Event 6");
    for val in rx_timer_event6.iter() {
        println!("{}", val);
    }
    println!("Event 7");
    for val in rx_timer_event7.iter() {
        println!("{}", val);
    }
}
