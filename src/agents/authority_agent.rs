use crate::messages::outs::controller_messages::{ReqRegisterAgent_SenderType, ReqRegisterAgent_Action, ReqRegisterAgent, ControllerRound};
use protobuf::Message;
use zmq::Socket;
use crate::messages::outs::stats_message::{RoundResultRequest, RoundResultReply};
use crate::messages::outs::authority_messages::PubRegulations;
use std::thread::sleep;
use std::time;

pub struct AuthorityAgent {
    pub id: u32,
    mask_mandate: bool,
    mask_mandate_limit: u32,
    social_distancing: bool,
    social_distancing_limit: u32,
    context: zmq::Context,
}

#[allow(dead_code)]
impl AuthorityAgent {
    pub fn new(
        id: u32,
        mask_mandate: bool,
        mask_mandate_limit: u32,
        social_distancing: bool,
        social_distancing_limit: u32,
        context: &zmq::Context,
    ) -> AuthorityAgent {
        AuthorityAgent {
            id,
            mask_mandate,
            mask_mandate_limit,
            social_distancing,
            social_distancing_limit,
            context: context.clone(),
        }
    }

    pub fn run(&self) {
        let mut request_socket = self.context.socket(zmq::REQ).unwrap();
        request_socket = self.register_at_controller(request_socket);
        let mut subscribe_socket = self.context.socket(zmq::SUB).unwrap();
        assert!(subscribe_socket.connect("inproc://control_publisher").is_ok());
        assert!(subscribe_socket.set_subscribe(&[]).is_ok());
        let pub_socket = self.context.socket(zmq::PUB).unwrap();
        assert!(pub_socket.bind("inproc://authority_agent").is_ok());

        let mut iteration;
        let mut incidence_vec = vec![];
        'run: loop {
            match self.receive_round_from_controller(subscribe_socket) {
                None => { break 'run; }
                Some((round, socket)) => {
                    iteration = round;
                    subscribe_socket = socket;
                }
            }
            let infected = self.request_current_infected(&request_socket, iteration);
            let incidence = self.calculate_incidence(&mut incidence_vec, infected);
            // super bad workaround --> but a proper solution would require too much effort at this point
            // this is needed, as the agents are subscribing to the authority agent and controller agent on the same socket
            // if the authority agent is not waiting long enough, its message might arrive before the controller ones --> Error
            // better fix would be to kind of recognize from which sender the incoming messages are comming
            sleep(time::Duration::from_millis(100));
            self.publish_regulations(&pub_socket, incidence);
        }
    }


    pub fn calculate_incidence(&self, incidence_vec: &mut Vec<u32>, infected: u32) -> u32 {
        if incidence_vec.len() >= 7 {
            incidence_vec.remove(0);
        }
        incidence_vec.push(infected);
        incidence_vec.iter().sum()
    }

    pub fn register_at_controller(&self, socket: zmq::Socket) -> zmq::Socket {
        assert!(socket.connect("inproc://control_server").is_ok());
        let mut msg = ReqRegisterAgent::new();
        msg.sender_id = self.id;
        msg.sender_type = ReqRegisterAgent_SenderType::AUTHORITY_AGENT;
        msg.action = ReqRegisterAgent_Action::REGISTER;
        socket.send(msg.write_to_bytes().unwrap(), 0).unwrap();
        socket.recv_bytes(0).unwrap();
        assert!(socket.disconnect("inproc://control_server").is_ok());
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

    fn request_current_infected(&self, socket: &zmq::Socket, round: u32) -> u32 {
        assert!(socket.connect("inproc://statistic_agent").is_ok());
        let mut request = RoundResultRequest::new();
        request.sender_id = self.id;
        request.requested_round = round;
        socket.send(request.write_to_bytes().unwrap(), 0).unwrap();
        let in_msg = socket.recv_bytes(0).unwrap();
        let reply = RoundResultReply::parse_from_bytes(&in_msg).unwrap();
        assert!(socket.disconnect("inproc://statistic_agent").is_ok());
        reply.agents_infected
    }

    fn publish_regulations(&self, socket: &Socket, incidence: u32) {
        let mut regulations_msg = PubRegulations::new();
        regulations_msg.sender_id = self.id;
        if self.mask_mandate && incidence >= self.mask_mandate_limit {
            regulations_msg.mask_mandate = true;
        } else { regulations_msg.mask_mandate = false; }
        if self.social_distancing && incidence >= self.social_distancing_limit {
            regulations_msg.social_distancing = true;
        } else { regulations_msg.social_distancing = false; }

        socket.send(regulations_msg.write_to_bytes().unwrap(), 0).unwrap();
    }
}

