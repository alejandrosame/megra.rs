use crate::{
    builtin_types::GlobalParameters,
    event::{EventOperation, InterpretableEvent, StaticEvent},
    generator_processor::GeneratorProcessor,
    markov_sequence_generator::MarkovSequenceGenerator,
};
use ruffbox_synth::building_blocks::{SynthParameterLabel, SynthParameterValue};
use std::boxed::Box;
use std::collections::BTreeSet;
use std::sync::*;

// little helper struct for fixed time operations
#[derive(Clone)]
pub struct TimeMod {
    val: f32,
    op: EventOperation,
}

impl TimeMod {
    fn apply_to(&self, ev: &mut StaticEvent) {
        if let SynthParameterValue::ScalarF32(old_val) = ev.params[&SynthParameterLabel::Duration] {
            let new_val = match self.op {
                EventOperation::Multiply => old_val * self.val,
                EventOperation::Divide => old_val / self.val,
                EventOperation::Add => old_val + self.val,
                EventOperation::Subtract => old_val - self.val,
                EventOperation::Replace => self.val,
            };
            ev.params.insert(
                SynthParameterLabel::Duration,
                SynthParameterValue::ScalarF32(new_val),
            );
        }
    }
}

#[derive(Clone)]
pub struct Generator {
    pub id_tags: BTreeSet<String>,
    pub root_generator: MarkovSequenceGenerator,
    pub processors: Vec<Box<dyn GeneratorProcessor + Send>>,
    pub time_mods: Vec<TimeMod>,
    pub keep_root: bool,
}

impl Generator {
    pub fn transfer_state(&mut self, other: &Generator) {
        self.root_generator.transfer_state(&other.root_generator);
        // this will only work if the generators remain in the same order,
        // but it'll still be helpful I think ..
        for (idx, gp) in self.processors.iter_mut().enumerate() {
            if let Some(g) = other.processors.get(idx) {
                gp.set_state(g.get_state());
            }
        }
    }

    pub fn reached_end_state(&self) -> bool {
        self.root_generator.reached_end_state()
    }

    pub fn current_events(
        &mut self,
        global_parameters: &Arc<GlobalParameters>,
    ) -> Vec<InterpretableEvent> {
        let mut events = self.root_generator.current_events();

        for ev in events.iter_mut() {
            if let InterpretableEvent::Sound(s) = ev {
                s.tags = self.id_tags.union(&s.tags).cloned().collect();
            }
        }

        // temporarily take ownership of processors ...
        // that way we can pass "self" to the "process_generator"
        // function without having to pass the components individually ...
        let mut tmp_procs = Vec::new();
        tmp_procs.append(&mut self.processors);

        for proc in tmp_procs.iter_mut() {
            proc.process_events(&mut events, global_parameters);

            proc.process_generator(self, global_parameters);
        }

        // and back home ...
        self.processors.append(&mut tmp_procs);

        if events.is_empty() {
            println!("no events");
        }

        events
    }

    pub fn current_transition(&mut self, global_parameters: &Arc<GlobalParameters>) -> StaticEvent {
        let mut trans = self.root_generator.current_transition();
        for proc in self.processors.iter_mut() {
            proc.process_transition(&mut trans, global_parameters);
        }
        if let Some(tmod) = self.time_mods.pop() {
            //println!("apply time mod");
            tmod.apply_to(&mut trans);
        }
        trans
    }
}

mod modifier_functions;
pub use modifier_functions::*;

pub mod modifier_functions_raw;
