import SwiftUI
import Combine

@MainActor
class TaskViewModel: ObservableObject {
    @Published var tasks: [Task] = []
    @Published var isLoading: Bool = false
    @Published var errorMessage: String?
    
    private var cancellables = Set<AnyCancellable>()
    
    struct Task: Identifiable {
        let id: UUID
        var title: String
        var isCompleted: Bool
    }
    
    init() {
        loadTasks()
    }
    
    func loadTasks() {
        isLoading = true
        
        Task {
            do {
                try await Task.sleep(nanoseconds: 500_000_000)
                
                await MainActor.run {
                    self.tasks = [
                        Task(id: UUID(), title: "Build un1c0", isCompleted: true),
                        Task(id: UUID(), title: "Kill all languages", isCompleted: false)
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
    
    func addTask(title: String) {
        let newTask = Task(id: UUID(), title: title, isCompleted: false)
        tasks.append(newTask)
    }
    
    func toggleTask(id: UUID) {
        if let index = tasks.firstIndex(where: { $0.id == id }) {
            tasks[index].isCompleted.toggle()
        }
    }
    
    func deleteTask(id: UUID) {
        tasks.removeAll(where: { $0.id == id })
    }
}

struct TaskListView: View {
    @StateObject private var viewModel = TaskViewModel()
    @State private var newTaskTitle = ""
    
    var body: some View {
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

struct ErrorWrapper: Identifiable {
    let id = UUID()
    let message: String
}
