use std::collections::HashMap;
use rand::Rng;
use crate::environment::social_network::Side::{Left, Right};

#[allow(dead_code)]
pub struct SocialNetwork {
    members: u32,
    avg_friends: u8,
    pub network: HashMap<u32, Vec<u32>>
}

enum Side {
    Right,
    Left
}

impl SocialNetwork {
    pub fn new(
        members: u32,
        avg_friends: u8,
    ) -> SocialNetwork {
        SocialNetwork {
            members,
            avg_friends: if avg_friends % 2 == 0 { avg_friends } else { avg_friends + 1 },
            network: init_sn(members, if avg_friends % 2 == 0 { avg_friends } else { avg_friends + 1 })
        }
    }

    // pub fn init(&mut self) {
    //     self.network = init_sn(self.members, self.avg_friends);
    // }
}

fn init_sn(n: u32, k: u8) -> HashMap<u32, Vec<u32>> {
    let ring_mesh = create_ring(n, k);
    let sn = ring2sn(n, k, ring_mesh);
    sn
}

// creating a ring-mesh as foundation for the social network
fn create_ring(n: u32, k: u8) -> HashMap<u32, Vec<u32>> {
    let mut neighbor_map = HashMap::new();
    for agent_id in 1..=n {
        let mut neighbor_vec = Vec::new();
        for neighbor in 1..=k/2 {
            neighbor_vec.push(determine_neighbor_id(n as i32, agent_id as i32, neighbor as i32, Left));
            neighbor_vec.push(determine_neighbor_id(n as i32, agent_id as i32, neighbor as i32, Right));
        }
        neighbor_map.insert(agent_id, neighbor_vec);
    }
    neighbor_map
}

// transform the ring-mesh into a social network using the Watts-Strogatz algorithm
fn ring2sn(n: u32, k: u8, mut ring: HashMap<u32, Vec<u32>>) -> HashMap<u32, Vec<u32>> {
    let mut rng = rand::thread_rng();

    for (agent_id, _) in ring.clone() {
        for contact_distance in 1..=k/2 {
            // only delete edge with 50% chance and when its a right-side neighbor
            if rng.gen_bool(0.5){
                // delete the old contact bridge
                let contact_del = determine_neighbor_id(n as i32, agent_id as i32, contact_distance as i32, Right);
                ring.get_mut(&agent_id).unwrap().retain(|&contact| contact != contact_del);
                ring.get_mut(&contact_del).unwrap().retain(|&contact| contact != agent_id);
                // create a new contact bridge to a randomly selected node
                let mut contact_new = rng.gen_range(1..=n);
                while contact_new == contact_del || contact_new == agent_id || ring.get_mut(&agent_id).unwrap().contains(&contact_new) {
                    contact_new = rng.gen_range(1..=n);
                }
                ring.get_mut(&agent_id).unwrap().append(&mut vec![contact_new]);
                ring.get_mut(&contact_new).unwrap().append(&mut vec![agent_id]);
            }
        }
    }
    ring
}

// function to determine the ID of an neighbor based on the relative distance of one specific node
fn determine_neighbor_id(n: i32, id: i32, neighbor_distance: i32, side: Side) -> u32 {
    match side {
        // counterclockwise neighbors
        Left =>
            if (id - (neighbor_distance)) >= 1 && (id - (neighbor_distance)) <= n {
                (id - (neighbor_distance)) as u32
            }
            else {
                (n - (id - (neighbor_distance)) * (-1)) as u32
            }
        // clockwise neighbors
        Right =>
            if (id + neighbor_distance) >= 1 && (id + neighbor_distance) <= n {
                (id + neighbor_distance) as u32
            }
            else {
                ((id + neighbor_distance) % n) as u32
            },
    }
}
