'use client'

import { useState } from 'react'
import { submitForm } from './actions'

export default function TaskManager() {
  const [tasks, setTasks] = useState<string[]>([])
  const [input, setInput] = useState('')
  const [loading, setLoading] = useState(false)

  async function handleSubmit() {
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
        onChange={(e) => setInput(e.target.value)}
        placeholder="New task"
        disabled={loading}
      />
      <button onClick={handleSubmit} disabled={loading}>
        {loading ? 'Adding...' : 'Add Task'}
      </button>
      <ul>
        {tasks.map((task, index) => (
          <li key={index}>{task}</li>
        ))}
      </ul>
    </div>
  )
}
