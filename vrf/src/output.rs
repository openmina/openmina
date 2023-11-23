use mina_hasher::{Hashable, ROInput};

use super::message::VrfMessage;
use super::CurvePoint;

#[derive(Clone, Debug)]
pub struct VrfOutputHashInput {
    message: VrfMessage,
    g: CurvePoint,
}

impl VrfOutputHashInput {
    pub fn new(message: VrfMessage, g: CurvePoint) -> Self {
        Self { message, g }
    }
}

impl Hashable for VrfOutputHashInput {
    type D = ();

    fn to_roinput(&self) -> ROInput {
        ROInput::new()
            .append_roinput(self.message.to_roinput())
            .append_field(self.g.x)
            .append_field(self.g.y)
    }

    fn domain_string(_: Self::D) -> Option<String> {
        "MinaVrfOutput".to_string().into()
    }
}