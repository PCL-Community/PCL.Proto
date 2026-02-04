use std::collections::HashMap;

use crate::scaffolding::{
    self, room::member::Member, server::ScaffoldingServer, util::byte_buffer::ByteBuffer,
};

struct Sender {
    member: Member,
    // connection
}

type Handler = Box<
    dyn Fn(
        &mut Sender,
        &mut ByteBuffer,
    ) -> Result<scaffolding::Response, scaffolding::ScaffoldingError>,
>;

pub struct RequestHandler {
    server: Option<ScaffoldingServer>,
    // 翻译Swift实现    private var handlers: [String: (Sender, ByteBuffer) throws -> Scaffolding.Response] = [:]
    handlers: HashMap<&'static str, Handler>,
}

impl RequestHandler {
    pub fn new() -> Self {
        let mut request_handler = Self {
            server: None,
            handlers: HashMap::new(),
        };
        request_handler.register_handler(
            "c:ping",
            Box::new(|_sender, request_body| {
                Ok(scaffolding::Response {
                    status: 0,
                    data: request_body.data.clone(),
                })
            }),
        );
        request_handler.register_handler(
            "c:protocols",
            Box::new(|_sender, _request_body| {
                Ok(scaffolding::Response {
                    status: 0,
                    data: Vec::new(),
                })
            }),
        );
        request_handler
    }

    fn register_handler(&mut self, protocol: &'static str, handler: Handler) {
        self.handlers.insert(protocol, handler);
    }

    pub fn protocols(&self) -> Vec<&'static str> {
        let keys: Vec<&'static str> = self.handlers.keys().cloned().collect();
        keys
    }
}
