use crate::{TIME_SYMPTOMATIC};

#[derive(Clone)]
pub struct Agent {
    pub id : u32,
    pub compartment : Compartment,
    pub days_infected : u8,
    pub contacts : Vec<u32>,
    state : State,
    // actions : Vec<Action>,
    // sensors : Vec<Sensor>,
}

#[derive(Clone)]
#[allow(dead_code)]
enum State {
    Active,
    Suspended,
    Waiting,
    Transit,
    Initiated,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Compartment {
    Susceptible,
    Infected,
    Removed,
}

impl Agent {
    pub fn new(
        id: u32,
        contacts: Vec<u32>,
    ) -> Agent {
        Agent {
            id,
            compartment: Compartment::Susceptible,
            days_infected: 0,
            contacts,
            state: State::Initiated,
            // actions: vec![],
            // sensors: vec![],
        }
    }


    pub fn run_agent(&mut self) {
        let context = zmq::Context::new();
        let subscriber = context.socket(zmq::SUB).unwrap();
        let s = context.socket(zmq::SUB).unwrap();
        s.bind(&*format!("{}\n{}", "ipc://agent", &self.id.to_string()));
        assert!(subscriber.connect("tcp://localhost:5555").is_ok());
        assert!(subscriber.set_subscribe(&[]).is_ok());
        let requester = context.socket(zmq::REQ).unwrap();
        assert!(requester.connect("tcp://localhost:5556").is_ok());

        // inform controller, that agent is ready
        requester.send("agent setup up", 0).unwrap();
        let mut msg = zmq::Message::new();
        requester.recv(&mut msg, 0).unwrap();
        loop {
            subscriber.recv_string(0);
            self.check_infectiousness();
            // self.agent_routine();
            requester.send("work done", 0).unwrap();
            let mut msg = zmq::Message::new();
            requester.recv(&mut msg, 0).unwrap();
        }
    }


    pub fn set_compartment(&mut self, compartment: Compartment) {
        if compartment == Compartment::Infected {
            self.days_infected = 1
        }
        self.compartment = compartment;
    }


    pub fn greet(&self) {
        println!("I bims der Agent {}.", self.id)
    }


    pub fn infects(&self, agent: &Agent) -> bool {
        match self.compartment {
            Compartment::Susceptible => match agent.compartment {
                Compartment::Infected => true,
                _ => false
            },
            Compartment::Infected => match agent.compartment {
                Compartment::Susceptible => true,
                _ => false
            },
            Compartment::Removed => false
        }
    }


    pub fn check_infectiousness(&mut self) {
        if self.compartment == Compartment::Infected && self.days_infected >= TIME_SYMPTOMATIC {
            self.compartment = Compartment::Removed;
            self.days_infected = 0;
        } else if self.compartment == Compartment::Infected && self.days_infected < TIME_SYMPTOMATIC {
            self.days_infected += 1;
        }
    }


    pub fn request_restrictions(&self) {
        let context = zmq::Context::new();
        let receiver = context.socket(zmq::PULL).unwrap();
        assert!(receiver.connect("tcp://localhost:5555").is_ok());

        let string = receiver.recv_string(0).unwrap().unwrap();
    }




    // fn agent_routine(&self) {
    //     let mut rng = rand::thread_rng();
    //     for contact in &self.contacts {
    //         if rng.gen_bool(CONTACT_PROB) {
    //             if self.infects() { }
    //         }
    //     }
    // }
}


//         for (contact, _) in agent.contacts.clone().iter() {
//             if rng.gen_bool(CONTACT_PROB) {
//                 let contact_agent = agents.get_mut(&contact).unwrap();
//                 if agent.meet(contact_agent) {
//                     contact_list.push(contact_agent.id);
//                 }
//             }
//         }
//         if !(contact_list.is_empty()) && rng.gen_bool(INFECTION_RISK) {
//             agents.get_mut(&id).unwrap().set_compartment(Compartment::Infected);
//             for contact in contact_list {
//                 agents.get_mut(&contact).unwrap().set_compartment(Compartment::Infected);
//             }
//         }
//     }
