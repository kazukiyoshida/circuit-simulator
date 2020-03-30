use super::super::simulator::Equation;
use std::any::Any;

pub trait Element: std::fmt::Debug {
    fn connect_pin_to_node(&mut self, pin_id: usize, node_id: usize);
    fn stamp(&self, eq: &mut Equation);
    fn as_any(&mut self) -> &mut dyn Any;
    fn is_voltage_or_current_src(&self) -> bool {
        false
    }
}

