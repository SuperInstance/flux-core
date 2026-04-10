use crate::vm::Interpreter;
use crate::bytecode::assembler::Assembler;
use crate::a2a::messages::{A2AMessage, MessageType};
use std::collections::HashMap;

/// An A2A agent that runs FLUX bytecode.
pub struct Agent {
    pub id: String,
    pub role: String,
    pub trust: f32,
    pub inbox: Vec<A2AMessage>,
    pub generation: u32,
    bytecode: Vec<u8>,
}

impl Agent {
    pub fn new(id: &str, bytecode: Vec<u8>, role: &str) -> Self {
        Self {
            id: id.to_string(),
            role: role.to_string(),
            trust: 1.0,
            inbox: Vec::new(),
            generation: 0,
            bytecode,
        }
    }

    /// Execute the agent's bytecode and return cycles consumed.
    pub fn step(&mut self) -> Result<u32, crate::error::FluxError> {
        let mut vm = Interpreter::new(&self.bytecode);
        vm.execute()?;
        self.generation += 1;
        Ok(vm.cycle_count as u32)
    }

    /// Get result from register.
    pub fn result(&self, reg: u8) -> i32 {
        let mut vm = Interpreter::new(&self.bytecode);
        let _ = vm.execute();
        vm.read_gp(reg)
    }

    /// Send a message to another agent.
    pub fn tell(&self, other: &mut Agent, payload: Vec<u8>) {
        let msg = A2AMessage::new(
            [0u8; 16], // simplified sender
            [0u8; 16], // simplified receiver
            MessageType::Tell,
            payload,
        );
        other.inbox.push(msg);
    }
}

/// A swarm of agents that coordinate via A2A protocol.
pub struct Swarm {
    pub agents: HashMap<String, Agent>,
}

impl Swarm {
    pub fn new() -> Self {
        Self { agents: HashMap::new() }
    }

    pub fn add(&mut self, agent: Agent) {
        self.agents.insert(agent.id.clone(), agent);
    }

    /// Execute all agents one step. Returns total cycles.
    pub fn tick(&mut self) -> u32 {
        let mut total = 0u32;
        for agent in self.agents.values_mut() {
            if let Ok(cycles) = agent.step() {
                total += cycles;
            }
        }
        total
    }

    /// Majority vote across agents on register value.
    pub fn vote(&self, reg: u8) -> HashMap<i32, usize> {
        let mut counts = HashMap::new();
        for agent in self.agents.values() {
            let val = agent.result(reg);
            *counts.entry(val).or_insert(0) += 1;
        }
        counts
    }

    /// Get the majority consensus value.
    pub fn consensus(&self, reg: u8) -> Option<i32> {
        let counts = self.vote(reg);
        counts.into_iter().max_by_key(|(_, c)| *c).map(|(v, _)| v)
    }
}
