import { render, screen } from '@testing-library/react'
import { describe, it, expect } from 'vitest'
import { StatusBadge } from './status-badge'

describe('StatusBadge', () => {
  describe('story type', () => {
    it('renders "To Do" for todo status', () => {
      render(<StatusBadge type="story" value="todo" />)
      expect(screen.getByText('To Do')).toBeInTheDocument()
    })

    it('renders "In Progress" for in_progress status', () => {
      render(<StatusBadge type="story" value="in_progress" />)
      expect(screen.getByText('In Progress')).toBeInTheDocument()
    })

    it('renders "Done" for done status', () => {
      render(<StatusBadge type="story" value="done" />)
      expect(screen.getByText('Done')).toBeInTheDocument()
    })
  })

  describe('task type', () => {
    it('renders "Done" for task done status', () => {
      render(<StatusBadge type="task" value="done" />)
      expect(screen.getByText('Done')).toBeInTheDocument()
    })

    it('renders "AI working" for running status', () => {
      render(<StatusBadge type="task" value="running" />)
      expect(screen.getByText('AI working')).toBeInTheDocument()
    })

    it('renders "Needs input" for needs_input status', () => {
      render(<StatusBadge type="task" value="needs_input" />)
      expect(screen.getByText('Needs input')).toBeInTheDocument()
    })

    it('renders "Blocked" for blocked status', () => {
      render(<StatusBadge type="task" value="blocked" />)
      expect(screen.getByText('Blocked')).toBeInTheDocument()
    })
  })
})
