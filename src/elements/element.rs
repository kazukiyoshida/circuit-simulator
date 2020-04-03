use super::super::simulator::Equation;
use std::any::Any;

pub trait Element {
    fn as_any(&mut self) -> &mut dyn Any;
    fn connect_pin_to_node(&mut self, pin_id: usize, node_id: usize);
    fn stamp(&self, eq: &mut Equation);
    fn clk(&self) -> bool {
        false
    }
    fn output_pins(&self) -> Vec<bool> {
        vec![]
    }
}
