use std::collections::VecDeque;

use bevy::prelude::*;

use super::command::RobotCommand;

/// Очередь команд для робота.
#[derive(Component, Default, Debug)]
pub struct CommandQueue {
    pub current: Option<RobotCommand>,
    pub queue: VecDeque<RobotCommand>,
}

impl CommandQueue {
    pub fn push(&mut self, cmd: RobotCommand) {
        self.queue.push_back(cmd);
    }

    pub fn push_front(&mut self, cmd: RobotCommand) {
        self.queue.push_front(cmd);
    }

    /// Берёт следующую команду из очереди.
    pub fn advance(&mut self) -> Option<RobotCommand> {
        self.current = self.queue.pop_front();
        self.current.clone()
    }
}
