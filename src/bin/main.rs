/* External Crates */
extern crate crossbeam_channel;
extern crate rand;
extern crate threadpool;

/* Use Statements */
use crossbeam_channel::{bounded, unbounded, Receiver, Sender};
use rand::Rng;
use rts_assignment_v2::*;
use std::format;
use std::thread;
use std::time::{Duration, Instant};
use threadpool::ThreadPool;

const PARKING_SPACE: usize = 6;

fn main() {
    let pool = ThreadPool::new(10);
    let dockbay_threadpool = ThreadPool::new(20);
    let (tx_main, rx_main): (Sender<Airplane>, Receiver<Airplane>) = unbounded();
    let (tx_dockbay, rx_dockbay): (Sender<Airplane>, Receiver<Airplane>) = bounded(PARKING_SPACE);

    // Duration from event is generated true until ATC notices
    let (tx_timer_event1, rx_timer_event1): (Sender<u128>, Receiver<u128>) = unbounded();

    // Duration from event is generated true until cleaner is dispatched
    let (tx_timer_event2, rx_timer_event2): (Sender<u128>, Receiver<u128>) = unbounded();

    // Duration from ATC notice until cleaner is notified
    let (tx_timer_event3, rx_timer_event3): (Sender<u128>, Receiver<u128>) = unbounded();

    // Duration from cleaner has finished until ATC resume regular task
    let (tx_timer_event4, rx_timer_event4): (Sender<u128>, Receiver<u128>) = unbounded();

    let tx_income_buffer = airport_control_center(
        pool.clone(),
        tx_main,
        tx_dockbay,
        tx_timer_event1,
        tx_timer_event2,
        tx_timer_event3,
        tx_timer_event4,
    );
    {
        for num_airplane in 1..60 {
            println!("{} Airplane has been generated", num_airplane);
            incoming_airplane(pool.clone(), tx_income_buffer.clone());
            docking_bay(
                dockbay_threadpool.clone(),
                rx_dockbay.clone(),
                tx_income_buffer.clone(),
            );
            thread::sleep(Duration::from_millis(1500));
        }
        incoming_airplane(pool, tx_income_buffer); // Need to be called so that the original tx_income_buffer is used and disconnected
    }
    println!("DONEEE!!!");
    for val in rx_main.iter() {
        println!("{}", val.get_code());
    } //To ensure graceful shutdown
    println!("event1");
    for val in rx_timer_event1.iter() {
        println!("{}", val)
    }
    println!("event2");
    for val in rx_timer_event2.iter() {
        println!("{}", val)
    }
    println!("event3");
    for val in rx_timer_event3.iter() {
        println!("{}", val)
    }
    println!("event4");
    for val in rx_timer_event4.iter() {
        println!("{}", val)
    }
    println!("End of main thread");
}

fn incoming_airplane(pool: ThreadPool, tx_income_buffer: Sender<Task>) {
    pool.execute(move || {
        let airline_code = vec!["AA", "EL", "AK", "CA", "GA"];
        let mut rng = rand::thread_rng();
        let random_airline_code = rng.gen_range(0..5);
        let random_airline_number = rng.gen_range(100..999);
        let incoming_plane = format!(
            "{}{}",
            airline_code[random_airline_code], random_airline_number
        );
        let airplane = Airplane::new(incoming_plane);
        println!(
            "{}: This is airplane {} requesting ATC to land",
            airplane.get_code(),
            airplane.get_code()
        );
        if let Err(_) = tx_income_buffer.send(Task::HandleIncoming(airplane)) {
            panic!("Error while trying to send airplane to ATC");
        };
    });
}

fn airport_control_center(
    pool: ThreadPool,
    tx_main: Sender<Airplane>,
    tx_dockbay: Sender<Airplane>,
    tx_timer_event1: Sender<u128>,
    tx_timer_event2: Sender<u128>,
    tx_timer_event3: Sender<u128>,
    tx_timer_event4: Sender<u128>,
) -> Sender<Task> {
    let (tx_task_buffer, rx_task_buffer): (Sender<Task>, Receiver<Task>) = unbounded();
    let (tx_event, rx_event): (Sender<(Event, Instant)>, Receiver<(Event, Instant)>) = bounded(1);

    let pool_clone1 = pool.clone();

    pool.execute(move || {
        for task in rx_task_buffer.iter() {
            // Every loop the ATC checks with event thread to see whether event is generated or not
            // Rather than having event notifying the ATC thread so that response is much quicker.
            // Although there might be an overhead in performance due to checking every iteration
            // Response is way faster than using the observer pattern
            generate_event(pool_clone1.clone(), tx_event.clone());
            let (event_occur, event_start_timer) = match rx_event.recv() {
                Ok((event, event_start_timer)) => (event.get_value(), event_start_timer),
                Err(_) => panic!("Error while receiving event value"),
            };
            if event_occur {
                if let Err(_) = tx_timer_event1.send(event_start_timer.clone().elapsed().as_nanos()) {
                    panic!("Timer Event1 Channel Disconnected");
                }
                println!("An event has occured");
                let (tx, rx): (Sender<(String, Instant)>, Receiver<(String, Instant)>) = bounded(1);
                let tx_cleaner = cleaner(pool_clone1.clone(), tx, tx_timer_event2.clone(), tx_timer_event3.clone());
                if let Err(_) = tx_cleaner.send((String::from("Clean"), event_start_timer, Instant::now())){
                    panic!("Something went wrong in atc thread while trying to notify cleaner thread");
                }
                let (_, cleaner_start_timer) = match rx.recv() {
                    Ok((val, cleaner_start_timer)) => (val , cleaner_start_timer),
                    Err(_) => panic!("Something went wrong in atc while receiving value from cleaner thread") 
                }; //Blocks current thread until the cleaner thread has finished.
                if let Err(_) = tx_timer_event4.clone().send(cleaner_start_timer.elapsed().as_nanos()) {
                    panic!("Timer Event4 Channel Disconnected");
                }
            }
            match task {
                Task::HandleIncoming(airplane) => {
                    println!(
                        "ATC: Airplane {} you are allowed to use the runway, please land now",
                        airplane.get_code()
                    );
                    if let Err(_) = tx_dockbay.send(airplane.clone()) {
                        panic!("Something went wrong while sending airplane to docking bay");
                    }
                    println!(
                        "ATC: Airplane {} you can park at the parking bay as there is an empty spot now",
                        airplane.get_code()
                    );
                }
                Task::HandleOutgoing(airplane) => {
                    println!(
                        "ATC: Airplane {} you are allowed to use the runway, have a safe flight",
                        airplane.get_code()
                    );
                    if let Err(_) = tx_main.send(airplane) {
                        panic!("Something went wrong while sending to main");
                    }
                }
            }
        }
        println!("Airport Control Center Terminated {:?}", Instant::now());
    });
    tx_task_buffer
}

fn docking_bay(pool: ThreadPool, rx_dockbay: Receiver<Airplane>, tx_atc: Sender<Task>) {
    pool.execute(move || {
        let mut rng = rand::thread_rng();
        let wait_time = rng.gen_range(1..3);
        thread::sleep(Duration::from_secs(wait_time));
        let airplane = match rx_dockbay.recv() {
            Ok(airplane) => airplane,
            Err(_) => panic!(
                "Something went wrong in docking bay thread while trying to receive airplane"
            ),
        };
        println!(
            "{}: Airplane {} is preparing for departure",
            airplane.get_code(),
            airplane.get_code()
        );

        /*
        Assumptions taken here is that the airplane has left the docking bay. the airplane can instantly take off or wait for other airplane in the
        runway but because it is not in the docking bay anymore, the docking bay thread does not take care of the airplane anymore
        */
        if let Err(_) = tx_atc.send(Task::HandleOutgoing(airplane)) {
            panic!("Something went wrong while plane is preparing to depart");
        }
    });
}

fn cleaner(
    pool: ThreadPool,
    sender: Sender<(String, Instant)>,
    tx_timer_event2: Sender<u128>,
    tx_timer_event3: Sender<u128>,
) -> Sender<(String, Instant, Instant)> {
    let (tx, rx): (
        Sender<(String, Instant, Instant)>,
        Receiver<(String, Instant, Instant)>,
    ) = bounded(1);
    pool.execute(move || {
        for (_, event_start_timer, atc_start_timer) in rx.iter() {
            if let Err(_) = tx_timer_event2.clone().send(event_start_timer.elapsed().as_nanos()) {
                panic!("Timer Event2 Channel Disconnected");
            }
            if let Err(_) = tx_timer_event3.clone().send(atc_start_timer.elapsed().as_nanos()) {
                panic!("Timer Event3 Channel Disconnected");
            }
            println!("Cleaner is cleaning");
            thread::sleep(Duration::from_millis(5000));
            println!("Cleaner finish cleaning");
            if let Err(_) = sender.send((String::from("Okay"), Instant::now())){
                panic!("Something went wrong in cleaner thread while trying to notify atc to resume normal operation");
            }
        }
    });
    tx
}

fn generate_event(pool: ThreadPool, tx: Sender<(Event, Instant)>) {
    pool.execute(move || {
        let mut rng = rand::thread_rng();
        let chance_event = rng.gen_range(0..101);
        if chance_event > 75 {
            let event = Event::new(true);
            if let Err(_) = tx.send((event, Instant::now())) {
                println!("Error while sending event value = true");
            }
        } else {
            let event = Event::new(false);
            if let Err(_) = tx.send((event, Instant::now())) {
                println!("Error while sending event value = false");
            }
        }
    });
}
