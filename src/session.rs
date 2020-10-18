use crate::scheduler::{Scheduler, SchedulerData};
use crate::generator::Generator;
use std::collections::HashMap;

pub struct Session {
    schedulers: HashMap<String, Scheduler>,
}

impl Session {

    pub fn start_generator(&mut self, gen: Box<Generator>) {
	self.schedulers.insert(gen.name.clone(), Scheduler::new());	

	let every_half = |data: &mut SchedulerData| -> f64 {
	    
	    println!{"diff: {0}", data.last_diff};
	    
	    match data.generator.root_generator.generator.next_symbol() {
		Some(sym) => println!(" {}", sym),
		None => println!(" NIL"),
	    };
	    
	    0.5
	};

	// start scheduler if it exists ...
	if let Some(sched) = self.schedulers.get_mut(&gen.name) {
	    sched.start(every_half, gen);
	}		
    }
}
