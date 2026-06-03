use chumsky::{extra::SimpleState, input, prelude::*};

use crate::{arena::Arena, program::Version};

pub struct State<'a> {
    pub arena: &'a Arena,
    pub env: Vec<&'a str>,
    pub version: Option<Version<'a>>,
    pub protocol_version: Option<u32>,
}

impl<'a> State<'a> {
    pub fn new(arena: &'a Arena, protocol_version: Option<u32>) -> Self {
        Self {
            arena,
            env: Vec::new(),
            version: None,
            protocol_version,
        }
    }

    pub fn set_version(&mut self, version: Version<'a>) {
        self.version = Some(version);
    }

    pub fn is_constr_case_available(&self) -> bool {
        let protocol_ok = self.protocol_version.map_or(true, |pv| pv >= 9);
        let version_ok = self
            .version
            .map_or(true, |v| v.is_constr_case_available());
        protocol_ok && version_ok
    }
}

pub type Extra<'a> = extra::Full<Rich<'a, char>, SimpleState<State<'a>>, ()>;
pub type MapExtra<'a, 'b> = input::MapExtra<'a, 'b, &'a str, Extra<'a>>;
