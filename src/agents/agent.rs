use crate::messages::outs::controller_messages::{ControllerRound, ReqRegisterAgent, ReqRegisterAgent_SenderType, ReqRegisterAgent_Action, ReqRoundStatus, ReqRoundStatus_Status};
use protobuf::{Message};
use rand::{Rng};
use crate::messages::outs::inter_agent_message::{EncounterRequest, EncounterReply, Compartment as Proto_Compartment};
use std::thread;
use std::thread::JoinHandle;
use crate::messages::outs::stats_message::StatsRequest;
use crate::disease::disease::Disease;
use crate::messages::outs::inter_agent_message::Compartment::{COMPARTMENT_SUSCEPTIBLE, COMPARTMENT_INFECTED, COMPARTMENT_REMOVED};
use crate::messages::outs::authority_messages::PubRegulations;
use crate::environment::regulations::Regulations;

/// A human being is represented here
#[allow(dead_code)]
pub struct Agent {
    /// A person must have a name, no matter how much Juliet may hate it
    pub id: u32,
    pub compartment: Compartment,
    pub days_infected: u8,
    pub contacts: Vec<u32>,
    pub disease: Option<Disease>,
    encountering_likelihood: f64,
    regulations: Regulations,
    state: State,
    context: zmq::Context,
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
    SUSCEPTIBLE,
    INFECTED,
    REMOVED,
}

#[allow(dead_code)]
impl Agent {
    /// Returns a person with the name given them
    ///
    /// # Arguments
    ///
    /// * `name` - A string slice that holds the name of the person
    pub fn new(
        id: u32,
        contacts: Vec<u32>,
        context: &zmq::Context,
    ) -> Agent {
        Agent {
            id,
            compartment: Compartment::SUSCEPTIBLE,
            days_infected: 0,
            contacts,
            disease: None,
            encountering_likelihood: 0.8,
            regulations: Regulations::new(),
            state: State::Initiated,
            context: context.clone(),
        }
    }

    /// Returns a person with the name given them
    pub fn run(&mut self) {
        let mut reply_socket = self.context.socket(zmq::REP).unwrap();
        let mut request_socket = self.context.socket(zmq::REQ).unwrap();
        let mut subscribe_socket = self.context.socket(zmq::SUB).unwrap();
        assert!(subscribe_socket.connect("inproc://control_publisher").is_ok());
        assert!(subscribe_socket.connect("inproc://authority_agent").is_ok());
        assert!(subscribe_socket.set_subscribe(&[]).is_ok());
        request_socket = self.register_at_controller(request_socket);
        let mut iteration;
        'sim: loop {
            match self.receive_round_from_controller(subscribe_socket) {
                None => {
                    break 'sim;
                }
                Some((round, socket)) => {
                    iteration = round;
                    subscribe_socket = socket
                }
            }
            self.receive_regulations(&subscribe_socket);
            let agents_to_meet = determine_contacts_to_meet(&self.contacts, self.encountering_likelihood, self.regulations.social_distancing);
            let handle = agent_replier(self.id.clone(), self.compartment.clone(), self.disease.clone(),agents_to_meet.clone(), reply_socket);
            request_socket = self.agent_routine(agents_to_meet, request_socket);
            reply_socket = handle.join().unwrap();
            request_socket = report_round_to_statistician(self.id, iteration, self.compartment, request_socket);
            request_socket = self.report_round_to_controller(iteration, request_socket);
        }
    }

    /// Returns a person with the name given them
    pub fn register_at_controller(&self, socket: zmq::Socket) -> zmq::Socket {
        assert!(socket.connect("inproc://control_server").is_ok());
        let mut msg = ReqRegisterAgent::new();
        msg.sender_id = self.id;
        msg.sender_type = ReqRegisterAgent_SenderType::SIMULATION_AGENT;
        msg.action = ReqRegisterAgent_Action::REGISTER;
        socket.send(msg.write_to_bytes().unwrap(), 0).unwrap();
        socket.recv_bytes(0).unwrap();
        assert!(socket.disconnect("inproc://control_server").is_ok());
        socket
    }

    /// The procedure that each agent goes through per round
    fn agent_routine(&mut self, contacts: Vec<(u32, bool)>, socket: zmq::Socket) -> zmq::Socket {
        self.update_infectiousness();
        let mut out_msg = EncounterRequest::new();
        out_msg.sender_id = self.id;
        out_msg.requester_compartment = parse_compartment(self.compartment);
        out_msg.disease = serde_json::to_string(&self.disease.clone()).unwrap();
        for (contact, _) in contacts {
            assert!(socket.connect(&format!("{}{}", "inproc://agent", contact)).is_ok());
            socket.send(out_msg.write_to_bytes().unwrap(), 0).unwrap();
            let in_msg = socket.recv_bytes(0).unwrap();
            let reply = EncounterReply::parse_from_bytes(&in_msg).unwrap();
            if reply.encounter_decision == true {
                match reply.replier_compartment {
                    Proto_Compartment::COMPARTMENT_SUSCEPTIBLE => {}
                    Proto_Compartment::COMPARTMENT_INFECTED => {
                        // let s:Result<Option<Disease>, serde_json::Error> = serde_json::from_str(&reply.disease);
                        self.virus_contact(serde_json::from_str(&reply.disease).unwrap())
                    }
                    Proto_Compartment::COMPARTMENT_REMOVED => {}
                }
            }
            assert!(socket.disconnect(&format!("{}{}", "inproc://agent", contact)).is_ok())
        }
        socket
    }

    fn receive_round_from_controller(&self, socket: zmq::Socket) -> Option<(u32, zmq::Socket)> {
        let in_bytes = socket.recv_bytes(0).unwrap();
        match ControllerRound::parse_from_bytes(&in_bytes) {
            Ok(in_msg) => {
                Some((in_msg.round, socket))
            }
            Err(_) => {
                None
            }
        }
    }

    fn report_round_to_controller(&self, iteration: u32, socket: zmq::Socket) -> zmq::Socket {
        assert!(socket.connect("inproc://control_server").is_ok());
        let mut msg = ReqRoundStatus::new();
        msg.sender_id = self.id;
        msg.finished_round = iteration;
        msg.status = ReqRoundStatus_Status::ROUND_FINISHED;
        socket.send(msg.write_to_bytes().unwrap(), 0).unwrap();
        socket.recv_bytes(0).unwrap();
        assert!(socket.disconnect("inproc://control_server").is_ok());
        socket
    }

    pub fn virus_contact(&mut self, disease: Option<Disease>) {
        let mut rng = rand::thread_rng();
        let mut infection_risk = disease.unwrap().infection_risk;
        if self.regulations.mask_mandate { infection_risk = infection_risk / 5.0 }
        if self.compartment == Compartment::SUSCEPTIBLE {
            if rng.gen_bool(infection_risk) {
                self.compartment = Compartment::INFECTED;
                self.disease = disease;
                self.days_infected = 1;
            }
        }
    }

    pub fn update_infectiousness(&mut self) {
        if self.compartment == Compartment::INFECTED && self.days_infected >= self.disease.unwrap().time_symptomatic {
            self.compartment = Compartment::REMOVED;
            self.days_infected = 0;
        } else if self.compartment == Compartment::INFECTED && self.days_infected < self.disease.unwrap().time_symptomatic {
            self.days_infected += 1;
        }
    }
    fn receive_regulations(&mut self, socket: &zmq::Socket) {
        let in_msg = socket.recv_bytes(0).unwrap();
        let regulation_msg = PubRegulations::parse_from_bytes(&in_msg).unwrap();
        self.regulations.mask_mandate = regulation_msg.mask_mandate;
        self.regulations.social_distancing = regulation_msg.social_distancing;
    }
}

fn report_round_to_statistician(id: u32, round: u32, compartment: Compartment, socket: zmq::Socket) -> zmq::Socket {
    assert!(socket.connect("inproc://statistic_agent").is_ok());
    let mut request = StatsRequest::new();
    request.sender_id = id;
    request.round = round;
    request.request_compartment = parse_compartment(compartment);
    socket.send(request.write_to_bytes().unwrap(), 0).unwrap();
    socket.recv_bytes(0).unwrap();
    assert!(socket.disconnect("inproc://statistic_agent").is_ok());
    socket
}

fn determine_contacts_to_meet(contacts: &Vec<u32>, mut encounter_likelihood: f64, social_distancing: bool) -> Vec<(u32, bool)> {
    let mut contacts_to_meet: Vec<(u32, bool)> = vec![];
    let mut rng = rand::thread_rng();
    if social_distancing { encounter_likelihood = encounter_likelihood / 3.0 }
    for contact in contacts {
        if rng.gen_bool(encounter_likelihood) {
            contacts_to_meet.push((contact.clone(), true));
        } else {
            contacts_to_meet.push((contact.clone(), false));
        }
    }
    contacts_to_meet
}

fn agent_replier(id: u32, compartment: Compartment, disease: Option<Disease>, agents_to_meet: Vec<(u32, bool)>, socket: zmq::Socket) -> JoinHandle<zmq::Socket> {
    assert!(socket.bind(&format!("{}{}", "inproc://agent", id)).is_ok());
    let handle = thread::spawn(move || {
        for _ in agents_to_meet.clone() {
            let req_msg = EncounterRequest::parse_from_bytes(&socket.recv_bytes(0).unwrap()).unwrap();
            let mut rep_msg = EncounterReply::new();
            rep_msg.sender_id = id;
            rep_msg.replier_compartment = parse_compartment(compartment);
            rep_msg.disease = serde_json::to_string(&disease).unwrap();
            if agents_to_meet.iter().any(|(x, _)| x == &req_msg.sender_id) {
                rep_msg.encounter_decision = agents_to_meet.clone().into_iter().find(|&(id, _)| id == req_msg.sender_id).unwrap().1;
            } else {
                panic!("This shouldn't happen.");
            }
            socket.send(rep_msg.write_to_bytes().unwrap(), 0).unwrap();
        }
        assert!(socket.disconnect(&format!("{}{}", "inproc://agent", id)).is_ok());
        socket
    });
    handle
}

fn parse_compartment(compartment: Compartment) -> Proto_Compartment {
    match compartment {
        Compartment::SUSCEPTIBLE => COMPARTMENT_SUSCEPTIBLE,
        Compartment::INFECTED => COMPARTMENT_INFECTED,
        Compartment::REMOVED => COMPARTMENT_REMOVED
    }
}