use std::{error::Error, fs, path, io::Read};
use chrono::{DateTime, Utc};
use crate::components::data::Task;

#[derive(Default, Clone)]
pub struct LocalStorage {
    pub current_id: u64,
    pub tasks: Vec<Task>,
    pub state: String
}

impl LocalStorage {
    pub fn select_task_list(
        &self, 
        active: bool, 
        completed: bool
    ) -> Vec<Task> {
        let mut selected_tasks: Vec<Task> = Vec::new();

        for task in &self.tasks{
            let mut ntask = task.clone();

            if !(task.is_completed) && active {
                selected_tasks.push(ntask);
            } else if task.is_completed && completed {
                selected_tasks.push(ntask);
            }
        }

        selected_tasks
    }

    pub fn create_task(
        &mut self, 
        name: String, 
    ) -> u64 {
        self.current_id += 1;

        let task = Task {
            id: self.current_id,
            name: name,
            is_completed: false,
            create_at: Utc::now(),
            completed_at: None,
        };

        self.tasks.push(task);

        return  self.current_id;
    }

    pub fn delete_task(&mut self, id: u64) -> bool {
        for i in 0..=self.tasks.len() {
            if self.tasks[i].id == id {
                self.tasks.remove(i);

                return true;
            }
        }

        false
    }

    pub fn update_task(
        &mut self,
        id: u64,
        name: Option<String>,
        is_completed: bool,
    ) -> Option<Task> {
        let current_datetime: DateTime<Utc> = Utc::now();

        for i in 0..=self.tasks.len() {
            if self.tasks[i].id == id {
                match name {
                    Some(name) => self.tasks[i].name = name,
                    None => (),
                };
                self.tasks[i].is_completed = is_completed;

                if is_completed {
                        self.tasks[i].completed_at = Some(current_datetime);
                }

                return Some(self.tasks[i].clone());
            }
        }

        return  None;
    }

    pub fn file_to_task(
        &mut self,
        path: &str
    ) {
        let mut file = fs::File::open(path);
        let mut content = String::new();
        file.unwrap().read_to_string(&mut content);

        self.tasks.clear();
        for line in content.split("\n") {
            self.create_task(line.to_string());
        }
    }

    pub fn task_to_file(
        &mut self,
        path: &str
    ) {
        let mut content: Vec<String> = Vec::new();

        for task in self.select_task_list(true, false) {
            content.push(task.name);
        }
        fs::write(path, content.join("\n"));
    }

    pub fn get_state(&mut self) -> String {
        self.state.clone()
    }

    pub fn set_current(&mut self) {
        self.state = String::from("current");
    }

    pub fn set_completed(&mut self) {
        self.state = String::from("completed");
    }
}