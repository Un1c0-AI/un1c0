import SwiftUI
import Combine

struct TaskManager: View {
    @State private var tasks: [String] = []
    @State private var input: String = ""
    @State private var loading: Bool = false

    var body: some View {
        VStack(spacing: 20) {
            Text("Task Manager")
                .font(.largeTitle)
                .bold()
            TextField("New task", text: $input)
                .textFieldStyle(RoundedBorderTextFieldStyle())
                .disabled(loading)
            Button(action: handleSubmit) {
                Text(loading ? "Adding..." : "Add Task")
            }
            .disabled(loading)
            List(tasks.indices, id: \.self) { index in
                Text(tasks[index])
            }
        }
        .padding()
    }

    func handleSubmit() {
        loading = true
        
        // Server action (stubbed)
        Task {
            try? await Task.sleep(nanoseconds: 500_000_000)
            tasks.append(input)
            input = ""
            loading = false
        }
    }
}
