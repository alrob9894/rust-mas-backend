// #[derive(Clone)]
pub struct AuthorityAgent {
    pub id: u32,
    pub restrictions: bool,
    context: zmq::Context,
    publisher: Option<zmq::Socket>,
}

impl AuthorityAgent {
    pub fn new(
        id: u32,
    ) -> AuthorityAgent {
        AuthorityAgent {
            id,
            restrictions: false,
            context: zmq::Context::new(),
            publisher: None,
        }
    }

    pub fn calculate_incidence(&self) -> u32 {
        5
    }
}