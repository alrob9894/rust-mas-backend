use protobuf::Message;
use rocket::serde::{json::Json, Serialize};

use crate::messages::outs::controller_messages::{ReqRegisterAgent, ReqRegisterAgent_SenderType, ReqRegisterAgent_Action};
use crate::messages::outs::stats_message::{StatsRequest, StatsReply, StatsReply_Status, RoundResultRequest, RoundResultReply};
use crate::messages::outs::inter_agent_message::Compartment;
use std::collections::HashMap;

#[derive(Clone)]
pub struct StatisticAgent {
    pub id: u32,
    iterations: u16,
    agent_num: u32,
    results: HashMap<u16, Stats>,
    context: zmq::Context,
}

#[derive(Serialize, Copy, Clone, Default)]
pub struct Stats {
    agents_susceptible: u32,
    agents_infected: u32,
    agents_removed: u32,
}

#[derive(Serialize, Copy, Clone, Default)]
pub struct RoundResult {
    round: u16,
    round_stats: Stats,
}

impl StatisticAgent {
    pub fn new(
        id: u32,
        iterations: u16,
        agent_num: u32,
        context: &zmq::Context,
    ) -> StatisticAgent {
        StatisticAgent {
            id,
            iterations,
            agent_num,
            results: init_results(iterations),
            context: context.clone(),
        }
    }

    pub fn run(&mut self) {
        let mut request_socket = self.context.socket(zmq::REQ).unwrap();
        request_socket = self.register_at_controller(request_socket);
        drop(request_socket);

        let reply_socket = self.context.socket(zmq::REP).unwrap();
        assert!(reply_socket.bind("inproc://statistic_agent").is_ok());
        // + 1 because of authority agent
        for _ in 1..=(self.iterations as u32 * (self.agent_num + 1)) {
            let in_msg = reply_socket.recv_bytes(0).unwrap();
            let request = StatsRequest::parse_from_bytes(&in_msg).unwrap();
            if request.sender_id != 0 {
                match request.request_compartment {
                    Compartment::COMPARTMENT_SUSCEPTIBLE => {
                        if request.round < 1 || request.round > 30 {
                            println!("{}", request.round)}
                        self.results.get_mut(&(request.round as u16)).unwrap().agents_susceptible += 1;
                    }
                    Compartment::COMPARTMENT_INFECTED => {
                        self.results.get_mut(&(request.round as u16)).unwrap().agents_infected += 1;
                    }
                    Compartment::COMPARTMENT_REMOVED => {
                        self.results.get_mut(&(request.round as u16)).unwrap().agents_removed += 1;
                    }
                }
                let mut reply = StatsReply::new();
                reply.sender_id = self.id;
                reply.status = StatsReply_Status::SUCCESS;
                reply_socket.send(reply.write_to_bytes().unwrap(), 0).unwrap();
            }
            else {
                let request = RoundResultRequest::parse_from_bytes(&in_msg).unwrap();
                let mut reply = RoundResultReply::new();
                reply.sender_id = self.id;
                reply.agents_infected = self.results.get(&(request.requested_round as u16)).clone().unwrap().agents_infected;
                reply_socket.send(reply.write_to_bytes().unwrap(), 0).unwrap();
            }

        }
    }

    /// Returns a person with the name given them
    fn register_at_controller(&self, socket: zmq::Socket) -> zmq::Socket {
        assert!(socket.connect("inproc://control_server").is_ok());
        let mut msg = ReqRegisterAgent::new();
        msg.sender_id = self.id;
        msg.sender_type = ReqRegisterAgent_SenderType::STATISTIC_AGENT;
        msg.action = ReqRegisterAgent_Action::REGISTER;
        socket.send(msg.write_to_bytes().unwrap(), 0).unwrap();
        socket.recv_bytes(0).unwrap();
        assert!(socket.disconnect("inproc://control_server").is_ok());
        socket
    }

    pub fn results_to_json(&self) -> Json<Vec<RoundResult>> {
        let mut vec:Vec<RoundResult> = vec![];
        for round in 1..=self.iterations {
            let rr = RoundResult{ round, round_stats: *self.results.get(&round).unwrap() };
            vec.push(rr);
        }
        Json(vec)
    }
}

fn init_results(rounds: u16) -> HashMap<u16, Stats> {
    let mut results = HashMap::new();
    for round in 1..=rounds {
        results.insert(
            round,
            Stats {
                agents_susceptible: 0,
                agents_infected: 0,
                agents_removed: 0
            });
    }
    results
}