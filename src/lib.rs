//use std::fmt;

pub enum Task {
    HandleIncoming(Airplane),
    HandleOutgoing(Airplane),
}

// Airplane
#[derive(Clone, PartialEq)]
pub struct Airplane {
    code: String,
}

impl Airplane {
    pub fn new(code: String) -> Airplane {
        Airplane { code }
    }

    pub fn get_code(&self) -> &str {
        &self.code
    }
}

#[derive(Clone)]
pub struct Event(bool);

impl Event {
    pub fn new(b: bool) -> Event {
        Event(b)
    }

    pub fn get_value(&self) -> bool {
        self.0
    }
}

// Docking Bay
// #[derive(Clone)]
// pub struct DockingBay {
//     pub capacity: usize,
//     pub docking_bay: Vec<Airplane>,
// }

// impl DockingBay {
//     pub fn new(capacity: usize) -> DockingBay {
//         DockingBay {
//             capacity,
//             docking_bay: Vec::with_capacity(capacity),
//         }
//     }

//     pub fn park(&mut self, airplane: Airplane) -> Result<(), FullCapacityError> {
//         println!("{}", self.docking_bay.capacity());
//         if self.docking_bay.len() < self.docking_bay.capacity() {
//             self.docking_bay.push(airplane);
//             println!("Parked!");
//             return Ok(());
//         } else {
//             Err(FullCapacityError)
//         }
//     }
//     pub fn depart(&mut self, airplane: Airplane) -> Result<(), DepartError> {
//         if self.docking_bay.len() <= 0 {
//             return Err(DepartError::Empty);
//         }
//         let self_clone = self.clone();
//         for (i, val) in self_clone.docking_bay.iter().enumerate() {
//             if val == &airplane {
//                 self.docking_bay.remove(i);
//                 println!("Airplane {} has departed", airplane.get_code());
//                 return Ok(());
//             }
//         }
//         Err(DepartError::NotFound)
//     }

//     // TODO: Temp Only
//     pub fn get_content(&mut self) {
//         for i in self.docking_bay.iter() {
//             println!("{}", i.get_code());
//         }
//         println!("length of parking lot now {} ", self.docking_bay.len());
//     }
// }

// Error When Parking Lot is Full
// #[derive(Debug, Clone)]
// pub struct FullCapacityError;

// impl fmt::Display for FullCapacityError {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         write!(f, "Docking bay is full")
//     }
// }

// pub enum DepartError {
//     NotFound,
//     Empty,
// }

// impl fmt::Display for DepartError {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         match *self {
//             DepartError::NotFound => write!(f, "Airplane is not found in the docking bay"),
//             DepartError::Empty => write!(f, "Docking bay is currently empty"),
//         }
//     }
// }
