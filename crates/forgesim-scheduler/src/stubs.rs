use forgesim_core::cluster::Cluster;
use forgesim_core::engine::Scheduler;
use forgesim_core::models::Placement;
use forgesim_core::resource::ResourceManager;

macro_rules! stub_scheduler {
    ($name:ident, $msg:literal) => {
        #[derive(Debug, Default, Clone)]
        pub struct $name;

        impl Scheduler for $name {
            fn schedule(
                &mut self,
                _cluster: &mut Cluster,
                _resource_manager: &ResourceManager,
            ) -> Vec<Placement> {
                eprintln!(concat!(stringify!($name), ": ", $msg));
                Vec::new()
            }
        }
    };
}

stub_scheduler!(PriorityScheduler, "not implemented — milestone 4");
stub_scheduler!(BestFitScheduler, "not implemented — milestone 2");
stub_scheduler!(ForgeScheduler, "not implemented — milestone 4");
