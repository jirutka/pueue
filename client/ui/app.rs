pub struct App {
    pub task_ids: Vec<usize>,
    pub selected_task: usize,
}

impl App {
    /// Select the next task in the task list.
    pub fn next(&mut self) {
        if self.task_ids.is_empty() {
            return;
        }

        // The keys are ordered in ascending order.
        // Select the first one, that's bigger than the current selection.
        for id in self.task_ids.iter() {
            if *id > self.selected_task {
                self.selected_task = *id;
                return;
            }
        }

        // If there's no higher key, jump back to the first task.
        self.selected_task = self.task_ids[0];
    }

    /// Select the previous task in the task list.
    pub fn previous(&mut self) {
        if self.task_ids.is_empty() {
            return;
        }

        // Get the first and last element for convenience purposes
        let mut previous = self.task_ids[0];
        let last = self.task_ids.iter().last();

        // The keys are ordered in ascending order.
        // Select the last one, that was smaller than the current selection.
        for id in self.task_ids.iter() {
            if *id >= self.selected_task {
                // The smallest one is already selected. Break and pick the last one.
                if previous == self.selected_task {
                    break;
                }
                self.selected_task = previous;
                return;
            }

            previous = *id;
        }

        // If there's no higher key, jump back to the first task.
        if let Some(last) = last {
            self.selected_task = *last;
        }
    }

    pub fn update_task_ids(&mut self, ids: Vec<usize>) {
        self.task_ids = ids;
    }
}
