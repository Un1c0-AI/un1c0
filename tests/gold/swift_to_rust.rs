use std::sync::Arc;
use std::sync::Mutex;
// Add: uuid = "1.0" to Cargo.toml
// use uuid::Uuid;

// Translated from Swift to Rust
// SwiftUI → Rust (UI framework: egui/iced/dioxus)
// @Published → Arc<Mutex<T>>
// async/await → tokio::spawn


pub struct TaskViewModel {
     pub tasks: Vec<Task> = []
     pub isLoading: bool = false
     pub errorMessage: Option<String>
    
    private pub cancellables = Vec<()>()
    
    #[derive(Clone, Debug)]
pub struct Task {
        let id: Uuid
        pub title: String
        pub isCompleted: bool
    }
    
    init() {
        loadTasks()
    }
    
    pub fn loadTasks() {
        isLoading = true
        
        Task {
            do {
                try await Task.sleep(nanoseconds: 500_000_000)
                
                await MainActor.run {
                    self.tasks = [
                        Task(id: Uuid(), title: "Build un1c0", isCompleted: true),
                        Task(id: Uuid(), title: "Kill all languages", isCompleted: false)
                    ]
                    self.isLoading = false
                }
            } catch {
                await MainActor.run {
                    self.errorMessage = error.localizedDescription
                    self.isLoading = false
                }
            }
        }
    }
    
    pub fn addTask(title: String) {
        let newTask = Task(id: Uuid(), title: title, isCompleted: false)
        tasks.append(newTask)
    }
    
    pub fn toggleTask(id: Uuid) {
        if let index = tasks.firstIndex(where: { $0.id == id }) {
            tasks[index].isCompleted.toggle()
        }
    }
    
    pub fn deleteTask(id: Uuid) {
        tasks.removeAll(where: { $0.id == id })
    }
}

#[derive(Clone, Debug)]
pub struct TaskListView {
     private pub viewModel = TaskViewModel()
     private pub newTaskTitle = ""
    
    pub body: some View {
        NavigationView {
            VStack {
                if viewModel.isLoading {
                    ProgressView("Loading...")
                } else {
                    List {
                        ForEach(viewModel.tasks) { task in
                            HStack {
                                Button(action: {
                                    viewModel.toggleTask(id: task.id)
                                }) {
                                    Image(systemName: task.isCompleted ? "checkmark.circle.fill" : "circle")
                                }
                                
                                Text(task.title)
                                    .strikethrough(task.isCompleted)
                                
                                Spacer()
                                
                                Button(action: {
                                    viewModel.deleteTask(id: task.id)
                                }) {
                                    Image(systemName: "trash")
                                        .foregroundColor(.red)
                                }
                            }
                        }
                    }
                    
                    HStack {
                        TextField("New task", text: $newTaskTitle)
                            .textFieldStyle(RoundedBorderTextFieldStyle())
                        
                        Button("Add") {
                            if !newTaskTitle.isEmpty {
                                viewModel.addTask(title: newTaskTitle)
                                newTaskTitle = ""
                            }
                        }
                        .disabled(newTaskTitle.isEmpty)
                    }
                    .padding()
                }
            }
            .navigationTitle("Tasks")
            .alert(item: Binding(
                get: { viewModel.errorMessage.map { ErrorWrapper(message: $0) } },
                set: { _ in viewModel.errorMessage = nil }
            )) { error in
                Alert(title: Text("Error"), message: Text(error.message))
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct ErrorWrapper {
    let id = UUID()
    let message: String
}


// Note: Swift async/await, Combine, SwiftUI require runtime support
// Recommend: tokio for async, egui/iced for UI
