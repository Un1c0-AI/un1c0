// Example TypeScript React component for UN1C⓪DE translation testing
// This is a demonstration file - not meant to be compiled standalone
// @ts-nocheck

interface TaskManagerState {
  tasks: string[]
  input: string
  loading: boolean
}

interface SubmitResult {
  success: boolean
}

// Mock React hooks for demonstration
declare function useState<T>(initialValue: T): [T, (value: T) => void]

// Mock action function
declare function submitForm(input: string): Promise<SubmitResult>

export default function TaskManager() {
  const [tasks, setTasks] = useState<string[]>([])
  const [input, setInput] = useState('')
  const [loading, setLoading] = useState(false)

  async function handleSubmit(): Promise<void> {
    setLoading(true)
    const result = await submitForm(input)
    if (result.success) {
      setTasks([...tasks, input])
      setInput('')
    }
    setLoading(false)
  }

  return (
    <div className="container">
      <h1>Task Manager</h1>
      <input
        value={input}
        onChange={(e: any) => setInput(e.target.value)}
        placeholder="New task"
        disabled={loading}
      />
      <button onClick={handleSubmit} disabled={loading}>
        {loading ? 'Adding...' : 'Add Task'}
      </button>
      <ul>
        {tasks.map((task: string, index: number) => (
          <li key={index}>{task}</li>
        ))}
      </ul>
    </div>
  )
}
