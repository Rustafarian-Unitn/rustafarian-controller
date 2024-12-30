use wg_2024::drone::Drone;

pub trait Runnable: Send {
    fn run(&mut self);
}

impl<T: Drone + Send> Runnable for T {
    fn run(&mut self) {
        self.run();
    }
}
